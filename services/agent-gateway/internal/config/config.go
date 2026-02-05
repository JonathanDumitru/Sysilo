package config

import (
	"fmt"
	"os"

	"gopkg.in/yaml.v3"
)

// Config holds all agent gateway configuration
type Config struct {
	Server  ServerConfig  `yaml:"server"`
	TLS     TLSConfig     `yaml:"tls"`
	Kafka   KafkaConfig   `yaml:"kafka"`
	Logging LoggingConfig `yaml:"logging"`
}

// KafkaConfig holds Kafka connection settings
type KafkaConfig struct {
	Enabled         bool   `yaml:"enabled"`
	Brokers         string `yaml:"brokers"`
	TaskResultTopic string `yaml:"task_result_topic"`
	LogsTopic       string `yaml:"logs_topic"`
	GroupID         string `yaml:"group_id"`
}

// ServerConfig holds server settings
type ServerConfig struct {
	Address              string `yaml:"address"`
	MaxConnectionsPerTenant int    `yaml:"max_connections_per_tenant"`
	HeartbeatTimeout     int    `yaml:"heartbeat_timeout_seconds"`
}

// TLSConfig holds TLS/mTLS settings
type TLSConfig struct {
	Enabled    bool   `yaml:"enabled"`
	CertFile   string `yaml:"cert_file"`
	KeyFile    string `yaml:"key_file"`
	CACertFile string `yaml:"ca_cert_file"`
}

// LoggingConfig holds logging settings
type LoggingConfig struct {
	Level  string `yaml:"level"`
	Format string `yaml:"format"`
}

// Default returns a configuration with sensible defaults
func Default() *Config {
	return &Config{
		Server: ServerConfig{
			Address:                 ":9090",
			MaxConnectionsPerTenant: 100,
			HeartbeatTimeout:        90,
		},
		TLS: TLSConfig{
			Enabled: false,
		},
		Kafka: KafkaConfig{
			Enabled:         true,
			Brokers:         "localhost:9092",
			TaskResultTopic: "sysilo.results",
			LogsTopic:       "sysilo.logs",
			GroupID:         "agent-gateway",
		},
		Logging: LoggingConfig{
			Level:  "info",
			Format: "json",
		},
	}
}

// Load reads configuration from a YAML file
func Load(path string) (*Config, error) {
	cfg := Default()

	if path == "" {
		// Try default locations
		defaultPaths := []string{
			"./agent-gateway.yaml",
			"./config/agent-gateway.yaml",
			"/etc/sysilo/agent-gateway.yaml",
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

	return cfg, nil
}

// applyEnvOverrides applies environment variable overrides
func (c *Config) applyEnvOverrides() {
	if v := os.Getenv("SYSILO_GATEWAY_ADDRESS"); v != "" {
		c.Server.Address = v
	}
	if v := os.Getenv("SYSILO_LOG_LEVEL"); v != "" {
		c.Logging.Level = v
	}
	if v := os.Getenv("KAFKA_BROKERS"); v != "" {
		c.Kafka.Brokers = v
	}
	if v := os.Getenv("KAFKA_ENABLED"); v == "false" || v == "0" {
		c.Kafka.Enabled = false
	}
}
