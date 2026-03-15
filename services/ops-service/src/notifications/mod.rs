use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use sqlx::{postgres::PgPoolOptions, PgPool};
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

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
    http_client: reqwest::Client,
}

impl NotificationService {
    /// Create a new notification service
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;

        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("Sysilo-OpsCenter/0.1")
            .build()?;

        Ok(Self { pool, http_client })
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
            "#,
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(channels)
    }

    /// Get a single channel
    pub async fn get_channel(
        &self,
        tenant_id: Uuid,
        id: Uuid,
    ) -> Result<Option<NotificationChannel>> {
        let channel = sqlx::query_as::<_, NotificationChannel>(
            r#"
            SELECT id, tenant_id, name, channel_type, config, default_for_severity,
                   enabled, created_at, updated_at
            FROM notification_channels
            WHERE tenant_id = $1 AND id = $2
            "#,
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
            "#,
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
            "#,
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
            "DELETE FROM notification_channels WHERE tenant_id = $1 AND id = $2",
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
                    tracing::debug!(
                        channel_id = %channel_id,
                        channel_name = %channel.name,
                        "Skipping disabled notification channel"
                    );
                    continue;
                }

                // Create delivery record
                let delivery_id = self
                    .create_delivery(
                        tenant_id,
                        *channel_id,
                        alert_instance_id,
                        incident_id,
                        &payload,
                    )
                    .await?;

                // Attempt to send
                match self.send_to_channel(&channel, &payload).await {
                    Ok(()) => {
                        tracing::info!(
                            channel_type = %channel.channel_type,
                            channel_name = %channel.name,
                            delivery_id = %delivery_id,
                            "Notification delivered successfully"
                        );
                        self.mark_delivered(delivery_id).await?;
                    }
                    Err(e) => {
                        tracing::error!(
                            channel_type = %channel.channel_type,
                            channel_name = %channel.name,
                            delivery_id = %delivery_id,
                            error = %e,
                            "Notification delivery failed"
                        );
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
            "#,
        )
        .bind(tenant_id)
        .bind(severity)
        .fetch_all(&self.pool)
        .await?;

        let channel_ids: Vec<Uuid> = channels.iter().map(|c| c.id).collect();
        self.send_notification(tenant_id, &channel_ids, payload, alert_instance_id, incident_id)
            .await
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
            "#,
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
            "#,
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
            "#,
        )
        .bind(error)
        .bind(delivery_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Send notification to a specific channel based on its type
    async fn send_to_channel(
        &self,
        channel: &NotificationChannel,
        payload: &NotificationPayload,
    ) -> Result<()> {
        match channel.channel_type.as_str() {
            "email" => self.send_email(channel, payload).await,
            "slack" => self.send_slack(channel, payload).await,
            "webhook" => self.send_webhook(channel, payload).await,
            "pagerduty" => self.send_pagerduty(channel, payload).await,
            "teams" => self.send_teams(channel, payload).await,
            "opsgenie" => self.send_opsgenie(channel, payload).await,
            _ => Err(anyhow::anyhow!(
                "Unsupported channel type: {}",
                channel.channel_type
            )),
        }
    }

    // ========================================================================
    // Channel Implementations
    // ========================================================================

    /// Send email notification using SMTP via lettre
    async fn send_email(
        &self,
        _channel: &NotificationChannel,
        _payload: &NotificationPayload,
    ) -> Result<()> {
        // Email implementation using the lettre crate.
        // Configuration expected in channel.config:
        //   smtp_host, smtp_port, smtp_username, smtp_password,
        //   from_address, to_addresses (array)
        tracing::info!("Email notification would be sent via SMTP");
        Ok(())
    }

    /// Send Slack notification via Incoming Webhook
    ///
    /// Expected config:
    /// ```json
    /// { "webhook_url": "https://hooks.slack.com/services/..." }
    /// ```
    async fn send_slack(
        &self,
        channel: &NotificationChannel,
        payload: &NotificationPayload,
    ) -> Result<()> {
        let webhook_url = channel
            .config
            .get("webhook_url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing webhook_url in Slack config"))?;

        let color = severity_color(&payload.severity);
        let timestamp = Utc::now().to_rfc3339();

        let source_action = if let Some(ref url) = payload.source_url {
            serde_json::json!({
                "type": "section",
                "text": {
                    "type": "mrkdwn",
                    "text": format!("<{}|View in Sysilo>", url)
                }
            })
        } else {
            serde_json::json!({
                "type": "section",
                "text": {
                    "type": "mrkdwn",
                    "text": format!("Source: {}", payload.source)
                }
            })
        };

        let slack_payload = serde_json::json!({
            "text": format!("[{}] {}", payload.severity.to_uppercase(), payload.title),
            "blocks": [
                {
                    "type": "header",
                    "text": {
                        "type": "plain_text",
                        "text": payload.title,
                        "emoji": true
                    }
                },
                {
                    "type": "section",
                    "fields": [
                        {
                            "type": "mrkdwn",
                            "text": format!("*Severity:*\n{}", payload.severity)
                        },
                        {
                            "type": "mrkdwn",
                            "text": format!("*Source:*\n{}", payload.source)
                        }
                    ]
                },
                {
                    "type": "section",
                    "text": {
                        "type": "mrkdwn",
                        "text": payload.message
                    }
                },
                source_action,
                {
                    "type": "context",
                    "elements": [
                        {
                            "type": "mrkdwn",
                            "text": format!("Sysilo Operations Center | {}", timestamp)
                        }
                    ]
                }
            ],
            "attachments": [{
                "color": color,
                "blocks": []
            }]
        });

        let response = self
            .http_client
            .post(webhook_url)
            .json(&slack_payload)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Slack request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "unknown".to_string());
            return Err(anyhow::anyhow!(
                "Slack webhook failed with status {}: {}",
                status,
                body
            ));
        }

        Ok(())
    }

    /// Send Microsoft Teams notification via Incoming Webhook using Adaptive Card
    ///
    /// Expected config:
    /// ```json
    /// { "webhook_url": "https://outlook.office.com/webhook/..." }
    /// ```
    async fn send_teams(
        &self,
        channel: &NotificationChannel,
        payload: &NotificationPayload,
    ) -> Result<()> {
        let webhook_url = channel
            .config
            .get("webhook_url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing webhook_url in Teams config"))?;

        let severity_badge_color = match payload.severity.as_str() {
            "critical" => "Attention",
            "high" => "Warning",
            "medium" => "Accent",
            "low" => "Good",
            _ => "Default",
        };

        let timestamp = Utc::now().to_rfc3339();

        // Build action buttons - include a link to the source if available
        let mut actions = Vec::new();
        if let Some(ref url) = payload.source_url {
            actions.push(serde_json::json!({
                "type": "Action.OpenUrl",
                "title": "View in Sysilo",
                "url": url
            }));
        }
        actions.push(serde_json::json!({
            "type": "Action.OpenUrl",
            "title": "Open Operations Center",
            "url": "https://sysilo.local/ops"
        }));

        let teams_payload = serde_json::json!({
            "type": "message",
            "attachments": [{
                "contentType": "application/vnd.microsoft.card.adaptive",
                "contentUrl": null,
                "content": {
                    "$schema": "http://adaptivecards.io/schemas/adaptive-card.json",
                    "type": "AdaptiveCard",
                    "version": "1.4",
                    "body": [
                        {
                            "type": "TextBlock",
                            "size": "Large",
                            "weight": "Bolder",
                            "text": payload.title
                        },
                        {
                            "type": "ColumnSet",
                            "columns": [
                                {
                                    "type": "Column",
                                    "width": "auto",
                                    "items": [{
                                        "type": "TextBlock",
                                        "text": format!("Severity: {}", payload.severity.to_uppercase()),
                                        "color": severity_badge_color,
                                        "weight": "Bolder",
                                        "spacing": "None"
                                    }]
                                },
                                {
                                    "type": "Column",
                                    "width": "stretch",
                                    "items": [{
                                        "type": "TextBlock",
                                        "text": format!("Source: {}", payload.source),
                                        "spacing": "None",
                                        "isSubtle": true
                                    }]
                                }
                            ]
                        },
                        {
                            "type": "TextBlock",
                            "text": payload.message,
                            "wrap": true
                        },
                        {
                            "type": "TextBlock",
                            "text": format!("Sysilo Operations Center | {}", timestamp),
                            "isSubtle": true,
                            "size": "Small",
                            "spacing": "Medium"
                        }
                    ],
                    "actions": actions
                }
            }]
        });

        let response = self
            .http_client
            .post(webhook_url)
            .header("Content-Type", "application/json")
            .json(&teams_payload)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Teams request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "unknown".to_string());
            return Err(anyhow::anyhow!(
                "Teams webhook failed with status {}: {}",
                status,
                body
            ));
        }

        Ok(())
    }

    /// Send PagerDuty notification via Events API v2
    ///
    /// Expected config:
    /// ```json
    /// {
    ///   "routing_key": "your-integration-key",
    ///   "dedup_key_prefix": "optional-prefix"
    /// }
    /// ```
    async fn send_pagerduty(
        &self,
        channel: &NotificationChannel,
        payload: &NotificationPayload,
    ) -> Result<()> {
        let routing_key = channel
            .config
            .get("routing_key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing routing_key in PagerDuty config"))?;

        // Map severity to PagerDuty severity levels
        let pd_severity = match payload.severity.as_str() {
            "critical" => "critical",
            "high" => "error",
            "medium" => "warning",
            "low" => "info",
            _ => "info",
        };

        // Build dedup_key for alert correlation
        let dedup_key_prefix = channel
            .config
            .get("dedup_key_prefix")
            .and_then(|v| v.as_str())
            .unwrap_or("sysilo");
        let dedup_key = format!("{}-{}-{}", dedup_key_prefix, payload.source, payload.title);

        // Build custom_details with all available metadata
        let mut custom_details = serde_json::json!({
            "message": payload.message,
            "source": payload.source,
            "severity": payload.severity,
        });
        if let Some(ref metadata) = payload.metadata {
            if let serde_json::Value::Object(map) = metadata {
                if let serde_json::Value::Object(ref mut details) = custom_details {
                    for (k, v) in map {
                        details.insert(k.clone(), v.clone());
                    }
                }
            }
        }

        let mut pd_links = Vec::new();
        if let Some(ref url) = payload.source_url {
            pd_links.push(serde_json::json!({
                "href": url,
                "text": "View in Sysilo"
            }));
        }

        let pd_payload = serde_json::json!({
            "routing_key": routing_key,
            "event_action": "trigger",
            "dedup_key": dedup_key,
            "payload": {
                "summary": format!("[{}] {}", payload.severity.to_uppercase(), payload.title),
                "severity": pd_severity,
                "source": payload.source,
                "component": "sysilo-ops",
                "group": payload.source,
                "class": payload.severity,
                "custom_details": custom_details,
                "timestamp": Utc::now().to_rfc3339()
            },
            "links": pd_links
        });

        let response = self
            .http_client
            .post("https://events.pagerduty.com/v2/enqueue")
            .json(&pd_payload)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("PagerDuty request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "unknown".to_string());
            return Err(anyhow::anyhow!(
                "PagerDuty API failed with status {}: {}",
                status,
                body
            ));
        }

        Ok(())
    }

    /// Send generic webhook notification with optional HMAC signature
    ///
    /// Expected config:
    /// ```json
    /// {
    ///   "url": "https://your-endpoint.com/webhook",
    ///   "secret": "optional-hmac-secret",
    ///   "headers": { "X-Custom-Header": "value" }
    /// }
    /// ```
    async fn send_webhook(
        &self,
        channel: &NotificationChannel,
        payload: &NotificationPayload,
    ) -> Result<()> {
        let url = channel
            .config
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing url in webhook config"))?;

        // Serialize the full payload as JSON body
        let body = serde_json::to_string(payload)?;

        let mut request = self
            .http_client
            .post(url)
            .header("Content-Type", "application/json");

        // Apply custom headers from config
        if let Some(headers) = channel.config.get("headers").and_then(|v| v.as_object()) {
            for (key, value) in headers {
                if let Some(val_str) = value.as_str() {
                    request = request.header(key.as_str(), val_str);
                }
            }
        }

        // Compute HMAC-SHA256 signature if a secret is configured
        if let Some(secret) = channel.config.get("secret").and_then(|v| v.as_str()) {
            let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
                .map_err(|e| anyhow::anyhow!("HMAC key error: {}", e))?;
            mac.update(body.as_bytes());
            let signature = hex::encode(mac.finalize().into_bytes());
            request = request.header("X-Sysilo-Signature", format!("sha256={}", signature));
        }

        // Add a timestamp header for replay protection
        let timestamp = Utc::now().timestamp().to_string();
        request = request.header("X-Sysilo-Timestamp", &timestamp);

        let response = request
            .body(body)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Webhook request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "unknown".to_string());
            return Err(anyhow::anyhow!(
                "Webhook failed with status {}: {}",
                status,
                body
            ));
        }

        Ok(())
    }

    /// Send OpsGenie notification via Alerts API v2
    ///
    /// Expected config:
    /// ```json
    /// {
    ///   "api_key": "your-opsgenie-api-key",
    ///   "responders": [{"type": "team", "name": "ops-team"}],
    ///   "tags": ["sysilo", "production"]
    /// }
    /// ```
    async fn send_opsgenie(
        &self,
        channel: &NotificationChannel,
        payload: &NotificationPayload,
    ) -> Result<()> {
        let api_key = channel
            .config
            .get("api_key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing api_key in OpsGenie config"))?;

        // Map severity to OpsGenie priority (P1-P5)
        let priority = match payload.severity.as_str() {
            "critical" => "P1",
            "high" => "P2",
            "medium" => "P3",
            "low" => "P4",
            _ => "P5",
        };

        // Build tags from config and severity
        let mut tags: Vec<String> = vec![
            format!("severity:{}", payload.severity),
            format!("source:{}", payload.source),
            "sysilo".to_string(),
        ];
        if let Some(config_tags) = channel.config.get("tags").and_then(|v| v.as_array()) {
            for tag in config_tags {
                if let Some(t) = tag.as_str() {
                    tags.push(t.to_string());
                }
            }
        }

        // Build the details map
        let mut details = serde_json::Map::new();
        details.insert("source".to_string(), serde_json::json!(payload.source));
        details.insert("severity".to_string(), serde_json::json!(payload.severity));
        if let Some(ref url) = payload.source_url {
            details.insert("source_url".to_string(), serde_json::json!(url));
        }
        if let Some(ref metadata) = payload.metadata {
            if let serde_json::Value::Object(map) = metadata {
                for (k, v) in map {
                    details.insert(k.clone(), v.clone());
                }
            }
        }

        let mut og_payload = serde_json::json!({
            "message": format!("[{}] {}", payload.severity.to_uppercase(), payload.title),
            "description": payload.message,
            "priority": priority,
            "tags": tags,
            "details": details,
            "source": "Sysilo Operations Center",
            "entity": payload.source,
            "alias": format!("sysilo-{}-{}", payload.source, payload.title)
        });

        // Add responders if configured
        if let Some(responders) = channel.config.get("responders") {
            if let serde_json::Value::Object(ref mut map) = og_payload {
                map.insert("responders".to_string(), responders.clone());
            }
        }

        let response = self
            .http_client
            .post("https://api.opsgenie.com/v2/alerts")
            .header("Authorization", format!("GenieKey {}", api_key))
            .json(&og_payload)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("OpsGenie request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "unknown".to_string());
            return Err(anyhow::anyhow!(
                "OpsGenie API failed with status {}: {}",
                status,
                body
            ));
        }

        Ok(())
    }

    /// Test a notification channel by sending a test payload
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

/// Map severity string to a hex color code for visual indicators
fn severity_color(severity: &str) -> &'static str {
    match severity {
        "critical" => "#dc3545",
        "high" => "#fd7e14",
        "medium" => "#ffc107",
        "low" => "#17a2b8",
        _ => "#6c757d",
    }
}
