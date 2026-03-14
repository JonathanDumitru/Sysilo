package eventmesh

import (
	"encoding/json"
	"net/http"
)

// Handlers provides HTTP handlers for the event mesh
type Handlers struct {
	mesh *EventMesh
}

// NewHandlers creates a new Handlers instance
func NewHandlers(mesh *EventMesh) *Handlers {
	return &Handlers{mesh: mesh}
}

// HandlePublish handles POST /api/v1/events
func (h *Handlers) HandlePublish(w http.ResponseWriter, r *http.Request) {
	var event Event
	if err := json.NewDecoder(r.Body).Decode(&event); err != nil {
		writeJSON(w, http.StatusBadRequest, map[string]string{"error": "invalid request body"})
		return
	}

	if err := h.mesh.Publish(&event); err != nil {
		writeJSON(w, http.StatusInternalServerError, map[string]string{"error": err.Error()})
		return
	}

	writeJSON(w, http.StatusAccepted, map[string]interface{}{
		"event_id": event.ID,
		"status":   "accepted",
	})
}

// HandleSubscribe handles POST /api/v1/events/subscriptions
func (h *Handlers) HandleSubscribe(w http.ResponseWriter, r *http.Request) {
	var sub Subscription
	if err := json.NewDecoder(r.Body).Decode(&sub); err != nil {
		writeJSON(w, http.StatusBadRequest, map[string]string{"error": "invalid request body"})
		return
	}

	h.mesh.Subscribe(&sub)
	writeJSON(w, http.StatusCreated, sub)
}

// HandleUnsubscribe handles DELETE /api/v1/events/subscriptions/{id}
func (h *Handlers) HandleUnsubscribe(w http.ResponseWriter, r *http.Request) {
	id := r.PathValue("id")
	if id == "" {
		writeJSON(w, http.StatusBadRequest, map[string]string{"error": "subscription id required"})
		return
	}

	h.mesh.Unsubscribe(id)
	writeJSON(w, http.StatusOK, map[string]string{"status": "unsubscribed"})
}

// HandleListSubscriptions handles GET /api/v1/events/subscriptions
func (h *Handlers) HandleListSubscriptions(w http.ResponseWriter, r *http.Request) {
	tenantID := r.URL.Query().Get("tenant_id")
	if tenantID == "" {
		writeJSON(w, http.StatusBadRequest, map[string]string{"error": "tenant_id required"})
		return
	}

	subs := h.mesh.ListSubscriptions(tenantID)
	writeJSON(w, http.StatusOK, map[string]interface{}{
		"subscriptions": subs,
	})
}

// HandleAddRoute handles POST /api/v1/events/routes
func (h *Handlers) HandleAddRoute(w http.ResponseWriter, r *http.Request) {
	var route TopicRoute
	if err := json.NewDecoder(r.Body).Decode(&route); err != nil {
		writeJSON(w, http.StatusBadRequest, map[string]string{"error": "invalid request body"})
		return
	}

	h.mesh.AddRoute(&route)
	writeJSON(w, http.StatusCreated, route)
}

// HandleListRoutes handles GET /api/v1/events/routes
func (h *Handlers) HandleListRoutes(w http.ResponseWriter, r *http.Request) {
	tenantID := r.URL.Query().Get("tenant_id")
	if tenantID == "" {
		writeJSON(w, http.StatusBadRequest, map[string]string{"error": "tenant_id required"})
		return
	}

	routes := h.mesh.ListRoutes(tenantID)
	writeJSON(w, http.StatusOK, map[string]interface{}{
		"routes": routes,
	})
}

// HandleRegisterNode handles POST /api/v1/control-plane/nodes
func (h *Handlers) HandleRegisterNode(w http.ResponseWriter, r *http.Request) {
	var node ControlPlaneNode
	if err := json.NewDecoder(r.Body).Decode(&node); err != nil {
		writeJSON(w, http.StatusBadRequest, map[string]string{"error": "invalid request body"})
		return
	}

	h.mesh.RegisterNode(&node)
	writeJSON(w, http.StatusCreated, node)
}

// HandleListNodes handles GET /api/v1/control-plane/nodes
func (h *Handlers) HandleListNodes(w http.ResponseWriter, r *http.Request) {
	tenantID := r.URL.Query().Get("tenant_id")
	if tenantID == "" {
		writeJSON(w, http.StatusBadRequest, map[string]string{"error": "tenant_id required"})
		return
	}

	nodes := h.mesh.ListNodes(tenantID)
	writeJSON(w, http.StatusOK, map[string]interface{}{
		"nodes": nodes,
	})
}

// HandleNodeHeartbeat handles POST /api/v1/control-plane/nodes/{id}/heartbeat
func (h *Handlers) HandleNodeHeartbeat(w http.ResponseWriter, r *http.Request) {
	id := r.PathValue("id")
	if id == "" {
		writeJSON(w, http.StatusBadRequest, map[string]string{"error": "node id required"})
		return
	}

	h.mesh.UpdateNodeHeartbeat(id)
	writeJSON(w, http.StatusOK, map[string]string{"status": "ok"})
}

// HandleStats handles GET /api/v1/events/stats
func (h *Handlers) HandleStats(w http.ResponseWriter, r *http.Request) {
	stats := h.mesh.Stats()
	writeJSON(w, http.StatusOK, stats)
}

func writeJSON(w http.ResponseWriter, status int, v interface{}) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	json.NewEncoder(w).Encode(v)
}
