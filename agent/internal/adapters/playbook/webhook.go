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
	return &WebhookHandler{logger: logger.Named("webhook")}
}

// Execute makes an HTTP request based on the step config
func (h *WebhookHandler) Execute(ctx context.Context, config *StepConfig) (*StepResult, error) {
	// TODO: Implement in Task 5
	return nil, fmt.Errorf("webhook handler not yet implemented")
}
