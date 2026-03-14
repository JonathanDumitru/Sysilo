package resources

import (
	"context"
	"encoding/json"
	"fmt"
	"io"
	"log/slog"
	"net/http"
	"strings"
	"time"

	"github.com/sysilo/sysilo/services/mcp-server/internal/config"
	"github.com/sysilo/sysilo/services/mcp-server/internal/mcp"
)

// Provider exposes Sysilo data catalog and asset registry entries as MCP resources.
type Provider struct {
	dataServiceURL  string
	assetServiceURL string
	httpClient      *http.Client
	logger          *slog.Logger
}

// NewProvider creates a new resource provider.
func NewProvider(cfg config.ServicesConfig, logger *slog.Logger) *Provider {
	return &Provider{
		dataServiceURL:  cfg.DataService,
		assetServiceURL: cfg.AssetService,
		httpClient: &http.Client{
			Timeout: 15 * time.Second,
		},
		logger: logger,
	}
}

// catalogEntry represents an entity from the data catalog service.
type catalogEntry struct {
	ID          string `json:"id"`
	Name        string `json:"name"`
	Description string `json:"description"`
	Type        string `json:"type"`
}

// assetEntry represents an entity from the asset registry service.
type assetEntry struct {
	ID          string `json:"id"`
	Name        string `json:"name"`
	Description string `json:"description"`
	Type        string `json:"type"`
	Status      string `json:"status"`
}

// ListResources returns all available MCP resources by querying the data catalog
// and asset registry services.
func (p *Provider) ListResources(ctx context.Context, tenantID string) ([]mcp.Resource, error) {
	var resources []mcp.Resource

	// Fetch catalog entities.
	catalogResources, err := p.fetchCatalogResources(ctx, tenantID)
	if err != nil {
		p.logger.WarnContext(ctx, "failed to fetch catalog resources",
			slog.String("error", err.Error()),
			slog.String("tenant_id", tenantID),
		)
		// Continue with partial results rather than failing entirely.
	} else {
		resources = append(resources, catalogResources...)
	}

	// Fetch asset registry entries.
	assetResources, err := p.fetchAssetResources(ctx, tenantID)
	if err != nil {
		p.logger.WarnContext(ctx, "failed to fetch asset resources",
			slog.String("error", err.Error()),
			slog.String("tenant_id", tenantID),
		)
	} else {
		resources = append(resources, assetResources...)
	}

	return resources, nil
}

// ReadResource reads a single resource by its MCP URI.
// URI scheme: sysilo://catalog/{entity_id} or sysilo://assets/{asset_id}
func (p *Provider) ReadResource(ctx context.Context, uri, tenantID string) (*mcp.ReadResourceResult, error) {
	// Parse the sysilo:// URI.
	resourceType, resourceID, err := parseResourceURI(uri)
	if err != nil {
		return nil, err
	}

	switch resourceType {
	case "catalog":
		return p.readCatalogResource(ctx, resourceID, tenantID)
	case "assets":
		return p.readAssetResource(ctx, resourceID, tenantID)
	default:
		return nil, fmt.Errorf("unknown resource type: %s", resourceType)
	}
}

func (p *Provider) fetchCatalogResources(ctx context.Context, tenantID string) ([]mcp.Resource, error) {
	url := fmt.Sprintf("%s/catalog/entities", strings.TrimRight(p.dataServiceURL, "/"))

	body, err := p.doGet(ctx, url, tenantID)
	if err != nil {
		return nil, err
	}

	var entries []catalogEntry
	if err := json.Unmarshal(body, &entries); err != nil {
		// Try unwrapping from a data envelope.
		var envelope struct {
			Data []catalogEntry `json:"data"`
		}
		if err2 := json.Unmarshal(body, &envelope); err2 != nil {
			return nil, fmt.Errorf("decode catalog response: %w", err)
		}
		entries = envelope.Data
	}

	resources := make([]mcp.Resource, 0, len(entries))
	for _, e := range entries {
		resources = append(resources, mcp.Resource{
			URI:         fmt.Sprintf("sysilo://catalog/%s", e.ID),
			Name:        e.Name,
			Description: fmt.Sprintf("[%s] %s", e.Type, e.Description),
			MIMEType:    "application/json",
		})
	}
	return resources, nil
}

func (p *Provider) fetchAssetResources(ctx context.Context, tenantID string) ([]mcp.Resource, error) {
	url := fmt.Sprintf("%s/assets", strings.TrimRight(p.assetServiceURL, "/"))

	body, err := p.doGet(ctx, url, tenantID)
	if err != nil {
		return nil, err
	}

	var entries []assetEntry
	if err := json.Unmarshal(body, &entries); err != nil {
		var envelope struct {
			Data []assetEntry `json:"data"`
		}
		if err2 := json.Unmarshal(body, &envelope); err2 != nil {
			return nil, fmt.Errorf("decode asset response: %w", err)
		}
		entries = envelope.Data
	}

	resources := make([]mcp.Resource, 0, len(entries))
	for _, e := range entries {
		desc := e.Description
		if e.Status != "" {
			desc = fmt.Sprintf("[%s] [%s] %s", e.Type, e.Status, e.Description)
		}
		resources = append(resources, mcp.Resource{
			URI:         fmt.Sprintf("sysilo://assets/%s", e.ID),
			Name:        e.Name,
			Description: desc,
			MIMEType:    "application/json",
		})
	}
	return resources, nil
}

func (p *Provider) readCatalogResource(ctx context.Context, entityID, tenantID string) (*mcp.ReadResourceResult, error) {
	url := fmt.Sprintf("%s/catalog/entities/%s", strings.TrimRight(p.dataServiceURL, "/"), entityID)

	body, err := p.doGet(ctx, url, tenantID)
	if err != nil {
		return nil, err
	}

	return &mcp.ReadResourceResult{
		Contents: []mcp.ResourceContent{
			{
				URI:      fmt.Sprintf("sysilo://catalog/%s", entityID),
				MIMEType: "application/json",
				Text:     string(body),
			},
		},
	}, nil
}

func (p *Provider) readAssetResource(ctx context.Context, assetID, tenantID string) (*mcp.ReadResourceResult, error) {
	url := fmt.Sprintf("%s/assets/%s", strings.TrimRight(p.assetServiceURL, "/"), assetID)

	body, err := p.doGet(ctx, url, tenantID)
	if err != nil {
		return nil, err
	}

	return &mcp.ReadResourceResult{
		Contents: []mcp.ResourceContent{
			{
				URI:      fmt.Sprintf("sysilo://assets/%s", assetID),
				MIMEType: "application/json",
				Text:     string(body),
			},
		},
	}, nil
}

func (p *Provider) doGet(ctx context.Context, url, tenantID string) ([]byte, error) {
	req, err := http.NewRequestWithContext(ctx, http.MethodGet, url, nil)
	if err != nil {
		return nil, fmt.Errorf("create request: %w", err)
	}
	req.Header.Set("Accept", "application/json")
	req.Header.Set("X-Tenant-ID", tenantID)

	resp, err := p.httpClient.Do(req)
	if err != nil {
		return nil, fmt.Errorf("fetch %s: %w", url, err)
	}
	defer resp.Body.Close()

	body, err := io.ReadAll(io.LimitReader(resp.Body, 5*1024*1024))
	if err != nil {
		return nil, fmt.Errorf("read response: %w", err)
	}

	if resp.StatusCode >= 400 {
		return nil, fmt.Errorf("service returned status %d: %s", resp.StatusCode, string(body))
	}

	return body, nil
}

// parseResourceURI extracts the resource type and ID from a sysilo:// URI.
func parseResourceURI(uri string) (resourceType, resourceID string, err error) {
	const prefix = "sysilo://"
	if !strings.HasPrefix(uri, prefix) {
		return "", "", fmt.Errorf("invalid resource URI scheme: %s (expected sysilo://...)", uri)
	}

	path := strings.TrimPrefix(uri, prefix)
	parts := strings.SplitN(path, "/", 2)
	if len(parts) != 2 || parts[0] == "" || parts[1] == "" {
		return "", "", fmt.Errorf("invalid resource URI format: %s (expected sysilo://<type>/<id>)", uri)
	}

	return parts[0], parts[1], nil
}
