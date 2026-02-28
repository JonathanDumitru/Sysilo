package handlers

import (
	"encoding/json"
	"io"
	"net/http"
	"os"

	"github.com/sysilo/sysilo/services/api-gateway/internal/middleware"
	"go.uber.org/zap"
)

// Stripe config is loaded from environment
func getStripeKey() string {
	return os.Getenv("STRIPE_SECRET_KEY")
}

func getStripeWebhookSecret() string {
	return os.Getenv("STRIPE_WEBHOOK_SECRET")
}

// CreateCheckoutSession creates a Stripe Checkout session for plan upgrade
func (h *Handler) CreateCheckoutSession(w http.ResponseWriter, r *http.Request) {
	tenantID := middleware.GetTenantID(r.Context())

	var req struct {
		PlanName  string `json:"plan_name"`
		SuccessURL string `json:"success_url"`
		CancelURL  string `json:"cancel_url"`
	}

	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		respondError(w, http.StatusBadRequest, "invalid request body")
		return
	}

	if req.PlanName == "" {
		respondError(w, http.StatusBadRequest, "plan_name is required")
		return
	}

	// Get the target plan
	plan, err := h.DB.Plans.GetByName(r.Context(), req.PlanName)
	if err != nil {
		h.Logger.Error("Failed to get plan", zap.Error(err))
		respondError(w, http.StatusInternalServerError, "failed to get plan")
		return
	}
	if plan == nil {
		respondError(w, http.StatusNotFound, "plan not found")
		return
	}

	// Get tenant's current billing info
	tenantPlan, err := h.DB.Plans.GetTenantPlan(r.Context(), tenantID)
	if err != nil {
		h.Logger.Error("Failed to get tenant plan", zap.Error(err))
		respondError(w, http.StatusInternalServerError, "failed to get tenant")
		return
	}

	stripeKey := getStripeKey()
	if stripeKey == "" {
		// Development mode: simulate checkout
		respondJSON(w, http.StatusOK, map[string]interface{}{
			"checkout_url": req.SuccessURL + "?session_id=dev_session_" + tenantID,
			"session_id":   "dev_session_" + tenantID,
			"mode":         "development",
		})
		return
	}

	// Build Stripe Checkout session request
	stripePriceID := plan.StripePriceStr
	if stripePriceID == "" {
		respondError(w, http.StatusBadRequest, "plan is not configured for billing")
		return
	}

	successURL := req.SuccessURL
	if successURL == "" {
		successURL = "http://localhost:3000/settings?tab=billing&status=success"
	}
	cancelURL := req.CancelURL
	if cancelURL == "" {
		cancelURL = "http://localhost:3000/settings?tab=billing&status=cancelled"
	}

	// Create checkout session via Stripe API
	checkoutBody := map[string]interface{}{
		"mode":        "subscription",
		"success_url": successURL + "?session_id={CHECKOUT_SESSION_ID}",
		"cancel_url":  cancelURL,
		"line_items": []map[string]interface{}{
			{
				"price":    stripePriceID,
				"quantity": 1,
			},
		},
		"metadata": map[string]string{
			"tenant_id": tenantID,
			"plan_name": req.PlanName,
		},
	}

	// If tenant already has a Stripe customer, use it
	if tenantPlan != nil && tenantPlan.StripeCustomerStr != "" {
		checkoutBody["customer"] = tenantPlan.StripeCustomerStr
	}

	// In production, this would call Stripe API
	// For now, return the checkout configuration
	respondJSON(w, http.StatusOK, map[string]interface{}{
		"checkout_config": checkoutBody,
		"message":         "Stripe integration pending - configure STRIPE_SECRET_KEY",
	})
}

// CreatePortalSession creates a Stripe Customer Portal session
func (h *Handler) CreatePortalSession(w http.ResponseWriter, r *http.Request) {
	tenantID := middleware.GetTenantID(r.Context())

	tenantPlan, err := h.DB.Plans.GetTenantPlan(r.Context(), tenantID)
	if err != nil {
		h.Logger.Error("Failed to get tenant plan", zap.Error(err))
		respondError(w, http.StatusInternalServerError, "failed to get tenant")
		return
	}

	if tenantPlan == nil || tenantPlan.StripeCustomerStr == "" {
		respondError(w, http.StatusBadRequest, "no billing account found")
		return
	}

	stripeKey := getStripeKey()
	if stripeKey == "" {
		respondJSON(w, http.StatusOK, map[string]interface{}{
			"portal_url": "http://localhost:3000/settings?tab=billing",
			"mode":       "development",
		})
		return
	}

	// In production, create Stripe Billing Portal session
	respondJSON(w, http.StatusOK, map[string]interface{}{
		"message": "Stripe portal integration pending - configure STRIPE_SECRET_KEY",
	})
}

// GetSubscription returns the current subscription status
func (h *Handler) GetSubscription(w http.ResponseWriter, r *http.Request) {
	tenantID := middleware.GetTenantID(r.Context())

	tenantPlan, err := h.DB.Plans.GetTenantPlan(r.Context(), tenantID)
	if err != nil {
		h.Logger.Error("Failed to get tenant plan", zap.Error(err))
		respondError(w, http.StatusInternalServerError, "failed to get subscription")
		return
	}
	if tenantPlan == nil {
		respondError(w, http.StatusNotFound, "tenant not found")
		return
	}

	respondJSON(w, http.StatusOK, tenantPlan)
}

// HandleStripeWebhook processes incoming Stripe webhook events
func (h *Handler) HandleStripeWebhook(w http.ResponseWriter, r *http.Request) {
	body, err := io.ReadAll(io.LimitReader(r.Body, 65536))
	if err != nil {
		respondError(w, http.StatusBadRequest, "failed to read body")
		return
	}

	// In production, verify webhook signature using STRIPE_WEBHOOK_SECRET
	// sig := r.Header.Get("Stripe-Signature")

	var event struct {
		Type string          `json:"type"`
		Data json.RawMessage `json:"data"`
	}

	if err := json.Unmarshal(body, &event); err != nil {
		respondError(w, http.StatusBadRequest, "invalid webhook payload")
		return
	}

	h.Logger.Info("Stripe webhook received", zap.String("type", event.Type))

	var eventData struct {
		Object json.RawMessage `json:"object"`
	}
	if err := json.Unmarshal(event.Data, &eventData); err != nil {
		respondError(w, http.StatusBadRequest, "invalid event data")
		return
	}

	switch event.Type {
	case "checkout.session.completed":
		h.handleCheckoutCompleted(r, eventData.Object)

	case "customer.subscription.updated":
		h.handleSubscriptionUpdated(r, eventData.Object)

	case "customer.subscription.deleted":
		h.handleSubscriptionDeleted(r, eventData.Object)

	case "invoice.payment_failed":
		h.handlePaymentFailed(r, eventData.Object)

	case "invoice.paid":
		h.handleInvoicePaid(r, eventData.Object)
	}

	// Always return 200 to Stripe
	respondJSON(w, http.StatusOK, map[string]string{"status": "received"})
}

func (h *Handler) handleCheckoutCompleted(r *http.Request, data json.RawMessage) {
	var session struct {
		CustomerID     string `json:"customer"`
		SubscriptionID string `json:"subscription"`
		Metadata       struct {
			TenantID string `json:"tenant_id"`
			PlanName string `json:"plan_name"`
		} `json:"metadata"`
	}

	if err := json.Unmarshal(data, &session); err != nil {
		h.Logger.Error("Failed to parse checkout session", zap.Error(err))
		return
	}

	ctx := r.Context()
	tenantID := session.Metadata.TenantID

	// Get the plan
	plan, err := h.DB.Plans.GetByName(ctx, session.Metadata.PlanName)
	if err != nil || plan == nil {
		h.Logger.Error("Failed to find plan for checkout", zap.String("plan", session.Metadata.PlanName))
		return
	}

	// Update tenant
	if err := h.DB.Plans.SetStripeCustomer(ctx, tenantID, session.CustomerID); err != nil {
		h.Logger.Error("Failed to set Stripe customer", zap.Error(err))
	}
	if err := h.DB.Plans.SetStripeSubscription(ctx, tenantID, session.SubscriptionID); err != nil {
		h.Logger.Error("Failed to set Stripe subscription", zap.Error(err))
	}
	if err := h.DB.Plans.UpdateTenantPlan(ctx, tenantID, plan.ID, "active"); err != nil {
		h.Logger.Error("Failed to update tenant plan", zap.Error(err))
	}

	h.Logger.Info("Checkout completed",
		zap.String("tenant_id", tenantID),
		zap.String("plan", session.Metadata.PlanName),
	)
}

func (h *Handler) handleSubscriptionUpdated(r *http.Request, data json.RawMessage) {
	var sub struct {
		ID       string `json:"id"`
		Status   string `json:"status"`
		Customer string `json:"customer"`
		Items    struct {
			Data []struct {
				Price struct {
					ID string `json:"id"`
				} `json:"price"`
			} `json:"data"`
		} `json:"items"`
	}

	if err := json.Unmarshal(data, &sub); err != nil {
		h.Logger.Error("Failed to parse subscription update", zap.Error(err))
		return
	}

	h.Logger.Info("Subscription updated",
		zap.String("subscription_id", sub.ID),
		zap.String("status", sub.Status),
	)
}

func (h *Handler) handleSubscriptionDeleted(r *http.Request, data json.RawMessage) {
	var sub struct {
		ID       string `json:"id"`
		Customer string `json:"customer"`
	}

	if err := json.Unmarshal(data, &sub); err != nil {
		h.Logger.Error("Failed to parse subscription deletion", zap.Error(err))
		return
	}

	// Find tenant by subscription ID and suspend
	h.Logger.Info("Subscription deleted",
		zap.String("subscription_id", sub.ID),
	)
}

func (h *Handler) handlePaymentFailed(r *http.Request, data json.RawMessage) {
	var invoice struct {
		Customer     string `json:"customer"`
		Subscription string `json:"subscription"`
	}

	if err := json.Unmarshal(data, &invoice); err != nil {
		h.Logger.Error("Failed to parse payment failure", zap.Error(err))
		return
	}

	h.Logger.Warn("Payment failed",
		zap.String("customer", invoice.Customer),
		zap.String("subscription", invoice.Subscription),
	)
}

func (h *Handler) handleInvoicePaid(r *http.Request, data json.RawMessage) {
	var invoice struct {
		Customer     string `json:"customer"`
		Subscription string `json:"subscription"`
	}

	if err := json.Unmarshal(data, &invoice); err != nil {
		h.Logger.Error("Failed to parse invoice paid", zap.Error(err))
		return
	}

	h.Logger.Info("Invoice paid",
		zap.String("customer", invoice.Customer),
		zap.String("subscription", invoice.Subscription),
	)
}
