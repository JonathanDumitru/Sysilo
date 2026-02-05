use tracing::info;
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
        let step_states: Vec<StepState> = steps
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
