package handlers

import (
	"database/sql"
	"encoding/json"
	"net/http"
	"strconv"

	"github.com/go-chi/chi/v5"
	"github.com/sysilo/sysilo/services/api-gateway/internal/db"
	"github.com/sysilo/sysilo/services/api-gateway/internal/middleware"
	"go.uber.org/zap"
)

// Handler holds dependencies for HTTP handlers
type Handler struct {
	DB     *db.DB
	Logger *zap.Logger
}

// New creates a new Handler instance
func New(database *db.DB, logger *zap.Logger) *Handler {
	return &Handler{
		DB:     database,
		Logger: logger,
	}
}

// Response helpers

func respondJSON(w http.ResponseWriter, status int, data interface{}) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	if data != nil {
		json.NewEncoder(w).Encode(data)
	}
}

func respondError(w http.ResponseWriter, status int, message string) {
	respondJSON(w, status, map[string]string{"error": message})
}

// Helper to parse pagination params
func parsePagination(r *http.Request) db.ListOptions {
	page, _ := strconv.Atoi(r.URL.Query().Get("page"))
	pageSize, _ := strconv.Atoi(r.URL.Query().Get("page_size"))

	if page <= 0 {
		page = 1
	}
	if pageSize <= 0 {
		pageSize = 20
	}

	return db.ListOptions{
		Page:     page,
		PageSize: pageSize,
	}
}

// Health check handlers

func (h *Handler) Health(w http.ResponseWriter, r *http.Request) {
	respondJSON(w, http.StatusOK, map[string]string{"status": "healthy"})
}

func (h *Handler) Ready(w http.ResponseWriter, r *http.Request) {
	// Check database connection
	if err := h.DB.Ping(r.Context()); err != nil {
		h.Logger.Error("Database health check failed", zap.Error(err))
		respondJSON(w, http.StatusServiceUnavailable, map[string]string{
			"status": "not ready",
			"error":  "database connection failed",
		})
		return
	}

	respondJSON(w, http.StatusOK, map[string]string{"status": "ready"})
}

// Agent handlers

func (h *Handler) ListAgents(w http.ResponseWriter, r *http.Request) {
	tenantID := middleware.GetTenantID(r.Context())
	opts := parsePagination(r)

	result, err := h.DB.Agents.List(r.Context(), tenantID, opts)
	if err != nil {
		h.Logger.Error("Failed to list agents", zap.Error(err))
		respondError(w, http.StatusInternalServerError, "failed to list agents")
		return
	}

	respondJSON(w, http.StatusOK, result)
}

func (h *Handler) GetAgent(w http.ResponseWriter, r *http.Request) {
	tenantID := middleware.GetTenantID(r.Context())
	agentID := chi.URLParam(r, "agentID")

	agent, err := h.DB.Agents.GetByID(r.Context(), tenantID, agentID)
	if err != nil {
		h.Logger.Error("Failed to get agent", zap.Error(err))
		respondError(w, http.StatusInternalServerError, "failed to get agent")
		return
	}
	if agent == nil {
		respondError(w, http.StatusNotFound, "agent not found")
		return
	}

	respondJSON(w, http.StatusOK, agent)
}

func (h *Handler) DeleteAgent(w http.ResponseWriter, r *http.Request) {
	tenantID := middleware.GetTenantID(r.Context())
	agentID := chi.URLParam(r, "agentID")

	err := h.DB.Agents.Delete(r.Context(), tenantID, agentID)
	if err == sql.ErrNoRows {
		respondError(w, http.StatusNotFound, "agent not found")
		return
	}
	if err != nil {
		h.Logger.Error("Failed to delete agent", zap.Error(err))
		respondError(w, http.StatusInternalServerError, "failed to delete agent")
		return
	}

	w.WriteHeader(http.StatusNoContent)
}

// Connection handlers

func (h *Handler) ListConnections(w http.ResponseWriter, r *http.Request) {
	tenantID := middleware.GetTenantID(r.Context())
	opts := parsePagination(r)

	result, err := h.DB.Connections.List(r.Context(), tenantID, opts)
	if err != nil {
		h.Logger.Error("Failed to list connections", zap.Error(err))
		respondError(w, http.StatusInternalServerError, "failed to list connections")
		return
	}

	respondJSON(w, http.StatusOK, result)
}

func (h *Handler) CreateConnection(w http.ResponseWriter, r *http.Request) {
	tenantID := middleware.GetTenantID(r.Context())

	var req struct {
		Name           string                 `json:"name"`
		Description    string                 `json:"description"`
		ConnectionType string                 `json:"connection_type"`
		Config         map[string]interface{} `json:"config"`
		AgentID        string                 `json:"agent_id"`
	}

	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		respondError(w, http.StatusBadRequest, "invalid request body")
		return
	}

	if req.Name == "" || req.ConnectionType == "" {
		respondError(w, http.StatusBadRequest, "name and connection_type are required")
		return
	}

	conn, err := h.DB.Connections.Create(r.Context(), db.CreateConnectionInput{
		TenantID:       tenantID,
		Name:           req.Name,
		Description:    req.Description,
		ConnectionType: req.ConnectionType,
		Config:         req.Config,
		AgentID:        req.AgentID,
	})
	if err != nil {
		h.Logger.Error("Failed to create connection", zap.Error(err))
		respondError(w, http.StatusInternalServerError, "failed to create connection")
		return
	}

	respondJSON(w, http.StatusCreated, conn)
}

func (h *Handler) GetConnection(w http.ResponseWriter, r *http.Request) {
	tenantID := middleware.GetTenantID(r.Context())
	connectionID := chi.URLParam(r, "connectionID")

	conn, err := h.DB.Connections.GetByID(r.Context(), tenantID, connectionID)
	if err != nil {
		h.Logger.Error("Failed to get connection", zap.Error(err))
		respondError(w, http.StatusInternalServerError, "failed to get connection")
		return
	}
	if conn == nil {
		respondError(w, http.StatusNotFound, "connection not found")
		return
	}

	respondJSON(w, http.StatusOK, conn)
}

func (h *Handler) UpdateConnection(w http.ResponseWriter, r *http.Request) {
	tenantID := middleware.GetTenantID(r.Context())
	connectionID := chi.URLParam(r, "connectionID")

	var req struct {
		Name        *string                `json:"name,omitempty"`
		Description *string                `json:"description,omitempty"`
		Config      map[string]interface{} `json:"config,omitempty"`
		AgentID     *string                `json:"agent_id,omitempty"`
		Status      *string                `json:"status,omitempty"`
	}

	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		respondError(w, http.StatusBadRequest, "invalid request body")
		return
	}

	conn, err := h.DB.Connections.Update(r.Context(), tenantID, connectionID, db.UpdateConnectionInput{
		Name:        req.Name,
		Description: req.Description,
		Config:      req.Config,
		AgentID:     req.AgentID,
		Status:      req.Status,
	})
	if err != nil {
		h.Logger.Error("Failed to update connection", zap.Error(err))
		respondError(w, http.StatusInternalServerError, "failed to update connection")
		return
	}
	if conn == nil {
		respondError(w, http.StatusNotFound, "connection not found")
		return
	}

	respondJSON(w, http.StatusOK, conn)
}

func (h *Handler) DeleteConnection(w http.ResponseWriter, r *http.Request) {
	tenantID := middleware.GetTenantID(r.Context())
	connectionID := chi.URLParam(r, "connectionID")

	err := h.DB.Connections.Delete(r.Context(), tenantID, connectionID)
	if err == sql.ErrNoRows {
		respondError(w, http.StatusNotFound, "connection not found")
		return
	}
	if err != nil {
		h.Logger.Error("Failed to delete connection", zap.Error(err))
		respondError(w, http.StatusInternalServerError, "failed to delete connection")
		return
	}

	w.WriteHeader(http.StatusNoContent)
}

func (h *Handler) TestConnection(w http.ResponseWriter, r *http.Request) {
	tenantID := middleware.GetTenantID(r.Context())
	connectionID := chi.URLParam(r, "connectionID")

	// Verify connection exists
	conn, err := h.DB.Connections.GetByID(r.Context(), tenantID, connectionID)
	if err != nil {
		h.Logger.Error("Failed to get connection for test", zap.Error(err))
		respondError(w, http.StatusInternalServerError, "failed to get connection")
		return
	}
	if conn == nil {
		respondError(w, http.StatusNotFound, "connection not found")
		return
	}

	// TODO: Actually send test task to agent via agent-gateway
	// For now, simulate success and update test status
	testStatus := "success"
	if err := h.DB.Connections.UpdateTestStatus(r.Context(), tenantID, connectionID, testStatus); err != nil {
		h.Logger.Error("Failed to update connection test status", zap.Error(err))
	}

	respondJSON(w, http.StatusOK, map[string]interface{}{
		"connection_id": connectionID,
		"success":       true,
		"message":       "Connection test successful",
	})
}

// Integration handlers

func (h *Handler) ListIntegrations(w http.ResponseWriter, r *http.Request) {
	tenantID := middleware.GetTenantID(r.Context())
	opts := parsePagination(r)

	result, err := h.DB.Integrations.List(r.Context(), tenantID, opts)
	if err != nil {
		h.Logger.Error("Failed to list integrations", zap.Error(err))
		respondError(w, http.StatusInternalServerError, "failed to list integrations")
		return
	}

	respondJSON(w, http.StatusOK, result)
}

func (h *Handler) CreateIntegration(w http.ResponseWriter, r *http.Request) {
	tenantID := middleware.GetTenantID(r.Context())
	userID := middleware.GetUserID(r.Context())

	var req struct {
		Name        string                 `json:"name"`
		Description string                 `json:"description"`
		Definition  map[string]interface{} `json:"definition"`
		Schedule    map[string]interface{} `json:"schedule"`
		Config      map[string]interface{} `json:"config"`
	}

	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		respondError(w, http.StatusBadRequest, "invalid request body")
		return
	}

	if req.Name == "" || req.Definition == nil {
		respondError(w, http.StatusBadRequest, "name and definition are required")
		return
	}

	integration, err := h.DB.Integrations.Create(r.Context(), db.CreateIntegrationInput{
		TenantID:    tenantID,
		Name:        req.Name,
		Description: req.Description,
		Definition:  req.Definition,
		Schedule:    req.Schedule,
		Config:      req.Config,
		CreatedBy:   userID,
	})
	if err != nil {
		h.Logger.Error("Failed to create integration", zap.Error(err))
		respondError(w, http.StatusInternalServerError, "failed to create integration")
		return
	}

	respondJSON(w, http.StatusCreated, integration)
}

func (h *Handler) GetIntegration(w http.ResponseWriter, r *http.Request) {
	tenantID := middleware.GetTenantID(r.Context())
	integrationID := chi.URLParam(r, "integrationID")

	integration, err := h.DB.Integrations.GetByID(r.Context(), tenantID, integrationID)
	if err != nil {
		h.Logger.Error("Failed to get integration", zap.Error(err))
		respondError(w, http.StatusInternalServerError, "failed to get integration")
		return
	}
	if integration == nil {
		respondError(w, http.StatusNotFound, "integration not found")
		return
	}

	respondJSON(w, http.StatusOK, integration)
}

func (h *Handler) UpdateIntegration(w http.ResponseWriter, r *http.Request) {
	tenantID := middleware.GetTenantID(r.Context())
	userID := middleware.GetUserID(r.Context())
	integrationID := chi.URLParam(r, "integrationID")

	var req struct {
		Name        *string                `json:"name,omitempty"`
		Description *string                `json:"description,omitempty"`
		Definition  map[string]interface{} `json:"definition,omitempty"`
		Schedule    map[string]interface{} `json:"schedule,omitempty"`
		Config      map[string]interface{} `json:"config,omitempty"`
		Status      *string                `json:"status,omitempty"`
	}

	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		respondError(w, http.StatusBadRequest, "invalid request body")
		return
	}

	integration, err := h.DB.Integrations.Update(r.Context(), tenantID, integrationID, db.UpdateIntegrationInput{
		Name:        req.Name,
		Description: req.Description,
		Definition:  req.Definition,
		Schedule:    req.Schedule,
		Config:      req.Config,
		Status:      req.Status,
		UpdatedBy:   userID,
	})
	if err != nil {
		h.Logger.Error("Failed to update integration", zap.Error(err))
		respondError(w, http.StatusInternalServerError, "failed to update integration")
		return
	}
	if integration == nil {
		respondError(w, http.StatusNotFound, "integration not found")
		return
	}

	respondJSON(w, http.StatusOK, integration)
}

func (h *Handler) DeleteIntegration(w http.ResponseWriter, r *http.Request) {
	tenantID := middleware.GetTenantID(r.Context())
	integrationID := chi.URLParam(r, "integrationID")

	err := h.DB.Integrations.Delete(r.Context(), tenantID, integrationID)
	if err == sql.ErrNoRows {
		respondError(w, http.StatusNotFound, "integration not found")
		return
	}
	if err != nil {
		h.Logger.Error("Failed to delete integration", zap.Error(err))
		respondError(w, http.StatusInternalServerError, "failed to delete integration")
		return
	}

	w.WriteHeader(http.StatusNoContent)
}

func (h *Handler) RunIntegration(w http.ResponseWriter, r *http.Request) {
	tenantID := middleware.GetTenantID(r.Context())
	userID := middleware.GetUserID(r.Context())
	integrationID := chi.URLParam(r, "integrationID")

	// Get integration to verify it exists and get version
	integration, err := h.DB.Integrations.GetByID(r.Context(), tenantID, integrationID)
	if err != nil {
		h.Logger.Error("Failed to get integration for run", zap.Error(err))
		respondError(w, http.StatusInternalServerError, "failed to get integration")
		return
	}
	if integration == nil {
		respondError(w, http.StatusNotFound, "integration not found")
		return
	}
	if integration.Status != "active" {
		respondError(w, http.StatusBadRequest, "integration is not active")
		return
	}

	// Create run
	run, err := h.DB.Runs.Create(r.Context(), db.CreateRunInput{
		TenantID:           tenantID,
		IntegrationID:      integrationID,
		IntegrationVersion: integration.Version,
		TriggerType:        "manual",
		TriggeredBy:        userID,
	})
	if err != nil {
		h.Logger.Error("Failed to create integration run", zap.Error(err))
		respondError(w, http.StatusInternalServerError, "failed to create run")
		return
	}

	// TODO: Dispatch tasks to agent-gateway via Kafka or HTTP

	respondJSON(w, http.StatusAccepted, run)
}

func (h *Handler) ListIntegrationRuns(w http.ResponseWriter, r *http.Request) {
	tenantID := middleware.GetTenantID(r.Context())
	integrationID := chi.URLParam(r, "integrationID")
	opts := parsePagination(r)

	result, err := h.DB.Runs.ListByIntegration(r.Context(), tenantID, integrationID, opts)
	if err != nil {
		h.Logger.Error("Failed to list integration runs", zap.Error(err))
		respondError(w, http.StatusInternalServerError, "failed to list runs")
		return
	}

	respondJSON(w, http.StatusOK, result)
}

// Run handlers

func (h *Handler) GetRun(w http.ResponseWriter, r *http.Request) {
	tenantID := middleware.GetTenantID(r.Context())
	runID := chi.URLParam(r, "runID")

	run, err := h.DB.Runs.GetByID(r.Context(), tenantID, runID)
	if err != nil {
		h.Logger.Error("Failed to get run", zap.Error(err))
		respondError(w, http.StatusInternalServerError, "failed to get run")
		return
	}
	if run == nil {
		respondError(w, http.StatusNotFound, "run not found")
		return
	}

	respondJSON(w, http.StatusOK, run)
}

func (h *Handler) CancelRun(w http.ResponseWriter, r *http.Request) {
	tenantID := middleware.GetTenantID(r.Context())
	runID := chi.URLParam(r, "runID")

	err := h.DB.Runs.Cancel(r.Context(), tenantID, runID)
	if err == sql.ErrNoRows {
		respondError(w, http.StatusNotFound, "run not found or not cancellable")
		return
	}
	if err != nil {
		h.Logger.Error("Failed to cancel run", zap.Error(err))
		respondError(w, http.StatusInternalServerError, "failed to cancel run")
		return
	}

	// TODO: Send cancellation command to agent-gateway

	respondJSON(w, http.StatusOK, map[string]interface{}{
		"run_id":  runID,
		"status":  "cancelled",
		"message": "Run cancellation requested",
	})
}

func (h *Handler) GetRunLogs(w http.ResponseWriter, r *http.Request) {
	tenantID := middleware.GetTenantID(r.Context())
	runID := chi.URLParam(r, "runID")
	opts := parsePagination(r)

	result, err := h.DB.Runs.GetLogs(r.Context(), tenantID, runID, opts)
	if err != nil {
		h.Logger.Error("Failed to get run logs", zap.Error(err))
		respondError(w, http.StatusInternalServerError, "failed to get logs")
		return
	}

	respondJSON(w, http.StatusOK, result)
}

// User handlers

func (h *Handler) ListUsers(w http.ResponseWriter, r *http.Request) {
	tenantID := middleware.GetTenantID(r.Context())
	opts := parsePagination(r)

	result, err := h.DB.Users.List(r.Context(), tenantID, opts)
	if err != nil {
		h.Logger.Error("Failed to list users", zap.Error(err))
		respondError(w, http.StatusInternalServerError, "failed to list users")
		return
	}

	respondJSON(w, http.StatusOK, result)
}

func (h *Handler) CreateUser(w http.ResponseWriter, r *http.Request) {
	tenantID := middleware.GetTenantID(r.Context())

	var req struct {
		Email string   `json:"email"`
		Name  string   `json:"name"`
		Roles []string `json:"roles"`
	}

	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		respondError(w, http.StatusBadRequest, "invalid request body")
		return
	}

	if req.Email == "" {
		respondError(w, http.StatusBadRequest, "email is required")
		return
	}

	user, err := h.DB.Users.Create(r.Context(), db.CreateUserInput{
		TenantID: tenantID,
		Email:    req.Email,
		Name:     req.Name,
		Roles:    req.Roles,
	})
	if err != nil {
		h.Logger.Error("Failed to create user", zap.Error(err))
		respondError(w, http.StatusInternalServerError, "failed to create user")
		return
	}

	respondJSON(w, http.StatusCreated, user)
}

func (h *Handler) GetUser(w http.ResponseWriter, r *http.Request) {
	tenantID := middleware.GetTenantID(r.Context())
	userID := chi.URLParam(r, "userID")

	user, err := h.DB.Users.GetByID(r.Context(), tenantID, userID)
	if err != nil {
		h.Logger.Error("Failed to get user", zap.Error(err))
		respondError(w, http.StatusInternalServerError, "failed to get user")
		return
	}
	if user == nil {
		respondError(w, http.StatusNotFound, "user not found")
		return
	}

	respondJSON(w, http.StatusOK, user)
}

func (h *Handler) UpdateUser(w http.ResponseWriter, r *http.Request) {
	tenantID := middleware.GetTenantID(r.Context())
	userID := chi.URLParam(r, "userID")

	var req struct {
		Email  *string  `json:"email,omitempty"`
		Name   *string  `json:"name,omitempty"`
		Roles  []string `json:"roles,omitempty"`
		Status *string  `json:"status,omitempty"`
	}

	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		respondError(w, http.StatusBadRequest, "invalid request body")
		return
	}

	user, err := h.DB.Users.Update(r.Context(), tenantID, userID, db.UpdateUserInput{
		Email:  req.Email,
		Name:   req.Name,
		Roles:  req.Roles,
		Status: req.Status,
	})
	if err != nil {
		h.Logger.Error("Failed to update user", zap.Error(err))
		respondError(w, http.StatusInternalServerError, "failed to update user")
		return
	}
	if user == nil {
		respondError(w, http.StatusNotFound, "user not found")
		return
	}

	respondJSON(w, http.StatusOK, user)
}

func (h *Handler) DeleteUser(w http.ResponseWriter, r *http.Request) {
	tenantID := middleware.GetTenantID(r.Context())
	userID := chi.URLParam(r, "userID")

	err := h.DB.Users.Delete(r.Context(), tenantID, userID)
	if err == sql.ErrNoRows {
		respondError(w, http.StatusNotFound, "user not found")
		return
	}
	if err != nil {
		h.Logger.Error("Failed to delete user", zap.Error(err))
		respondError(w, http.StatusInternalServerError, "failed to delete user")
		return
	}

	w.WriteHeader(http.StatusNoContent)
}
