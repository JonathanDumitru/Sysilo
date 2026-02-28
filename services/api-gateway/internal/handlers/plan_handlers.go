package handlers

import (
	"net/http"

	"github.com/sysilo/sysilo/services/api-gateway/internal/middleware"
	"go.uber.org/zap"
)

// GetCurrentPlan returns the tenant's current plan with features and limits
func (h *Handler) GetCurrentPlan(w http.ResponseWriter, r *http.Request) {
	tenantID := middleware.GetTenantID(r.Context())

	tenantPlan, err := h.DB.Plans.GetTenantPlan(r.Context(), tenantID)
	if err != nil {
		h.Logger.Error("Failed to get tenant plan", zap.Error(err))
		respondError(w, http.StatusInternalServerError, "failed to get plan")
		return
	}
	if tenantPlan == nil {
		respondError(w, http.StatusNotFound, "tenant not found")
		return
	}

	respondJSON(w, http.StatusOK, tenantPlan)
}

// GetPlanUsage returns the tenant's current usage counters and resource counts
func (h *Handler) GetPlanUsage(w http.ResponseWriter, r *http.Request) {
	tenantID := middleware.GetTenantID(r.Context())

	usage, err := h.DB.Usage.GetOrCreateCurrentPeriod(r.Context(), tenantID)
	if err != nil {
		h.Logger.Error("Failed to get usage counters", zap.Error(err))
		respondError(w, http.StatusInternalServerError, "failed to get usage")
		return
	}

	counts, err := h.DB.Usage.CountTenantResources(r.Context(), tenantID)
	if err != nil {
		h.Logger.Error("Failed to count resources", zap.Error(err))
		respondError(w, http.StatusInternalServerError, "failed to count resources")
		return
	}

	respondJSON(w, http.StatusOK, map[string]interface{}{
		"period":    usage,
		"resources": counts,
	})
}

// ListPlans returns all active plans (for pricing page)
func (h *Handler) ListPlans(w http.ResponseWriter, r *http.Request) {
	plans, err := h.DB.Plans.ListActive(r.Context())
	if err != nil {
		h.Logger.Error("Failed to list plans", zap.Error(err))
		respondError(w, http.StatusInternalServerError, "failed to list plans")
		return
	}

	respondJSON(w, http.StatusOK, map[string]interface{}{
		"plans": plans,
	})
}
