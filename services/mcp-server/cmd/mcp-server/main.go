package main

import (
	"context"
	"encoding/json"
	"flag"
	"fmt"
	"log/slog"
	"net/http"
	"os"
	"os/signal"
	"strings"
	"sync"
	"syscall"
	"time"

	"github.com/go-chi/chi/v5"
	chimiddleware "github.com/go-chi/chi/v5/middleware"
	"github.com/golang-jwt/jwt/v5"
	"github.com/google/uuid"

	"github.com/sysilo/sysilo/services/mcp-server/internal/client"
	"github.com/sysilo/sysilo/services/mcp-server/internal/config"
	"github.com/sysilo/sysilo/services/mcp-server/internal/governance"
	"github.com/sysilo/sysilo/services/mcp-server/internal/mcp"
	"github.com/sysilo/sysilo/services/mcp-server/internal/resources"
	"github.com/sysilo/sysilo/services/mcp-server/internal/tools"
)

var (
	version   = "dev"
	commit    = "unknown"
	buildDate = "unknown"
)

// Context keys for request-scoped values.
type contextKey string

const (
	ctxTenantID contextKey = "tenant_id"
	ctxUserID   contextKey = "user_id"
	ctxRoles    contextKey = "roles"
)

func main() {
	configPath := flag.String("config", "", "Path to configuration file")
	showVersion := flag.Bool("version", false, "Show version information")
	flag.Parse()

	if *showVersion {
		fmt.Printf("Sysilo MCP Server %s (commit: %s, built: %s)\n", version, commit, buildDate)
		os.Exit(0)
	}

	// Initialize structured logger.
	logger := slog.New(slog.NewJSONHandler(os.Stdout, &slog.HandlerOptions{
		Level: slog.LevelInfo,
	}))
	slog.SetDefault(logger)

	logger.Info("starting Sysilo MCP Server",
		slog.String("version", version),
		slog.String("commit", commit),
	)

	// Load configuration.
	cfg, err := config.Load(*configPath)
	if err != nil {
		logger.Error("failed to load configuration", slog.String("error", err.Error()))
		os.Exit(1)
	}

	// Initialize components.
	toolRegistry := tools.NewRegistry()
	serviceClient := client.NewServiceClient(cfg.Services, logger)
	resourceProvider := resources.NewProvider(cfg.Services, logger)
	govGate := governance.NewGate(governance.GateConfig{
		GovernanceServiceURL: cfg.Services.GovernanceService,
		RateLimitEnabled:     cfg.RateLimit.Enabled,
		RequestsPerMinute:    cfg.RateLimit.RequestsPerMin,
		BurstSize:            cfg.RateLimit.BurstSize,
	}, logger)

	handler := &mcpHandler{
		cfg:              cfg,
		logger:           logger,
		toolRegistry:     toolRegistry,
		serviceClient:    serviceClient,
		resourceProvider: resourceProvider,
		govGate:          govGate,
	}

	// Build router.
	r := chi.NewRouter()
	r.Use(chimiddleware.RequestID)
	r.Use(chimiddleware.RealIP)
	r.Use(chimiddleware.Recoverer)
	r.Use(requestLogger(logger))

	// Health/readiness (no auth).
	r.Get("/health", handler.handleHealth)
	r.Get("/ready", handler.handleReady)

	// MCP SSE endpoint (auth required).
	r.Route("/mcp", func(r chi.Router) {
		r.Use(authMiddleware(logger, cfg.Auth))
		r.Get("/sse", handler.handleSSE)
		r.Post("/message", handler.handleMessage)
	})

	srv := &http.Server{
		Addr:         cfg.Server.Address,
		Handler:      r,
		ReadTimeout:  15 * time.Second,
		WriteTimeout: 0, // SSE connections are long-lived
		IdleTimeout:  120 * time.Second,
	}

	go func() {
		logger.Info("MCP Server listening", slog.String("address", cfg.Server.Address))
		if err := srv.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			logger.Error("server error", slog.String("error", err.Error()))
			os.Exit(1)
		}
	}()

	sigCh := make(chan os.Signal, 1)
	signal.Notify(sigCh, syscall.SIGINT, syscall.SIGTERM)
	<-sigCh

	logger.Info("shutting down MCP server...")

	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	if err := srv.Shutdown(ctx); err != nil {
		logger.Error("server forced to shutdown", slog.String("error", err.Error()))
	}

	logger.Info("MCP Server shutdown complete")
}

// --- HTTP Handlers ---

type mcpHandler struct {
	cfg              *config.Config
	logger           *slog.Logger
	toolRegistry     *tools.Registry
	serviceClient    *client.ServiceClient
	resourceProvider *resources.Provider
	govGate          *governance.Gate
}

func (h *mcpHandler) handleHealth(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusOK)
	json.NewEncoder(w).Encode(map[string]string{
		"status":  "healthy",
		"service": "mcp-server",
		"version": version,
	})
}

func (h *mcpHandler) handleReady(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusOK)
	json.NewEncoder(w).Encode(map[string]string{
		"status": "ready",
	})
}

// handleSSE establishes an SSE connection for the MCP protocol.
// The client sends JSON-RPC messages via POST to /mcp/message with a session_id query param,
// and receives responses via this SSE stream.
func (h *mcpHandler) handleSSE(w http.ResponseWriter, r *http.Request) {
	flusher, ok := w.(http.Flusher)
	if !ok {
		http.Error(w, "streaming not supported", http.StatusInternalServerError)
		return
	}

	sessionID := uuid.New().String()

	w.Header().Set("Content-Type", "text/event-stream")
	w.Header().Set("Cache-Control", "no-cache")
	w.Header().Set("Connection", "keep-alive")
	w.Header().Set("X-Accel-Buffering", "no") // Disable nginx buffering

	// Send the endpoint event so the client knows where to POST messages.
	messageEndpoint := fmt.Sprintf("/mcp/message?session_id=%s", sessionID)
	fmt.Fprintf(w, "event: endpoint\ndata: %s\n\n", messageEndpoint)
	flusher.Flush()

	// Register this session for receiving responses.
	session := &sseSession{
		messages: make(chan []byte, 64),
		done:     make(chan struct{}),
	}
	sessionStore.Store(sessionID, session)
	defer func() {
		sessionStore.Delete(sessionID)
		close(session.done)
	}()

	h.logger.InfoContext(r.Context(), "SSE session established",
		slog.String("session_id", sessionID),
		slog.String("tenant_id", getTenantID(r.Context())),
	)

	// Stream messages to the client until the connection closes.
	ctx := r.Context()
	for {
		select {
		case <-ctx.Done():
			h.logger.InfoContext(r.Context(), "SSE session closed",
				slog.String("session_id", sessionID),
			)
			return
		case msg := <-session.messages:
			fmt.Fprintf(w, "event: message\ndata: %s\n\n", msg)
			flusher.Flush()
		}
	}
}

// handleMessage receives a JSON-RPC message from the client and routes it to the
// appropriate handler. The response is sent back via the SSE stream.
func (h *mcpHandler) handleMessage(w http.ResponseWriter, r *http.Request) {
	sessionID := r.URL.Query().Get("session_id")
	if sessionID == "" {
		http.Error(w, "missing session_id query parameter", http.StatusBadRequest)
		return
	}

	sessionVal, ok := sessionStore.Load(sessionID)
	if !ok {
		http.Error(w, "unknown session; establish an SSE connection first", http.StatusNotFound)
		return
	}
	session := sessionVal.(*sseSession)

	var req mcp.Request
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		resp := mcp.NewErrorResponse(nil, mcp.CodeParseError, "invalid JSON", nil)
		h.sendSSE(session, resp)
		w.WriteHeader(http.StatusAccepted)
		return
	}

	if req.JSONRPC != "2.0" {
		resp := mcp.NewErrorResponse(req.ID, mcp.CodeInvalidRequest, "jsonrpc must be '2.0'", nil)
		h.sendSSE(session, resp)
		w.WriteHeader(http.StatusAccepted)
		return
	}

	// Route by method.
	var resp mcp.Response
	switch req.Method {
	case "initialize":
		resp = h.handleInitialize(req)
	case "initialized":
		// Client acknowledgement; no response needed.
		w.WriteHeader(http.StatusAccepted)
		return
	case "ping":
		resp = mcp.NewResponse(req.ID, map[string]interface{}{})
	case "tools/list":
		resp = h.handleListTools(req)
	case "tools/call":
		resp = h.handleCallTool(r.Context(), req)
	case "resources/list":
		resp = h.handleListResources(r.Context(), req)
	case "resources/read":
		resp = h.handleReadResource(r.Context(), req)
	default:
		resp = mcp.NewErrorResponse(req.ID, mcp.CodeMethodNotFound,
			fmt.Sprintf("method not found: %s", req.Method), nil)
	}

	h.sendSSE(session, resp)
	w.WriteHeader(http.StatusAccepted)
}

func (h *mcpHandler) sendSSE(session *sseSession, resp mcp.Response) {
	data, err := json.Marshal(resp)
	if err != nil {
		h.logger.Error("failed to marshal response", slog.String("error", err.Error()))
		return
	}

	select {
	case session.messages <- data:
	default:
		h.logger.Warn("SSE session message buffer full, dropping message")
	}
}

// --- MCP Method Handlers ---

func (h *mcpHandler) handleInitialize(req mcp.Request) mcp.Response {
	result := mcp.InitializeResult{
		ProtocolVersion: mcp.ProtocolVersion,
		Capabilities: mcp.ServerCapabilities{
			Tools: &mcp.ToolsCapability{
				ListChanged: false,
			},
			Resources: &mcp.ResourcesCapability{
				Subscribe:   false,
				ListChanged: false,
			},
			Logging: &mcp.LoggingCapability{},
		},
		ServerInfo: mcp.Implementation{
			Name:    "sysilo-mcp-server",
			Version: version,
		},
		Instructions: "Sysilo MCP Server exposes enterprise integration, data management, asset registry, governance, and AI capabilities as tools. Use tools/list to discover available operations. All operations are governed by tenant-scoped RBAC policies and audit-logged.",
	}
	return mcp.NewResponse(req.ID, result)
}

func (h *mcpHandler) handleListTools(req mcp.Request) mcp.Response {
	result := mcp.ListToolsResult{
		Tools: h.toolRegistry.List(),
	}
	return mcp.NewResponse(req.ID, result)
}

func (h *mcpHandler) handleCallTool(ctx context.Context, req mcp.Request) mcp.Response {
	var params mcp.CallToolParams
	if err := json.Unmarshal(req.Params, &params); err != nil {
		return mcp.NewErrorResponse(req.ID, mcp.CodeInvalidParams,
			"invalid tool call parameters", nil)
	}

	// Look up the tool.
	toolDef, ok := h.toolRegistry.Get(params.Name)
	if !ok {
		return mcp.NewErrorResponse(req.ID, mcp.CodeInvalidParams,
			fmt.Sprintf("unknown tool: %s", params.Name), nil)
	}

	tenantID := getTenantID(ctx)
	userID := getUserID(ctx)
	roles := getRoles(ctx)
	requestID := uuid.New().String()

	// Check RBAC scope.
	if !hasScope(roles, toolDef.RequiredScope) {
		h.logger.WarnContext(ctx, "tool call denied: insufficient permissions",
			slog.String("tool", params.Name),
			slog.String("required_scope", toolDef.RequiredScope),
			slog.String("user_id", userID),
			slog.String("tenant_id", tenantID),
		)
		return mcp.NewResponse(req.ID, mcp.ErrorResult(
			fmt.Sprintf("Permission denied: this tool requires the '%s' scope.", toolDef.RequiredScope)))
	}

	// Governance gate check.
	govResult, err := h.govGate.Check(ctx, governance.PolicyEvaluationRequest{
		Action:       "tool_call",
		ResourceType: "mcp_tool",
		ToolName:     params.Name,
		TenantID:     tenantID,
		UserID:       userID,
		Roles:        roles,
		Arguments:    params.Arguments,
	})
	if err != nil {
		h.logger.ErrorContext(ctx, "governance check failed",
			slog.String("tool", params.Name),
			slog.String("error", err.Error()),
		)
		return mcp.NewResponse(req.ID, mcp.ErrorResult("Internal error during governance check."))
	}

	if !govResult.Allowed {
		h.logger.WarnContext(ctx, "tool call denied by governance",
			slog.String("tool", params.Name),
			slog.String("reason", govResult.Reason),
			slog.String("tenant_id", tenantID),
		)
		return mcp.NewResponse(req.ID, mcp.ErrorResult(govResult.Reason))
	}

	// Make a copy of arguments so path template resolution doesn't mutate the original.
	argsCopy := make(map[string]interface{}, len(params.Arguments))
	for k, v := range params.Arguments {
		argsCopy[k] = v
	}

	// Proxy the call to the backend service.
	callResult, err := h.serviceClient.Call(ctx, toolDef.ServiceRoute, argsCopy, client.CallContext{
		TenantID:  tenantID,
		UserID:    userID,
		RequestID: requestID,
	})

	// Log to audit trail (fire and forget).
	auditResult := "success"
	if err != nil {
		auditResult = "error"
	} else if callResult.StatusCode >= 400 {
		auditResult = "failure"
	}
	go h.govGate.LogAuditEntry(context.Background(), governance.AuditLogEntry{
		Action:    "tool_call",
		ToolName:  params.Name,
		TenantID:  tenantID,
		UserID:    userID,
		Arguments: params.Arguments,
		Result:    auditResult,
		RequestID: requestID,
	})

	if err != nil {
		h.logger.ErrorContext(ctx, "tool call failed",
			slog.String("tool", params.Name),
			slog.String("error", err.Error()),
			slog.String("request_id", requestID),
		)
		return mcp.NewResponse(req.ID, mcp.ErrorResult(
			fmt.Sprintf("Failed to execute tool: %s", err.Error())))
	}

	// Build the response content.
	var content []mcp.ContentBlock

	// Add governance warnings if any.
	if len(govResult.Warnings) > 0 {
		warningText := "Governance warnings:\n"
		for _, w := range govResult.Warnings {
			warningText += fmt.Sprintf("- %s\n", w)
		}
		content = append(content, mcp.TextContent(warningText))
	}

	// Format the backend response.
	if callResult.StatusCode >= 400 {
		content = append(content, mcp.TextContent(
			fmt.Sprintf("Service returned error (HTTP %d): %s", callResult.StatusCode, string(callResult.Body))))
		return mcp.NewResponse(req.ID, mcp.CallToolResult{
			Content: content,
			IsError: true,
		})
	}

	// Pretty-print JSON responses for readability.
	var prettyBody json.RawMessage
	if err := json.Unmarshal(callResult.Body, &prettyBody); err == nil {
		formatted, err := json.MarshalIndent(prettyBody, "", "  ")
		if err == nil {
			content = append(content, mcp.TextContent(string(formatted)))
		} else {
			content = append(content, mcp.TextContent(string(callResult.Body)))
		}
	} else {
		content = append(content, mcp.TextContent(string(callResult.Body)))
	}

	return mcp.NewResponse(req.ID, mcp.CallToolResult{
		Content: content,
	})
}

func (h *mcpHandler) handleListResources(ctx context.Context, req mcp.Request) mcp.Response {
	tenantID := getTenantID(ctx)

	resourceList, err := h.resourceProvider.ListResources(ctx, tenantID)
	if err != nil {
		return mcp.NewErrorResponse(req.ID, mcp.CodeInternalError,
			"failed to list resources", nil)
	}

	return mcp.NewResponse(req.ID, mcp.ListResourcesResult{
		Resources: resourceList,
	})
}

func (h *mcpHandler) handleReadResource(ctx context.Context, req mcp.Request) mcp.Response {
	var params mcp.ReadResourceParams
	if err := json.Unmarshal(req.Params, &params); err != nil {
		return mcp.NewErrorResponse(req.ID, mcp.CodeInvalidParams,
			"invalid resource read parameters", nil)
	}

	tenantID := getTenantID(ctx)

	result, err := h.resourceProvider.ReadResource(ctx, params.URI, tenantID)
	if err != nil {
		return mcp.NewErrorResponse(req.ID, mcp.CodeInternalError,
			fmt.Sprintf("failed to read resource: %s", err.Error()), nil)
	}

	return mcp.NewResponse(req.ID, result)
}

// --- Auth Middleware ---

func authMiddleware(logger *slog.Logger, cfg config.AuthConfig) func(http.Handler) http.Handler {
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			authHeader := r.Header.Get("Authorization")
			if authHeader == "" {
				http.Error(w, "missing authorization header", http.StatusUnauthorized)
				return
			}

			parts := strings.SplitN(authHeader, " ", 2)
			if len(parts) != 2 || strings.ToLower(parts[0]) != "bearer" {
				http.Error(w, "invalid authorization header format", http.StatusUnauthorized)
				return
			}

			tokenString := parts[1]

			token, err := jwt.Parse(tokenString, func(token *jwt.Token) (interface{}, error) {
				if _, ok := token.Method.(*jwt.SigningMethodHMAC); !ok {
					return nil, jwt.ErrSignatureInvalid
				}
				return []byte(cfg.JWTSecret), nil
			})
			if err != nil {
				logger.Debug("MCP token validation failed", slog.String("error", err.Error()))
				http.Error(w, "invalid token", http.StatusUnauthorized)
				return
			}

			claims, ok := token.Claims.(jwt.MapClaims)
			if !ok || !token.Valid {
				http.Error(w, "invalid token claims", http.StatusUnauthorized)
				return
			}

			userID, _ := claims["sub"].(string)
			tenantID, _ := claims["tenant_id"].(string)
			if userID == "" || tenantID == "" {
				http.Error(w, "token missing required claims (sub, tenant_id)", http.StatusUnauthorized)
				return
			}

			// Extract roles.
			var roleStrings []string
			if roles, ok := claims["roles"].([]interface{}); ok {
				for _, r := range roles {
					if s, ok := r.(string); ok {
						roleStrings = append(roleStrings, s)
					}
				}
			}

			// Extract scopes (used for RBAC checks on tools).
			if scope, ok := claims["scope"].(string); ok && scope != "" {
				roleStrings = append(roleStrings, strings.Fields(scope)...)
			}
			if scopes, ok := claims["scopes"].([]interface{}); ok {
				for _, s := range scopes {
					if str, ok := s.(string); ok {
						roleStrings = append(roleStrings, str)
					}
				}
			}

			ctx := r.Context()
			ctx = context.WithValue(ctx, ctxTenantID, tenantID)
			ctx = context.WithValue(ctx, ctxUserID, userID)
			ctx = context.WithValue(ctx, ctxRoles, roleStrings)

			next.ServeHTTP(w, r.WithContext(ctx))
		})
	}
}

// --- Request Logger Middleware ---

func requestLogger(logger *slog.Logger) func(http.Handler) http.Handler {
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			start := time.Now()
			ww := chimiddleware.NewWrapResponseWriter(w, r.ProtoMajor)

			defer func() {
				logger.Info("request",
					slog.String("method", r.Method),
					slog.String("path", r.URL.Path),
					slog.Int("status", ww.Status()),
					slog.Int("bytes", ww.BytesWritten()),
					slog.Duration("duration", time.Since(start)),
					slog.String("request_id", chimiddleware.GetReqID(r.Context())),
					slog.String("remote_addr", r.RemoteAddr),
				)
			}()

			next.ServeHTTP(ww, r)
		})
	}
}

// --- SSE Session Store ---

var sessionStore sync.Map

type sseSession struct {
	messages chan []byte
	done     chan struct{}
}

// --- Context Helpers ---

func getTenantID(ctx context.Context) string {
	if v, ok := ctx.Value(ctxTenantID).(string); ok {
		return v
	}
	return ""
}

func getUserID(ctx context.Context) string {
	if v, ok := ctx.Value(ctxUserID).(string); ok {
		return v
	}
	return ""
}

func getRoles(ctx context.Context) []string {
	if v, ok := ctx.Value(ctxRoles).([]string); ok {
		return v
	}
	return nil
}

// hasScope checks whether the user's roles/scopes contain the required scope.
// It supports exact matches and colon-delimited hierarchical matching
// (e.g., role "integrations:admin" satisfies scope "integrations:run").
func hasScope(roles []string, required string) bool {
	if required == "" {
		return true
	}

	requiredParts := strings.SplitN(required, ":", 2)
	requiredResource := requiredParts[0]

	for _, role := range roles {
		if role == required {
			return true
		}

		// Admin role grants all scopes.
		if role == "admin" || role == "super_admin" {
			return true
		}

		// Resource-level admin (e.g., "integrations:admin") grants all
		// actions on that resource (e.g., "integrations:run").
		roleParts := strings.SplitN(role, ":", 2)
		if len(roleParts) == 2 && roleParts[0] == requiredResource && roleParts[1] == "admin" {
			return true
		}

		// Wildcard scope (e.g., "integrations:*").
		if len(roleParts) == 2 && roleParts[0] == requiredResource && roleParts[1] == "*" {
			return true
		}
	}

	return false
}
