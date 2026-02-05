package config

import (
	"fmt"
	"os"
	"strconv"

	"gopkg.in/yaml.v3"
)

// Config holds all API gateway configuration
type Config struct {
	Server    ServerConfig    `yaml:"server"`
	Database  DatabaseConfig  `yaml:"database"`
	Redis     RedisConfig     `yaml:"redis"`
	Auth      AuthConfig      `yaml:"auth"`
	CORS      CORSConfig      `yaml:"cors"`
	RateLimit RateLimitConfig `yaml:"rate_limit"`
	Services  ServicesConfig  `yaml:"services"`
	Logging   LoggingConfig   `yaml:"logging"`
}

// ServerConfig holds server settings
type ServerConfig struct {
	Address string `yaml:"address"`
}

// DatabaseConfig holds PostgreSQL connection settings
type DatabaseConfig struct {
	Host         string `yaml:"host"`
	Port         int    `yaml:"port"`
	User         string `yaml:"user"`
	Password     string `yaml:"password"`
	Database     string `yaml:"database"`
	SSLMode      string `yaml:"ssl_mode"`
	MaxOpenConns int    `yaml:"max_open_conns"`
	MaxIdleConns int    `yaml:"max_idle_conns"`
}

// DSN returns the PostgreSQL connection string
func (c DatabaseConfig) DSN() string {
	return fmt.Sprintf(
		"host=%s port=%d user=%s password=%s dbname=%s sslmode=%s",
		c.Host, c.Port, c.User, c.Password, c.Database, c.SSLMode,
	)
}

// RedisConfig holds Redis connection settings
type RedisConfig struct {
	Address  string `yaml:"address"`
	Password string `yaml:"password"`
	DB       int    `yaml:"db"`
}

// AuthConfig holds authentication settings
type AuthConfig struct {
	JWTSecret      string   `yaml:"jwt_secret"`
	JWTIssuer      string   `yaml:"jwt_issuer"`
	TokenExpiry    int      `yaml:"token_expiry_minutes"`
	AllowedIssuers []string `yaml:"allowed_issuers"`
}

// CORSConfig holds CORS settings
type CORSConfig struct {
	AllowedOrigins   []string `yaml:"allowed_origins"`
	AllowedMethods   []string `yaml:"allowed_methods"`
	AllowedHeaders   []string `yaml:"allowed_headers"`
	ExposedHeaders   []string `yaml:"exposed_headers"`
	AllowCredentials bool     `yaml:"allow_credentials"`
	MaxAge           int      `yaml:"max_age"`
}

// RateLimitConfig holds rate limiting settings
type RateLimitConfig struct {
	Enabled        bool `yaml:"enabled"`
	RequestsPerMin int  `yaml:"requests_per_minute"`
	BurstSize      int  `yaml:"burst_size"`
}

// ServicesConfig holds internal service addresses
type ServicesConfig struct {
	AgentGateway       string `yaml:"agent_gateway"`
	IntegrationService string `yaml:"integration_service"`
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
			Address: ":8081",
		},
		Database: DatabaseConfig{
			Host:         "localhost",
			Port:         5432,
			User:         "sysilo",
			Password:     "sysilo",
			Database:     "sysilo",
			SSLMode:      "disable",
			MaxOpenConns: 25,
			MaxIdleConns: 5,
		},
		Redis: RedisConfig{
			Address: "localhost:6379",
			DB:      0,
		},
		Auth: AuthConfig{
			JWTSecret:   "dev-secret-change-in-production",
			JWTIssuer:   "sysilo",
			TokenExpiry: 60, // 1 hour
		},
		CORS: CORSConfig{
			AllowedOrigins:   []string{"http://localhost:3000"},
			AllowedMethods:   []string{"GET", "POST", "PUT", "DELETE", "OPTIONS"},
			AllowedHeaders:   []string{"Accept", "Authorization", "Content-Type", "X-Tenant-ID"},
			ExposedHeaders:   []string{"X-Request-ID"},
			AllowCredentials: true,
			MaxAge:           86400, // 24 hours
		},
		RateLimit: RateLimitConfig{
			Enabled:        true,
			RequestsPerMin: 1000,
			BurstSize:      50,
		},
		Services: ServicesConfig{
			AgentGateway:       "localhost:8082",
			IntegrationService: "localhost:8085",
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
			"./api-gateway.yaml",
			"./config/api-gateway.yaml",
			"/etc/sysilo/api-gateway.yaml",
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
	if v := os.Getenv("SYSILO_API_ADDRESS"); v != "" {
		c.Server.Address = v
	}
	if v := os.Getenv("SYSILO_JWT_SECRET"); v != "" {
		c.Auth.JWTSecret = v
	}
	if v := os.Getenv("SYSILO_LOG_LEVEL"); v != "" {
		c.Logging.Level = v
	}

	// Database overrides
	if v := os.Getenv("SYSILO_DB_HOST"); v != "" {
		c.Database.Host = v
	}
	if v := os.Getenv("SYSILO_DB_PORT"); v != "" {
		if port, err := strconv.Atoi(v); err == nil {
			c.Database.Port = port
		}
	}
	if v := os.Getenv("SYSILO_DB_USER"); v != "" {
		c.Database.User = v
	}
	if v := os.Getenv("SYSILO_DB_PASSWORD"); v != "" {
		c.Database.Password = v
	}
	if v := os.Getenv("SYSILO_DB_NAME"); v != "" {
		c.Database.Database = v
	}
	if v := os.Getenv("SYSILO_DB_SSLMODE"); v != "" {
		c.Database.SSLMode = v
	}

	// Redis overrides
	if v := os.Getenv("SYSILO_REDIS_ADDRESS"); v != "" {
		c.Redis.Address = v
	}
	if v := os.Getenv("SYSILO_REDIS_PASSWORD"); v != "" {
		c.Redis.Password = v
	}

	// Service overrides
	if v := os.Getenv("SYSILO_AGENT_GATEWAY_ADDRESS"); v != "" {
		c.Services.AgentGateway = v
	}
	if v := os.Getenv("SYSILO_INTEGRATION_SERVICE_ADDRESS"); v != "" {
		c.Services.IntegrationService = v
	}
}
