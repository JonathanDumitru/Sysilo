package middleware

import (
	"context"
	"net/http"
	"strings"
	"time"

	"github.com/go-chi/chi/v5/middleware"
	"github.com/golang-jwt/jwt/v5"
	"github.com/sysilo/sysilo/services/api-gateway/internal/config"
	"go.uber.org/zap"
)

// Context keys
type contextKey string

const (
	ContextKeyTenantID contextKey = "tenant_id"
	ContextKeyUserID   contextKey = "user_id"
	ContextKeyRoles    contextKey = "roles"
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

			w.Header().Set("Access-Control-Allow-Methods", strings.Join(cfg.AllowedMethods, ", "))
			w.Header().Set("Access-Control-Allow-Headers", strings.Join(cfg.AllowedHeaders, ", "))

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

			// Extract user info from claims
			userID, _ := claims["sub"].(string)
			tenantID, _ := claims["tenant_id"].(string)
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

			// Check if user has any of the required roles
			hasRole := false
			for _, required := range requiredRoles {
				for _, role := range roles {
					if role == required {
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
