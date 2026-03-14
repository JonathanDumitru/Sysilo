package config

import (
	"fmt"
	"os"
	"strconv"

	"gopkg.in/yaml.v3"
)

// Config holds all MCP server configuration.
type Config struct {
	Server    ServerConfig    `yaml:"server"`
	Auth      AuthConfig      `yaml:"auth"`
	Services  ServicesConfig  `yaml:"services"`
	RateLimit RateLimitConfig `yaml:"rate_limit"`
	Logging   LoggingConfig   `yaml:"logging"`
}

// ServerConfig holds HTTP server settings.
type ServerConfig struct {
	Address string `yaml:"address"`
}

// AuthConfig holds JWT authentication settings.
type AuthConfig struct {
	JWTSecret      string   `yaml:"jwt_secret"`
	JWTIssuer      string   `yaml:"jwt_issuer"`
	AllowedIssuers []string `yaml:"allowed_issuers"`
}

// ServicesConfig holds addresses for backend Sysilo services.
type ServicesConfig struct {
	IntegrationService    string `yaml:"integration_service"`
	DataService           string `yaml:"data_service"`
	AssetService          string `yaml:"asset_service"`
	OpsService            string `yaml:"ops_service"`
	GovernanceService     string `yaml:"governance_service"`
	RationalizationService string `yaml:"rationalization_service"`
	AIService             string `yaml:"ai_service"`
}

// RateLimitConfig holds per-tenant rate limiting settings for MCP calls.
type RateLimitConfig struct {
	Enabled        bool `yaml:"enabled"`
	RequestsPerMin int  `yaml:"requests_per_minute"`
	BurstSize      int  `yaml:"burst_size"`
}

// LoggingConfig holds structured logging settings.
type LoggingConfig struct {
	Level  string `yaml:"level"`
	Format string `yaml:"format"`
}

// Default returns a configuration with sensible defaults.
func Default() *Config {
	return &Config{
		Server: ServerConfig{
			Address: ":8091",
		},
		Auth: AuthConfig{
			JWTSecret: "dev-secret-change-in-production",
			JWTIssuer: "sysilo",
		},
		Services: ServicesConfig{
			IntegrationService:    "http://localhost:8081",
			DataService:           "http://localhost:8083",
			AssetService:          "http://localhost:8084",
			OpsService:            "http://localhost:8085",
			GovernanceService:     "http://localhost:8086",
			RationalizationService: "http://localhost:8087",
			AIService:             "http://localhost:8090",
		},
		RateLimit: RateLimitConfig{
			Enabled:        true,
			RequestsPerMin: 300,
			BurstSize:      20,
		},
		Logging: LoggingConfig{
			Level:  "info",
			Format: "json",
		},
	}
}

// Load reads configuration from a YAML file, falling back to defaults.
func Load(path string) (*Config, error) {
	cfg := Default()

	if path == "" {
		candidates := []string{
			"./mcp-server.yaml",
			"./config/mcp-server.yaml",
			"/etc/sysilo/mcp-server.yaml",
		}
		for _, p := range candidates {
			if _, err := os.Stat(p); err == nil {
				path = p
				break
			}
		}
	}

	if path != "" {
		data, err := os.ReadFile(path)
		if err != nil {
			return nil, fmt.Errorf("failed to read config file: %w", err)
		}
		if err := yaml.Unmarshal(data, cfg); err != nil {
			return nil, fmt.Errorf("failed to parse config file: %w", err)
		}
	}

	cfg.applyEnvOverrides()
	return cfg, nil
}

func (c *Config) applyEnvOverrides() {
	if v := os.Getenv("MCP_SERVER_ADDRESS"); v != "" {
		c.Server.Address = v
	}
	if v := os.Getenv("SYSILO_JWT_SECRET"); v != "" {
		c.Auth.JWTSecret = v
	}
	if v := os.Getenv("MCP_LOG_LEVEL"); v != "" {
		c.Logging.Level = v
	}

	// Service address overrides
	if v := os.Getenv("SYSILO_INTEGRATION_SERVICE_URL"); v != "" {
		c.Services.IntegrationService = v
	}
	if v := os.Getenv("SYSILO_DATA_SERVICE_URL"); v != "" {
		c.Services.DataService = v
	}
	if v := os.Getenv("SYSILO_ASSET_SERVICE_URL"); v != "" {
		c.Services.AssetService = v
	}
	if v := os.Getenv("SYSILO_OPS_SERVICE_URL"); v != "" {
		c.Services.OpsService = v
	}
	if v := os.Getenv("SYSILO_GOVERNANCE_SERVICE_URL"); v != "" {
		c.Services.GovernanceService = v
	}
	if v := os.Getenv("SYSILO_RATIONALIZATION_SERVICE_URL"); v != "" {
		c.Services.RationalizationService = v
	}
	if v := os.Getenv("SYSILO_AI_SERVICE_URL"); v != "" {
		c.Services.AIService = v
	}

	// Rate limit overrides
	if v := os.Getenv("MCP_RATE_LIMIT_ENABLED"); v != "" {
		if b, err := strconv.ParseBool(v); err == nil {
			c.RateLimit.Enabled = b
		}
	}
	if v := os.Getenv("MCP_RATE_LIMIT_RPM"); v != "" {
		if n, err := strconv.Atoi(v); err == nil {
			c.RateLimit.RequestsPerMin = n
		}
	}
}
