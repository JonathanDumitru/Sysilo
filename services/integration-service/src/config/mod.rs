use serde::Deserialize;
use std::env;

/// Application configuration
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub kafka: KafkaConfig,
    pub engine: EngineConfig,
    pub consumer: ConsumerConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub address: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct KafkaConfig {
    /// Kafka broker addresses (alias: bootstrap_servers)
    #[serde(alias = "bootstrap_servers")]
    pub brokers: String,
    pub group_id: String,
    pub task_topic: String,
    pub result_topic: String,
}

impl KafkaConfig {
    /// Get bootstrap servers (compatibility method for kafka producer)
    pub fn bootstrap_servers(&self) -> &str {
        &self.brokers
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct EngineConfig {
    pub max_concurrent_runs: usize,
    pub default_timeout_seconds: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConsumerConfig {
    pub bootstrap_servers: String,
    pub group_id: String,
    pub asset_service_url: String,
    pub enabled: bool,
}

impl Config {
    /// Load configuration from environment variables with defaults
    pub fn load() -> anyhow::Result<Self> {
        Ok(Config {
            server: ServerConfig {
                address: env::var("SYSILO_SERVER_ADDRESS")
                    .unwrap_or_else(|_| "0.0.0.0:8082".to_string()),
            },
            database: DatabaseConfig {
                url: env::var("DATABASE_URL").unwrap_or_else(|_| {
                    "postgres://sysilo:sysilo_dev@localhost:5432/sysilo".to_string()
                }),
                max_connections: env::var("DATABASE_MAX_CONNECTIONS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(10),
            },
            kafka: KafkaConfig {
                brokers: env::var("KAFKA_BROKERS")
                    .unwrap_or_else(|_| "localhost:9092".to_string()),
                group_id: env::var("KAFKA_GROUP_ID")
                    .unwrap_or_else(|_| "integration-service".to_string()),
                task_topic: env::var("KAFKA_TASK_TOPIC")
                    .unwrap_or_else(|_| "sysilo.tasks".to_string()),
                result_topic: env::var("KAFKA_RESULT_TOPIC")
                    .unwrap_or_else(|_| "sysilo.results".to_string()),
            },
            engine: EngineConfig {
                max_concurrent_runs: env::var("ENGINE_MAX_CONCURRENT_RUNS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(100),
                default_timeout_seconds: env::var("ENGINE_DEFAULT_TIMEOUT_SECONDS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(300),
            },
            consumer: ConsumerConfig {
                bootstrap_servers: env::var("CONSUMER_BOOTSTRAP_SERVERS")
                    .unwrap_or_else(|_| "localhost:9092".to_string()),
                group_id: env::var("CONSUMER_GROUP_ID")
                    .unwrap_or_else(|_| "integration-service-consumers".to_string()),
                asset_service_url: env::var("CONSUMER_ASSET_SERVICE_URL")
                    .unwrap_or_else(|_| "http://localhost:8082".to_string()),
                enabled: env::var("CONSUMER_ENABLED")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(true),
            },
        })
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::load().expect("Failed to load default config")
    }
}
