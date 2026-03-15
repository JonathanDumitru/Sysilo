package orchestration

import (
	"encoding/json"
	"net/http"
	"strings"
)

// Handlers exposes OrchestrationHub functionality over HTTP.
type Handlers struct {
	hub *OrchestrationHub
}

// NewHandlers creates a new Handlers instance backed by the given hub.
func NewHandlers(hub *OrchestrationHub) *Handlers {
	return &Handlers{hub: hub}
}

// RegisterRoutes attaches the orchestration HTTP handlers to the given mux.
func (h *Handlers) RegisterRoutes(mux *http.ServeMux) {
	mux.HandleFunc("/orchestration/agents", h.handleAgents)
	mux.HandleFunc("/orchestration/agents/", h.handleAgentByID)
	mux.HandleFunc("/orchestration/agents/discover", h.handleDiscoverAgents)
	mux.HandleFunc("/orchestration/agents/permissions/check", h.handleCheckPermission)
	mux.HandleFunc("/orchestration/agents/metrics", h.handleRecordMetrics)
	mux.HandleFunc("/orchestration/messages", h.handleMessages)
	mux.HandleFunc("/orchestration/portfolio", h.handlePortfolio)
	mux.HandleFunc("/orchestration/escalation/rules", h.handleEscalationRules)
	mux.HandleFunc("/orchestration/escalation/check", h.handleEscalationCheck)
}

// handleAgents handles listing and registering agents.
//
//	GET  /orchestration/agents?tenant_id=...&type=...
//	POST /orchestration/agents
func (h *Handlers) handleAgents(w http.ResponseWriter, r *http.Request) {
	switch r.Method {
	case http.MethodGet:
		tenantID := r.URL.Query().Get("tenant_id")
		if tenantID == "" {
			writeError(w, http.StatusBadRequest, "tenant_id query parameter is required")
			return
		}

		agentType := r.URL.Query().Get("type")
		var agents []*EnterpriseAgent
		if agentType != "" {
			agents = h.hub.ListAgentsByType(tenantID, agentType)
		} else {
			agents = h.hub.ListAgentsByTenant(tenantID)
		}

		if agents == nil {
			agents = []*EnterpriseAgent{}
		}
		writeJSON(w, http.StatusOK, agents)

	case http.MethodPost:
		var agent EnterpriseAgent
		if err := json.NewDecoder(r.Body).Decode(&agent); err != nil {
			writeError(w, http.StatusBadRequest, "invalid request body: "+err.Error())
			return
		}
		if agent.ID == "" {
			writeError(w, http.StatusBadRequest, "agent id is required")
			return
		}
		if agent.TenantID == "" {
			writeError(w, http.StatusBadRequest, "tenant_id is required")
			return
		}
		h.hub.RegisterAgent(&agent)
		writeJSON(w, http.StatusCreated, agent)

	default:
		writeError(w, http.StatusMethodNotAllowed, "method not allowed")
	}
}

// handleAgentByID handles operations on a single agent.
//
//	GET    /orchestration/agents/{id}
//	DELETE /orchestration/agents/{id}
func (h *Handlers) handleAgentByID(w http.ResponseWriter, r *http.Request) {
	// Extract agent ID from the path: /orchestration/agents/{id}
	path := strings.TrimPrefix(r.URL.Path, "/orchestration/agents/")
	agentID := strings.TrimRight(path, "/")

	if agentID == "" || agentID == "discover" || agentID == "permissions" || agentID == "metrics" {
		// These are handled by other routes; reject here.
		writeError(w, http.StatusNotFound, "not found")
		return
	}

	switch r.Method {
	case http.MethodGet:
		agent, ok := h.hub.GetAgent(agentID)
		if !ok {
			writeError(w, http.StatusNotFound, "agent not found")
			return
		}
		writeJSON(w, http.StatusOK, agent)

	case http.MethodDelete:
		h.hub.UnregisterAgent(agentID)
		w.WriteHeader(http.StatusNoContent)

	default:
		writeError(w, http.StatusMethodNotAllowed, "method not allowed")
	}
}

// handleDiscoverAgents finds agents with a required capability.
//
//	GET /orchestration/agents/discover?tenant_id=...&capability=...
func (h *Handlers) handleDiscoverAgents(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodGet {
		writeError(w, http.StatusMethodNotAllowed, "method not allowed")
		return
	}

	tenantID := r.URL.Query().Get("tenant_id")
	capability := r.URL.Query().Get("capability")
	if tenantID == "" || capability == "" {
		writeError(w, http.StatusBadRequest, "tenant_id and capability query parameters are required")
		return
	}

	agents := h.hub.DiscoverAgents(tenantID, capability)
	if agents == nil {
		agents = []*EnterpriseAgent{}
	}
	writeJSON(w, http.StatusOK, agents)
}

// checkPermissionRequest is the body for permission check requests.
type checkPermissionRequest struct {
	AgentID  string `json:"agent_id"`
	Resource string `json:"resource"`
	Action   string `json:"action"`
}

// checkPermissionResponse is returned from permission check requests.
type checkPermissionResponse struct {
	Allowed bool `json:"allowed"`
}

// handleCheckPermission checks whether an agent has a specific permission.
//
//	POST /orchestration/agents/permissions/check
func (h *Handlers) handleCheckPermission(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		writeError(w, http.StatusMethodNotAllowed, "method not allowed")
		return
	}

	var req checkPermissionRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		writeError(w, http.StatusBadRequest, "invalid request body: "+err.Error())
		return
	}
	if req.AgentID == "" || req.Resource == "" || req.Action == "" {
		writeError(w, http.StatusBadRequest, "agent_id, resource, and action are required")
		return
	}

	allowed := h.hub.CheckPermission(req.AgentID, req.Resource, req.Action)
	writeJSON(w, http.StatusOK, checkPermissionResponse{Allowed: allowed})
}

// recordMetricsRequest is the body for recording agent metrics.
type recordMetricsRequest struct {
	AgentID   string  `json:"agent_id"`
	LatencyMs float64 `json:"latency_ms"`
	Success   bool    `json:"success"`
}

// handleRecordMetrics records a metrics data point for an agent.
//
//	POST /orchestration/agents/metrics
func (h *Handlers) handleRecordMetrics(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		writeError(w, http.StatusMethodNotAllowed, "method not allowed")
		return
	}

	var req recordMetricsRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		writeError(w, http.StatusBadRequest, "invalid request body: "+err.Error())
		return
	}
	if req.AgentID == "" {
		writeError(w, http.StatusBadRequest, "agent_id is required")
		return
	}

	h.hub.RecordMetrics(req.AgentID, req.LatencyMs, req.Success)
	w.WriteHeader(http.StatusNoContent)
}

// handleMessages sends agent-to-agent messages.
//
//	POST /orchestration/messages
func (h *Handlers) handleMessages(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		writeError(w, http.StatusMethodNotAllowed, "method not allowed")
		return
	}

	var msg A2AMessage
	if err := json.NewDecoder(r.Body).Decode(&msg); err != nil {
		writeError(w, http.StatusBadRequest, "invalid request body: "+err.Error())
		return
	}

	if err := h.hub.SendMessage(&msg); err != nil {
		writeError(w, http.StatusUnprocessableEntity, err.Error())
		return
	}
	writeJSON(w, http.StatusAccepted, msg)
}

// handlePortfolio returns the portfolio view for a tenant.
//
//	GET /orchestration/portfolio?tenant_id=...
func (h *Handlers) handlePortfolio(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodGet {
		writeError(w, http.StatusMethodNotAllowed, "method not allowed")
		return
	}

	tenantID := r.URL.Query().Get("tenant_id")
	if tenantID == "" {
		writeError(w, http.StatusBadRequest, "tenant_id query parameter is required")
		return
	}

	view := h.hub.GetPortfolioView(tenantID)
	writeJSON(w, http.StatusOK, view)
}

// handleEscalationRules manages escalation rules.
//
//	POST /orchestration/escalation/rules
func (h *Handlers) handleEscalationRules(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		writeError(w, http.StatusMethodNotAllowed, "method not allowed")
		return
	}

	var rule EscalationRule
	if err := json.NewDecoder(r.Body).Decode(&rule); err != nil {
		writeError(w, http.StatusBadRequest, "invalid request body: "+err.Error())
		return
	}
	if rule.AgentType == "" || rule.ActionType == "" || rule.RiskLevel == "" {
		writeError(w, http.StatusBadRequest, "agent_type, action_type, and risk_level are required")
		return
	}

	h.hub.AddEscalationRule(&rule)
	writeJSON(w, http.StatusCreated, rule)
}

// escalationCheckRequest is the body for escalation check requests.
type escalationCheckRequest struct {
	AgentType  string `json:"agent_type"`
	ActionType string `json:"action_type"`
	RiskLevel  string `json:"risk_level"`
}

// escalationCheckResponse is returned from escalation check requests.
type escalationCheckResponse struct {
	ShouldEscalate bool            `json:"should_escalate"`
	Rule           *EscalationRule `json:"rule,omitempty"`
}

// handleEscalationCheck checks whether an action should be escalated.
//
//	POST /orchestration/escalation/check
func (h *Handlers) handleEscalationCheck(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		writeError(w, http.StatusMethodNotAllowed, "method not allowed")
		return
	}

	var req escalationCheckRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		writeError(w, http.StatusBadRequest, "invalid request body: "+err.Error())
		return
	}
	if req.AgentType == "" || req.ActionType == "" || req.RiskLevel == "" {
		writeError(w, http.StatusBadRequest, "agent_type, action_type, and risk_level are required")
		return
	}

	rule, shouldEscalate := h.hub.ShouldEscalate(req.AgentType, req.ActionType, req.RiskLevel)
	writeJSON(w, http.StatusOK, escalationCheckResponse{
		ShouldEscalate: shouldEscalate,
		Rule:           rule,
	})
}

// --- helpers ---

// errorResponse is the standard error envelope.
type errorResponse struct {
	Error string `json:"error"`
}

func writeJSON(w http.ResponseWriter, status int, v interface{}) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	json.NewEncoder(w).Encode(v)
}

func writeError(w http.ResponseWriter, status int, msg string) {
	writeJSON(w, status, errorResponse{Error: msg})
}
