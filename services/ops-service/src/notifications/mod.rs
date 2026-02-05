use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, postgres::PgPoolOptions};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Notification channel types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChannelType {
    Email,
    Slack,
    Webhook,
    Pagerduty,
    Teams,
    Opsgenie,
}

impl ChannelType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ChannelType::Email => "email",
            ChannelType::Slack => "slack",
            ChannelType::Webhook => "webhook",
            ChannelType::Pagerduty => "pagerduty",
            ChannelType::Teams => "teams",
            ChannelType::Opsgenie => "opsgenie",
        }
    }
}

/// A notification channel configuration
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct NotificationChannel {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub channel_type: String,
    pub config: serde_json::Value,
    pub default_for_severity: Option<Vec<String>>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request to create a notification channel
#[derive(Debug, Clone, Deserialize)]
pub struct CreateChannelRequest {
    pub name: String,
    pub channel_type: ChannelType,
    pub config: serde_json::Value,
    pub default_for_severity: Option<Vec<String>>,
}

/// Request to update a notification channel
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateChannelRequest {
    pub name: Option<String>,
    pub config: Option<serde_json::Value>,
    pub default_for_severity: Option<Vec<String>>,
    pub enabled: Option<bool>,
}

/// A notification delivery record
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct NotificationDelivery {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub channel_id: Uuid,
    pub alert_instance_id: Option<Uuid>,
    pub incident_id: Option<Uuid>,
    pub status: String,
    pub attempt_count: i32,
    pub last_attempt_at: Option<DateTime<Utc>>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub payload: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

/// Notification payload for sending
#[derive(Debug, Clone, Serialize)]
pub struct NotificationPayload {
    pub title: String,
    pub message: String,
    pub severity: String,
    pub source: String,
    pub source_url: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// Trait for notification senders
#[async_trait]
pub trait NotificationSender: Send + Sync {
    async fn send(&self, payload: &NotificationPayload, config: &serde_json::Value) -> Result<()>;
    fn channel_type(&self) -> ChannelType;
}

/// Service for managing notifications
pub struct NotificationService {
    pool: PgPool,
}

impl NotificationService {
    /// Create a new notification service
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;

        Ok(Self { pool })
    }

    // ========================================================================
    // Channel Management
    // ========================================================================

    /// List notification channels
    pub async fn list_channels(&self, tenant_id: Uuid) -> Result<Vec<NotificationChannel>> {
        let channels = sqlx::query_as::<_, NotificationChannel>(
            r#"
            SELECT id, tenant_id, name, channel_type, config, default_for_severity,
                   enabled, created_at, updated_at
            FROM notification_channels
            WHERE tenant_id = $1
            ORDER BY name
            "#
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(channels)
    }

    /// Get a single channel
    pub async fn get_channel(&self, tenant_id: Uuid, id: Uuid) -> Result<Option<NotificationChannel>> {
        let channel = sqlx::query_as::<_, NotificationChannel>(
            r#"
            SELECT id, tenant_id, name, channel_type, config, default_for_severity,
                   enabled, created_at, updated_at
            FROM notification_channels
            WHERE tenant_id = $1 AND id = $2
            "#
        )
        .bind(tenant_id)
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(channel)
    }

    /// Create a notification channel
    pub async fn create_channel(
        &self,
        tenant_id: Uuid,
        req: CreateChannelRequest,
    ) -> Result<NotificationChannel> {
        let channel = sqlx::query_as::<_, NotificationChannel>(
            r#"
            INSERT INTO notification_channels
                (tenant_id, name, channel_type, config, default_for_severity)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, tenant_id, name, channel_type, config, default_for_severity,
                      enabled, created_at, updated_at
            "#
        )
        .bind(tenant_id)
        .bind(&req.name)
        .bind(req.channel_type.as_str())
        .bind(&req.config)
        .bind(&req.default_for_severity)
        .fetch_one(&self.pool)
        .await?;

        Ok(channel)
    }

    /// Update a notification channel
    pub async fn update_channel(
        &self,
        tenant_id: Uuid,
        id: Uuid,
        req: UpdateChannelRequest,
    ) -> Result<Option<NotificationChannel>> {
        let existing = self.get_channel(tenant_id, id).await?;
        if existing.is_none() {
            return Ok(None);
        }
        let existing = existing.unwrap();

        let name = req.name.unwrap_or(existing.name);
        let config = req.config.unwrap_or(existing.config);
        let default_for_severity = req.default_for_severity.or(existing.default_for_severity);
        let enabled = req.enabled.unwrap_or(existing.enabled);

        let channel = sqlx::query_as::<_, NotificationChannel>(
            r#"
            UPDATE notification_channels SET
                name = $1, config = $2, default_for_severity = $3, enabled = $4,
                updated_at = NOW()
            WHERE tenant_id = $5 AND id = $6
            RETURNING id, tenant_id, name, channel_type, config, default_for_severity,
                      enabled, created_at, updated_at
            "#
        )
        .bind(&name)
        .bind(&config)
        .bind(&default_for_severity)
        .bind(enabled)
        .bind(tenant_id)
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(Some(channel))
    }

    /// Delete a notification channel
    pub async fn delete_channel(&self, tenant_id: Uuid, id: Uuid) -> Result<bool> {
        let result = sqlx::query(
            "DELETE FROM notification_channels WHERE tenant_id = $1 AND id = $2"
        )
        .bind(tenant_id)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    // ========================================================================
    // Notification Sending
    // ========================================================================

    /// Send a notification to specified channels
    pub async fn send_notification(
        &self,
        tenant_id: Uuid,
        channel_ids: &[Uuid],
        payload: NotificationPayload,
        alert_instance_id: Option<Uuid>,
        incident_id: Option<Uuid>,
    ) -> Result<Vec<Uuid>> {
        let mut delivery_ids = Vec::new();

        for channel_id in channel_ids {
            let channel = self.get_channel(tenant_id, *channel_id).await?;
            if let Some(channel) = channel {
                if !channel.enabled {
                    continue;
                }

                // Create delivery record
                let delivery_id = self.create_delivery(
                    tenant_id,
                    *channel_id,
                    alert_instance_id,
                    incident_id,
                    &payload,
                ).await?;

                // Attempt to send
                match self.send_to_channel(&channel, &payload).await {
                    Ok(()) => {
                        self.mark_delivered(delivery_id).await?;
                    }
                    Err(e) => {
                        self.mark_failed(delivery_id, &e.to_string()).await?;
                    }
                }

                delivery_ids.push(delivery_id);
            }
        }

        Ok(delivery_ids)
    }

    /// Send notification to channels matching severity
    pub async fn send_by_severity(
        &self,
        tenant_id: Uuid,
        severity: &str,
        payload: NotificationPayload,
        alert_instance_id: Option<Uuid>,
        incident_id: Option<Uuid>,
    ) -> Result<Vec<Uuid>> {
        // Find channels that handle this severity
        let channels = sqlx::query_as::<_, NotificationChannel>(
            r#"
            SELECT id, tenant_id, name, channel_type, config, default_for_severity,
                   enabled, created_at, updated_at
            FROM notification_channels
            WHERE tenant_id = $1
              AND enabled = true
              AND ($2 = ANY(default_for_severity) OR default_for_severity IS NULL)
            "#
        )
        .bind(tenant_id)
        .bind(severity)
        .fetch_all(&self.pool)
        .await?;

        let channel_ids: Vec<Uuid> = channels.iter().map(|c| c.id).collect();
        self.send_notification(tenant_id, &channel_ids, payload, alert_instance_id, incident_id).await
    }

    /// Create a delivery record
    async fn create_delivery(
        &self,
        tenant_id: Uuid,
        channel_id: Uuid,
        alert_instance_id: Option<Uuid>,
        incident_id: Option<Uuid>,
        payload: &NotificationPayload,
    ) -> Result<Uuid> {
        let payload_json = serde_json::to_value(payload)?;

        let id: (Uuid,) = sqlx::query_as(
            r#"
            INSERT INTO notification_deliveries
                (tenant_id, channel_id, alert_instance_id, incident_id, payload)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id
            "#
        )
        .bind(tenant_id)
        .bind(channel_id)
        .bind(alert_instance_id)
        .bind(incident_id)
        .bind(&payload_json)
        .fetch_one(&self.pool)
        .await?;

        Ok(id.0)
    }

    /// Mark delivery as successful
    async fn mark_delivered(&self, delivery_id: Uuid) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE notification_deliveries SET
                status = 'delivered',
                delivered_at = NOW(),
                last_attempt_at = NOW(),
                attempt_count = attempt_count + 1
            WHERE id = $1
            "#
        )
        .bind(delivery_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Mark delivery as failed
    async fn mark_failed(&self, delivery_id: Uuid, error: &str) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE notification_deliveries SET
                status = 'failed',
                error_message = $1,
                last_attempt_at = NOW(),
                attempt_count = attempt_count + 1
            WHERE id = $2
            "#
        )
        .bind(error)
        .bind(delivery_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Send notification to a specific channel
    async fn send_to_channel(
        &self,
        channel: &NotificationChannel,
        payload: &NotificationPayload,
    ) -> Result<()> {
        match channel.channel_type.as_str() {
            "slack" => self.send_slack(channel, payload).await,
            "webhook" => self.send_webhook(channel, payload).await,
            "email" => self.send_email(channel, payload).await,
            "pagerduty" => self.send_pagerduty(channel, payload).await,
            "teams" => self.send_teams(channel, payload).await,
            _ => Err(anyhow::anyhow!("Unsupported channel type: {}", channel.channel_type)),
        }
    }

    /// Send Slack notification
    async fn send_slack(&self, channel: &NotificationChannel, payload: &NotificationPayload) -> Result<()> {
        let webhook_url = channel.config.get("webhook_url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing webhook_url in Slack config"))?;

        let color = match payload.severity.as_str() {
            "critical" => "#dc3545",
            "high" => "#fd7e14",
            "medium" => "#ffc107",
            "low" => "#17a2b8",
            _ => "#6c757d",
        };

        let slack_payload = serde_json::json!({
            "attachments": [{
                "color": color,
                "title": payload.title,
                "text": payload.message,
                "fields": [
                    {"title": "Severity", "value": payload.severity, "short": true},
                    {"title": "Source", "value": payload.source, "short": true}
                ],
                "footer": "Sysilo Operations Center"
            }]
        });

        let client = reqwest::Client::new();
        let response = client
            .post(webhook_url)
            .json(&slack_payload)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Slack webhook failed: {}", response.status()));
        }

        Ok(())
    }

    /// Send generic webhook notification
    async fn send_webhook(&self, channel: &NotificationChannel, payload: &NotificationPayload) -> Result<()> {
        let url = channel.config.get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing url in webhook config"))?;

        let client = reqwest::Client::new();
        let response = client
            .post(url)
            .json(payload)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Webhook failed: {}", response.status()));
        }

        Ok(())
    }

    /// Send email notification (placeholder)
    async fn send_email(&self, _channel: &NotificationChannel, _payload: &NotificationPayload) -> Result<()> {
        // Implementation would use lettre crate
        // For now, just log
        tracing::info!("Email notification would be sent");
        Ok(())
    }

    /// Send PagerDuty notification
    async fn send_pagerduty(&self, channel: &NotificationChannel, payload: &NotificationPayload) -> Result<()> {
        let routing_key = channel.config.get("routing_key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing routing_key in PagerDuty config"))?;

        let pd_payload = serde_json::json!({
            "routing_key": routing_key,
            "event_action": "trigger",
            "payload": {
                "summary": payload.title,
                "severity": match payload.severity.as_str() {
                    "critical" => "critical",
                    "high" => "error",
                    "medium" => "warning",
                    _ => "info"
                },
                "source": payload.source,
                "custom_details": {
                    "message": payload.message
                }
            }
        });

        let client = reqwest::Client::new();
        let response = client
            .post("https://events.pagerduty.com/v2/enqueue")
            .json(&pd_payload)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("PagerDuty API failed: {}", response.status()));
        }

        Ok(())
    }

    /// Send Microsoft Teams notification
    async fn send_teams(&self, channel: &NotificationChannel, payload: &NotificationPayload) -> Result<()> {
        let webhook_url = channel.config.get("webhook_url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing webhook_url in Teams config"))?;

        let color = match payload.severity.as_str() {
            "critical" => "dc3545",
            "high" => "fd7e14",
            "medium" => "ffc107",
            "low" => "17a2b8",
            _ => "6c757d",
        };

        let teams_payload = serde_json::json!({
            "@type": "MessageCard",
            "@context": "http://schema.org/extensions",
            "themeColor": color,
            "summary": payload.title,
            "sections": [{
                "activityTitle": payload.title,
                "facts": [
                    {"name": "Severity", "value": payload.severity},
                    {"name": "Source", "value": payload.source}
                ],
                "text": payload.message
            }]
        });

        let client = reqwest::Client::new();
        let response = client
            .post(webhook_url)
            .json(&teams_payload)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Teams webhook failed: {}", response.status()));
        }

        Ok(())
    }

    /// Test a notification channel
    pub async fn test_channel(&self, tenant_id: Uuid, id: Uuid) -> Result<bool> {
        let channel = self.get_channel(tenant_id, id).await?;
        if let Some(channel) = channel {
            let test_payload = NotificationPayload {
                title: "Test Notification".to_string(),
                message: "This is a test notification from Sysilo Operations Center.".to_string(),
                severity: "info".to_string(),
                source: "test".to_string(),
                source_url: None,
                metadata: None,
            };

            self.send_to_channel(&channel, &test_payload).await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
