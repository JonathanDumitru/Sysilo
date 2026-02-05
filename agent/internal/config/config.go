package config

import (
	"fmt"
	"os"

	"gopkg.in/yaml.v3"
)

// Config holds all agent configuration
type Config struct {
	Agent   AgentConfig   `yaml:"agent"`
	Gateway GatewayConfig `yaml:"gateway"`
	TLS     TLSConfig     `yaml:"tls"`
	Logging LoggingConfig `yaml:"logging"`
}

// AgentConfig holds agent-specific settings
type AgentConfig struct {
	ID                 string            `yaml:"id"`
	Name               string            `yaml:"name"`
	TenantID           string            `yaml:"tenant_id"`
	MaxConcurrentTasks int               `yaml:"max_concurrent_tasks"`
	Labels             map[string]string `yaml:"labels"`
}

// GatewayConfig holds connection settings to the control plane
type GatewayConfig struct {
	Address           string `yaml:"address"`
	ReconnectInterval int    `yaml:"reconnect_interval_seconds"`
	HeartbeatInterval int    `yaml:"heartbeat_interval_seconds"`
}

// TLSConfig holds TLS/mTLS settings
type TLSConfig struct {
	Enabled    bool   `yaml:"enabled"`
	CertFile   string `yaml:"cert_file"`
	KeyFile    string `yaml:"key_file"`
	CACertFile string `yaml:"ca_cert_file"`
	ServerName string `yaml:"server_name"`
}

// LoggingConfig holds logging settings
type LoggingConfig struct {
	Level  string `yaml:"level"`
	Format string `yaml:"format"`
}

// Default returns a configuration with sensible defaults
func Default() *Config {
	return &Config{
		Agent: AgentConfig{
			ID:                 "",
			Name:               "sysilo-agent",
			TenantID:           "",
			MaxConcurrentTasks: 10,
			Labels:             make(map[string]string),
		},
		Gateway: GatewayConfig{
			Address:           "localhost:9090",
			ReconnectInterval: 5,
			HeartbeatInterval: 30,
		},
		TLS: TLSConfig{
			Enabled: false,
		},
		Logging: LoggingConfig{
			Level:  "info",
			Format: "json",
		},
	}
}

// Load reads configuration from a YAML file, falling back to defaults
func Load(path string) (*Config, error) {
	cfg := Default()

	if path == "" {
		// Try default locations
		defaultPaths := []string{
			"./agent.yaml",
			"./config/agent.yaml",
			"/etc/sysilo/agent.yaml",
		}
		for _, p := range defaultPaths {
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

	// Override with environment variables
	cfg.applyEnvOverrides()

	// Validate configuration
	if err := cfg.Validate(); err != nil {
		return nil, fmt.Errorf("invalid configuration: %w", err)
	}

	return cfg, nil
}

// applyEnvOverrides applies environment variable overrides
func (c *Config) applyEnvOverrides() {
	if v := os.Getenv("SYSILO_AGENT_ID"); v != "" {
		c.Agent.ID = v
	}
	if v := os.Getenv("SYSILO_TENANT_ID"); v != "" {
		c.Agent.TenantID = v
	}
	if v := os.Getenv("SYSILO_GATEWAY_ADDRESS"); v != "" {
		c.Gateway.Address = v
	}
	if v := os.Getenv("SYSILO_LOG_LEVEL"); v != "" {
		c.Logging.Level = v
	}
}

// Validate checks that required configuration is present
func (c *Config) Validate() error {
	if c.Agent.ID == "" {
		return fmt.Errorf("agent.id is required")
	}
	if c.Agent.TenantID == "" {
		return fmt.Errorf("agent.tenant_id is required")
	}
	if c.Gateway.Address == "" {
		return fmt.Errorf("gateway.address is required")
	}
	return nil
}
