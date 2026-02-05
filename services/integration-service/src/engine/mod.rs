use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::config::Config;
use crate::kafka::{IntegrationEvent, KafkaConfig, KafkaError, TaskProducer};

/// Errors that can occur during integration execution
#[derive(Debug, Error)]
pub enum EngineError {
    #[error("Integration not found: {0}")]
    IntegrationNotFound(Uuid),

    #[error("Run not found: {0}")]
    RunNotFound(Uuid),

    #[error("Invalid integration definition: {0}")]
    InvalidDefinition(String),

    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    #[error("No available agent for tenant {0}")]
    NoAvailableAgent(String),

    #[error("Kafka error: {0}")]
    KafkaError(#[from] KafkaError),
}

/// Integration definition structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationDefinition {
    /// Source configuration
    pub source: StepConfig,

    /// Optional transformation steps
    #[serde(default)]
    pub transforms: Vec<TransformConfig>,

    /// Target configuration
    pub target: StepConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepConfig {
    /// Connection ID to use
    pub connection_id: Uuid,

    /// Step type (e.g., "query", "api_call", "file_read")
    pub step_type: String,

    /// Step-specific configuration
    pub config: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformConfig {
    /// Transform type (e.g., "map", "filter", "aggregate")
    pub transform_type: String,

    /// Transform configuration
    pub config: serde_json::Value,
}

/// Status of an integration run
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// An integration run instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationRun {
    pub id: Uuid,
    pub integration_id: Uuid,
    pub tenant_id: String,
    pub status: RunStatus,
    pub trigger_type: String,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub metrics: RunMetrics,
    /// Task IDs associated with this run
    #[serde(default)]
    pub task_ids: Vec<Uuid>,
    /// Number of completed tasks
    #[serde(default)]
    pub completed_tasks: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RunMetrics {
    pub records_read: i64,
    pub records_written: i64,
    pub bytes_processed: i64,
    pub duration_ms: i64,
}

/// Task to be sent to an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub run_id: Uuid,
    pub integration_id: Uuid,
    pub tenant_id: String,
    pub task_type: String,
    pub config: serde_json::Value,
    pub priority: i32,
    pub timeout_seconds: u64,
    /// Sequence number in the flow (for ordering)
    #[serde(default)]
    pub sequence: u32,
    /// IDs of tasks this task depends on
    #[serde(default)]
    pub depends_on: Vec<Uuid>,
}

/// The integration execution engine
pub struct Engine {
    config: Config,
    /// Active runs being tracked
    active_runs: Arc<RwLock<HashMap<Uuid, IntegrationRun>>>,
    /// Kafka producer for task dispatch
    kafka_producer: Option<TaskProducer>,
}

impl Engine {
    /// Create a new engine instance
    pub fn new(config: Config) -> Self {
        // Try to create Kafka producer, log warning if fails
        let kafka_producer = match TaskProducer::new(&KafkaConfig {
            bootstrap_servers: config.kafka.brokers.clone(),
            client_id: "integration-service".to_string(),
            acks: "all".to_string(),
            retries: 3,
            linger_ms: 5,
        }) {
            Ok(producer) => Some(producer),
            Err(e) => {
                warn!("Failed to initialize Kafka producer: {}. Running in local mode.", e);
                None
            }
        };

        Self {
            config,
            active_runs: Arc::new(RwLock::new(HashMap::new())),
            kafka_producer,
        }
    }

    /// Start a new integration run
    pub async fn start_run(
        &self,
        integration_id: Uuid,
        tenant_id: String,
        definition: IntegrationDefinition,
        trigger_type: String,
    ) -> Result<IntegrationRun, EngineError> {
        let run_id = Uuid::new_v4();

        info!(
            run_id = %run_id,
            integration_id = %integration_id,
            tenant_id = %tenant_id,
            "Starting integration run"
        );

        // Generate tasks from the definition
        let tasks = self.generate_tasks(run_id, integration_id, &tenant_id, &definition)?;
        let task_ids: Vec<Uuid> = tasks.iter().map(|t| t.id).collect();

        // Create the run
        let mut run = IntegrationRun {
            id: run_id,
            integration_id,
            tenant_id: tenant_id.clone(),
            status: RunStatus::Pending,
            trigger_type,
            started_at: None,
            completed_at: None,
            error_message: None,
            metrics: RunMetrics::default(),
            task_ids: task_ids.clone(),
            completed_tasks: 0,
        };

        // Track the run
        {
            let mut runs = self.active_runs.write().await;
            runs.insert(run_id, run.clone());
        }

        // Send tasks to Kafka for agent gateway to pick up
        if let Some(ref producer) = self.kafka_producer {
            // Only send tasks with no dependencies (starting tasks)
            let starting_tasks: Vec<&Task> = tasks.iter().filter(|t| t.depends_on.is_empty()).collect();

            for task in starting_tasks {
                if let Err(e) = producer.send_task(task).await {
                    error!(task_id = %task.id, error = %e, "Failed to send task to Kafka");
                    // Mark run as failed if we can't dispatch tasks
                    let mut runs = self.active_runs.write().await;
                    if let Some(r) = runs.get_mut(&run_id) {
                        r.status = RunStatus::Failed;
                        r.error_message = Some(format!("Failed to dispatch tasks: {}", e));
                        r.completed_at = Some(Utc::now());
                    }
                    return Err(EngineError::ExecutionFailed(format!(
                        "Failed to send task to Kafka: {}",
                        e
                    )));
                }
            }

            // Send run started event
            let event = IntegrationEvent::run_started(run_id, integration_id, tenant_id.clone());
            if let Err(e) = producer.send_event(&event).await {
                warn!(error = %e, "Failed to send run started event");
            }
        } else {
            // Local mode - just log the tasks
            for task in &tasks {
                info!(
                    task_id = %task.id,
                    task_type = %task.task_type,
                    sequence = task.sequence,
                    "Generated task (local mode - no Kafka)"
                );
            }
        }

        // Update status to running
        {
            let mut runs = self.active_runs.write().await;
            if let Some(r) = runs.get_mut(&run_id) {
                r.status = RunStatus::Running;
                r.started_at = Some(Utc::now());
                run = r.clone();
            }
        }

        Ok(run)
    }

    /// Generate tasks from an integration definition
    fn generate_tasks(
        &self,
        run_id: Uuid,
        integration_id: Uuid,
        tenant_id: &str,
        definition: &IntegrationDefinition,
    ) -> Result<Vec<Task>, EngineError> {
        let mut tasks = Vec::new();
        let mut sequence: u32 = 0;
        let mut prev_task_id: Option<Uuid> = None;

        // Source task (sequence 0, no dependencies)
        let source_task_id = Uuid::new_v4();
        tasks.push(Task {
            id: source_task_id,
            run_id,
            integration_id,
            tenant_id: tenant_id.to_string(),
            task_type: definition.source.step_type.clone(),
            config: serde_json::json!({
                "step": "source",
                "connection_id": definition.source.connection_id,
                "config": definition.source.config,
            }),
            priority: 2,
            timeout_seconds: self.config.engine.default_timeout_seconds,
            sequence,
            depends_on: vec![],
        });
        prev_task_id = Some(source_task_id);
        sequence += 1;

        // Transform tasks (each depends on previous)
        for transform in &definition.transforms {
            let transform_task_id = Uuid::new_v4();
            tasks.push(Task {
                id: transform_task_id,
                run_id,
                integration_id,
                tenant_id: tenant_id.to_string(),
                task_type: format!("transform_{}", transform.transform_type),
                config: serde_json::json!({
                    "step": "transform",
                    "transform_type": transform.transform_type,
                    "config": transform.config,
                }),
                priority: 2,
                timeout_seconds: self.config.engine.default_timeout_seconds,
                sequence,
                depends_on: prev_task_id.map(|id| vec![id]).unwrap_or_default(),
            });
            prev_task_id = Some(transform_task_id);
            sequence += 1;
        }

        // Target task (depends on last transform or source)
        let target_task_id = Uuid::new_v4();
        tasks.push(Task {
            id: target_task_id,
            run_id,
            integration_id,
            tenant_id: tenant_id.to_string(),
            task_type: definition.target.step_type.clone(),
            config: serde_json::json!({
                "step": "target",
                "connection_id": definition.target.connection_id,
                "config": definition.target.config,
            }),
            priority: 2,
            timeout_seconds: self.config.engine.default_timeout_seconds,
            sequence,
            depends_on: prev_task_id.map(|id| vec![id]).unwrap_or_default(),
        });

        info!(
            run_id = %run_id,
            task_count = tasks.len(),
            "Generated tasks for integration run"
        );

        Ok(tasks)
    }

    /// Get the status of a run
    pub async fn get_run(&self, run_id: Uuid) -> Option<IntegrationRun> {
        let runs = self.active_runs.read().await;
        runs.get(&run_id).cloned()
    }

    /// Cancel a running integration
    pub async fn cancel_run(&self, run_id: Uuid) -> Result<IntegrationRun, EngineError> {
        let mut runs = self.active_runs.write().await;

        let run = runs
            .get_mut(&run_id)
            .ok_or(EngineError::RunNotFound(run_id))?;

        if run.status == RunStatus::Running || run.status == RunStatus::Pending {
            run.status = RunStatus::Cancelled;
            run.completed_at = Some(Utc::now());

            // Send cancel commands to agents via Kafka
            if let Some(ref producer) = self.kafka_producer {
                for task_id in &run.task_ids {
                    if let Err(e) = producer
                        .send_cancel(*task_id, run_id, &run.tenant_id, "User requested cancellation")
                        .await
                    {
                        warn!(task_id = %task_id, error = %e, "Failed to send cancel command");
                    }
                }

                // Send run cancelled event
                let event = IntegrationEvent::run_cancelled(
                    run_id,
                    run.integration_id,
                    run.tenant_id.clone(),
                    "User requested cancellation",
                );
                if let Err(e) = producer.send_event(&event).await {
                    warn!(error = %e, "Failed to send run cancelled event");
                }
            }

            info!(run_id = %run_id, "Integration run cancelled");
        } else {
            warn!(
                run_id = %run_id,
                status = ?run.status,
                "Cannot cancel run in current status"
            );
        }

        Ok(run.clone())
    }

    /// Handle a task result from an agent
    pub async fn handle_task_result(
        &self,
        run_id: Uuid,
        task_id: Uuid,
        status: RunStatus,
        metrics: RunMetrics,
        error: Option<String>,
    ) -> Result<(), EngineError> {
        let mut runs = self.active_runs.write().await;

        let run = runs
            .get_mut(&run_id)
            .ok_or(EngineError::RunNotFound(run_id))?;

        // Update metrics
        run.metrics.records_read += metrics.records_read;
        run.metrics.records_written += metrics.records_written;
        run.metrics.bytes_processed += metrics.bytes_processed;

        // Handle task completion
        match status {
            RunStatus::Completed => {
                run.completed_tasks += 1;

                // Check if all tasks are complete
                if run.completed_tasks >= run.task_ids.len() {
                    run.status = RunStatus::Completed;
                    run.completed_at = Some(Utc::now());
                    run.metrics.duration_ms = run
                        .started_at
                        .map(|start| (Utc::now() - start).num_milliseconds())
                        .unwrap_or(0);

                    info!(
                        run_id = %run_id,
                        records_read = run.metrics.records_read,
                        records_written = run.metrics.records_written,
                        duration_ms = run.metrics.duration_ms,
                        "Integration run completed successfully"
                    );

                    // Send completed event
                    if let Some(ref producer) = self.kafka_producer {
                        let event = IntegrationEvent::run_completed(
                            run_id,
                            run.integration_id,
                            run.tenant_id.clone(),
                            serde_json::to_value(&run.metrics).unwrap_or_default(),
                        );
                        if let Err(e) = producer.send_event(&event).await {
                            warn!(error = %e, "Failed to send run completed event");
                        }
                    }
                }
                // If not all tasks complete, the next dependent task will be triggered
                // by the agent gateway based on the task graph
            }
            RunStatus::Failed => {
                run.status = RunStatus::Failed;
                run.error_message = error.clone();
                run.completed_at = Some(Utc::now());

                error!(
                    run_id = %run_id,
                    task_id = %task_id,
                    error = ?error,
                    "Integration run failed"
                );

                // Send failed event
                if let Some(ref producer) = self.kafka_producer {
                    let event = IntegrationEvent::run_failed(
                        run_id,
                        run.integration_id,
                        run.tenant_id.clone(),
                        &error.unwrap_or_else(|| "Unknown error".to_string()),
                    );
                    if let Err(e) = producer.send_event(&event).await {
                        warn!(error = %e, "Failed to send run failed event");
                    }
                }
            }
            _ => {
                // Other statuses (Running, Pending, Cancelled) - just log
                info!(
                    run_id = %run_id,
                    task_id = %task_id,
                    status = ?status,
                    "Processed task result"
                );
            }
        }

        Ok(())
    }

    /// Get all active runs
    pub async fn active_runs(&self) -> Vec<IntegrationRun> {
        let runs = self.active_runs.read().await;
        runs.values().cloned().collect()
    }

    /// Clean up completed runs from memory (should be called periodically)
    pub async fn cleanup_completed_runs(&self, max_age_hours: i64) {
        let cutoff = Utc::now() - chrono::Duration::hours(max_age_hours);
        let mut runs = self.active_runs.write().await;

        let to_remove: Vec<Uuid> = runs
            .iter()
            .filter(|(_, run)| {
                matches!(
                    run.status,
                    RunStatus::Completed | RunStatus::Failed | RunStatus::Cancelled
                ) && run.completed_at.map(|t| t < cutoff).unwrap_or(false)
            })
            .map(|(id, _)| *id)
            .collect();

        for id in to_remove {
            runs.remove(&id);
            info!(run_id = %id, "Cleaned up completed run from memory");
        }
    }

    /// Get reference to Kafka producer (if available)
    pub fn kafka_producer(&self) -> Option<&TaskProducer> {
        self.kafka_producer.as_ref()
    }
}
