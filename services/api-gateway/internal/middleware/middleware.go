package middleware

import (
	"context"
	"crypto/subtle"
	"database/sql"
	"fmt"
	"net/http"
	"os"
	"strconv"
	"strings"
	"sync"
	"time"

	"github.com/go-chi/chi/v5/middleware"
	"github.com/golang-jwt/jwt/v5"
	_ "github.com/lib/pq"
	"github.com/sysilo/sysilo/services/api-gateway/internal/authorization"
	"github.com/sysilo/sysilo/services/api-gateway/internal/config"
	"go.uber.org/zap"
)

// Context keys
type contextKey string

const (
	ContextKeyTenantID contextKey = "tenant_id"
	ContextKeyUserID   contextKey = "user_id"
	ContextKeyRoles    contextKey = "roles"
	ContextKeyEnv      contextKey = "environment"
	ContextKeyTeamID   contextKey = "team_id"
	ContextKeySCIMScopes contextKey = "scim_scopes"
)

var (
	authProfileDB     *sql.DB
	authProfileDBErr  error
	authProfileDBOnce sync.Once
)

// Logger returns a middleware that logs requests
func Logger(logger *zap.Logger) func(next http.Handler) http.Handler {
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			start := time.Now()
			ww := middleware.NewWrapResponseWriter(w, r.ProtoMajor)

			defer func() {
				logger.Info("request",
					zap.String("method", r.Method),
					zap.String("path", r.URL.Path),
					zap.Int("status", ww.Status()),
					zap.Int("bytes", ww.BytesWritten()),
					zap.Duration("duration", time.Since(start)),
					zap.String("request_id", middleware.GetReqID(r.Context())),
					zap.String("remote_addr", r.RemoteAddr),
				)
			}()

			next.ServeHTTP(ww, r)
		})
	}
}

// CORS returns a middleware that handles CORS
func CORS(cfg config.CORSConfig) func(next http.Handler) http.Handler {
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			origin := r.Header.Get("Origin")

			// Check if origin is allowed
			allowed := false
			for _, o := range cfg.AllowedOrigins {
				if o == "*" || o == origin {
					allowed = true
					break
				}
			}

			if allowed {
				w.Header().Set("Access-Control-Allow-Origin", origin)
			}

			if cfg.AllowCredentials {
				w.Header().Set("Access-Control-Allow-Credentials", "true")
			}

			allowedHeaders := ensureHeader(cfg.AllowedHeaders, "X-Environment")
			allowedHeaders = ensureHeader(allowedHeaders, "X-Team-ID")
			w.Header().Set("Access-Control-Allow-Methods", strings.Join(cfg.AllowedMethods, ", "))
			w.Header().Set("Access-Control-Allow-Headers", strings.Join(allowedHeaders, ", "))

			if len(cfg.ExposedHeaders) > 0 {
				w.Header().Set("Access-Control-Expose-Headers", strings.Join(cfg.ExposedHeaders, ", "))
			}

			// Handle preflight
			if r.Method == "OPTIONS" {
				w.Header().Set("Access-Control-Max-Age", string(rune(cfg.MaxAge)))
				w.WriteHeader(http.StatusNoContent)
				return
			}

			next.ServeHTTP(w, r)
		})
	}
}

// Auth returns a middleware that validates JWT tokens
func Auth(logger *zap.Logger, cfg config.AuthConfig) func(next http.Handler) http.Handler {
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			// Get token from Authorization header
			authHeader := r.Header.Get("Authorization")
			if authHeader == "" {
				http.Error(w, "missing authorization header", http.StatusUnauthorized)
				return
			}

			// Parse "Bearer <token>"
			parts := strings.SplitN(authHeader, " ", 2)
			if len(parts) != 2 || strings.ToLower(parts[0]) != "bearer" {
				http.Error(w, "invalid authorization header format", http.StatusUnauthorized)
				return
			}

			tokenString := parts[1]

			// Parse and validate token
			token, err := jwt.Parse(tokenString, func(token *jwt.Token) (interface{}, error) {
				// Validate signing method
				if _, ok := token.Method.(*jwt.SigningMethodHMAC); !ok {
					return nil, jwt.ErrSignatureInvalid
				}
				return []byte(cfg.JWTSecret), nil
			})

			if err != nil {
				logger.Debug("Token validation failed", zap.Error(err))
				http.Error(w, "invalid token", http.StatusUnauthorized)
				return
			}

			claims, ok := token.Claims.(jwt.MapClaims)
			if !ok || !token.Valid {
				http.Error(w, "invalid token claims", http.StatusUnauthorized)
				return
			}
			tokenType, _ := claims["token_type"].(string)
			if tokenType != "access" {
				http.Error(w, "invalid token type", http.StatusUnauthorized)
				return
			}

			// Extract user info from claims
			userID, _ := claims["sub"].(string)
			tenantID, _ := claims["tenant_id"].(string)
			status, _ := claims["status"].(string)
			sessionVersion, err := intClaim(claims["session_version"])
			if err != nil {
				http.Error(w, "invalid token claims", http.StatusUnauthorized)
				return
			}
			if userID == "" || tenantID == "" || status != "active" {
				http.Error(w, "invalid token claims", http.StatusUnauthorized)
				return
			}

			profile, err := loadAuthProfile(r.Context(), tenantID, userID)
			if err != nil {
				logger.Warn("Failed to load request-time auth profile", zap.Error(err))
				http.Error(w, "invalid token", http.StatusUnauthorized)
				return
			}
			if profile == nil || profile.Status != "active" || profile.SessionVersion != sessionVersion {
				http.Error(w, "session is no longer valid", http.StatusUnauthorized)
				return
			}

			roles, _ := claims["roles"].([]interface{})

			// Convert roles to string slice
			roleStrings := make([]string, 0, len(roles))
			for _, r := range roles {
				if s, ok := r.(string); ok {
					roleStrings = append(roleStrings, s)
				}
			}

			// Add to context
			ctx := r.Context()
			ctx = context.WithValue(ctx, ContextKeyUserID, userID)
			ctx = context.WithValue(ctx, ContextKeyTenantID, tenantID)
			ctx = context.WithValue(ctx, ContextKeyRoles, roleStrings)

			next.ServeHTTP(w, r.WithContext(ctx))
		})
	}
}

type authProfile struct {
	Status         string
	SessionVersion int
}

func loadAuthProfile(ctx context.Context, tenantID, userID string) (*authProfile, error) {
	db, err := authProfileConn()
	if err != nil {
		return nil, err
	}

	var profile authProfile
	err = db.QueryRowContext(ctx, `
		SELECT status, session_version
		FROM users
		WHERE tenant_id = $1 AND id = $2
	`, tenantID, userID).Scan(&profile.Status, &profile.SessionVersion)
	if err == sql.ErrNoRows {
		return nil, nil
	}
	if err != nil {
		return nil, err
	}
	return &profile, nil
}

func authProfileConn() (*sql.DB, error) {
	authProfileDBOnce.Do(func() {
		host := envOrDefault("SYSILO_DB_HOST", "localhost")
		port := envOrDefault("SYSILO_DB_PORT", "5432")
		user := envOrDefault("SYSILO_DB_USER", "sysilo")
		password := envOrDefault("SYSILO_DB_PASSWORD", "sysilo")
		database := envOrDefault("SYSILO_DB_NAME", "sysilo")
		sslMode := envOrDefault("SYSILO_DB_SSLMODE", "disable")

		dsn := fmt.Sprintf("host=%s port=%s user=%s password=%s dbname=%s sslmode=%s",
			host, port, user, password, database, sslMode,
		)
		conn, err := sql.Open("postgres", dsn)
		if err != nil {
			authProfileDBErr = err
			return
		}
		conn.SetMaxOpenConns(4)
		conn.SetMaxIdleConns(2)
		conn.SetConnMaxLifetime(2 * time.Minute)
		if err := conn.Ping(); err != nil {
			authProfileDBErr = err
			conn.Close()
			return
		}
		authProfileDB = conn
	})
	if authProfileDBErr != nil {
		return nil, authProfileDBErr
	}
	if authProfileDB == nil {
		return nil, fmt.Errorf("auth profile connection not initialized")
	}
	return authProfileDB, nil
}

func envOrDefault(key, fallback string) string {
	if value := os.Getenv(key); value != "" {
		return value
	}
	return fallback
}

func intClaim(v interface{}) (int, error) {
	switch value := v.(type) {
	case float64:
		return int(value), nil
	case int:
		return value, nil
	case string:
		n, err := strconv.Atoi(value)
		if err != nil {
			return 0, err
		}
		return n, nil
	default:
		return 0, fmt.Errorf("invalid integer claim")
	}
}

// TenantContext ensures tenant context is set
func TenantContext(logger *zap.Logger) func(next http.Handler) http.Handler {
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			tenantID := r.Context().Value(ContextKeyTenantID)
			if tenantID == nil || tenantID.(string) == "" {
				// Check header as fallback (for service-to-service calls)
				headerTenant := r.Header.Get("X-Tenant-ID")
				if headerTenant != "" {
					ctx := context.WithValue(r.Context(), ContextKeyTenantID, headerTenant)
					r = r.WithContext(ctx)
				} else {
					http.Error(w, "tenant context required", http.StatusBadRequest)
					return
				}
			}

			rawEnvironment := r.Header.Get("X-Environment")
			environment, ok := authorization.ParseEnvironment(rawEnvironment)
			if !ok {
				http.Error(w, "valid environment context required", http.StatusBadRequest)
				return
			}
			rawTeamID := normalizeTeamID(r.Header.Get("X-Team-ID"))
			if !isValidTeamID(rawTeamID) {
				http.Error(w, "valid team context required", http.StatusBadRequest)
				return
			}

			ctx := context.WithValue(r.Context(), ContextKeyEnv, string(environment))
			ctx = context.WithValue(ctx, ContextKeyTeamID, rawTeamID)
			r = r.WithContext(ctx)

			next.ServeHTTP(w, r)
		})
	}
}

// RequireRole returns a middleware that requires specific roles
func RequireRole(requiredRoles ...string) func(next http.Handler) http.Handler {
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			roles, ok := r.Context().Value(ContextKeyRoles).([]string)
			if !ok {
				http.Error(w, "forbidden", http.StatusForbidden)
				return
			}
			environment := GetEnvironment(r.Context())
			if environment == "" {
				http.Error(w, "valid environment context required", http.StatusBadRequest)
				return
			}
			teamID := GetTeamID(r.Context())
			if teamID == "" {
				http.Error(w, "valid team context required", http.StatusBadRequest)
				return
			}

			// Check if user has any of the required roles
			hasRole := false
			for _, required := range requiredRoles {
				for _, role := range roles {
					if roleMatchesRequired(role, required) {
						hasRole = true
						break
					}
				}
				if hasRole {
					break
				}
			}

			if !hasRole {
				http.Error(w, "forbidden", http.StatusForbidden)
				return
			}
			action := actionForRequest(r)
			parsedEnvironment, ok := authorization.ParseEnvironment(environment)
			if !ok {
				http.Error(w, "valid environment context required", http.StatusBadRequest)
				return
			}
			if !authorization.Authorize(roles, parsedEnvironment, action, teamID) {
				http.Error(w, "forbidden", http.StatusForbidden)
				return
			}

			next.ServeHTTP(w, r)
		})
	}
}

// PlanLoader is a function that loads plan info for a tenant
type PlanLoader func(ctx context.Context, tenantID string) (*PlanInfo, error)

// LoadTenantPlan loads the tenant's plan into context after tenant is resolved
func LoadTenantPlan(logger *zap.Logger, loader PlanLoader) func(next http.Handler) http.Handler {
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			tenantID := GetTenantID(r.Context())
			if tenantID == "" {
				next.ServeHTTP(w, r)
				return
			}

			info, err := loader(r.Context(), tenantID)
			if err != nil {
				logger.Error("Failed to load tenant plan", zap.Error(err), zap.String("tenant_id", tenantID))
				// Don't block the request, just proceed without plan context
				next.ServeHTTP(w, r)
				return
			}

			if info != nil {
				ctx := SetPlanContext(r.Context(), *info)
				r = r.WithContext(ctx)
			}

			next.ServeHTTP(w, r)
		})
	}
}

// RateLimit returns a middleware that rate limits requests
func RateLimit(cfg config.RateLimitConfig) func(next http.Handler) http.Handler {
	// TODO: Implement proper rate limiting with Redis
	// For now, this is a placeholder
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			if !cfg.Enabled {
				next.ServeHTTP(w, r)
				return
			}

			// TODO: Check rate limit using Redis
			// tenantID := r.Context().Value(ContextKeyTenantID).(string)
			// key := fmt.Sprintf("ratelimit:%s", tenantID)

			next.ServeHTTP(w, r)
		})
	}
}

// GetTenantID extracts tenant ID from request context
func GetTenantID(ctx context.Context) string {
	if v := ctx.Value(ContextKeyTenantID); v != nil {
		return v.(string)
	}
	return ""
}

// GetUserID extracts user ID from request context
func GetUserID(ctx context.Context) string {
	if v := ctx.Value(ContextKeyUserID); v != nil {
		return v.(string)
	}
	return ""
}

// GetRoles extracts roles from request context
func GetRoles(ctx context.Context) []string {
	if v := ctx.Value(ContextKeyRoles); v != nil {
		return v.([]string)
	}
	return nil
}

// GetEnvironment extracts environment from request context.
func GetEnvironment(ctx context.Context) string {
	if v := ctx.Value(ContextKeyEnv); v != nil {
		return v.(string)
	}
	return ""
}

// GetTeamID extracts team ID from request context.
func GetTeamID(ctx context.Context) string {
	if v := ctx.Value(ContextKeyTeamID); v != nil {
		return v.(string)
	}
	return ""
}

func actionForRequest(r *http.Request) authorization.Action {
	if strings.HasPrefix(r.URL.Path, "/api/admin") || strings.Contains(r.URL.Path, "/admin/") {
		return authorization.ActionAdmin
	}
	switch r.Method {
	case http.MethodGet, http.MethodHead, http.MethodOptions:
		return authorization.ActionRead
	default:
		return authorization.ActionWrite
	}
}

func ensureHeader(headers []string, required string) []string {
	for _, header := range headers {
		if strings.EqualFold(header, required) {
			return headers
		}
	}
	return append(headers, required)
}

func normalizeTeamID(raw string) string {
	return strings.ToLower(strings.TrimSpace(raw))
}

func isValidTeamID(raw string) bool {
	if raw == "" || len(raw) > 64 {
		return false
	}
	for _, r := range raw {
		if (r >= 'a' && r <= 'z') || (r >= 'A' && r <= 'Z') || (r >= '0' && r <= '9') || r == '-' || r == '_' {
			continue
		}
		return false
	}
	return true
}

func roleMatchesRequired(role, required string) bool {
	role = strings.ToLower(strings.TrimSpace(role))
	required = strings.ToLower(strings.TrimSpace(required))
	if role == required {
		return true
	}
	return strings.HasSuffix(role, ":"+required) ||
		strings.HasPrefix(role, required+"#") ||
		strings.HasSuffix(role, "/"+required)
}

// RequireSCIMToken validates SCIM bearer credentials at the route boundary.
// It accepts either a static token (SCIM_BEARER_TOKEN) or a signed JWT.
func RequireSCIMToken(logger *zap.Logger, cfg config.AuthConfig) func(next http.Handler) http.Handler {
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			authz := strings.TrimSpace(r.Header.Get("Authorization"))
			if !strings.HasPrefix(strings.ToLower(authz), "bearer ") {
				http.Error(w, "missing bearer token", http.StatusUnauthorized)
				return
			}
			tokenString := strings.TrimSpace(authz[len("Bearer "):])
			if tokenString == "" {
				http.Error(w, "missing bearer token", http.StatusUnauthorized)
				return
			}

			expected := strings.TrimSpace(os.Getenv("SCIM_BEARER_TOKEN"))
			if expected != "" && subtle.ConstantTimeCompare([]byte(tokenString), []byte(expected)) == 1 {
				ctx := context.WithValue(r.Context(), ContextKeySCIMScopes, []string{"scim:admin"})
				next.ServeHTTP(w, r.WithContext(ctx))
				return
			}

			token, err := jwt.Parse(tokenString, func(token *jwt.Token) (interface{}, error) {
				if _, ok := token.Method.(*jwt.SigningMethodHMAC); !ok {
					return nil, jwt.ErrSignatureInvalid
				}
				return []byte(cfg.JWTSecret), nil
			})
			if err != nil {
				logger.Debug("SCIM token validation failed", zap.Error(err))
				http.Error(w, "invalid scim token", http.StatusUnauthorized)
				return
			}
			claims, ok := token.Claims.(jwt.MapClaims)
			if !ok || !token.Valid {
				http.Error(w, "invalid scim token claims", http.StatusUnauthorized)
				return
			}

			scopes := extractScopes(claims)
			ctx := context.WithValue(r.Context(), ContextKeySCIMScopes, scopes)
			next.ServeHTTP(w, r.WithContext(ctx))
		})
	}
}

// RequireSCIMAdminScope enforces a SCIM admin scope on provision/deprovision routes.
func RequireSCIMAdminScope() func(next http.Handler) http.Handler {
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			scopes, _ := r.Context().Value(ContextKeySCIMScopes).([]string)
			if hasScope(scopes, "scim:admin") {
				next.ServeHTTP(w, r)
				return
			}
			http.Error(w, "forbidden", http.StatusForbidden)
		})
	}
}

func extractScopes(claims jwt.MapClaims) []string {
	var scopes []string
	if raw, ok := claims["scope"].(string); ok {
		scopes = append(scopes, strings.Fields(raw)...)
	}
	if raw, ok := claims["scopes"].([]interface{}); ok {
		for _, item := range raw {
			if s, isString := item.(string); isString && s != "" {
				scopes = append(scopes, s)
			}
		}
	}
	return scopes
}

func hasScope(scopes []string, scope string) bool {
	for _, candidate := range scopes {
		if candidate == scope {
			return true
		}
	}
	return false
}
