use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use tokio::time;
use tracing::{info, warn, error};

use crate::alerts::{AlertRule, AlertsService, AlertCondition};
use crate::metrics::MetricsService;
use crate::notifications::{NotificationService, NotificationPayload};

/// Parses a condition string into an AlertCondition enum.
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

/// The alert evaluation engine that periodically checks metrics against
/// configured alert rules and fires or resolves alerts accordingly.
pub struct AlertEvaluator {
    alerts: Arc<AlertsService>,
    metrics: Arc<MetricsService>,
    notifications: Arc<NotificationService>,
    interval: Duration,
}

impl AlertEvaluator {
    /// Create a new AlertEvaluator with the given services and evaluation interval.
    pub fn new(
        alerts: Arc<AlertsService>,
        metrics: Arc<MetricsService>,
        notifications: Arc<NotificationService>,
        interval_seconds: u64,
    ) -> Self {
        Self {
            alerts,
            metrics,
            notifications,
            interval: Duration::from_secs(interval_seconds),
        }
    }

    /// Spawn a background tokio task that runs the evaluation loop.
    /// Returns a JoinHandle for the spawned task.
    pub fn start(self) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            info!(
                interval_secs = self.interval.as_secs(),
                "Alert evaluator started"
            );

            let mut ticker = time::interval(self.interval);

            loop {
                ticker.tick().await;

                match self.run_evaluation_cycle().await {
                    Ok((rules_checked, fired, resolved)) => {
                        info!(
                            rules_checked = rules_checked,
                            alerts_fired = fired,
                            alerts_resolved = resolved,
                            "Alert evaluation cycle complete"
                        );
                    }
                    Err(e) => {
                        error!(error = %e, "Alert evaluation cycle failed");
                    }
                }
            }
        })
    }

    /// Run a single evaluation cycle across all tenants and rules.
    /// Returns (rules_checked, alerts_fired, alerts_resolved).
    async fn run_evaluation_cycle(&self) -> Result<(usize, usize, usize)> {
        // Fetch all enabled alert rules across all tenants
        let rules = self.fetch_all_enabled_rules().await?;
        let rules_checked = rules.len();
        let mut fired = 0usize;
        let mut resolved = 0usize;

        for rule in &rules {
            match self.evaluate_rule(rule).await {
                Ok(EvalOutcome::Fired) => fired += 1,
                Ok(EvalOutcome::Resolved) => resolved += 1,
                Ok(EvalOutcome::NoChange) => {}
                Err(e) => {
                    warn!(
                        rule_id = %rule.id,
                        rule_name = %rule.name,
                        error = %e,
                        "Failed to evaluate alert rule, skipping"
                    );
                }
            }
        }

        Ok((rules_checked, fired, resolved))
    }

    /// Fetch all enabled alert rules across every tenant.
    async fn fetch_all_enabled_rules(&self) -> Result<Vec<AlertRule>> {
        let rules = self.alerts.list_all_enabled_rules().await?;
        Ok(rules)
    }

    /// Evaluate a single alert rule against the latest metric value.
    async fn evaluate_rule(&self, rule: &AlertRule) -> Result<EvalOutcome> {
        // Query the latest metric values matching this rule's metric_name for the rule's tenant
        let latest_metrics = self.metrics.query_metrics(
            rule.tenant_id,
            None,               // any resource_type
            None,               // any resource_id
            Some(rule.metric_name.clone()),
            None,               // no start_time filter (defaults to last hour)
            None,               // no end_time filter
            1,                  // just the latest value
        ).await?;

        let current_value = match latest_metrics.first() {
            Some(m) => m.metric_value,
            None => {
                // No metric data available for this rule; nothing to evaluate
                return Ok(EvalOutcome::NoChange);
            }
        };

        let condition = parse_condition(&rule.condition);
        let threshold_breached = condition.evaluate(current_value, rule.threshold);

        // Check if there is already an active (firing) alert instance for this rule
        let active_instances = self.alerts.list_instances(
            rule.tenant_id,
            Some("firing".to_string()),
            100,
        ).await?;

        let active_for_rule = active_instances
            .iter()
            .find(|inst| inst.instance.rule_id == rule.id);

        if threshold_breached {
            if active_for_rule.is_some() {
                // Already firing, nothing to do
                return Ok(EvalOutcome::NoChange);
            }

            // Fire a new alert
            let instance = self.alerts.fire_alert(
                rule.tenant_id,
                rule.id,
                current_value,
                Some(serde_json::json!({
                    "condition": rule.condition,
                    "threshold": rule.threshold,
                    "metric_name": rule.metric_name,
                    "evaluated_value": current_value,
                })),
            ).await?;

            info!(
                rule_id = %rule.id,
                rule_name = %rule.name,
                value = current_value,
                threshold = rule.threshold,
                condition = %rule.condition,
                "Alert fired"
            );

            // Send notifications via configured channels and severity-based routing
            let payload = NotificationPayload {
                title: format!("Alert: {}", rule.name),
                message: format!(
                    "{} - metric '{}' value {:.2} {} threshold {:.2}",
                    rule.description.as_deref().unwrap_or("Alert condition met"),
                    rule.metric_name,
                    current_value,
                    rule.condition,
                    rule.threshold,
                ),
                severity: rule.severity.clone(),
                source: "alert-evaluator".to_string(),
                source_url: None,
                metadata: Some(serde_json::json!({
                    "rule_id": rule.id,
                    "metric_name": rule.metric_name,
                    "current_value": current_value,
                    "threshold": rule.threshold,
                })),
            };

            // Send to explicitly configured channels on the rule
            if !rule.channels.is_empty() {
                if let Err(e) = self.notifications.send_notification(
                    rule.tenant_id,
                    &rule.channels,
                    payload.clone(),
                    Some(instance.id),
                    None,
                ).await {
                    warn!(
                        rule_id = %rule.id,
                        error = %e,
                        "Failed to send notification to configured channels"
                    );
                }
            }

            // Also send via severity-based routing
            if let Err(e) = self.notifications.send_by_severity(
                rule.tenant_id,
                &rule.severity,
                NotificationPayload {
                    title: format!("Alert: {}", rule.name),
                    message: format!(
                        "{} - metric '{}' value {:.2} {} threshold {:.2}",
                        rule.description.as_deref().unwrap_or("Alert condition met"),
                        rule.metric_name,
                        current_value,
                        rule.condition,
                        rule.threshold,
                    ),
                    severity: rule.severity.clone(),
                    source: "alert-evaluator".to_string(),
                    source_url: None,
                    metadata: Some(serde_json::json!({
                        "rule_id": rule.id,
                        "metric_name": rule.metric_name,
                        "current_value": current_value,
                        "threshold": rule.threshold,
                    })),
                },
                Some(instance.id),
                None,
            ).await {
                warn!(
                    rule_id = %rule.id,
                    error = %e,
                    "Failed to send severity-based notification"
                );
            }

            Ok(EvalOutcome::Fired)
        } else {
            // Condition not met - auto-resolve if there is a firing instance
            if let Some(active) = active_for_rule {
                self.alerts.resolve_alert(rule.tenant_id, active.instance.id).await?;

                info!(
                    rule_id = %rule.id,
                    rule_name = %rule.name,
                    instance_id = %active.instance.id,
                    value = current_value,
                    threshold = rule.threshold,
                    "Alert auto-resolved"
                );

                Ok(EvalOutcome::Resolved)
            } else {
                Ok(EvalOutcome::NoChange)
            }
        }
    }
}

/// Outcome of evaluating a single alert rule.
enum EvalOutcome {
    /// A new alert was fired.
    Fired,
    /// An existing alert was auto-resolved.
    Resolved,
    /// No state change occurred.
    NoChange,
}
