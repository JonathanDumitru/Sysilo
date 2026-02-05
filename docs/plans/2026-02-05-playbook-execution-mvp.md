# Playbook Execution MVP Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Enable end-to-end playbook execution with webhook steps, fixing critical pipeline bugs.

**Architecture:** Frontend triggers run → API creates run record → Executor dispatches steps to Kafka → Agent executes webhook requests → Consumer routes results back → Result handler advances DAG → Frontend polls status.

**Tech Stack:** Rust (Axum), Go (Agent), TypeScript (React), Kafka, PostgreSQL

---

## Task 1: Fix run_id Propagation in Executor

**Problem:** The executor sends `run_id` in the `Task` struct but not in the `config` JSON. The consumer expects `run_id` in the task output, but the agent doesn't know to echo it back.

**Files:**
- Modify: `services/integration-service/src/playbooks/executor.rs:121-127`

**Step 1: Add run_id to task config**

In `dispatch_step()`, add `run_id` to the config JSON so it's available in the agent's task config:

```rust
let task = Task {
    id: task_id,
    run_id,
    integration_id: playbook_id,
    tenant_id: tenant_id.to_string(),
    task_type: "playbook_step".to_string(),
    config: serde_json::json!({
        "run_id": run_id.to_string(),  // ADD THIS LINE
        "step_id": step.id,
        "step_type": step_type_str,
        "step_name": step.name,
        "step_config": step.config,
        "variables": variables,
    }),
    priority: 2,
    timeout_seconds: 300,
    sequence: 0,
    depends_on: vec![],
};
```

**Step 2: Verify with existing test**

Run: `cargo test --package integration-service playbooks::executor`
Expected: All tests pass (this doesn't break existing tests)

**Step 3: Commit**

```bash
git add services/integration-service/src/playbooks/executor.rs
git commit -m "fix(playbooks): include run_id in task config for result correlation"
```

---

## Task 2: Fix Approval Endpoint Return Types

**Problem:** `approve_run` and `reject_run` return `ApprovalResponse` but frontend expects `PlaybookRun`. Frontend tries to access `data.id` and `data.playbook_id` which don't exist on `ApprovalResponse`.

**Files:**
- Modify: `services/integration-service/src/playbooks/api.rs:503-611`

**Step 1: Update approve_run to return PlaybookRun**

Replace the approval response with a full run fetch:

```rust
/// POST /runs/:id/approve - Approve a playbook run waiting for approval
pub async fn approve_run(
    State(state): State<Arc<AppState>>,
    Extension(tenant): Extension<TenantContext>,
    Path(id): Path<Uuid>,
    Json(req): Json<ApprovalRequest>,
) -> Result<Json<PlaybookRunResponse>, ApiError> {
    let tenant_id = tenant.tenant_id.to_string();

    // Get the current run
    let row = state
        .storage
        .get_playbook_run(&tenant_id, id)
        .await
        .map_err(|e| ApiError {
            error: "not_found".to_string(),
            message: e.to_string(),
        })?;

    // Check if the run is waiting for approval
    if row.status != "waiting_approval" {
        return Err(ApiError {
            error: "invalid_state".to_string(),
            message: format!(
                "Run is in '{}' state, expected 'waiting_approval'",
                row.status
            ),
        });
    }

    // Update status to running (resume execution)
    state
        .storage
        .update_playbook_run(id, "running", row.step_states.clone())
        .await
        .map_err(|e| ApiError {
            error: "database_error".to_string(),
            message: e.to_string(),
        })?;

    tracing::info!(
        run_id = %id,
        tenant_id = %tenant_id,
        reason = ?req.reason,
        "Playbook run approved"
    );

    // Re-fetch the updated run to return
    let updated_row = state
        .storage
        .get_playbook_run(&tenant_id, id)
        .await
        .map_err(|e| ApiError {
            error: "database_error".to_string(),
            message: e.to_string(),
        })?;

    let response = run_row_to_response(updated_row)?;
    Ok(Json(response))
}
```

**Step 2: Update reject_run to return PlaybookRun**

Same pattern:

```rust
/// POST /runs/:id/reject - Reject a playbook run waiting for approval
pub async fn reject_run(
    State(state): State<Arc<AppState>>,
    Extension(tenant): Extension<TenantContext>,
    Path(id): Path<Uuid>,
    Json(req): Json<ApprovalRequest>,
) -> Result<Json<PlaybookRunResponse>, ApiError> {
    let tenant_id = tenant.tenant_id.to_string();

    // Get the current run
    let row = state
        .storage
        .get_playbook_run(&tenant_id, id)
        .await
        .map_err(|e| ApiError {
            error: "not_found".to_string(),
            message: e.to_string(),
        })?;

    // Check if the run is waiting for approval
    if row.status != "waiting_approval" {
        return Err(ApiError {
            error: "invalid_state".to_string(),
            message: format!(
                "Run is in '{}' state, expected 'waiting_approval'",
                row.status
            ),
        });
    }

    // Update status to cancelled
    state
        .storage
        .update_playbook_run(id, "cancelled", row.step_states.clone())
        .await
        .map_err(|e| ApiError {
            error: "database_error".to_string(),
            message: e.to_string(),
        })?;

    tracing::info!(
        run_id = %id,
        tenant_id = %tenant_id,
        reason = ?req.reason,
        "Playbook run rejected"
    );

    // Re-fetch the updated run to return
    let updated_row = state
        .storage
        .get_playbook_run(&tenant_id, id)
        .await
        .map_err(|e| ApiError {
            error: "database_error".to_string(),
            message: e.to_string(),
        })?;

    let response = run_row_to_response(updated_row)?;
    Ok(Json(response))
}
```

**Step 3: Build and verify**

Run: `cargo check --package integration-service`
Expected: Compiles without errors

**Step 4: Commit**

```bash
git add services/integration-service/src/playbooks/api.rs
git commit -m "fix(playbooks): return PlaybookRun from approve/reject endpoints"
```

---

## Task 3: Add Approval Resumption Logic

**Problem:** After approval, the endpoint just sets status to `running` but doesn't dispatch the next steps. The run will hang.

**Files:**
- Modify: `services/integration-service/src/playbooks/api.rs:503-555` (approve_run function)

**Step 1: Add next step dispatch after approval**

After the approval update, find and dispatch the next steps. The approval step's `on_success` contains the next step IDs:

```rust
/// POST /runs/:id/approve - Approve a playbook run waiting for approval
pub async fn approve_run(
    State(state): State<Arc<AppState>>,
    Extension(tenant): Extension<TenantContext>,
    Path(id): Path<Uuid>,
    Json(req): Json<ApprovalRequest>,
) -> Result<Json<PlaybookRunResponse>, ApiError> {
    let tenant_id = tenant.tenant_id.to_string();

    // Get the current run
    let row = state
        .storage
        .get_playbook_run(&tenant_id, id)
        .await
        .map_err(|e| ApiError {
            error: "not_found".to_string(),
            message: e.to_string(),
        })?;

    // Check if the run is waiting for approval
    if row.status != "waiting_approval" {
        return Err(ApiError {
            error: "invalid_state".to_string(),
            message: format!(
                "Run is in '{}' state, expected 'waiting_approval'",
                row.status
            ),
        });
    }

    // Load playbook to get step definitions
    let playbook = state
        .storage
        .get_playbook(&tenant_id, row.playbook_id)
        .await
        .map_err(|e| ApiError {
            error: "not_found".to_string(),
            message: e.to_string(),
        })?;

    let steps: Vec<crate::playbooks::Step> = serde_json::from_value(playbook.steps.clone())
        .map_err(|e| ApiError {
            error: "parse_error".to_string(),
            message: e.to_string(),
        })?;

    // Parse step states to find the approval step that's currently running
    let mut step_states: Vec<crate::playbooks::StepState> =
        serde_json::from_value(row.step_states.clone()).map_err(|e| ApiError {
            error: "parse_error".to_string(),
            message: e.to_string(),
        })?;

    // Find the running approval step and its next steps
    let mut next_step_ids: Vec<String> = Vec::new();
    for step_state in &mut step_states {
        if step_state.status == crate::playbooks::StepStatus::Running {
            if let Some(step_def) = steps.iter().find(|s| s.id == step_state.step_id) {
                if matches!(step_def.step_type, crate::playbooks::StepType::Approval) {
                    // Mark approval step as completed
                    step_state.status = crate::playbooks::StepStatus::Completed;
                    step_state.completed_at = Some(chrono::Utc::now());
                    step_state.output = Some(serde_json::json!({
                        "approved": true,
                        "reason": req.reason,
                    }));
                    // Get next steps
                    next_step_ids = step_def.on_success.clone();
                    break;
                }
            }
        }
    }

    // Mark next steps as running
    for next_id in &next_step_ids {
        if let Some(ns) = step_states.iter_mut().find(|s| s.step_id == *next_id) {
            ns.status = crate::playbooks::StepStatus::Running;
            ns.started_at = Some(chrono::Utc::now());
        }
    }

    let step_states_json =
        serde_json::to_value(&step_states).map_err(|e| ApiError {
            error: "serialize_error".to_string(),
            message: e.to_string(),
        })?;

    // Update run status
    let new_status = if next_step_ids.is_empty() { "completed" } else { "running" };
    state
        .storage
        .update_playbook_run(id, new_status, step_states_json)
        .await
        .map_err(|e| ApiError {
            error: "database_error".to_string(),
            message: e.to_string(),
        })?;

    // Dispatch next steps if Kafka producer available
    if !next_step_ids.is_empty() {
        if let Some(producer) = state.engine.kafka_producer() {
            for next_id in &next_step_ids {
                if let Some(next_step) = steps.iter().find(|s| s.id == *next_id) {
                    if let Err(e) = crate::playbooks::executor::PlaybookExecutor::dispatch_step(
                        producer,
                        id,
                        row.playbook_id,
                        &tenant_id,
                        next_step,
                        &row.variables,
                    )
                    .await
                    {
                        tracing::error!(
                            run_id = %id,
                            step_id = %next_id,
                            error = %e,
                            "Failed to dispatch step after approval"
                        );
                    }
                }
            }
        }
    }

    tracing::info!(
        run_id = %id,
        tenant_id = %tenant_id,
        reason = ?req.reason,
        next_steps = ?next_step_ids,
        "Playbook run approved and resumed"
    );

    // Re-fetch the updated run
    let updated_row = state
        .storage
        .get_playbook_run(&tenant_id, id)
        .await
        .map_err(|e| ApiError {
            error: "database_error".to_string(),
            message: e.to_string(),
        })?;

    let response = run_row_to_response(updated_row)?;
    Ok(Json(response))
}
```

**Step 2: Build and verify**

Run: `cargo check --package integration-service`
Expected: Compiles without errors

**Step 3: Commit**

```bash
git add services/integration-service/src/playbooks/api.rs
git commit -m "fix(playbooks): dispatch next steps after approval"
```

---

## Task 4: Create Playbook Step Handler Structure in Go Agent

**Files:**
- Create: `agent/internal/adapters/playbook/handler.go`
- Modify: `agent/cmd/agent/main.go:62-63`

**Step 1: Create the playbook step handler**

```go
package playbook

import (
	"context"
	"encoding/json"
	"fmt"

	"github.com/sysilo/sysilo/agent/internal/executor"
	"go.uber.org/zap"
)

// Handler processes playbook_step tasks by routing to sub-handlers based on step_type
type Handler struct {
	logger      *zap.Logger
	subHandlers map[string]StepHandler
}

// StepHandler processes a specific step type
type StepHandler interface {
	Execute(ctx context.Context, config *StepConfig) (*StepResult, error)
}

// StepConfig represents the common configuration passed to all step handlers
type StepConfig struct {
	RunID      string                 `json:"run_id"`
	StepID     string                 `json:"step_id"`
	StepType   string                 `json:"step_type"`
	StepName   string                 `json:"step_name"`
	StepConfig map[string]interface{} `json:"step_config"`
	Variables  map[string]interface{} `json:"variables"`
}

// StepResult is the result from executing a step
type StepResult struct {
	Output    interface{} `json:"output,omitempty"`
	NextSteps []string    `json:"next_steps,omitempty"` // For condition steps
	Error     string      `json:"error,omitempty"`
}

// NewHandler creates a new playbook step handler
func NewHandler(logger *zap.Logger) *Handler {
	h := &Handler{
		logger:      logger,
		subHandlers: make(map[string]StepHandler),
	}

	// Register sub-handlers
	h.subHandlers["webhook"] = NewWebhookHandler(logger)

	return h
}

// Type returns the task type this handler processes
func (h *Handler) Type() string {
	return "playbook_step"
}

// Execute routes to the appropriate sub-handler based on step_type
func (h *Handler) Execute(ctx context.Context, task *executor.Task) (*executor.TaskResult, error) {
	h.logger.Info("Executing playbook step",
		zap.String("task_id", task.ID),
		zap.Any("config_keys", getConfigKeys(task.Config)),
	)

	// Parse the step config
	configBytes, err := json.Marshal(task.Config)
	if err != nil {
		return nil, fmt.Errorf("failed to marshal task config: %w", err)
	}

	var stepConfig StepConfig
	if err := json.Unmarshal(configBytes, &stepConfig); err != nil {
		return nil, fmt.Errorf("failed to parse step config: %w", err)
	}

	// Extract step_type from the JSON value (it's serialized as a string like "\"webhook\"")
	stepType := extractStepType(stepConfig.StepType)

	h.logger.Info("Routing to sub-handler",
		zap.String("step_type", stepType),
		zap.String("step_id", stepConfig.StepID),
		zap.String("run_id", stepConfig.RunID),
	)

	// Find the sub-handler
	subHandler, ok := h.subHandlers[stepType]
	if !ok {
		return &executor.TaskResult{
			Output: map[string]interface{}{
				"run_id":  stepConfig.RunID,
				"step_id": stepConfig.StepID,
				"error":   fmt.Sprintf("unknown step type: %s", stepType),
			},
			Error: &executor.TaskError{
				Code:      "unknown_step_type",
				Message:   fmt.Sprintf("No handler registered for step type: %s", stepType),
				Retryable: false,
			},
		}, nil
	}

	// Execute the sub-handler
	result, err := subHandler.Execute(ctx, &stepConfig)
	if err != nil {
		return &executor.TaskResult{
			Output: map[string]interface{}{
				"run_id":  stepConfig.RunID,
				"step_id": stepConfig.StepID,
				"error":   err.Error(),
			},
			Error: &executor.TaskError{
				Code:      "step_execution_failed",
				Message:   err.Error(),
				Retryable: false,
			},
		}, nil
	}

	// Build the output with run_id and step_id for correlation
	output := map[string]interface{}{
		"run_id":  stepConfig.RunID,
		"step_id": stepConfig.StepID,
	}

	if result.Output != nil {
		output["result"] = result.Output
	}
	if len(result.NextSteps) > 0 {
		output["next_steps"] = result.NextSteps
	}

	return &executor.TaskResult{
		Output: output,
	}, nil
}

// extractStepType handles the serialized step_type value
// The Rust side serializes it as JSON, so we might get "\"webhook\"" or just "webhook"
func extractStepType(raw string) string {
	// Try to unquote if it's a JSON string
	var unquoted string
	if err := json.Unmarshal([]byte(raw), &unquoted); err == nil {
		return unquoted
	}
	return raw
}

// getConfigKeys returns the keys of a map for logging
func getConfigKeys(m map[string]interface{}) []string {
	keys := make([]string, 0, len(m))
	for k := range m {
		keys = append(keys, k)
	}
	return keys
}
```

**Step 2: Register the handler in main.go**

Add import and registration:

```go
import (
	// ... existing imports ...
	"github.com/sysilo/sysilo/agent/internal/adapters/playbook"
)

// In main(), after existing handler registrations:
exec.RegisterHandler(playbook.NewHandler(logger))
```

**Step 3: Create empty webhook handler file (to be implemented in Task 5)**

Create `agent/internal/adapters/playbook/webhook.go`:

```go
package playbook

import (
	"context"
	"fmt"

	"go.uber.org/zap"
)

// WebhookHandler executes HTTP webhook requests
type WebhookHandler struct {
	logger *zap.Logger
}

// NewWebhookHandler creates a new webhook step handler
func NewWebhookHandler(logger *zap.Logger) *WebhookHandler {
	return &WebhookHandler{logger: logger}
}

// Execute makes an HTTP request based on the step config
func (h *WebhookHandler) Execute(ctx context.Context, config *StepConfig) (*StepResult, error) {
	// TODO: Implement in Task 5
	return nil, fmt.Errorf("webhook handler not yet implemented")
}
```

**Step 4: Verify builds** (Note: Go may not be installed locally)

Run: `cd agent && go build ./cmd/agent`
Expected: Build succeeds (or skip if Go not installed)

**Step 5: Commit**

```bash
git add agent/internal/adapters/playbook/handler.go agent/internal/adapters/playbook/webhook.go agent/cmd/agent/main.go
git commit -m "feat(agent): add playbook_step handler structure with routing"
```

---

## Task 5: Implement Webhook Handler

**Files:**
- Modify: `agent/internal/adapters/playbook/webhook.go`

**Step 1: Implement the full webhook handler**

```go
package playbook

import (
	"bytes"
	"context"
	"encoding/base64"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"regexp"
	"strings"
	"time"

	"go.uber.org/zap"
)

// WebhookHandler executes HTTP webhook requests
type WebhookHandler struct {
	logger *zap.Logger
	client *http.Client
}

// WebhookConfig represents the configuration for a webhook step
type WebhookConfig struct {
	Method              string            `json:"method"`
	URL                 string            `json:"url"`
	Headers             map[string]string `json:"headers"`
	Body                interface{}       `json:"body"`
	TimeoutSeconds      int               `json:"timeout_seconds"`
	Auth                *AuthConfig       `json:"auth"`
	FollowRedirects     *bool             `json:"follow_redirects"`
	ExpectedStatusCodes []int             `json:"expected_status_codes"`
	ExtractResponsePath string            `json:"extract_response_path"`
}

// AuthConfig represents authentication configuration
type AuthConfig struct {
	Type     string `json:"type"` // "none", "basic", "bearer"
	Username string `json:"username,omitempty"`
	Password string `json:"password,omitempty"`
	Token    string `json:"token,omitempty"`
}

// WebhookResult is the result of a webhook execution
type WebhookResult struct {
	StatusCode   int               `json:"status_code"`
	Headers      map[string]string `json:"headers"`
	Body         interface{}       `json:"body"`
	ExtractedVal interface{}       `json:"extracted_value,omitempty"`
	DurationMs   int64             `json:"duration_ms"`
}

// NewWebhookHandler creates a new webhook step handler
func NewWebhookHandler(logger *zap.Logger) *WebhookHandler {
	return &WebhookHandler{
		logger: logger,
		client: &http.Client{
			Timeout: 30 * time.Second,
		},
	}
}

// Execute makes an HTTP request based on the step config
func (h *WebhookHandler) Execute(ctx context.Context, config *StepConfig) (*StepResult, error) {
	h.logger.Info("Executing webhook step",
		zap.String("step_id", config.StepID),
		zap.String("step_name", config.StepName),
	)

	// Parse webhook-specific config
	configBytes, err := json.Marshal(config.StepConfig)
	if err != nil {
		return nil, fmt.Errorf("failed to marshal step config: %w", err)
	}

	var webhookConfig WebhookConfig
	if err := json.Unmarshal(configBytes, &webhookConfig); err != nil {
		return nil, fmt.Errorf("failed to parse webhook config: %w", err)
	}

	// Apply defaults
	if webhookConfig.Method == "" {
		webhookConfig.Method = "GET"
	}
	if webhookConfig.TimeoutSeconds == 0 {
		webhookConfig.TimeoutSeconds = 30
	}
	if webhookConfig.FollowRedirects == nil {
		followRedirects := true
		webhookConfig.FollowRedirects = &followRedirects
	}

	// Validate required fields
	if webhookConfig.URL == "" {
		return nil, fmt.Errorf("webhook URL is required")
	}

	// Interpolate variables in URL, headers, and body
	webhookConfig.URL = h.interpolateVariables(webhookConfig.URL, config.Variables)
	for k, v := range webhookConfig.Headers {
		webhookConfig.Headers[k] = h.interpolateVariables(v, config.Variables)
	}

	// Build the request
	var bodyReader io.Reader
	if webhookConfig.Body != nil {
		switch b := webhookConfig.Body.(type) {
		case string:
			bodyReader = strings.NewReader(h.interpolateVariables(b, config.Variables))
		default:
			bodyBytes, err := json.Marshal(b)
			if err != nil {
				return nil, fmt.Errorf("failed to marshal request body: %w", err)
			}
			// Interpolate variables in JSON body
			interpolated := h.interpolateVariables(string(bodyBytes), config.Variables)
			bodyReader = strings.NewReader(interpolated)
		}
	}

	// Create HTTP client with timeout
	client := &http.Client{
		Timeout: time.Duration(webhookConfig.TimeoutSeconds) * time.Second,
	}
	if !*webhookConfig.FollowRedirects {
		client.CheckRedirect = func(req *http.Request, via []*http.Request) error {
			return http.ErrUseLastResponse
		}
	}

	// Create the request
	req, err := http.NewRequestWithContext(ctx, webhookConfig.Method, webhookConfig.URL, bodyReader)
	if err != nil {
		return nil, fmt.Errorf("failed to create request: %w", err)
	}

	// Set headers
	for k, v := range webhookConfig.Headers {
		req.Header.Set(k, v)
	}

	// Set default Content-Type for POST/PUT/PATCH if body is present and no Content-Type set
	if bodyReader != nil && req.Header.Get("Content-Type") == "" {
		req.Header.Set("Content-Type", "application/json")
	}

	// Apply authentication
	if webhookConfig.Auth != nil {
		switch webhookConfig.Auth.Type {
		case "basic":
			auth := base64.StdEncoding.EncodeToString(
				[]byte(webhookConfig.Auth.Username + ":" + webhookConfig.Auth.Password),
			)
			req.Header.Set("Authorization", "Basic "+auth)
		case "bearer":
			token := h.interpolateVariables(webhookConfig.Auth.Token, config.Variables)
			req.Header.Set("Authorization", "Bearer "+token)
		}
	}

	h.logger.Info("Making HTTP request",
		zap.String("method", webhookConfig.Method),
		zap.String("url", webhookConfig.URL),
	)

	// Execute the request
	startTime := time.Now()
	resp, err := client.Do(req)
	duration := time.Since(startTime)

	if err != nil {
		return nil, fmt.Errorf("HTTP request failed: %w", err)
	}
	defer resp.Body.Close()

	// Read response body
	respBody, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, fmt.Errorf("failed to read response body: %w", err)
	}

	// Parse response body as JSON if possible
	var parsedBody interface{}
	if err := json.Unmarshal(respBody, &parsedBody); err != nil {
		// Not JSON, use as string
		parsedBody = string(respBody)
	}

	// Build response headers map
	respHeaders := make(map[string]string)
	for k, v := range resp.Header {
		if len(v) > 0 {
			respHeaders[k] = v[0]
		}
	}

	result := &WebhookResult{
		StatusCode: resp.StatusCode,
		Headers:    respHeaders,
		Body:       parsedBody,
		DurationMs: duration.Milliseconds(),
	}

	// Extract value if path specified
	if webhookConfig.ExtractResponsePath != "" && parsedBody != nil {
		extracted := h.extractPath(parsedBody, webhookConfig.ExtractResponsePath)
		result.ExtractedVal = extracted
	}

	// Check expected status codes
	if len(webhookConfig.ExpectedStatusCodes) > 0 {
		found := false
		for _, code := range webhookConfig.ExpectedStatusCodes {
			if code == resp.StatusCode {
				found = true
				break
			}
		}
		if !found {
			return &StepResult{
				Output: result,
				Error:  fmt.Sprintf("unexpected status code %d, expected one of %v", resp.StatusCode, webhookConfig.ExpectedStatusCodes),
			}, nil
		}
	} else {
		// Default: accept 2xx status codes
		if resp.StatusCode < 200 || resp.StatusCode >= 300 {
			return &StepResult{
				Output: result,
				Error:  fmt.Sprintf("HTTP request failed with status %d", resp.StatusCode),
			}, nil
		}
	}

	h.logger.Info("Webhook request completed",
		zap.Int("status_code", resp.StatusCode),
		zap.Int64("duration_ms", duration.Milliseconds()),
	)

	return &StepResult{
		Output: result,
	}, nil
}

// interpolateVariables replaces ${var} patterns with values from variables map
func (h *WebhookHandler) interpolateVariables(input string, variables map[string]interface{}) string {
	if variables == nil {
		return input
	}

	re := regexp.MustCompile(`\$\{([^}]+)\}`)
	return re.ReplaceAllStringFunc(input, func(match string) string {
		// Extract variable name from ${name}
		varName := match[2 : len(match)-1]

		// Look up in variables
		if val, ok := variables[varName]; ok {
			switch v := val.(type) {
			case string:
				return v
			default:
				// Convert to JSON string for complex types
				jsonBytes, err := json.Marshal(v)
				if err != nil {
					return match // Keep original if can't convert
				}
				return string(jsonBytes)
			}
		}

		return match // Keep original if not found
	})
}

// extractPath extracts a value from a nested structure using dot notation
// e.g., "data.user.id" extracts obj["data"]["user"]["id"]
func (h *WebhookHandler) extractPath(data interface{}, path string) interface{} {
	parts := strings.Split(path, ".")
	current := data

	for _, part := range parts {
		switch c := current.(type) {
		case map[string]interface{}:
			var ok bool
			current, ok = c[part]
			if !ok {
				return nil
			}
		default:
			return nil
		}
	}

	return current
}
```

**Step 2: Verify builds** (Note: Go may not be installed locally)

Run: `cd agent && go build ./cmd/agent`
Expected: Build succeeds

**Step 3: Commit**

```bash
git add agent/internal/adapters/playbook/webhook.go
git commit -m "feat(agent): implement webhook step handler with full configuration"
```

---

## Task 6: Verify TypeScript Types

**Problem:** Frontend service declares `approveRun` and `rejectRun` return `PlaybookRun`, which is now correct. Verify types align.

**Files:**
- Read: `packages/frontend/web-app/src/services/playbooks.ts:131-141`
- Read: `packages/frontend/web-app/src/hooks/usePlaybooks.ts:126-146`

**Step 1: Verify service types**

The service already declares the correct return types:

```typescript
export async function approveRun(runId: string): Promise<PlaybookRun> {
  return apiFetch<PlaybookRun>(`/integrations/playbook-runs/${runId}/approve`, {
    method: 'POST',
  });
}

export async function rejectRun(runId: string): Promise<PlaybookRun> {
  return apiFetch<PlaybookRun>(`/integrations/playbook-runs/${runId}/reject`, {
    method: 'POST',
  });
}
```

With Task 2's backend fix, these now match the actual response shape.

**Step 2: Verify hook usage**

The hooks correctly use `data.id` and `data.playbook_id`:

```typescript
onSuccess: (data) => {
  queryClient.invalidateQueries({ queryKey: playbookKeys.run(data.id) });
  queryClient.invalidateQueries({ queryKey: playbookKeys.runs(data.playbook_id) });
},
```

**Step 3: Run TypeScript check**

Run: `cd packages/frontend/web-app && npx tsc --noEmit`
Expected: No errors

**Step 4: No commit needed** — this is verification only

---

## Task 7: Build Verification

**Step 1: Verify Rust builds**

Run: `cd services/integration-service && cargo check`
Expected: No errors (warnings OK)

**Step 2: Verify TypeScript builds**

Run: `cd packages/frontend/web-app && npx tsc --noEmit`
Expected: No errors

**Step 3: Run Rust tests**

Run: `cd services/integration-service && cargo test`
Expected: All tests pass

**Step 4: Commit any final fixes if needed**

---

## Summary

| Task | Description | Files |
|------|-------------|-------|
| 1 | Fix run_id propagation | executor.rs |
| 2 | Fix approval return types | api.rs |
| 3 | Add approval resumption | api.rs |
| 4 | Create playbook handler structure | handler.go, main.go |
| 5 | Implement webhook handler | webhook.go |
| 6 | Verify TypeScript types | (verification only) |
| 7 | Build verification | (verification only) |

**End-to-end test after all tasks:**

1. Create a playbook with a single webhook step that calls `https://httpbin.org/post`
2. Run the playbook
3. Verify the run transitions through: `pending` → `running` → `completed`
4. Check the step output contains the HTTP response

**For playbooks with approval steps:**

1. Create a playbook: webhook → approval → webhook
2. Run the playbook
3. First webhook executes
4. Run pauses at `waiting_approval`
5. Approve the run via API
6. Second webhook executes
7. Run completes
