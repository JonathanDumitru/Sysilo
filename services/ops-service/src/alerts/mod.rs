use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, postgres::PgPoolOptions};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Comparison condition for alert rules
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AlertCondition {
    Gt,   // Greater than
    Lt,   // Less than
    Eq,   // Equal to
    Gte,  // Greater than or equal
    Lte,  // Less than or equal
    Ne,   // Not equal
}

impl AlertCondition {
    pub fn as_str(&self) -> &'static str {
        match self {
            AlertCondition::Gt => "gt",
            AlertCondition::Lt => "lt",
            AlertCondition::Eq => "eq",
            AlertCondition::Gte => "gte",
            AlertCondition::Lte => "lte",
            AlertCondition::Ne => "ne",
        }
    }

    pub fn evaluate(&self, value: f64, threshold: f64) -> bool {
        match self {
            AlertCondition::Gt => value > threshold,
            AlertCondition::Lt => value < threshold,
            AlertCondition::Eq => (value - threshold).abs() < f64::EPSILON,
            AlertCondition::Gte => value >= threshold,
            AlertCondition::Lte => value <= threshold,
            AlertCondition::Ne => (value - threshold).abs() >= f64::EPSILON,
        }
    }
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AlertSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

impl AlertSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            AlertSeverity::Critical => "critical",
            AlertSeverity::High => "high",
            AlertSeverity::Medium => "medium",
            AlertSeverity::Low => "low",
            AlertSeverity::Info => "info",
        }
    }
}

/// An alert rule definition
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AlertRule {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub metric_name: String,
    pub condition: String,
    pub threshold: f64,
    pub duration_seconds: i32,
    pub severity: String,
    pub channels: Vec<Uuid>,
    pub labels: Option<serde_json::Value>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request to create an alert rule
#[derive(Debug, Clone, Deserialize)]
pub struct CreateAlertRuleRequest {
    pub name: String,
    pub description: Option<String>,
    pub metric_name: String,
    pub condition: AlertCondition,
    pub threshold: f64,
    pub duration_seconds: Option<i32>,
    pub severity: AlertSeverity,
    pub channels: Option<Vec<Uuid>>,
    pub labels: Option<serde_json::Value>,
}

/// Request to update an alert rule
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateAlertRuleRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub metric_name: Option<String>,
    pub condition: Option<AlertCondition>,
    pub threshold: Option<f64>,
    pub duration_seconds: Option<i32>,
    pub severity: Option<AlertSeverity>,
    pub channels: Option<Vec<Uuid>>,
    pub labels: Option<serde_json::Value>,
    pub enabled: Option<bool>,
}

/// Alert instance status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AlertStatus {
    Firing,
    Resolved,
    Acknowledged,
    Silenced,
}

impl AlertStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            AlertStatus::Firing => "firing",
            AlertStatus::Resolved => "resolved",
            AlertStatus::Acknowledged => "acknowledged",
            AlertStatus::Silenced => "silenced",
        }
    }
}

/// A triggered alert instance
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AlertInstance {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub rule_id: Uuid,
    pub status: String,
    pub triggered_value: Option<f64>,
    pub triggered_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub acknowledged_by: Option<Uuid>,
    pub incident_id: Option<Uuid>,
    pub metadata: Option<serde_json::Value>,
}

/// Alert instance with rule details (flat row for database query)
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AlertInstanceWithRuleRow {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub rule_id: Uuid,
    pub status: String,
    pub triggered_value: Option<f64>,
    pub triggered_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub acknowledged_by: Option<Uuid>,
    pub incident_id: Option<Uuid>,
    pub metadata: Option<serde_json::Value>,
    pub rule_name: String,
    pub severity: String,
}

/// Alert instance with rule details (API response)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertInstanceWithRule {
    pub instance: AlertInstance,
    pub rule_name: String,
    pub severity: String,
}

impl From<AlertInstanceWithRuleRow> for AlertInstanceWithRule {
    fn from(row: AlertInstanceWithRuleRow) -> Self {
        AlertInstanceWithRule {
            instance: AlertInstance {
                id: row.id,
                tenant_id: row.tenant_id,
                rule_id: row.rule_id,
                status: row.status,
                triggered_value: row.triggered_value,
                triggered_at: row.triggered_at,
                resolved_at: row.resolved_at,
                acknowledged_at: row.acknowledged_at,
                acknowledged_by: row.acknowledged_by,
                incident_id: row.incident_id,
                metadata: row.metadata,
            },
            rule_name: row.rule_name,
            severity: row.severity,
        }
    }
}

/// Service for managing alerts
pub struct AlertsService {
    pool: PgPool,
}

impl AlertsService {
    /// Create a new alerts service
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;

        Ok(Self { pool })
    }

    // ========================================================================
    // Alert Rules
    // ========================================================================

    /// List all enabled alert rules across all tenants (for the evaluator)
    pub async fn list_all_enabled_rules(&self) -> Result<Vec<AlertRule>> {
        let rules = sqlx::query_as::<_, AlertRule>(
            r#"
            SELECT id, tenant_id, name, description, metric_name, condition,
                   threshold, duration_seconds, severity, channels, labels,
                   enabled, created_at, updated_at
            FROM alert_rules
            WHERE enabled = true
            ORDER BY tenant_id, name
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rules)
    }

    /// List alert rules
    pub async fn list_rules(
        &self,
        tenant_id: Uuid,
        enabled_only: bool,
    ) -> Result<Vec<AlertRule>> {
        let rules = if enabled_only {
            sqlx::query_as::<_, AlertRule>(
                r#"
                SELECT id, tenant_id, name, description, metric_name, condition,
                       threshold, duration_seconds, severity, channels, labels,
                       enabled, created_at, updated_at
                FROM alert_rules
                WHERE tenant_id = $1 AND enabled = true
                ORDER BY name
                "#
            )
            .bind(tenant_id)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, AlertRule>(
                r#"
                SELECT id, tenant_id, name, description, metric_name, condition,
                       threshold, duration_seconds, severity, channels, labels,
                       enabled, created_at, updated_at
                FROM alert_rules
                WHERE tenant_id = $1
                ORDER BY name
                "#
            )
            .bind(tenant_id)
            .fetch_all(&self.pool)
            .await?
        };

        Ok(rules)
    }

    /// Get a single alert rule
    pub async fn get_rule(&self, tenant_id: Uuid, id: Uuid) -> Result<Option<AlertRule>> {
        let rule = sqlx::query_as::<_, AlertRule>(
            r#"
            SELECT id, tenant_id, name, description, metric_name, condition,
                   threshold, duration_seconds, severity, channels, labels,
                   enabled, created_at, updated_at
            FROM alert_rules
            WHERE tenant_id = $1 AND id = $2
            "#
        )
        .bind(tenant_id)
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(rule)
    }

    /// Create an alert rule
    pub async fn create_rule(
        &self,
        tenant_id: Uuid,
        req: CreateAlertRuleRequest,
    ) -> Result<AlertRule> {
        let duration = req.duration_seconds.unwrap_or(60);
        let channels = req.channels.unwrap_or_default();

        let rule = sqlx::query_as::<_, AlertRule>(
            r#"
            INSERT INTO alert_rules
                (tenant_id, name, description, metric_name, condition, threshold,
                 duration_seconds, severity, channels, labels)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id, tenant_id, name, description, metric_name, condition,
                      threshold, duration_seconds, severity, channels, labels,
                      enabled, created_at, updated_at
            "#
        )
        .bind(tenant_id)
        .bind(&req.name)
        .bind(&req.description)
        .bind(&req.metric_name)
        .bind(req.condition.as_str())
        .bind(req.threshold)
        .bind(duration)
        .bind(req.severity.as_str())
        .bind(&channels)
        .bind(&req.labels)
        .fetch_one(&self.pool)
        .await?;

        Ok(rule)
    }

    /// Update an alert rule
    pub async fn update_rule(
        &self,
        tenant_id: Uuid,
        id: Uuid,
        req: UpdateAlertRuleRequest,
    ) -> Result<Option<AlertRule>> {
        // Get existing rule
        let existing = self.get_rule(tenant_id, id).await?;
        if existing.is_none() {
            return Ok(None);
        }
        let existing = existing.unwrap();

        let name = req.name.unwrap_or(existing.name);
        let description = req.description.or(existing.description);
        let metric_name = req.metric_name.unwrap_or(existing.metric_name);
        let condition = req.condition.map(|c| c.as_str().to_string()).unwrap_or(existing.condition);
        let threshold = req.threshold.unwrap_or(existing.threshold);
        let duration = req.duration_seconds.unwrap_or(existing.duration_seconds);
        let severity = req.severity.map(|s| s.as_str().to_string()).unwrap_or(existing.severity);
        let channels = req.channels.unwrap_or(existing.channels);
        let labels = req.labels.or(existing.labels);
        let enabled = req.enabled.unwrap_or(existing.enabled);

        let rule = sqlx::query_as::<_, AlertRule>(
            r#"
            UPDATE alert_rules SET
                name = $1, description = $2, metric_name = $3, condition = $4,
                threshold = $5, duration_seconds = $6, severity = $7, channels = $8,
                labels = $9, enabled = $10, updated_at = NOW()
            WHERE tenant_id = $11 AND id = $12
            RETURNING id, tenant_id, name, description, metric_name, condition,
                      threshold, duration_seconds, severity, channels, labels,
                      enabled, created_at, updated_at
            "#
        )
        .bind(&name)
        .bind(&description)
        .bind(&metric_name)
        .bind(&condition)
        .bind(threshold)
        .bind(duration)
        .bind(&severity)
        .bind(&channels)
        .bind(&labels)
        .bind(enabled)
        .bind(tenant_id)
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(Some(rule))
    }

    /// Delete an alert rule
    pub async fn delete_rule(&self, tenant_id: Uuid, id: Uuid) -> Result<bool> {
        let result = sqlx::query(
            "DELETE FROM alert_rules WHERE tenant_id = $1 AND id = $2"
        )
        .bind(tenant_id)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    // ========================================================================
    // Alert Instances
    // ========================================================================

    /// List alert instances
    pub async fn list_instances(
        &self,
        tenant_id: Uuid,
        status: Option<String>,
        limit: i64,
    ) -> Result<Vec<AlertInstanceWithRule>> {
        let rows = sqlx::query_as::<_, AlertInstanceWithRuleRow>(
            r#"
            SELECT ai.id, ai.tenant_id, ai.rule_id, ai.status, ai.triggered_value,
                   ai.triggered_at, ai.resolved_at, ai.acknowledged_at,
                   ai.acknowledged_by, ai.incident_id, ai.metadata,
                   ar.name AS rule_name, ar.severity
            FROM alert_instances ai
            JOIN alert_rules ar ON ar.id = ai.rule_id
            WHERE ai.tenant_id = $1
              AND ($2::text IS NULL OR ai.status = $2)
            ORDER BY ai.triggered_at DESC
            LIMIT $3
            "#
        )
        .bind(tenant_id)
        .bind(status)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let result = rows.into_iter().map(AlertInstanceWithRule::from).collect();

        Ok(result)
    }

    /// Create an alert instance (fire an alert)
    pub async fn fire_alert(
        &self,
        tenant_id: Uuid,
        rule_id: Uuid,
        triggered_value: f64,
        metadata: Option<serde_json::Value>,
    ) -> Result<AlertInstance> {
        let instance = sqlx::query_as::<_, AlertInstance>(
            r#"
            INSERT INTO alert_instances
                (tenant_id, rule_id, triggered_value, metadata)
            VALUES ($1, $2, $3, $4)
            RETURNING id, tenant_id, rule_id, status, triggered_value, triggered_at,
                      resolved_at, acknowledged_at, acknowledged_by, incident_id, metadata
            "#
        )
        .bind(tenant_id)
        .bind(rule_id)
        .bind(triggered_value)
        .bind(&metadata)
        .fetch_one(&self.pool)
        .await?;

        Ok(instance)
    }

    /// Acknowledge an alert
    pub async fn acknowledge_alert(
        &self,
        tenant_id: Uuid,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<AlertInstance>> {
        let instance = sqlx::query_as::<_, AlertInstance>(
            r#"
            UPDATE alert_instances SET
                status = 'acknowledged',
                acknowledged_at = NOW(),
                acknowledged_by = $1
            WHERE tenant_id = $2 AND id = $3 AND status = 'firing'
            RETURNING id, tenant_id, rule_id, status, triggered_value, triggered_at,
                      resolved_at, acknowledged_at, acknowledged_by, incident_id, metadata
            "#
        )
        .bind(user_id)
        .bind(tenant_id)
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(instance)
    }

    /// Resolve an alert
    pub async fn resolve_alert(&self, tenant_id: Uuid, id: Uuid) -> Result<Option<AlertInstance>> {
        let instance = sqlx::query_as::<_, AlertInstance>(
            r#"
            UPDATE alert_instances SET
                status = 'resolved',
                resolved_at = NOW()
            WHERE tenant_id = $1 AND id = $2 AND status IN ('firing', 'acknowledged')
            RETURNING id, tenant_id, rule_id, status, triggered_value, triggered_at,
                      resolved_at, acknowledged_at, acknowledged_by, incident_id, metadata
            "#
        )
        .bind(tenant_id)
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(instance)
    }

    /// Link an alert to an incident
    pub async fn link_to_incident(
        &self,
        tenant_id: Uuid,
        alert_id: Uuid,
        incident_id: Uuid,
    ) -> Result<bool> {
        let result = sqlx::query(
            r#"
            UPDATE alert_instances SET incident_id = $1
            WHERE tenant_id = $2 AND id = $3
            "#
        )
        .bind(incident_id)
        .bind(tenant_id)
        .bind(alert_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}

/// Parse condition string to enum
#[allow(dead_code)]
fn parse_condition(s: &str) -> AlertCondition {
    match s {
        "gt" => AlertCondition::Gt,
        "lt" => AlertCondition::Lt,
        "eq" => AlertCondition::Eq,
        "gte" => AlertCondition::Gte,
        "lte" => AlertCondition::Lte,
        "ne" => AlertCondition::Ne,
        _ => AlertCondition::Gt,
    }
}
