package governance

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"log/slog"
	"net/http"
	"sync"
	"time"
)

// Gate enforces governance policies and rate limits before tool execution.
type Gate struct {
	governanceURL string
	httpClient    *http.Client
	logger        *slog.Logger

	// Per-tenant sliding window rate limiter.
	rateLimiter *tenantRateLimiter
}

// GateConfig holds configuration for the governance gate.
type GateConfig struct {
	GovernanceServiceURL string
	RateLimitEnabled     bool
	RequestsPerMinute    int
	BurstSize            int
}

// NewGate creates a new governance gate.
func NewGate(cfg GateConfig, logger *slog.Logger) *Gate {
	return &Gate{
		governanceURL: cfg.GovernanceServiceURL,
		httpClient: &http.Client{
			Timeout: 10 * time.Second,
		},
		logger:      logger,
		rateLimiter: newTenantRateLimiter(cfg.RequestsPerMinute, cfg.BurstSize, cfg.RateLimitEnabled),
	}
}

// CheckResult is the outcome of a governance check.
type CheckResult struct {
	Allowed  bool     `json:"allowed"`
	Warnings []string `json:"warnings,omitempty"`
	Reason   string   `json:"reason,omitempty"`
}

// PolicyEvaluationRequest is sent to the governance service.
type PolicyEvaluationRequest struct {
	Action       string                 `json:"action"`
	ResourceType string                 `json:"resource_type"`
	ToolName     string                 `json:"tool_name"`
	TenantID     string                 `json:"tenant_id"`
	UserID       string                 `json:"user_id"`
	Roles        []string               `json:"roles"`
	Arguments    map[string]interface{} `json:"arguments,omitempty"`
	Source       string                 `json:"source"`
}

// PolicyEvaluationResponse is returned by the governance service.
type PolicyEvaluationResponse struct {
	Decision   string   `json:"decision"`   // "allow", "warn", "enforce"
	Violations []string `json:"violations"` // human-readable violation messages
	AuditID    string   `json:"audit_id"`
}

// AuditLogEntry is sent to the governance service audit trail.
type AuditLogEntry struct {
	Action    string                 `json:"action"`
	ToolName  string                 `json:"tool_name"`
	TenantID  string                 `json:"tenant_id"`
	UserID    string                 `json:"user_id"`
	Arguments map[string]interface{} `json:"arguments,omitempty"`
	Result    string                 `json:"result"`
	Source    string                 `json:"source"`
	Timestamp time.Time              `json:"timestamp"`
	RequestID string                 `json:"request_id"`
}

// Check evaluates governance policies for a tool call.
// It checks rate limits, calls the governance service for policy evaluation,
// and logs the action to the audit trail.
func (g *Gate) Check(ctx context.Context, req PolicyEvaluationRequest) (*CheckResult, error) {
	// Check rate limit first (fast, local check).
	if !g.rateLimiter.allow(req.TenantID) {
		g.logger.WarnContext(ctx, "rate limit exceeded for tenant",
			slog.String("tenant_id", req.TenantID),
			slog.String("tool", req.ToolName),
		)
		return &CheckResult{
			Allowed: false,
			Reason:  "Rate limit exceeded. Too many MCP tool calls for this tenant. Please wait before retrying.",
		}, nil
	}

	// Call governance service for policy evaluation.
	result, err := g.evaluatePolicy(ctx, req)
	if err != nil {
		// If governance service is unavailable, log the error but fail open
		// to avoid blocking all MCP operations. The audit trail will capture
		// that governance was unavailable.
		g.logger.ErrorContext(ctx, "governance service unavailable, failing open",
			slog.String("tool", req.ToolName),
			slog.String("tenant_id", req.TenantID),
			slog.String("error", err.Error()),
		)
		return &CheckResult{
			Allowed:  true,
			Warnings: []string{"Governance service unavailable; policy evaluation skipped."},
		}, nil
	}

	return result, nil
}

// LogAuditEntry sends an audit log entry to the governance service.
func (g *Gate) LogAuditEntry(ctx context.Context, entry AuditLogEntry) {
	entry.Timestamp = time.Now().UTC()
	entry.Source = "mcp-server"

	body, err := json.Marshal(entry)
	if err != nil {
		g.logger.ErrorContext(ctx, "failed to marshal audit entry",
			slog.String("error", err.Error()),
		)
		return
	}

	url := fmt.Sprintf("%s/audit/log", g.governanceURL)
	req, err := http.NewRequestWithContext(ctx, http.MethodPost, url, bytes.NewReader(body))
	if err != nil {
		g.logger.ErrorContext(ctx, "failed to create audit request",
			slog.String("error", err.Error()),
		)
		return
	}
	req.Header.Set("Content-Type", "application/json")

	resp, err := g.httpClient.Do(req)
	if err != nil {
		g.logger.ErrorContext(ctx, "failed to send audit log",
			slog.String("error", err.Error()),
		)
		return
	}
	defer resp.Body.Close()
	io.Copy(io.Discard, resp.Body)

	if resp.StatusCode >= 400 {
		g.logger.WarnContext(ctx, "audit log returned error status",
			slog.Int("status", resp.StatusCode),
		)
	}
}

func (g *Gate) evaluatePolicy(ctx context.Context, evalReq PolicyEvaluationRequest) (*CheckResult, error) {
	evalReq.Source = "mcp-server"

	body, err := json.Marshal(evalReq)
	if err != nil {
		return nil, fmt.Errorf("marshal policy request: %w", err)
	}

	url := fmt.Sprintf("%s/policies/evaluate", g.governanceURL)
	req, err := http.NewRequestWithContext(ctx, http.MethodPost, url, bytes.NewReader(body))
	if err != nil {
		return nil, fmt.Errorf("create policy request: %w", err)
	}
	req.Header.Set("Content-Type", "application/json")

	resp, err := g.httpClient.Do(req)
	if err != nil {
		return nil, fmt.Errorf("call governance service: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode >= 500 {
		return nil, fmt.Errorf("governance service returned status %d", resp.StatusCode)
	}

	var evalResp PolicyEvaluationResponse
	if err := json.NewDecoder(resp.Body).Decode(&evalResp); err != nil {
		return nil, fmt.Errorf("decode policy response: %w", err)
	}

	switch evalResp.Decision {
	case "enforce":
		reason := "Policy violation"
		if len(evalResp.Violations) > 0 {
			reason = fmt.Sprintf("Policy violation: %s", evalResp.Violations[0])
		}
		return &CheckResult{
			Allowed: false,
			Reason:  reason,
		}, nil

	case "warn":
		return &CheckResult{
			Allowed:  true,
			Warnings: evalResp.Violations,
		}, nil

	default: // "allow" or any other value
		return &CheckResult{
			Allowed: true,
		}, nil
	}
}

// --- In-memory per-tenant rate limiter ---

type tenantRateLimiter struct {
	mu       sync.Mutex
	enabled  bool
	perMin   int
	burst    int
	tenants  map[string]*tokenBucket
}

type tokenBucket struct {
	tokens     float64
	maxTokens  float64
	refillRate float64 // tokens per second
	lastRefill time.Time
}

func newTenantRateLimiter(requestsPerMin, burst int, enabled bool) *tenantRateLimiter {
	if burst <= 0 {
		burst = requestsPerMin / 6 // default burst = ~10 seconds worth
		if burst < 1 {
			burst = 1
		}
	}
	return &tenantRateLimiter{
		enabled: enabled,
		perMin:  requestsPerMin,
		burst:   burst,
		tenants: make(map[string]*tokenBucket),
	}
}

func (rl *tenantRateLimiter) allow(tenantID string) bool {
	if !rl.enabled {
		return true
	}

	rl.mu.Lock()
	defer rl.mu.Unlock()

	bucket, ok := rl.tenants[tenantID]
	if !ok {
		maxTokens := float64(rl.burst)
		bucket = &tokenBucket{
			tokens:     maxTokens,
			maxTokens:  maxTokens,
			refillRate: float64(rl.perMin) / 60.0,
			lastRefill: time.Now(),
		}
		rl.tenants[tenantID] = bucket
	}

	// Refill tokens based on elapsed time.
	now := time.Now()
	elapsed := now.Sub(bucket.lastRefill).Seconds()
	bucket.tokens += elapsed * bucket.refillRate
	if bucket.tokens > bucket.maxTokens {
		bucket.tokens = bucket.maxTokens
	}
	bucket.lastRefill = now

	if bucket.tokens < 1 {
		return false
	}

	bucket.tokens--
	return true
}
