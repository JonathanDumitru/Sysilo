package middleware

import (
	"context"
	"encoding/json"
	"net/http"
	"strings"

	"go.uber.org/zap"
)

const (
	ContextKeyPlanFeatures contextKey = "plan_features"
	ContextKeyPlanLimits   contextKey = "plan_limits"
	ContextKeyPlanName     contextKey = "plan_name"
	ContextKeyPlanStatus   contextKey = "plan_status"
)

// PlanInfo is loaded during auth and stored in context
type PlanInfo struct {
	Name     string          `json:"name"`
	Status   string          `json:"status"`
	Features json.RawMessage `json:"features"`
	Limits   json.RawMessage `json:"limits"`
}

// featureGate maps route prefixes to required feature keys
var featureGate = map[string]string{
	"/api/v1/governance":     "governance_enabled",
	"/api/v1/compliance":     "compliance_enabled",
	"/api/v1/rationalization": "rationalization_enabled",
	"/api/v1/ai":             "ai_enabled",
	"/api/v1/ops/advanced":   "advanced_ops_enabled",
}

// PlanGate checks if the tenant's plan allows access to the requested feature
func PlanGate(logger *zap.Logger) func(next http.Handler) http.Handler {
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			path := r.URL.Path
			planName := GetPlanName(r.Context())
			planStatus := GetPlanStatus(r.Context())

			// Suspended tenants: read-only access
			if planStatus == "suspended" {
				if r.Method != "GET" && r.Method != "HEAD" && r.Method != "OPTIONS" {
					w.Header().Set("Content-Type", "application/json")
					w.WriteHeader(http.StatusForbidden)
					json.NewEncoder(w).Encode(map[string]interface{}{
						"error":        "account_suspended",
						"message":      "Your account is suspended. Please upgrade to restore access.",
						"current_plan": planName,
					})
					return
				}
			}

			// Check feature gates
			for prefix, featureKey := range featureGate {
				if strings.HasPrefix(path, prefix) {
					if !hasFeature(r.Context(), featureKey) {
						requiredPlan := requiredPlanFor(featureKey)
						w.Header().Set("Content-Type", "application/json")
						w.WriteHeader(http.StatusForbidden)
						json.NewEncoder(w).Encode(map[string]interface{}{
							"error":         "upgrade_required",
							"feature":       featureKey,
							"current_plan":  planName,
							"required_plan": requiredPlan,
						})
						logger.Info("Feature gated",
							zap.String("feature", featureKey),
							zap.String("plan", planName),
							zap.String("path", path),
						)
						return
					}
					break
				}
			}

			next.ServeHTTP(w, r)
		})
	}
}

func hasFeature(ctx context.Context, featureKey string) bool {
	featuresRaw := ctx.Value(ContextKeyPlanFeatures)
	if featuresRaw == nil {
		return false
	}

	features, ok := featuresRaw.(json.RawMessage)
	if !ok {
		return false
	}

	var featureMap map[string]interface{}
	if err := json.Unmarshal(features, &featureMap); err != nil {
		return false
	}

	val, exists := featureMap[featureKey]
	if !exists {
		return false
	}

	enabled, ok := val.(bool)
	return ok && enabled
}

func requiredPlanFor(featureKey string) string {
	switch featureKey {
	case "compliance_enabled", "rationalization_enabled":
		return "enterprise"
	case "governance_enabled", "ai_enabled", "advanced_ops_enabled":
		return "business"
	default:
		return "business"
	}
}

// GetPlanName extracts plan name from context
func GetPlanName(ctx context.Context) string {
	if v := ctx.Value(ContextKeyPlanName); v != nil {
		return v.(string)
	}
	return ""
}

// GetPlanStatus extracts plan status from context
func GetPlanStatus(ctx context.Context) string {
	if v := ctx.Value(ContextKeyPlanStatus); v != nil {
		return v.(string)
	}
	return ""
}

// GetPlanFeatures extracts plan features from context
func GetPlanFeatures(ctx context.Context) json.RawMessage {
	if v := ctx.Value(ContextKeyPlanFeatures); v != nil {
		return v.(json.RawMessage)
	}
	return nil
}

// GetPlanLimits extracts plan limits from context
func GetPlanLimits(ctx context.Context) json.RawMessage {
	if v := ctx.Value(ContextKeyPlanLimits); v != nil {
		return v.(json.RawMessage)
	}
	return nil
}

// SetPlanContext injects plan info into the request context
func SetPlanContext(ctx context.Context, info PlanInfo) context.Context {
	ctx = context.WithValue(ctx, ContextKeyPlanName, info.Name)
	ctx = context.WithValue(ctx, ContextKeyPlanStatus, info.Status)
	ctx = context.WithValue(ctx, ContextKeyPlanFeatures, info.Features)
	ctx = context.WithValue(ctx, ContextKeyPlanLimits, info.Limits)
	return ctx
}
