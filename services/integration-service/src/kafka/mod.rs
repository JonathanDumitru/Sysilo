use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::Timeout;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;
use tracing::info;
use uuid::Uuid;

use crate::engine::Task;

/// Kafka topic names
pub mod topics {
    /// Tasks to be dispatched to agents
    pub const TASKS: &str = "sysilo.tasks";
    /// Task results from agents
    pub const RESULTS: &str = "sysilo.results";
    /// Cancel commands for running tasks
    pub const CANCEL: &str = "sysilo.cancel";
    /// Integration events (run started, completed, etc.)
    pub const EVENTS: &str = "sysilo.integration.events";
}

// =============================================================================
// Consumer Types - for receiving task results from agents
// =============================================================================

/// Task result received from agents via Kafka.
/// Matches the ResultMessage struct published by agent-gateway.
#[derive(Debug, Deserialize)]
pub struct TaskResult {
    pub task_id: String,
    pub agent_id: String,
    pub integration_id: String,
    pub tenant_id: String,
    pub status: String,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: chrono::DateTime<chrono::Utc>,
    pub output: Option<serde_json::Value>,
    pub error: Option<TaskError>,
    pub metrics: std::collections::HashMap<String, serde_json::Value>,
}

/// Error details from a failed task.
/// Matches the ErrorDetail struct from agent-gateway.
#[derive(Debug, Deserialize)]
pub struct TaskError {
    pub code: String,
    pub message: String,
    pub details: Option<String>,
    pub retryable: bool,
}

/// Discovered asset from agent discovery task.
/// Used when processing discovery results to create assets in the Asset Registry.
#[derive(Debug, Deserialize)]
pub struct DiscoveredAsset {
    pub name: String,
    pub asset_type: String,
    pub description: Option<String>,
    pub vendor: Option<String>,
    pub version: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

// =============================================================================
// Producer Configuration
// =============================================================================

/// Kafka configuration
#[derive(Debug, Clone)]
pub struct KafkaConfig {
    pub bootstrap_servers: String,
    pub client_id: String,
    pub acks: String,
    pub retries: i32,
    pub linger_ms: i32,
}

impl Default for KafkaConfig {
    fn default() -> Self {
        Self {
            bootstrap_servers: "localhost:9092".to_string(),
            client_id: "integration-service".to_string(),
            acks: "all".to_string(),
            retries: 3,
            linger_ms: 5,
        }
    }
}

/// Errors that can occur in Kafka operations
#[derive(Debug, Error)]
pub enum KafkaError {
    #[error("Failed to create producer: {0}")]
    ProducerCreationFailed(String),

    #[error("Failed to send message: {0}")]
    SendFailed(String),

    #[error("Serialization error: {0}")]
    SerializationFailed(String),
}

/// Kafka message producer for integration service
pub struct TaskProducer {
    producer: FutureProducer,
}

impl TaskProducer {
    /// Create a new task producer
    pub fn new(config: &KafkaConfig) -> Result<Self, KafkaError> {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", &config.bootstrap_servers)
            .set("client.id", &config.client_id)
            .set("acks", &config.acks)
            .set("retries", config.retries.to_string())
            .set("linger.ms", config.linger_ms.to_string())
            .set("message.timeout.ms", "30000")
            .create()
            .map_err(|e| KafkaError::ProducerCreationFailed(e.to_string()))?;

        info!("Kafka producer initialized for {}", config.bootstrap_servers);

        Ok(Self { producer })
    }

    /// Send a task to the tasks topic for agent dispatch
    pub async fn send_task(&self, task: &Task) -> Result<(), KafkaError> {
        let payload = serde_json::to_string(task)
            .map_err(|e| KafkaError::SerializationFailed(e.to_string()))?;

        // Use tenant_id as partition key for task locality
        let key = task.tenant_id.clone();

        let record = FutureRecord::to(topics::TASKS)
            .key(&key)
            .payload(&payload)
            .headers(rdkafka::message::OwnedHeaders::new()
                .insert(rdkafka::message::Header {
                    key: "task_id",
                    value: Some(task.id.to_string().as_bytes()),
                })
                .insert(rdkafka::message::Header {
                    key: "run_id",
                    value: Some(task.run_id.to_string().as_bytes()),
                })
                .insert(rdkafka::message::Header {
                    key: "tenant_id",
                    value: Some(task.tenant_id.as_bytes()),
                }));

        self.producer
            .send(record, Timeout::After(Duration::from_secs(10)))
            .await
            .map_err(|(e, _)| KafkaError::SendFailed(e.to_string()))?;

        info!(
            task_id = %task.id,
            run_id = %task.run_id,
            task_type = %task.task_type,
            "Task sent to Kafka"
        );

        Ok(())
    }

    /// Send multiple tasks in a batch
    pub async fn send_tasks(&self, tasks: &[Task]) -> Result<(), KafkaError> {
        for task in tasks {
            self.send_task(task).await?;
        }
        Ok(())
    }

    /// Send a cancel command for a task
    pub async fn send_cancel(&self, task_id: Uuid, run_id: Uuid, tenant_id: &str, reason: &str) -> Result<(), KafkaError> {
        let cancel_msg = CancelTaskMessage {
            task_id,
            run_id,
            tenant_id: tenant_id.to_string(),
            reason: reason.to_string(),
        };

        let payload = serde_json::to_string(&cancel_msg)
            .map_err(|e| KafkaError::SerializationFailed(e.to_string()))?;

        let record = FutureRecord::to(topics::CANCEL)
            .key(tenant_id)
            .payload(&payload);

        self.producer
            .send(record, Timeout::After(Duration::from_secs(10)))
            .await
            .map_err(|(e, _)| KafkaError::SendFailed(e.to_string()))?;

        info!(
            task_id = %task_id,
            run_id = %run_id,
            reason = %reason,
            "Cancel command sent to Kafka"
        );

        Ok(())
    }

    /// Send an integration event
    pub async fn send_event(&self, event: &IntegrationEvent) -> Result<(), KafkaError> {
        let payload = serde_json::to_string(event)
            .map_err(|e| KafkaError::SerializationFailed(e.to_string()))?;

        let record = FutureRecord::to(topics::EVENTS)
            .key(&event.tenant_id)
            .payload(&payload);

        self.producer
            .send(record, Timeout::After(Duration::from_secs(10)))
            .await
            .map_err(|(e, _)| KafkaError::SendFailed(e.to_string()))?;

        info!(
            event_type = %event.event_type,
            run_id = %event.run_id,
            "Integration event sent to Kafka"
        );

        Ok(())
    }
}

/// Message to cancel a running task
#[derive(Debug, Serialize)]
pub struct CancelTaskMessage {
    pub task_id: Uuid,
    pub run_id: Uuid,
    pub tenant_id: String,
    pub reason: String,
}

/// Integration lifecycle events
#[derive(Debug, Serialize)]
pub struct IntegrationEvent {
    pub event_type: String,
    pub run_id: Uuid,
    pub integration_id: Uuid,
    pub tenant_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub details: serde_json::Value,
}

impl IntegrationEvent {
    pub fn run_started(run_id: Uuid, integration_id: Uuid, tenant_id: String) -> Self {
        Self {
            event_type: "run.started".to_string(),
            run_id,
            integration_id,
            tenant_id,
            timestamp: chrono::Utc::now(),
            details: serde_json::json!({}),
        }
    }

    pub fn run_completed(run_id: Uuid, integration_id: Uuid, tenant_id: String, metrics: serde_json::Value) -> Self {
        Self {
            event_type: "run.completed".to_string(),
            run_id,
            integration_id,
            tenant_id,
            timestamp: chrono::Utc::now(),
            details: metrics,
        }
    }

    pub fn run_failed(run_id: Uuid, integration_id: Uuid, tenant_id: String, error: &str) -> Self {
        Self {
            event_type: "run.failed".to_string(),
            run_id,
            integration_id,
            tenant_id,
            timestamp: chrono::Utc::now(),
            details: serde_json::json!({ "error": error }),
        }
    }

    pub fn run_cancelled(run_id: Uuid, integration_id: Uuid, tenant_id: String, reason: &str) -> Self {
        Self {
            event_type: "run.cancelled".to_string(),
            run_id,
            integration_id,
            tenant_id,
            timestamp: chrono::Utc::now(),
            details: serde_json::json!({ "reason": reason }),
        }
    }
}
