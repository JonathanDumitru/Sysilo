package middleware

import (
	"encoding/json"
	"fmt"
	"net/http"
	"strconv"
	"strings"
	"time"

	"github.com/sysilo/sysilo/services/api-gateway/internal/ratelimit"
	"go.uber.org/zap"
)

// RateLimitWithRedis returns a chi-compatible middleware that enforces
// sliding-window rate limits using a Redis-backed RateLimiter.
//
// It extracts the tenant ID from the request context (set by the Auth
// middleware) or from the X-Tenant-ID header as a fallback. Health-check
// endpoints (/health, /ready) are excluded from rate limiting.
//
// Standard rate-limit response headers are set on every response:
//
//	X-RateLimit-Limit, X-RateLimit-Remaining, X-RateLimit-Reset
//
// When the limit is exceeded the middleware returns 429 Too Many Requests
// with a Retry-After header.
func RateLimitWithRedis(logger *zap.Logger, limiter *ratelimit.RateLimiter, requestsPerMinute int) func(next http.Handler) http.Handler {
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			// Skip rate limiting for health check endpoints
			path := r.URL.Path
			if path == "/health" || path == "/ready" || strings.HasPrefix(path, "/health/") {
				next.ServeHTTP(w, r)
				return
			}

			// Extract tenant ID from context (set by Auth middleware) or header
			tenantID := GetTenantID(r.Context())
			if tenantID == "" {
				tenantID = r.Header.Get("X-Tenant-ID")
			}
			if tenantID == "" {
				// No tenant to rate limit against; let the request through
				// (auth middleware will reject unauthenticated requests anyway)
				next.ServeHTTP(w, r)
				return
			}

			// Derive a coarse endpoint identifier for per-endpoint limits
			endpoint := normalizeEndpoint(r.Method, path)

			allowed, remaining, resetAt, err := limiter.Check(r.Context(), tenantID, endpoint)
			if err != nil {
				logger.Error("Rate limiter error, allowing request",
					zap.Error(err),
					zap.String("tenant_id", tenantID),
				)
				// Fail open: if Redis is down, don't block traffic
				next.ServeHTTP(w, r)
				return
			}

			// Set standard rate-limit headers on every response
			w.Header().Set("X-RateLimit-Limit", strconv.Itoa(requestsPerMinute))
			w.Header().Set("X-RateLimit-Remaining", strconv.Itoa(remaining))
			w.Header().Set("X-RateLimit-Reset", strconv.FormatInt(resetAt.Unix(), 10))

			if !allowed {
				retryAfter := time.Until(resetAt)
				if retryAfter < time.Second {
					retryAfter = time.Second
				}
				w.Header().Set("Retry-After", strconv.Itoa(int(retryAfter.Seconds())))
				w.Header().Set("Content-Type", "application/json")
				w.WriteHeader(http.StatusTooManyRequests)
				json.NewEncoder(w).Encode(map[string]interface{}{
					"error":   "rate_limit_exceeded",
					"message": fmt.Sprintf("Rate limit exceeded. Try again in %d seconds.", int(retryAfter.Seconds())),
				})
				logger.Warn("Rate limit exceeded",
					zap.String("tenant_id", tenantID),
					zap.String("endpoint", endpoint),
					zap.Int("remaining", remaining),
				)
				return
			}

			next.ServeHTTP(w, r)
		})
	}
}

// normalizeEndpoint reduces the request path to a stable key suitable for
// per-endpoint rate limiting. It strips variable path segments (UUIDs, numeric
// IDs) so that e.g. GET /api/v1/agents/abc-123 and GET /api/v1/agents/def-456
// share the same bucket.
func normalizeEndpoint(method, path string) string {
	parts := strings.Split(strings.Trim(path, "/"), "/")
	normalized := make([]string, 0, len(parts))
	for _, p := range parts {
		if isIDSegment(p) {
			normalized = append(normalized, ":id")
		} else {
			normalized = append(normalized, p)
		}
	}
	return method + ":" + strings.Join(normalized, "/")
}

// isIDSegment returns true if the path segment looks like a variable identifier
// (UUID or numeric ID) that should be collapsed.
func isIDSegment(s string) bool {
	if len(s) == 0 {
		return false
	}
	// Numeric IDs
	if _, err := strconv.ParseInt(s, 10, 64); err == nil {
		return true
	}
	// UUID-like: 8-4-4-4-12 hex characters with dashes (36 chars) or without (32 chars)
	if len(s) == 36 && strings.Count(s, "-") == 4 {
		return true
	}
	if len(s) == 32 {
		for _, c := range s {
			if !((c >= '0' && c <= '9') || (c >= 'a' && c <= 'f') || (c >= 'A' && c <= 'F')) {
				return false
			}
		}
		return true
	}
	return false
}
