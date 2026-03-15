package tools

import (
	"encoding/json"
	"fmt"

	"github.com/sysilo/sysilo/services/mcp-server/internal/mcp"
)

// ToolDefinition extends the MCP Tool with metadata used for routing and authorization.
type ToolDefinition struct {
	mcp.Tool

	// RequiredScope is the RBAC permission scope needed to invoke this tool.
	RequiredScope string

	// ServiceRoute describes how to proxy the call to a backend service.
	ServiceRoute ServiceRoute
}

// ServiceRoute describes how a tool call maps to a backend Sysilo service.
type ServiceRoute struct {
	// Service is the logical service name (integration, data, asset, ops, governance, rationalization, ai).
	Service string

	// Method is the HTTP method for the proxied request (GET, POST, etc.).
	Method string

	// PathTemplate is a Go-template-style path with {param} placeholders.
	// Parameters are resolved from the tool call arguments.
	PathTemplate string
}

// Registry holds all registered MCP tools.
type Registry struct {
	tools map[string]ToolDefinition
	order []string // preserve insertion order for listing
}

// NewRegistry creates a registry pre-populated with all Sysilo MCP tools.
func NewRegistry() *Registry {
	r := &Registry{
		tools: make(map[string]ToolDefinition),
	}
	r.registerAll()
	return r
}

// Get returns a tool definition by name.
func (r *Registry) Get(name string) (ToolDefinition, bool) {
	t, ok := r.tools[name]
	return t, ok
}

// List returns all tools as MCP Tool objects for the tools/list response.
func (r *Registry) List() []mcp.Tool {
	out := make([]mcp.Tool, 0, len(r.order))
	for _, name := range r.order {
		out = append(out, r.tools[name].Tool)
	}
	return out
}

func (r *Registry) register(def ToolDefinition) {
	r.tools[def.Name] = def
	r.order = append(r.order, def.Name)
}

func (r *Registry) registerAll() {
	r.register(ToolDefinition{
		Tool: mcp.Tool{
			Name:        "sysilo_run_integration",
			Description: "Run a named Sysilo integration pipeline. The integration must already be configured in the Sysilo console. Returns the run ID and initial status.",
			InputSchema: mustSchema(map[string]interface{}{
				"type": "object",
				"properties": map[string]interface{}{
					"integration_id": map[string]interface{}{
						"type":        "string",
						"description": "The unique identifier of the integration to execute.",
					},
					"config": map[string]interface{}{
						"type":        "object",
						"description": "Optional runtime configuration overrides for this run.",
					},
				},
				"required": []string{"integration_id"},
			}),
		},
		RequiredScope: "integrations:run",
		ServiceRoute: ServiceRoute{
			Service:      "integration",
			Method:       "POST",
			PathTemplate: "/integrations/{integration_id}/run",
		},
	})

	r.register(ToolDefinition{
		Tool: mcp.Tool{
			Name:        "sysilo_run_playbook",
			Description: "Execute an automation playbook. Playbooks are predefined operational workflows that can orchestrate multiple steps across Sysilo services.",
			InputSchema: mustSchema(map[string]interface{}{
				"type": "object",
				"properties": map[string]interface{}{
					"playbook_id": map[string]interface{}{
						"type":        "string",
						"description": "The unique identifier of the playbook to execute.",
					},
					"variables": map[string]interface{}{
						"type":        "object",
						"description": "Key-value variables to pass into the playbook execution.",
					},
				},
				"required": []string{"playbook_id"},
			}),
		},
		RequiredScope: "playbooks:run",
		ServiceRoute: ServiceRoute{
			Service:      "ops",
			Method:       "POST",
			PathTemplate: "/playbooks/{playbook_id}/run",
		},
	})

	r.register(ToolDefinition{
		Tool: mcp.Tool{
			Name:        "sysilo_query_assets",
			Description: "Search the Sysilo asset registry. Returns applications, services, databases, and infrastructure components matching the query. Supports filtering by type, owner, lifecycle status, and tags.",
			InputSchema: mustSchema(map[string]interface{}{
				"type": "object",
				"properties": map[string]interface{}{
					"query": map[string]interface{}{
						"type":        "string",
						"description": "Free-text search query for asset names, descriptions, or tags.",
					},
					"filters": map[string]interface{}{
						"type":        "object",
						"description": "Structured filters: type (application|service|database|infrastructure), owner, status (active|deprecated|retired), tags (array of strings).",
						"properties": map[string]interface{}{
							"type": map[string]interface{}{
								"type": "string",
								"enum": []string{"application", "service", "database", "infrastructure"},
							},
							"owner":  map[string]interface{}{"type": "string"},
							"status": map[string]interface{}{"type": "string", "enum": []string{"active", "deprecated", "retired"}},
							"tags":   map[string]interface{}{"type": "array", "items": map[string]interface{}{"type": "string"}},
						},
					},
				},
				"required": []string{"query"},
			}),
		},
		RequiredScope: "assets:read",
		ServiceRoute: ServiceRoute{
			Service:      "asset",
			Method:       "GET",
			PathTemplate: "/assets",
		},
	})

	r.register(ToolDefinition{
		Tool: mcp.Tool{
			Name:        "sysilo_query_lineage",
			Description: "Retrieve data lineage for a specific entity. Shows upstream sources or downstream consumers of data, tracing how data flows through the organization's systems.",
			InputSchema: mustSchema(map[string]interface{}{
				"type": "object",
				"properties": map[string]interface{}{
					"entity_id": map[string]interface{}{
						"type":        "string",
						"description": "The unique identifier of the entity to trace lineage for.",
					},
					"direction": map[string]interface{}{
						"type":        "string",
						"enum":        []string{"upstream", "downstream", "both"},
						"description": "Direction of lineage traversal. Defaults to 'both'.",
					},
					"depth": map[string]interface{}{
						"type":        "integer",
						"description": "Maximum depth of lineage hops to traverse. Defaults to 3.",
						"minimum":     1,
						"maximum":     10,
					},
				},
				"required": []string{"entity_id"},
			}),
		},
		RequiredScope: "data:read",
		ServiceRoute: ServiceRoute{
			Service:      "data",
			Method:       "GET",
			PathTemplate: "/lineage/{entity_id}",
		},
	})

	r.register(ToolDefinition{
		Tool: mcp.Tool{
			Name:        "sysilo_check_policy",
			Description: "Evaluate governance policies against a resource. Returns whether the resource is compliant, any violations found, and recommended remediation steps.",
			InputSchema: mustSchema(map[string]interface{}{
				"type": "object",
				"properties": map[string]interface{}{
					"resource_type": map[string]interface{}{
						"type":        "string",
						"description": "The type of resource to evaluate (e.g., 'database', 'api', 'application').",
					},
					"resource_data": map[string]interface{}{
						"type":        "object",
						"description": "The resource metadata to evaluate against policies.",
					},
				},
				"required": []string{"resource_type", "resource_data"},
			}),
		},
		RequiredScope: "governance:read",
		ServiceRoute: ServiceRoute{
			Service:      "governance",
			Method:       "POST",
			PathTemplate: "/policies/evaluate",
		},
	})

	r.register(ToolDefinition{
		Tool: mcp.Tool{
			Name:        "sysilo_get_metrics",
			Description: "Query operational metrics from Sysilo. Returns time-series data for infrastructure, application, or business metrics.",
			InputSchema: mustSchema(map[string]interface{}{
				"type": "object",
				"properties": map[string]interface{}{
					"metric_name": map[string]interface{}{
						"type":        "string",
						"description": "The name of the metric to query (e.g., 'cpu_usage', 'error_rate', 'request_latency').",
					},
					"time_range": map[string]interface{}{
						"type":        "object",
						"description": "Time range for the query.",
						"properties": map[string]interface{}{
							"start": map[string]interface{}{"type": "string", "format": "date-time", "description": "Start time in ISO 8601 format."},
							"end":   map[string]interface{}{"type": "string", "format": "date-time", "description": "End time in ISO 8601 format."},
						},
					},
					"filters": map[string]interface{}{
						"type":        "object",
						"description": "Additional filters for the metric query (e.g., service name, environment).",
					},
				},
				"required": []string{"metric_name"},
			}),
		},
		RequiredScope: "ops:read",
		ServiceRoute: ServiceRoute{
			Service:      "ops",
			Method:       "GET",
			PathTemplate: "/metrics",
		},
	})

	r.register(ToolDefinition{
		Tool: mcp.Tool{
			Name:        "sysilo_analyze_portfolio",
			Description: "Get a TIME (Tolerate, Invest, Migrate, Eliminate) rationalization analysis of the application portfolio. Returns categorized applications with recommendations and effort estimates.",
			InputSchema: mustSchema(map[string]interface{}{
				"type": "object",
				"properties": map[string]interface{}{
					"filters": map[string]interface{}{
						"type":        "object",
						"description": "Filters to narrow the portfolio analysis.",
						"properties": map[string]interface{}{
							"business_unit": map[string]interface{}{"type": "string"},
							"category":      map[string]interface{}{"type": "string", "enum": []string{"tolerate", "invest", "migrate", "eliminate"}},
							"min_score":     map[string]interface{}{"type": "number", "description": "Minimum rationalization score (0-100)."},
						},
					},
				},
			}),
		},
		RequiredScope: "rationalization:read",
		ServiceRoute: ServiceRoute{
			Service:      "rationalization",
			Method:       "GET",
			PathTemplate: "/analysis",
		},
	})

	r.register(ToolDefinition{
		Tool: mcp.Tool{
			Name:        "sysilo_ai_chat",
			Description: "Chat with Sysilo's built-in AI assistant. The AI has context about the tenant's data, integrations, and infrastructure. Use this for natural-language questions about the tenant's Sysilo environment.",
			InputSchema: mustSchema(map[string]interface{}{
				"type": "object",
				"properties": map[string]interface{}{
					"message": map[string]interface{}{
						"type":        "string",
						"description": "The message or question to send to the AI assistant.",
					},
					"context": map[string]interface{}{
						"type":        "object",
						"description": "Additional context to provide to the AI (e.g., related entity IDs, previous conversation state).",
					},
				},
				"required": []string{"message"},
			}),
		},
		RequiredScope: "ai:chat",
		ServiceRoute: ServiceRoute{
			Service:      "ai",
			Method:       "POST",
			PathTemplate: "/chat",
		},
	})

	r.register(ToolDefinition{
		Tool: mcp.Tool{
			Name:        "sysilo_create_connection",
			Description: "Create a new data source connection in Sysilo. Connections are reusable configurations for databases, APIs, SaaS tools, and cloud services that integrations use to move and transform data.",
			InputSchema: mustSchema(map[string]interface{}{
				"type": "object",
				"properties": map[string]interface{}{
					"connector_type": map[string]interface{}{
						"type":        "string",
						"description": "The type of connector (e.g., 'postgres', 'mysql', 'snowflake', 'salesforce', 's3', 'rest_api').",
					},
					"config": map[string]interface{}{
						"type":        "object",
						"description": "Connection configuration including host, credentials, and connector-specific settings. Sensitive values will be encrypted at rest.",
					},
					"name": map[string]interface{}{
						"type":        "string",
						"description": "A human-readable name for the connection.",
					},
				},
				"required": []string{"connector_type", "config", "name"},
			}),
		},
		RequiredScope: "connections:write",
		ServiceRoute: ServiceRoute{
			Service:      "integration",
			Method:       "POST",
			PathTemplate: "/connections",
		},
	})

	r.register(ToolDefinition{
		Tool: mcp.Tool{
			Name:        "sysilo_discover_schema",
			Description: "Discover the schema of a data source through an existing connection. Returns tables, columns, data types, and relationships for database connections, or endpoint definitions for API connections.",
			InputSchema: mustSchema(map[string]interface{}{
				"type": "object",
				"properties": map[string]interface{}{
					"connection_id": map[string]interface{}{
						"type":        "string",
						"description": "The unique identifier of the connection to discover schema from.",
					},
				},
				"required": []string{"connection_id"},
			}),
		},
		RequiredScope: "connections:read",
		ServiceRoute: ServiceRoute{
			Service:      "integration",
			Method:       "POST",
			PathTemplate: "/connections/{connection_id}/discover",
		},
	})
}

// mustSchema marshals a Go map into JSON for use as a JSON Schema.
// Panics on marshal failure (only during init, so programming errors are caught immediately).
func mustSchema(v interface{}) json.RawMessage {
	b, err := json.Marshal(v)
	if err != nil {
		panic(fmt.Sprintf("tools: failed to marshal schema: %v", err))
	}
	return b
}
