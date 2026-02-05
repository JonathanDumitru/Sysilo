use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, postgres::PgPoolOptions};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Incident severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IncidentSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

impl IncidentSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            IncidentSeverity::Critical => "critical",
            IncidentSeverity::High => "high",
            IncidentSeverity::Medium => "medium",
            IncidentSeverity::Low => "low",
            IncidentSeverity::Info => "info",
        }
    }
}

/// Incident status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IncidentStatus {
    Open,
    Acknowledged,
    Investigating,
    Resolved,
    Closed,
}

impl IncidentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            IncidentStatus::Open => "open",
            IncidentStatus::Acknowledged => "acknowledged",
            IncidentStatus::Investigating => "investigating",
            IncidentStatus::Resolved => "resolved",
            IncidentStatus::Closed => "closed",
        }
    }
}

/// An incident record
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Incident {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub severity: String,
    pub status: String,
    pub priority: i32,
    pub assignee_id: Option<Uuid>,
    pub source: Option<String>,
    pub source_ref: Option<Uuid>,
    pub labels: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub closed_at: Option<DateTime<Utc>>,
}

/// Request to create an incident
#[derive(Debug, Clone, Deserialize)]
pub struct CreateIncidentRequest {
    pub title: String,
    pub description: Option<String>,
    pub severity: IncidentSeverity,
    pub priority: Option<i32>,
    pub assignee_id: Option<Uuid>,
    pub source: Option<String>,
    pub source_ref: Option<Uuid>,
    pub labels: Option<serde_json::Value>,
}

/// Request to update an incident
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateIncidentRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub severity: Option<IncidentSeverity>,
    pub status: Option<IncidentStatus>,
    pub priority: Option<i32>,
    pub assignee_id: Option<Uuid>,
    pub labels: Option<serde_json::Value>,
}

/// An incident timeline event
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct IncidentEvent {
    pub id: Uuid,
    pub incident_id: Uuid,
    pub event_type: String,
    pub content: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

/// Event types for incident timeline
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IncidentEventType {
    Created,
    StatusChanged,
    SeverityChanged,
    Assigned,
    Comment,
    AlertLinked,
    Resolved,
    Closed,
    Reopened,
}

impl IncidentEventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            IncidentEventType::Created => "created",
            IncidentEventType::StatusChanged => "status_changed",
            IncidentEventType::SeverityChanged => "severity_changed",
            IncidentEventType::Assigned => "assigned",
            IncidentEventType::Comment => "comment",
            IncidentEventType::AlertLinked => "alert_linked",
            IncidentEventType::Resolved => "resolved",
            IncidentEventType::Closed => "closed",
            IncidentEventType::Reopened => "reopened",
        }
    }
}

/// Request to add an incident event
#[derive(Debug, Clone, Deserialize)]
pub struct AddIncidentEventRequest {
    pub event_type: IncidentEventType,
    pub content: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// Service for managing incidents
pub struct IncidentsService {
    pool: PgPool,
}

impl IncidentsService {
    /// Create a new incidents service
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;

        Ok(Self { pool })
    }

    /// List incidents
    pub async fn list_incidents(
        &self,
        tenant_id: Uuid,
        status: Option<String>,
        severity: Option<String>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<Incident>, i64)> {
        let incidents = sqlx::query_as::<_, Incident>(
            r#"
            SELECT id, tenant_id, title, description, severity, status, priority,
                   assignee_id, source, source_ref, labels, created_at, updated_at,
                   acknowledged_at, resolved_at, closed_at
            FROM incidents
            WHERE tenant_id = $1
              AND ($2::text IS NULL OR status = $2)
              AND ($3::text IS NULL OR severity = $3)
            ORDER BY
                CASE severity
                    WHEN 'critical' THEN 1
                    WHEN 'high' THEN 2
                    WHEN 'medium' THEN 3
                    WHEN 'low' THEN 4
                    ELSE 5
                END,
                created_at DESC
            LIMIT $4 OFFSET $5
            "#
        )
        .bind(tenant_id)
        .bind(&status)
        .bind(&severity)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        let total: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*)
            FROM incidents
            WHERE tenant_id = $1
              AND ($2::text IS NULL OR status = $2)
              AND ($3::text IS NULL OR severity = $3)
            "#
        )
        .bind(tenant_id)
        .bind(&status)
        .bind(&severity)
        .fetch_one(&self.pool)
        .await?;

        Ok((incidents, total.0))
    }

    /// Get a single incident
    pub async fn get_incident(&self, tenant_id: Uuid, id: Uuid) -> Result<Option<Incident>> {
        let incident = sqlx::query_as::<_, Incident>(
            r#"
            SELECT id, tenant_id, title, description, severity, status, priority,
                   assignee_id, source, source_ref, labels, created_at, updated_at,
                   acknowledged_at, resolved_at, closed_at
            FROM incidents
            WHERE tenant_id = $1 AND id = $2
            "#
        )
        .bind(tenant_id)
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(incident)
    }

    /// Create an incident
    pub async fn create_incident(
        &self,
        tenant_id: Uuid,
        req: CreateIncidentRequest,
        created_by: Option<Uuid>,
    ) -> Result<Incident> {
        let priority = req.priority.unwrap_or(3);

        let incident = sqlx::query_as::<_, Incident>(
            r#"
            INSERT INTO incidents
                (tenant_id, title, description, severity, priority, assignee_id,
                 source, source_ref, labels)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id, tenant_id, title, description, severity, status, priority,
                      assignee_id, source, source_ref, labels, created_at, updated_at,
                      acknowledged_at, resolved_at, closed_at
            "#
        )
        .bind(tenant_id)
        .bind(&req.title)
        .bind(&req.description)
        .bind(req.severity.as_str())
        .bind(priority)
        .bind(req.assignee_id)
        .bind(&req.source)
        .bind(req.source_ref)
        .bind(&req.labels)
        .fetch_one(&self.pool)
        .await?;

        // Add creation event
        self.add_event(
            incident.id,
            IncidentEventType::Created,
            Some(format!("Incident created: {}", req.title)),
            None,
            created_by,
        ).await?;

        Ok(incident)
    }

    /// Update an incident
    pub async fn update_incident(
        &self,
        tenant_id: Uuid,
        id: Uuid,
        req: UpdateIncidentRequest,
        updated_by: Option<Uuid>,
    ) -> Result<Option<Incident>> {
        let existing = self.get_incident(tenant_id, id).await?;
        if existing.is_none() {
            return Ok(None);
        }
        let existing = existing.unwrap();

        let title = req.title.unwrap_or(existing.title);
        let description = req.description.or(existing.description);
        let severity = req.severity.map(|s| s.as_str().to_string()).unwrap_or(existing.severity);
        let status = req.status.clone().map(|s| s.as_str().to_string()).unwrap_or(existing.status.clone());
        let priority = req.priority.unwrap_or(existing.priority);
        let assignee_id = req.assignee_id.or(existing.assignee_id);
        let labels = req.labels.or(existing.labels);

        // Determine timestamp updates based on status change
        let mut acknowledged_at = existing.acknowledged_at;
        let mut resolved_at = existing.resolved_at;
        let mut closed_at = existing.closed_at;

        if let Some(new_status) = &req.status {
            match new_status {
                IncidentStatus::Acknowledged if existing.acknowledged_at.is_none() => {
                    acknowledged_at = Some(Utc::now());
                }
                IncidentStatus::Resolved if existing.resolved_at.is_none() => {
                    resolved_at = Some(Utc::now());
                }
                IncidentStatus::Closed if existing.closed_at.is_none() => {
                    closed_at = Some(Utc::now());
                }
                _ => {}
            }
        }

        let incident = sqlx::query_as::<_, Incident>(
            r#"
            UPDATE incidents SET
                title = $1, description = $2, severity = $3, status = $4,
                priority = $5, assignee_id = $6, labels = $7,
                acknowledged_at = $8, resolved_at = $9, closed_at = $10,
                updated_at = NOW()
            WHERE tenant_id = $11 AND id = $12
            RETURNING id, tenant_id, title, description, severity, status, priority,
                      assignee_id, source, source_ref, labels, created_at, updated_at,
                      acknowledged_at, resolved_at, closed_at
            "#
        )
        .bind(&title)
        .bind(&description)
        .bind(&severity)
        .bind(&status)
        .bind(priority)
        .bind(assignee_id)
        .bind(&labels)
        .bind(acknowledged_at)
        .bind(resolved_at)
        .bind(closed_at)
        .bind(tenant_id)
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        // Add status change event if status changed
        if existing.status != status {
            self.add_event(
                id,
                IncidentEventType::StatusChanged,
                Some(format!("Status changed from {} to {}", existing.status, status)),
                None,
                updated_by,
            ).await?;
        }

        Ok(Some(incident))
    }

    /// Resolve an incident
    pub async fn resolve_incident(
        &self,
        tenant_id: Uuid,
        id: Uuid,
        resolution_note: Option<String>,
        resolved_by: Option<Uuid>,
    ) -> Result<Option<Incident>> {
        let incident = sqlx::query_as::<_, Incident>(
            r#"
            UPDATE incidents SET
                status = 'resolved',
                resolved_at = NOW(),
                updated_at = NOW()
            WHERE tenant_id = $1 AND id = $2 AND status NOT IN ('resolved', 'closed')
            RETURNING id, tenant_id, title, description, severity, status, priority,
                      assignee_id, source, source_ref, labels, created_at, updated_at,
                      acknowledged_at, resolved_at, closed_at
            "#
        )
        .bind(tenant_id)
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(ref inc) = incident {
            self.add_event(
                inc.id,
                IncidentEventType::Resolved,
                resolution_note.or(Some("Incident resolved".to_string())),
                None,
                resolved_by,
            ).await?;
        }

        Ok(incident)
    }

    // ========================================================================
    // Incident Events
    // ========================================================================

    /// List events for an incident
    pub async fn list_events(&self, incident_id: Uuid) -> Result<Vec<IncidentEvent>> {
        let events = sqlx::query_as::<_, IncidentEvent>(
            r#"
            SELECT id, incident_id, event_type, content, metadata, created_by, created_at
            FROM incident_events
            WHERE incident_id = $1
            ORDER BY created_at ASC
            "#
        )
        .bind(incident_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(events)
    }

    /// Add an event to an incident timeline
    pub async fn add_event(
        &self,
        incident_id: Uuid,
        event_type: IncidentEventType,
        content: Option<String>,
        metadata: Option<serde_json::Value>,
        created_by: Option<Uuid>,
    ) -> Result<IncidentEvent> {
        let event = sqlx::query_as::<_, IncidentEvent>(
            r#"
            INSERT INTO incident_events (incident_id, event_type, content, metadata, created_by)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, incident_id, event_type, content, metadata, created_by, created_at
            "#
        )
        .bind(incident_id)
        .bind(event_type.as_str())
        .bind(&content)
        .bind(&metadata)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await?;

        Ok(event)
    }

    /// Create incident from alert (auto-creation for critical alerts)
    pub async fn create_from_alert(
        &self,
        tenant_id: Uuid,
        alert_id: Uuid,
        rule_name: &str,
        severity: &str,
        triggered_value: f64,
    ) -> Result<Incident> {
        let title = format!("Alert: {}", rule_name);
        let description = format!(
            "Automatically created from alert. Triggered value: {}",
            triggered_value
        );

        let incident_severity = match severity {
            "critical" => IncidentSeverity::Critical,
            "high" => IncidentSeverity::High,
            "medium" => IncidentSeverity::Medium,
            "low" => IncidentSeverity::Low,
            _ => IncidentSeverity::Info,
        };

        let incident = self.create_incident(
            tenant_id,
            CreateIncidentRequest {
                title,
                description: Some(description),
                severity: incident_severity,
                priority: Some(if severity == "critical" { 1 } else { 2 }),
                assignee_id: None,
                source: Some("alert".to_string()),
                source_ref: Some(alert_id),
                labels: None,
            },
            None,
        ).await?;

        // Link alert to incident
        sqlx::query(
            "INSERT INTO incident_alerts (incident_id, alert_instance_id) VALUES ($1, $2)"
        )
        .bind(incident.id)
        .bind(alert_id)
        .execute(&self.pool)
        .await?;

        Ok(incident)
    }
}
