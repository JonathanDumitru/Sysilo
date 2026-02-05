use tracing::{error, info};
use uuid::Uuid;

use crate::kafka::TaskProducer;
use crate::playbooks::executor::PlaybookExecutor;
use crate::playbooks::{Step, StepState, StepStatus};
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
        // 1. Load the run from DB (no tenant_id needed)
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
            // For condition steps, use the next_steps from output if present
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
            // No next steps — check if all steps are done
            let all_done = step_states.iter().all(|s| {
                matches!(
                    s.status,
                    StepStatus::Completed | StepStatus::Failed | StepStatus::Skipped
                )
            });

            if all_done {
                let any_failed = step_states
                    .iter()
                    .any(|s| matches!(s.status, StepStatus::Failed));
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
                    // Mark step as running before dispatching
                    if let Some(ns) = step_states.iter_mut().find(|s| s.step_id == *next_id) {
                        ns.status = StepStatus::Running;
                        ns.started_at = Some(chrono::Utc::now());
                    }

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
                }
            }

            // Save updated step_states with running status for dispatched steps
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
