use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Trigger types for playbooks
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TriggerType {
    #[default]
    Manual,
    Scheduled,
    Webhook,
    Event,
}

/// Step types within a playbook
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepType {
    Integration,
    Webhook,
    Wait,
    Condition,
    Approval,
}

/// Variable definition for playbook inputs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variable {
    pub name: String,
    pub var_type: String,
    pub required: bool,
    #[serde(default)]
    pub default_value: Option<String>,
}

/// A single step in the playbook workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    pub id: String,
    pub step_type: StepType,
    pub name: String,
    pub config: serde_json::Value,
    #[serde(default)]
    pub on_success: Vec<String>,
    #[serde(default)]
    pub on_failure: Vec<String>,
}

/// Status of a playbook run
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    #[default]
    Pending,
    Running,
    WaitingApproval,
    Completed,
    Failed,
    Cancelled,
}

/// Status of a single step within a run
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum StepStatus {
    #[default]
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
}

/// State of a step during execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepState {
    pub step_id: String,
    pub status: StepStatus,
    #[serde(default)]
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default)]
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default)]
    pub output: Option<serde_json::Value>,
    #[serde(default)]
    pub error: Option<String>,
}

/// Full playbook definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Playbook {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub trigger_type: TriggerType,
    pub steps: Vec<Step>,
    pub variables: Vec<Variable>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// A running instance of a playbook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybookRun {
    pub id: Uuid,
    pub playbook_id: Uuid,
    pub tenant_id: Uuid,
    pub status: RunStatus,
    pub variables: serde_json::Value,
    pub step_states: Vec<StepState>,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_step_type_serialization() {
        let step = Step {
            id: "step-1".to_string(),
            step_type: StepType::Integration,
            name: "Test Step".to_string(),
            config: serde_json::json!({"integration_id": "uuid"}),
            on_success: vec!["step-2".to_string()],
            on_failure: vec![],
        };
        let json = serde_json::to_string(&step).unwrap();
        assert!(json.contains("integration"));
    }

    #[test]
    fn test_run_status_default() {
        let status = RunStatus::default();
        assert!(matches!(status, RunStatus::Pending));
    }
}
