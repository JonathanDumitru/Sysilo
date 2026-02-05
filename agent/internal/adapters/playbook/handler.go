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
		logger:      logger.Named("playbook"),
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
