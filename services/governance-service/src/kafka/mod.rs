use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::Timeout;
use serde::Serialize;
use std::time::Duration;
use thiserror::Error;
use tracing::{info, warn};
use uuid::Uuid;

/// Kafka topic names for governance events
pub mod topics {
    /// Policy violations
    pub const VIOLATIONS: &str = "sysilo.governance.violations";
    /// Approval workflow events
    pub const APPROVALS: &str = "sysilo.governance.approvals";
    /// Audit log events
    pub const AUDIT: &str = "sysilo.governance.audit";
    /// Compliance events
    pub const COMPLIANCE: &str = "sysilo.governance.compliance";
}

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
            client_id: "governance-service".to_string(),
            acks: "all".to_string(),
            retries: 3,
            linger_ms: 5,
        }
    }
}

impl KafkaConfig {
    pub fn from_env() -> Self {
        let bootstrap_servers = std::env::var("KAFKA_BROKERS")
            .unwrap_or_else(|_| "localhost:9092".to_string());

        Self {
            bootstrap_servers,
            ..Default::default()
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

/// Kafka event producer for governance events
pub struct GovernanceEventProducer {
    producer: FutureProducer,
}

impl GovernanceEventProducer {
    /// Create a new event producer
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

        info!("Kafka producer initialized for governance events");

        Ok(Self { producer })
    }

    /// Try to create a producer, return None if it fails
    pub fn try_new(config: &KafkaConfig) -> Option<Self> {
        match Self::new(config) {
            Ok(producer) => Some(producer),
            Err(e) => {
                warn!("Failed to create Kafka producer: {}. Running without event publishing.", e);
                None
            }
        }
    }

    /// Send a policy violation event
    pub async fn send_violation(&self, event: &PolicyViolationEvent) -> Result<(), KafkaError> {
        self.send_event(topics::VIOLATIONS, &event.tenant_id, event).await
    }

    /// Send an approval workflow event
    pub async fn send_approval_event(&self, event: &ApprovalEvent) -> Result<(), KafkaError> {
        self.send_event(topics::APPROVALS, &event.tenant_id, event).await
    }

    /// Send an audit log event
    pub async fn send_audit_event(&self, event: &AuditEvent) -> Result<(), KafkaError> {
        self.send_event(topics::AUDIT, &event.tenant_id, event).await
    }

    /// Send a compliance event
    pub async fn send_compliance_event(&self, event: &ComplianceEvent) -> Result<(), KafkaError> {
        self.send_event(topics::COMPLIANCE, &event.tenant_id, event).await
    }

    /// Generic event sender
    async fn send_event<T: Serialize>(
        &self,
        topic: &str,
        tenant_id: &str,
        event: &T,
    ) -> Result<(), KafkaError> {
        let payload = serde_json::to_string(event)
            .map_err(|e| KafkaError::SerializationFailed(e.to_string()))?;

        let record = FutureRecord::to(topic)
            .key(tenant_id)
            .payload(&payload);

        self.producer
            .send(record, Timeout::After(Duration::from_secs(10)))
            .await
            .map_err(|(e, _)| KafkaError::SendFailed(e.to_string()))?;

        info!(topic = %topic, tenant_id = %tenant_id, "Governance event sent");

        Ok(())
    }
}

/// Policy violation event
#[derive(Debug, Clone, Serialize)]
pub struct PolicyViolationEvent {
    pub event_type: String,
    pub violation_id: Uuid,
    pub policy_id: Uuid,
    pub policy_name: String,
    pub tenant_id: String,
    pub resource_type: String,
    pub resource_id: Uuid,
    pub severity: String,
    pub details: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl PolicyViolationEvent {
    pub fn new(
        violation_id: Uuid,
        policy_id: Uuid,
        policy_name: String,
        tenant_id: String,
        resource_type: String,
        resource_id: Uuid,
        severity: String,
        details: serde_json::Value,
    ) -> Self {
        Self {
            event_type: "policy.violation.created".to_string(),
            violation_id,
            policy_id,
            policy_name,
            tenant_id,
            resource_type,
            resource_id,
            severity,
            details,
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn resolved(
        violation_id: Uuid,
        policy_id: Uuid,
        tenant_id: String,
        resolved_by: Option<Uuid>,
    ) -> Self {
        Self {
            event_type: "policy.violation.resolved".to_string(),
            violation_id,
            policy_id,
            policy_name: String::new(),
            tenant_id,
            resource_type: String::new(),
            resource_id: Uuid::nil(),
            severity: String::new(),
            details: serde_json::json!({ "resolved_by": resolved_by }),
            timestamp: chrono::Utc::now(),
        }
    }
}

/// Approval workflow event
#[derive(Debug, Clone, Serialize)]
pub struct ApprovalEvent {
    pub event_type: String,
    pub request_id: Uuid,
    pub workflow_id: Uuid,
    pub tenant_id: String,
    pub resource_type: String,
    pub resource_id: Uuid,
    pub requester_id: Uuid,
    pub status: String,
    pub details: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl ApprovalEvent {
    pub fn request_created(
        request_id: Uuid,
        workflow_id: Uuid,
        tenant_id: String,
        resource_type: String,
        resource_id: Uuid,
        requester_id: Uuid,
        auto_approved: bool,
    ) -> Self {
        Self {
            event_type: if auto_approved {
                "approval.auto_approved".to_string()
            } else {
                "approval.request.created".to_string()
            },
            request_id,
            workflow_id,
            tenant_id,
            resource_type,
            resource_id,
            requester_id,
            status: if auto_approved { "approved" } else { "pending" }.to_string(),
            details: serde_json::json!({ "auto_approved": auto_approved }),
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn decision_made(
        request_id: Uuid,
        workflow_id: Uuid,
        tenant_id: String,
        resource_type: String,
        resource_id: Uuid,
        approver_id: Uuid,
        decision: &str,
        new_status: &str,
    ) -> Self {
        Self {
            event_type: format!("approval.{}", decision),
            request_id,
            workflow_id,
            tenant_id,
            resource_type,
            resource_id,
            requester_id: Uuid::nil(), // Not relevant for decision events
            status: new_status.to_string(),
            details: serde_json::json!({ "approver_id": approver_id, "decision": decision }),
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn stage_advanced(
        request_id: Uuid,
        workflow_id: Uuid,
        tenant_id: String,
        new_stage: i32,
    ) -> Self {
        Self {
            event_type: "approval.stage.advanced".to_string(),
            request_id,
            workflow_id,
            tenant_id,
            resource_type: String::new(),
            resource_id: Uuid::nil(),
            requester_id: Uuid::nil(),
            status: "pending".to_string(),
            details: serde_json::json!({ "new_stage": new_stage }),
            timestamp: chrono::Utc::now(),
        }
    }
}

/// Audit log event
#[derive(Debug, Clone, Serialize)]
pub struct AuditEvent {
    pub event_type: String,
    pub audit_id: Uuid,
    pub tenant_id: String,
    pub actor_id: Option<Uuid>,
    pub actor_type: String,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<Uuid>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl AuditEvent {
    pub fn new(
        audit_id: Uuid,
        tenant_id: String,
        actor_id: Option<Uuid>,
        actor_type: String,
        action: String,
        resource_type: String,
        resource_id: Option<Uuid>,
    ) -> Self {
        Self {
            event_type: "audit.entry.created".to_string(),
            audit_id,
            tenant_id,
            actor_id,
            actor_type,
            action,
            resource_type,
            resource_id,
            timestamp: chrono::Utc::now(),
        }
    }
}

/// Compliance event
#[derive(Debug, Clone, Serialize)]
pub struct ComplianceEvent {
    pub event_type: String,
    pub tenant_id: String,
    pub framework_id: Uuid,
    pub framework_name: String,
    pub control_id: Option<String>,
    pub status: String,
    pub details: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl ComplianceEvent {
    pub fn assessment_completed(
        tenant_id: String,
        framework_id: Uuid,
        framework_name: String,
        status: String,
        summary: serde_json::Value,
    ) -> Self {
        Self {
            event_type: "compliance.assessment.completed".to_string(),
            tenant_id,
            framework_id,
            framework_name,
            control_id: None,
            status,
            details: summary,
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn control_status_changed(
        tenant_id: String,
        framework_id: Uuid,
        framework_name: String,
        control_id: String,
        old_status: &str,
        new_status: &str,
    ) -> Self {
        Self {
            event_type: "compliance.control.status_changed".to_string(),
            tenant_id,
            framework_id,
            framework_name,
            control_id: Some(control_id),
            status: new_status.to_string(),
            details: serde_json::json!({
                "old_status": old_status,
                "new_status": new_status
            }),
            timestamp: chrono::Utc::now(),
        }
    }
}
