package client

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"log/slog"
	"net/http"
	"net/url"
	"strings"
	"time"

	"github.com/sysilo/sysilo/services/mcp-server/internal/config"
	"github.com/sysilo/sysilo/services/mcp-server/internal/tools"
)

// ServiceClient proxies MCP tool calls to the appropriate backend Sysilo service.
type ServiceClient struct {
	httpClient *http.Client
	services   map[string]string // service name -> base URL
	logger     *slog.Logger
}

// NewServiceClient creates a new client with service address mappings.
func NewServiceClient(cfg config.ServicesConfig, logger *slog.Logger) *ServiceClient {
	return &ServiceClient{
		httpClient: &http.Client{
			Timeout: 30 * time.Second,
		},
		services: map[string]string{
			"integration":    cfg.IntegrationService,
			"data":           cfg.DataService,
			"asset":          cfg.AssetService,
			"ops":            cfg.OpsService,
			"governance":     cfg.GovernanceService,
			"rationalization": cfg.RationalizationService,
			"ai":             cfg.AIService,
		},
		logger: logger,
	}
}

// CallContext carries tenant and tracing context for a proxied call.
type CallContext struct {
	TenantID  string
	UserID    string
	RequestID string
}

// CallResult holds the raw response from a backend service.
type CallResult struct {
	StatusCode int
	Body       json.RawMessage
}

// Call executes a tool call by routing it to the appropriate backend service.
func (c *ServiceClient) Call(ctx context.Context, route tools.ServiceRoute, args map[string]interface{}, cc CallContext) (*CallResult, error) {
	baseURL, ok := c.services[route.Service]
	if !ok {
		return nil, fmt.Errorf("unknown service: %s", route.Service)
	}

	// Resolve path template with argument values.
	path := resolvePathTemplate(route.PathTemplate, args)

	// Build the full URL.
	fullURL := strings.TrimRight(baseURL, "/") + path

	// For GET requests, add remaining args as query parameters.
	// For POST/PUT, send args as JSON body.
	var body io.Reader
	if route.Method == http.MethodGet || route.Method == http.MethodHead {
		fullURL = appendQueryParams(fullURL, args, route.PathTemplate)
	} else {
		jsonBody, err := json.Marshal(args)
		if err != nil {
			return nil, fmt.Errorf("marshal request body: %w", err)
		}
		body = bytes.NewReader(jsonBody)
	}

	req, err := http.NewRequestWithContext(ctx, route.Method, fullURL, body)
	if err != nil {
		return nil, fmt.Errorf("create request: %w", err)
	}

	// Set standard headers for multi-tenant isolation and tracing.
	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("Accept", "application/json")
	req.Header.Set("X-Tenant-ID", cc.TenantID)
	req.Header.Set("X-MCP-Request-ID", cc.RequestID)
	if cc.UserID != "" {
		req.Header.Set("X-User-ID", cc.UserID)
	}

	c.logger.InfoContext(ctx, "proxying tool call to backend",
		slog.String("service", route.Service),
		slog.String("method", route.Method),
		slog.String("url", fullURL),
		slog.String("tenant_id", cc.TenantID),
		slog.String("request_id", cc.RequestID),
	)

	resp, err := c.httpClient.Do(req)
	if err != nil {
		return nil, fmt.Errorf("call %s service: %w", route.Service, err)
	}
	defer resp.Body.Close()

	respBody, err := io.ReadAll(io.LimitReader(resp.Body, 10*1024*1024)) // 10 MB limit
	if err != nil {
		return nil, fmt.Errorf("read response from %s: %w", route.Service, err)
	}

	return &CallResult{
		StatusCode: resp.StatusCode,
		Body:       json.RawMessage(respBody),
	}, nil
}

// resolvePathTemplate replaces {param} placeholders in the path with values from args.
// Used parameters are removed from args so they are not duplicated in query strings or bodies.
func resolvePathTemplate(tmpl string, args map[string]interface{}) string {
	result := tmpl
	for key, val := range args {
		placeholder := "{" + key + "}"
		if strings.Contains(result, placeholder) {
			strVal := fmt.Sprintf("%v", val)
			result = strings.ReplaceAll(result, placeholder, url.PathEscape(strVal))
			delete(args, key)
		}
	}
	return result
}

// appendQueryParams adds remaining args as URL query parameters for GET requests.
// It skips arguments that were already consumed as path parameters.
func appendQueryParams(baseURL string, args map[string]interface{}, pathTemplate string) string {
	if len(args) == 0 {
		return baseURL
	}

	params := url.Values{}
	for key, val := range args {
		switch v := val.(type) {
		case map[string]interface{}:
			// Serialize nested objects as JSON query param.
			b, err := json.Marshal(v)
			if err == nil {
				params.Set(key, string(b))
			}
		case []interface{}:
			for _, item := range v {
				params.Add(key, fmt.Sprintf("%v", item))
			}
		default:
			params.Set(key, fmt.Sprintf("%v", v))
		}
	}

	separator := "?"
	if strings.Contains(baseURL, "?") {
		separator = "&"
	}
	return baseURL + separator + params.Encode()
}
