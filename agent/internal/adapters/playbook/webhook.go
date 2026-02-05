package playbook

import (
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
		logger: logger.Named("webhook"),
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
