package monetization

import (
	"encoding/json"
	"net/http"
	"time"
)

// Handlers provides HTTP handlers for API monetization
type Handlers struct {
	engine *MonetizationEngine
}

// NewHandlers creates a new Handlers instance
func NewHandlers(engine *MonetizationEngine) *Handlers {
	return &Handlers{engine: engine}
}

// HandleRegisterProduct handles POST /api/v1/api-products
func (h *Handlers) HandleRegisterProduct(w http.ResponseWriter, r *http.Request) {
	var product APIProduct
	if err := json.NewDecoder(r.Body).Decode(&product); err != nil {
		writeJSON(w, http.StatusBadRequest, map[string]string{"error": "invalid request body"})
		return
	}

	h.engine.RegisterProduct(&product)
	writeJSON(w, http.StatusCreated, product)
}

// HandlePublishProduct handles POST /api/v1/api-products/{id}/publish
func (h *Handlers) HandlePublishProduct(w http.ResponseWriter, r *http.Request) {
	id := r.PathValue("id")
	if id == "" {
		writeJSON(w, http.StatusBadRequest, map[string]string{"error": "product id required"})
		return
	}

	product, ok := h.engine.PublishProduct(id)
	if !ok {
		writeJSON(w, http.StatusNotFound, map[string]string{"error": "product not found"})
		return
	}

	writeJSON(w, http.StatusOK, product)
}

// HandleGetProduct handles GET /api/v1/api-products/{id}
func (h *Handlers) HandleGetProduct(w http.ResponseWriter, r *http.Request) {
	id := r.PathValue("id")
	if id == "" {
		writeJSON(w, http.StatusBadRequest, map[string]string{"error": "product id required"})
		return
	}

	product, ok := h.engine.GetProduct(id)
	if !ok {
		writeJSON(w, http.StatusNotFound, map[string]string{"error": "product not found"})
		return
	}

	writeJSON(w, http.StatusOK, product)
}

// HandleListProducts handles GET /api/v1/api-products
func (h *Handlers) HandleListProducts(w http.ResponseWriter, r *http.Request) {
	tenantID := r.URL.Query().Get("tenant_id")
	if tenantID == "" {
		writeJSON(w, http.StatusBadRequest, map[string]string{"error": "tenant_id required"})
		return
	}

	products := h.engine.ListProducts(tenantID)
	writeJSON(w, http.StatusOK, map[string]interface{}{
		"products": products,
	})
}

// HandleCreateSubscription handles POST /api/v1/api-subscriptions
func (h *Handlers) HandleCreateSubscription(w http.ResponseWriter, r *http.Request) {
	var sub APISubscription
	if err := json.NewDecoder(r.Body).Decode(&sub); err != nil {
		writeJSON(w, http.StatusBadRequest, map[string]string{"error": "invalid request body"})
		return
	}

	if err := h.engine.CreateSubscription(&sub); err != nil {
		writeJSON(w, http.StatusBadRequest, map[string]string{"error": err.Error()})
		return
	}

	writeJSON(w, http.StatusCreated, sub)
}

// HandleListSubscriptions handles GET /api/v1/api-subscriptions
func (h *Handlers) HandleListSubscriptions(w http.ResponseWriter, r *http.Request) {
	productID := r.URL.Query().Get("product_id")
	if productID == "" {
		writeJSON(w, http.StatusBadRequest, map[string]string{"error": "product_id required"})
		return
	}

	subs := h.engine.ListSubscriptions(productID)
	writeJSON(w, http.StatusOK, map[string]interface{}{
		"subscriptions": subs,
	})
}

// HandleRecordUsage handles POST /api/v1/api-usage
func (h *Handlers) HandleRecordUsage(w http.ResponseWriter, r *http.Request) {
	var record UsageRecord
	if err := json.NewDecoder(r.Body).Decode(&record); err != nil {
		writeJSON(w, http.StatusBadRequest, map[string]string{"error": "invalid request body"})
		return
	}

	if err := h.engine.RecordUsage(&record); err != nil {
		writeJSON(w, http.StatusBadRequest, map[string]string{"error": err.Error()})
		return
	}

	writeJSON(w, http.StatusCreated, map[string]string{"status": "recorded"})
}

// HandleGetRevenueReport handles GET /api/v1/api-revenue
func (h *Handlers) HandleGetRevenueReport(w http.ResponseWriter, r *http.Request) {
	tenantID := r.URL.Query().Get("tenant_id")
	if tenantID == "" {
		writeJSON(w, http.StatusBadRequest, map[string]string{"error": "tenant_id required"})
		return
	}

	// Default to last 30 days
	end := time.Now()
	start := end.AddDate(0, -1, 0)

	if startStr := r.URL.Query().Get("start"); startStr != "" {
		if t, err := time.Parse(time.RFC3339, startStr); err == nil {
			start = t
		}
	}
	if endStr := r.URL.Query().Get("end"); endStr != "" {
		if t, err := time.Parse(time.RFC3339, endStr); err == nil {
			end = t
		}
	}

	report := h.engine.GetRevenueReport(tenantID, start, end)
	writeJSON(w, http.StatusOK, report)
}

// HandleGetProductAnalytics handles GET /api/v1/api-products/{id}/analytics
func (h *Handlers) HandleGetProductAnalytics(w http.ResponseWriter, r *http.Request) {
	id := r.PathValue("id")
	if id == "" {
		writeJSON(w, http.StatusBadRequest, map[string]string{"error": "product id required"})
		return
	}

	analytics := h.engine.GetProductAnalytics(id)
	writeJSON(w, http.StatusOK, analytics)
}

func writeJSON(w http.ResponseWriter, status int, v interface{}) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	json.NewEncoder(w).Encode(v)
}
