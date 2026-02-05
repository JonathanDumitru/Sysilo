# Playbook Kafka Task Dispatcher Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Enable actual execution of playbook steps via the existing Kafka task system, with DAG-based step orchestration driven by on_success/on_failure edges.

**Architecture:** PlaybookExecutor dispatches starting steps as Kafka tasks. When results arrive, PlaybookResultHandler updates step_states in the DB, dispatches the next steps based on success/failure edges, and detects run completion. Special handling for approval (pause), wait (agent sleep), and condition (branching) step types.

**Tech Stack:** Rust/Axum, rdkafka (FutureProducer), PostgreSQL (sqlx), serde_json

---

### Task 1: Create PlaybookExecutor

**Files:**
- Create: `services/integration-service/src/playbooks/executor.rs`
- Modify: `services/integration-service/src/playbooks/mod.rs`

**Step 1: Create executor.rs with PlaybookExecutor struct**

```rust
// services/integration-service/src/playbooks/executor.rs

use tracing::{error, info, warn};
use uuid::Uuid;

use crate::engine::Task;
use crate::kafka::TaskProducer;
use crate::playbooks::{Step, StepState, StepStatus};
use crate::storage::Storage;

/// Errors from playbook execution
#[derive(Debug, thiserror::Error)]
pub enum ExecutorError {
    #[error("Playbook not found: {0}")]
    PlaybookNotFound(Uuid),

    #[error("Run not found: {0}")]
    RunNotFound(Uuid),

    #[error("Kafka dispatch failed: {0}")]
    DispatchFailed(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Invalid playbook definition: {0}")]
    InvalidDefinition(String),
}

/// Dispatches playbook steps to Kafka for execution.
pub struct PlaybookExecutor;

impl PlaybookExecutor {
    /// Start executing a playbook run by dispatching its initial steps.
    ///
    /// Initial steps are those whose step_id does NOT appear in any other
    /// step's on_success or on_failure list (i.e., no incoming edges).
    pub async fn start_run(
        producer: &TaskProducer,
        storage: &Storage,
        run_id: Uuid,
        playbook_id: Uuid,
        tenant_id: &str,
        steps: &[Step],
        variables: &serde_json::Value,
    ) -> Result<(), ExecutorError> {
        let starting_steps = Self::find_starting_steps(steps);

        if starting_steps.is_empty() {
            return Err(ExecutorError::InvalidDefinition(
                "No starting steps found (all steps have incoming edges)".to_string(),
            ));
        }

        info!(
            run_id = %run_id,
            playbook_id = %playbook_id,
            starting_step_count = starting_steps.len(),
            "Dispatching starting steps for playbook run"
        );

        // Update step_states for starting steps to "running"
        let mut step_states: Vec<StepState> = steps
            .iter()
            .map(|s| {
                let is_starting = starting_steps.iter().any(|ss| ss.id == s.id);
                StepState {
                    step_id: s.id.clone(),
                    status: if is_starting {
                        StepStatus::Running
                    } else {
                        StepStatus::Pending
                    },
                    started_at: if is_starting {
                        Some(chrono::Utc::now())
                    } else {
                        None
                    },
                    completed_at: None,
                    output: None,
                    error: None,
                }
            })
            .collect();

        let step_states_json = serde_json::to_value(&step_states)
            .map_err(|e| ExecutorError::InvalidDefinition(e.to_string()))?;

        storage
            .update_playbook_run(run_id, "running", step_states_json)
            .await
            .map_err(|e| ExecutorError::StorageError(e.to_string()))?;

        // Dispatch each starting step
        for step in &starting_steps {
            Self::dispatch_step(producer, run_id, playbook_id, tenant_id, step, variables)
                .await?;
        }

        Ok(())
    }

    /// Dispatch a single step as a Kafka task.
    pub async fn dispatch_step(
        producer: &TaskProducer,
        run_id: Uuid,
        playbook_id: Uuid,
        tenant_id: &str,
        step: &Step,
        variables: &serde_json::Value,
    ) -> Result<Uuid, ExecutorError> {
        let task_id = Uuid::new_v4();

        let step_type_str = serde_json::to_value(&step.step_type)
            .map_err(|e| ExecutorError::InvalidDefinition(e.to_string()))?;

        let task = Task {
            id: task_id,
            run_id,
            integration_id: playbook_id,
            tenant_id: tenant_id.to_string(),
            task_type: "playbook_step".to_string(),
            config: serde_json::json!({
                "step_id": step.id,
                "step_type": step_type_str,
                "step_name": step.name,
                "step_config": step.config,
                "variables": variables,
            }),
            priority: 2,
            timeout_seconds: 300,
            sequence: 0,
            depends_on: vec![],
        };

        producer
            .send_task(&task)
            .await
            .map_err(|e| ExecutorError::DispatchFailed(e.to_string()))?;

        info!(
            task_id = %task_id,
            run_id = %run_id,
            step_id = %step.id,
            step_type = ?step.step_type,
            "Playbook step dispatched to Kafka"
        );

        Ok(task_id)
    }

    /// Find steps with no incoming edges (not referenced in any on_success/on_failure).
    fn find_starting_steps(steps: &[Step]) -> Vec<Step> {
        // Collect all step IDs that are targets of edges
        let mut referenced_ids: std::collections::HashSet<&str> =
            std::collections::HashSet::new();

        for step in steps {
            for target in &step.on_success {
                referenced_ids.insert(target.as_str());
            }
            for target in &step.on_failure {
                referenced_ids.insert(target.as_str());
            }
        }

        // Starting steps are those NOT referenced by any edge
        steps
            .iter()
            .filter(|s| !referenced_ids.contains(s.id.as_str()))
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::playbooks::StepType;

    fn make_step(id: &str, on_success: Vec<&str>, on_failure: Vec<&str>) -> Step {
        Step {
            id: id.to_string(),
            step_type: StepType::Integration,
            name: format!("Step {}", id),
            config: serde_json::json!({}),
            on_success: on_success.into_iter().map(String::from).collect(),
            on_failure: on_failure.into_iter().map(String::from).collect(),
        }
    }

    #[test]
    fn test_find_starting_steps_simple_chain() {
        let steps = vec![
            make_step("a", vec!["b"], vec![]),
            make_step("b", vec!["c"], vec![]),
            make_step("c", vec![], vec![]),
        ];
        let starting = PlaybookExecutor::find_starting_steps(&steps);
        assert_eq!(starting.len(), 1);
        assert_eq!(starting[0].id, "a");
    }

    #[test]
    fn test_find_starting_steps_parallel_start() {
        let steps = vec![
            make_step("a", vec!["c"], vec![]),
            make_step("b", vec!["c"], vec![]),
            make_step("c", vec![], vec![]),
        ];
        let starting = PlaybookExecutor::find_starting_steps(&steps);
        assert_eq!(starting.len(), 2);
        let ids: Vec<&str> = starting.iter().map(|s| s.id.as_str()).collect();
        assert!(ids.contains(&"a"));
        assert!(ids.contains(&"b"));
    }

    #[test]
    fn test_find_starting_steps_with_failure_edges() {
        let steps = vec![
            make_step("a", vec!["b"], vec!["c"]),
            make_step("b", vec![], vec![]),
            make_step("c", vec![], vec![]),
        ];
        let starting = PlaybookExecutor::find_starting_steps(&steps);
        assert_eq!(starting.len(), 1);
        assert_eq!(starting[0].id, "a");
    }

    #[test]
    fn test_find_starting_steps_single_step() {
        let steps = vec![make_step("only", vec![], vec![])];
        let starting = PlaybookExecutor::find_starting_steps(&steps);
        assert_eq!(starting.len(), 1);
        assert_eq!(starting[0].id, "only");
    }
}
```

**Step 2: Add executor module to playbooks/mod.rs**

Add `pub mod executor;` to the top of `services/integration-service/src/playbooks/mod.rs`, after `pub mod api;`.

**Step 3: Run tests**

Run: `cd services/integration-service && cargo test playbooks::executor --lib`
Expected: All 4 tests pass

**Step 4: Commit**

```bash
git add services/integration-service/src/playbooks/executor.rs services/integration-service/src/playbooks/mod.rs
git commit -m "feat(playbooks): add PlaybookExecutor for dispatching steps to Kafka"
```

---

### Task 2: Create PlaybookResultHandler

**Files:**
- Create: `services/integration-service/src/playbooks/result_handler.rs`
- Modify: `services/integration-service/src/playbooks/mod.rs`

**Step 1: Create result_handler.rs**

```rust
// services/integration-service/src/playbooks/result_handler.rs

use tracing::{error, info, warn};
use uuid::Uuid;

use crate::kafka::TaskProducer;
use crate::playbooks::executor::PlaybookExecutor;
use crate::playbooks::{RunStatus, Step, StepState, StepStatus};
use crate::storage::Storage;

/// Errors from result handling
#[derive(Debug, thiserror::Error)]
pub enum ResultHandlerError {
    #[error("Run not found: {0}")]
    RunNotFound(Uuid),

    #[error("Step not found in run: {0}")]
    StepNotFound(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Dispatch error: {0}")]
    DispatchError(String),

    #[error("Parse error: {0}")]
    ParseError(String),
}

/// Handles results from completed playbook steps and orchestrates the DAG.
pub struct PlaybookResultHandler;

impl PlaybookResultHandler {
    /// Handle a completed playbook step result.
    ///
    /// 1. Updates the step's state in the DB
    /// 2. Determines next steps based on on_success/on_failure
    /// 3. Dispatches next steps (or completes/fails the run)
    pub async fn handle_step_result(
        producer: &TaskProducer,
        storage: &Storage,
        run_id: Uuid,
        step_id: &str,
        success: bool,
        output: Option<serde_json::Value>,
        error_msg: Option<String>,
    ) -> Result<(), ResultHandlerError> {
        // 1. Load the run from DB
        //    We need tenant_id but don't have it here; load run without tenant filter
        let run = storage
            .get_playbook_run_by_id(run_id)
            .await
            .map_err(|e| ResultHandlerError::StorageError(e.to_string()))?;

        let tenant_id = run.tenant_id.to_string();
        let playbook_id = run.playbook_id;

        // 2. Load the playbook to get step definitions
        let playbook = storage
            .get_playbook(&tenant_id, playbook_id)
            .await
            .map_err(|e| ResultHandlerError::StorageError(e.to_string()))?;

        let steps: Vec<Step> = serde_json::from_value(playbook.steps)
            .map_err(|e| ResultHandlerError::ParseError(e.to_string()))?;

        // 3. Parse current step_states
        let mut step_states: Vec<StepState> = serde_json::from_value(run.step_states)
            .map_err(|e| ResultHandlerError::ParseError(e.to_string()))?;

        // 4. Update the completed step's state
        let step_state = step_states
            .iter_mut()
            .find(|s| s.step_id == step_id)
            .ok_or_else(|| ResultHandlerError::StepNotFound(step_id.to_string()))?;

        step_state.status = if success {
            StepStatus::Completed
        } else {
            StepStatus::Failed
        };
        step_state.completed_at = Some(chrono::Utc::now());
        step_state.output = output.clone();
        step_state.error = error_msg.clone();

        info!(
            run_id = %run_id,
            step_id = %step_id,
            success = success,
            "Step result processed"
        );

        // 5. Find the step definition to get edges
        let step_def = steps
            .iter()
            .find(|s| s.id == step_id)
            .ok_or_else(|| ResultHandlerError::StepNotFound(step_id.to_string()))?;

        // 6. Determine next steps
        let next_step_ids = if success {
            // For condition steps, use the next_steps from output
            if let Some(ref out) = output {
                if let Some(next) = out.get("next_steps") {
                    if let Some(arr) = next.as_array() {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect::<Vec<String>>()
                    } else {
                        step_def.on_success.clone()
                    }
                } else {
                    step_def.on_success.clone()
                }
            } else {
                step_def.on_success.clone()
            }
        } else {
            step_def.on_failure.clone()
        };

        // 7. Check for approval steps among next steps
        let variables = run.variables.clone();
        let mut has_approval = false;

        for next_id in &next_step_ids {
            if let Some(next_step) = steps.iter().find(|s| s.id == *next_id) {
                if matches!(next_step.step_type, crate::playbooks::StepType::Approval) {
                    has_approval = true;

                    // Mark approval step as running
                    if let Some(ns) = step_states.iter_mut().find(|s| s.step_id == *next_id) {
                        ns.status = StepStatus::Running;
                        ns.started_at = Some(chrono::Utc::now());
                    }

                    info!(
                        run_id = %run_id,
                        step_id = %next_id,
                        "Run paused at approval step"
                    );
                }
            }
        }

        // 8. Determine run status
        let new_run_status = if has_approval {
            "waiting_approval"
        } else if next_step_ids.is_empty() {
            // No next steps - check if all steps are done
            let all_done = step_states
                .iter()
                .all(|s| matches!(s.status, StepStatus::Completed | StepStatus::Failed | StepStatus::Skipped));

            if all_done {
                let any_failed = step_states.iter().any(|s| matches!(s.status, StepStatus::Failed));
                if any_failed && !success {
                    "failed"
                } else {
                    "completed"
                }
            } else {
                "running"
            }
        } else {
            "running"
        };

        // 9. Save updated step_states and status
        let step_states_json = serde_json::to_value(&step_states)
            .map_err(|e| ResultHandlerError::ParseError(e.to_string()))?;

        storage
            .update_playbook_run(run_id, new_run_status, step_states_json)
            .await
            .map_err(|e| ResultHandlerError::StorageError(e.to_string()))?;

        info!(
            run_id = %run_id,
            new_status = new_run_status,
            "Playbook run status updated"
        );

        // 10. Dispatch next non-approval steps
        if !has_approval && new_run_status == "running" {
            for next_id in &next_step_ids {
                if let Some(next_step) = steps.iter().find(|s| s.id == *next_id) {
                    if let Err(e) = PlaybookExecutor::dispatch_step(
                        producer,
                        run_id,
                        playbook_id,
                        &tenant_id,
                        next_step,
                        &variables,
                    )
                    .await
                    {
                        error!(
                            run_id = %run_id,
                            step_id = %next_id,
                            error = %e,
                            "Failed to dispatch next step"
                        );
                    }

                    // Update step state to running in memory (already saved above, update again)
                    // We re-load and update to avoid race conditions
                }
            }

            // Update step_states for newly dispatched steps
            let run = storage
                .get_playbook_run_by_id(run_id)
                .await
                .map_err(|e| ResultHandlerError::StorageError(e.to_string()))?;

            let mut step_states: Vec<StepState> = serde_json::from_value(run.step_states)
                .map_err(|e| ResultHandlerError::ParseError(e.to_string()))?;

            for next_id in &next_step_ids {
                if let Some(ns) = step_states.iter_mut().find(|s| s.step_id == *next_id) {
                    if matches!(ns.status, StepStatus::Pending) {
                        ns.status = StepStatus::Running;
                        ns.started_at = Some(chrono::Utc::now());
                    }
                }
            }

            let step_states_json = serde_json::to_value(&step_states)
                .map_err(|e| ResultHandlerError::ParseError(e.to_string()))?;

            storage
                .update_playbook_run(run_id, "running", step_states_json)
                .await
                .map_err(|e| ResultHandlerError::StorageError(e.to_string()))?;
        }

        Ok(())
    }
}
```

**Step 2: Add result_handler module to playbooks/mod.rs**

Add `pub mod result_handler;` to the top of `services/integration-service/src/playbooks/mod.rs`, after `pub mod executor;`.

**Step 3: Commit**

```bash
git add services/integration-service/src/playbooks/result_handler.rs services/integration-service/src/playbooks/mod.rs
git commit -m "feat(playbooks): add PlaybookResultHandler for DAG orchestration"
```

---

### Task 3: Add get_playbook_run_by_id to Storage

**Files:**
- Modify: `services/integration-service/src/storage/mod.rs`

**Step 1: Add the new method**

The result handler needs to look up a run by ID without knowing the tenant_id (since it comes from Kafka). Add this method to the `Storage` impl block, after `get_playbook_run`:

```rust
    /// Get a playbook run by ID only (used by result handler where tenant_id is unknown)
    pub async fn get_playbook_run_by_id(
        &self,
        run_id: Uuid,
    ) -> Result<PlaybookRunRow, StorageError> {
        let row: Option<PlaybookRunRow> = sqlx::query_as(
            r#"
            SELECT id, playbook_id, tenant_id, status, variables,
                   step_states, started_at, completed_at
            FROM playbook_runs
            WHERE id = $1
            "#,
        )
        .bind(run_id)
        .fetch_optional(&self.pool)
        .await?;

        row.ok_or_else(|| StorageError::NotFound(format!("PlaybookRun {}", run_id)))
    }
```

**Step 2: Commit**

```bash
git add services/integration-service/src/storage/mod.rs
git commit -m "feat(storage): add get_playbook_run_by_id for Kafka result handler"
```

---

### Task 4: Wire PlaybookExecutor into run_playbook API

**Files:**
- Modify: `services/integration-service/src/playbooks/api.rs`

**Step 1: Update run_playbook handler**

Replace the `run_playbook` function to call `PlaybookExecutor::start_run()` after creating the run record. The current handler at line 369-429 creates the run but doesn't dispatch anything.

Update imports at the top of api.rs to add:
```rust
use crate::playbooks::executor::PlaybookExecutor;
```

Then modify `run_playbook()` — after the `create_playbook_run` call and before the response, add the executor call:

```rust
    // Dispatch starting steps to Kafka (if producer available)
    if let Some(producer) = state.engine.kafka_producer() {
        let steps: Vec<Step> = serde_json::from_value(playbook.steps.clone()).map_err(|e| ApiError {
            error: "invalid_playbook".to_string(),
            message: format!("Failed to parse steps for dispatch: {}", e),
        })?;

        if let Err(e) = PlaybookExecutor::start_run(
            producer,
            &state.storage,
            row.id,
            id,
            &tenant_id,
            &steps,
            &req.variables,
        )
        .await
        {
            tracing::error!(
                run_id = %row.id,
                error = %e,
                "Failed to dispatch playbook steps - run created but not started"
            );
            // Run is created but stays in pending - user can retry
        }
    } else {
        tracing::warn!(
            run_id = %row.id,
            "No Kafka producer available - playbook run created but steps not dispatched"
        );
    }
```

Note: The playbook data is already fetched earlier in the handler (the `get_playbook` call). Reuse `playbook.steps` from there.

**Step 2: Commit**

```bash
git add services/integration-service/src/playbooks/api.rs
git commit -m "feat(playbooks): wire executor into run_playbook API endpoint"
```

---

### Task 5: Route Playbook Step Results in Consumer

**Files:**
- Modify: `services/integration-service/src/consumer/mod.rs`

**Step 1: Update ResultConsumer to accept Storage and TaskProducer references**

The consumer currently only handles discovery results. We need to also route `playbook_step` task results to the PlaybookResultHandler. Since the consumer is spawned in `main.rs`, we need to pass it references.

The simplest approach: add a new consumer specifically for playbook results, or extend the existing one.

Extend `ResultConsumer` to:
1. Accept a `Storage` reference and optional `TaskProducer`
2. Check if the result is a playbook step (via task headers or config)
3. Route accordingly

Update `ResultConsumer::new()` to accept storage and producer:
```rust
pub struct ResultConsumer {
    consumer: StreamConsumer,
    asset_service_url: String,
    http_client: reqwest::Client,
    storage: Option<Arc<Storage>>,
    kafka_producer: Option<Arc<TaskProducer>>,
}
```

Update `new()` to accept these optional params:
```rust
pub fn new(
    config: &ConsumerConfig,
    storage: Option<Arc<Storage>>,
    kafka_producer: Option<Arc<TaskProducer>>,
) -> Result<Self> {
    // ... existing code ...
    Ok(Self {
        consumer,
        asset_service_url: config.asset_service_url.clone(),
        http_client: reqwest::Client::new(),
        storage,
        kafka_producer,
    })
}
```

Update `process_message` to check task type from the result. Add before the existing discovery logic:

```rust
async fn process_message(&self, payload: &[u8]) -> Result<()> {
    let result: TaskResult = serde_json::from_slice(payload)?;

    info!(
        task_id = %result.task_id,
        tenant_id = %result.tenant_id,
        status = %result.status,
        "Processing task result"
    );

    // Check if this is a playbook step result
    if let Some(ref output) = result.output {
        if output.get("step_id").is_some() {
            return self.process_playbook_result(&result).await;
        }
    }

    // ... existing discovery result handling below ...
}
```

Add the new method:
```rust
async fn process_playbook_result(&self, result: &TaskResult) -> Result<()> {
    let (storage, producer) = match (&self.storage, &self.kafka_producer) {
        (Some(s), Some(p)) => (s, p),
        _ => {
            warn!("Cannot process playbook result - storage or producer not available");
            return Ok(());
        }
    };

    let step_id = result
        .output
        .as_ref()
        .and_then(|o| o.get("step_id"))
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    let run_id: Uuid = result.integration_id.parse().unwrap_or_default();
    // run_id is in the task's run_id field which maps to the integration_id parse
    // Actually the TaskResult has integration_id as String - but run_id should be in the task result
    // Let's use the run_id from output or parse from task headers
    let run_id_str = result
        .output
        .as_ref()
        .and_then(|o| o.get("run_id"))
        .and_then(|v| v.as_str());

    // Parse run_id - try output first, then fall back to parsing integration_id
    let run_id = if let Some(rid) = run_id_str {
        rid.parse::<Uuid>().unwrap_or_default()
    } else {
        result.integration_id.parse::<Uuid>().unwrap_or_default()
    };

    let success = result.status == "success";
    let output = result.output.clone();
    let error_msg = result.error.as_ref().map(|e| e.message.clone());

    info!(
        run_id = %run_id,
        step_id = %step_id,
        success = success,
        "Routing playbook step result to handler"
    );

    if let Err(e) = crate::playbooks::result_handler::PlaybookResultHandler::handle_step_result(
        producer,
        storage,
        run_id,
        step_id,
        success,
        output,
        error_msg,
    )
    .await
    {
        error!(
            run_id = %run_id,
            step_id = %step_id,
            error = %e,
            "Failed to handle playbook step result"
        );
    }

    Ok(())
}
```

Note: Add `use std::sync::Arc;` and `use crate::storage::Storage;` and `use crate::kafka::TaskProducer;` to the imports.

**Step 2: Update main.rs to pass Storage and TaskProducer to consumer**

In `main.rs`, the consumer is created at line 62-84. Update it to pass storage and producer references:

```rust
if config.consumer.enabled {
    let consumer_config = consumer::ConsumerConfig {
        bootstrap_servers: config.consumer.bootstrap_servers.clone(),
        group_id: config.consumer.group_id.clone(),
        asset_service_url: config.consumer.asset_service_url.clone(),
    };

    // Share storage and producer with consumer
    let consumer_storage = Arc::new(Storage::new(&config.database).await?);
    let consumer_producer = {
        let kafka_cfg = crate::kafka::KafkaConfig {
            bootstrap_servers: config.kafka.brokers.clone(),
            client_id: "integration-service-consumer".to_string(),
            acks: "all".to_string(),
            retries: 3,
            linger_ms: 5,
        };
        match crate::kafka::TaskProducer::new(&kafka_cfg) {
            Ok(p) => Some(Arc::new(p)),
            Err(e) => {
                tracing::warn!("Consumer Kafka producer not available: {}", e);
                None
            }
        }
    };

    tokio::spawn(async move {
        let consumer = match consumer::ResultConsumer::new(
            &consumer_config,
            Some(consumer_storage),
            consumer_producer,
        ) {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("Failed to create result consumer: {}", e);
                return;
            }
        };

        if let Err(e) = consumer.run().await {
            tracing::error!("Result consumer error: {}", e);
        }
    });

    info!("Result consumer started");
}
```

**Step 3: Commit**

```bash
git add services/integration-service/src/consumer/mod.rs services/integration-service/src/main.rs
git commit -m "feat(consumer): route playbook step results to PlaybookResultHandler"
```

---

### Task 6: Build Verification

**Step 1: Run cargo build**

Run: `cd services/integration-service && cargo build 2>&1`
Expected: Build succeeds with no errors

**Step 2: Run all tests**

Run: `cd services/integration-service && cargo test 2>&1`
Expected: All tests pass (including the new executor tests)

**Step 3: Final commit if any fixes needed**

```bash
git add -A
git commit -m "fix: address build issues in playbook kafka dispatcher"
```
