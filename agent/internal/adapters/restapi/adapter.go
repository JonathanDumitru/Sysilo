package restapi

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"strings"
	"time"

	"go.uber.org/zap"

	"github.com/sysilo/sysilo/agent/internal/executor"
)

// Adapter handles generic REST/HTTP API operations
type Adapter struct {
	logger *zap.Logger
}

// NewAdapter creates a new REST API adapter
func NewAdapter(logger *zap.Logger) *Adapter {
	return &Adapter{
		logger: logger.Named("rest_api"),
	}
}

// Type returns the adapter type identifier
func (a *Adapter) Type() string {
	return "rest_api"
}

// Execute runs a REST API task
func (a *Adapter) Execute(ctx context.Context, task *executor.Task) (*executor.TaskResult, error) {
	config, err := parseConfig(task.Config)
	if err != nil {
		return nil, fmt.Errorf("invalid task config: %w", err)
	}

	a.logger.Info("Executing REST API task",
		zap.String("task_id", task.ID),
		zap.String("operation", config.Operation),
	)

	switch config.Operation {
	case "request":
		return a.executeRequest(ctx, config)
	case "health_check":
		return a.healthCheck(ctx, config)
	default:
		return nil, fmt.Errorf("unknown operation: %s", config.Operation)
	}
}

// TaskConfig holds the configuration for a REST API task
type TaskConfig struct {
	Operation  string           `json:"operation"`
	Connection ConnectionConfig `json:"connection"`
	Request    RequestConfig    `json:"request"`
}

// ConnectionConfig holds base connection details
type ConnectionConfig struct {
	BaseURL string            `json:"base_url"`
	Headers map[string]string `json:"headers"`
	Auth    AuthConfig        `json:"auth"`
}

// AuthConfig holds authentication configuration
type AuthConfig struct {
	Type          string `json:"type"` // "none", "api_key", "bearer", "basic", "oauth2"
	ApiKey        string `json:"api_key"`
	ApiKeyHeader  string `json:"api_key_header"`
	BearerToken   string `json:"bearer_token"`
	BasicUser     string `json:"basic_user"`
	BasicPassword string `json:"basic_password"`
}

// RequestConfig holds individual request configuration
type RequestConfig struct {
	Method         string            `json:"method"`
	Path           string            `json:"path"`
	QueryParams    map[string]string `json:"query_params"`
	Body           interface{}       `json:"body"`
	Headers        map[string]string `json:"headers"`
	TimeoutSeconds int               `json:"timeout_seconds"`
}

func parseConfig(raw map[string]interface{}) (*TaskConfig, error) {
	data, err := json.Marshal(raw)
	if err != nil {
		return nil, err
	}

	var config TaskConfig
	if err := json.Unmarshal(data, &config); err != nil {
		return nil, err
	}

	// Defaults
	if config.Request.Method == "" {
		config.Request.Method = "GET"
	}
	if config.Request.TimeoutSeconds == 0 {
		config.Request.TimeoutSeconds = 30
	}
	if config.Connection.Auth.Type == "" {
		config.Connection.Auth.Type = "none"
	}
	if config.Connection.Auth.ApiKeyHeader == "" {
		config.Connection.Auth.ApiKeyHeader = "X-API-Key"
	}

	return &config, nil
}

func (a *Adapter) executeRequest(ctx context.Context, config *TaskConfig) (*executor.TaskResult, error) {
	// Build URL
	reqURL, err := buildURL(config.Connection.BaseURL, config.Request.Path, config.Request.QueryParams)
	if err != nil {
		return nil, fmt.Errorf("failed to build URL: %w", err)
	}

	// Marshal body if present
	var bodyReader io.Reader
	if config.Request.Body != nil {
		bodyBytes, err := json.Marshal(config.Request.Body)
		if err != nil {
			return nil, fmt.Errorf("failed to marshal request body: %w", err)
		}
		bodyReader = bytes.NewReader(bodyBytes)
	}

	// Create HTTP request
	req, err := http.NewRequestWithContext(ctx, strings.ToUpper(config.Request.Method), reqURL, bodyReader)
	if err != nil {
		return nil, fmt.Errorf("failed to create request: %w", err)
	}

	// Set default content type for requests with body
	if config.Request.Body != nil {
		req.Header.Set("Content-Type", "application/json")
	}

	// Apply connection-level headers
	for k, v := range config.Connection.Headers {
		req.Header.Set(k, v)
	}

	// Apply request-level headers (override connection headers)
	for k, v := range config.Request.Headers {
		req.Header.Set(k, v)
	}

	// Apply authentication
	applyAuth(req, config.Connection.Auth)

	// Execute request with timeout
	client := &http.Client{
		Timeout: time.Duration(config.Request.TimeoutSeconds) * time.Second,
	}

	start := time.Now()
	resp, err := client.Do(req)
	if err != nil {
		return nil, fmt.Errorf("request failed: %w", err)
	}
	defer resp.Body.Close()
	latency := time.Since(start)

	// Read response body
	respBody, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, fmt.Errorf("failed to read response body: %w", err)
	}

	// Parse response body (try JSON first, fallback to string)
	var parsedBody interface{}
	if err := json.Unmarshal(respBody, &parsedBody); err != nil {
		// Not valid JSON, use as string
		parsedBody = string(respBody)
	}

	// Collect response headers
	respHeaders := make(map[string]string)
	for k := range resp.Header {
		respHeaders[k] = resp.Header.Get(k)
	}

	output := map[string]interface{}{
		"status_code": resp.StatusCode,
		"headers":     respHeaders,
		"body":        parsedBody,
		"latency_ms":  latency.Milliseconds(),
	}

	bytesProcessed := int64(len(respBody))

	return &executor.TaskResult{
		Output: output,
		Metrics: executor.TaskMetrics{
			BytesProcessed: bytesProcessed,
		},
	}, nil
}

func (a *Adapter) healthCheck(ctx context.Context, config *TaskConfig) (*executor.TaskResult, error) {
	client := &http.Client{
		Timeout: 5 * time.Second,
	}

	req, err := http.NewRequestWithContext(ctx, "GET", config.Connection.BaseURL, nil)
	if err != nil {
		return nil, fmt.Errorf("failed to create health check request: %w", err)
	}

	// Apply connection-level headers
	for k, v := range config.Connection.Headers {
		req.Header.Set(k, v)
	}

	// Apply authentication
	applyAuth(req, config.Connection.Auth)

	start := time.Now()
	resp, err := client.Do(req)
	if err != nil {
		return nil, fmt.Errorf("health check failed: %w", err)
	}
	defer resp.Body.Close()
	latency := time.Since(start)

	healthy := resp.StatusCode >= 200 && resp.StatusCode < 300

	output := map[string]interface{}{
		"healthy":     healthy,
		"status_code": resp.StatusCode,
		"latency_ms":  latency.Milliseconds(),
	}

	return &executor.TaskResult{
		Output: output,
	}, nil
}

// buildURL constructs a full URL from base URL, path, and query parameters
func buildURL(baseURL, path string, queryParams map[string]string) (string, error) {
	u, err := url.Parse(baseURL)
	if err != nil {
		return "", fmt.Errorf("invalid base URL: %w", err)
	}

	// Append path
	if path != "" {
		// Ensure proper path joining
		basePath := strings.TrimRight(u.Path, "/")
		cleanPath := strings.TrimLeft(path, "/")
		u.Path = basePath + "/" + cleanPath
	}

	// Add query parameters
	if len(queryParams) > 0 {
		q := u.Query()
		for k, v := range queryParams {
			q.Set(k, v)
		}
		u.RawQuery = q.Encode()
	}

	return u.String(), nil
}

// applyAuth applies authentication to the HTTP request based on auth config
func applyAuth(req *http.Request, auth AuthConfig) {
	switch auth.Type {
	case "api_key":
		req.Header.Set(auth.ApiKeyHeader, auth.ApiKey)
	case "bearer":
		req.Header.Set("Authorization", "Bearer "+auth.BearerToken)
	case "basic":
		req.SetBasicAuth(auth.BasicUser, auth.BasicPassword)
	case "oauth2":
		// OAuth2 uses bearer token format
		req.Header.Set("Authorization", "Bearer "+auth.BearerToken)
	case "none":
		// No authentication
	}
}

// extractJSONPath performs simple dot-notation extraction from a parsed JSON value.
// For example, "data.items" would extract obj["data"]["items"].
func extractJSONPath(data interface{}, path string) (interface{}, error) {
	if path == "" {
		return data, nil
	}

	parts := strings.Split(path, ".")
	current := data

	for _, part := range parts {
		m, ok := current.(map[string]interface{})
		if !ok {
			return nil, fmt.Errorf("cannot traverse into non-object at key %q", part)
		}
		val, exists := m[part]
		if !exists {
			return nil, fmt.Errorf("key %q not found", part)
		}
		current = val
	}

	return current, nil
}
