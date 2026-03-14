use anyhow::Result;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{CommitMode, Consumer, StreamConsumer};
use rdkafka::message::Message;
use rust_decimal::Decimal;
use serde::Deserialize;
use tokio_stream::StreamExt;
use uuid::Uuid;

use super::{LiveScoringService, ScoreEvent};

/// Kafka event envelope used across Sysilo services
#[derive(Debug, Deserialize)]
struct KafkaEventEnvelope {
    pub tenant_id: Uuid,
    pub event_type: String,
    pub payload: serde_json::Value,
}

/// Start the Kafka consumer that subscribes to event topics and feeds them
/// into the LiveScoringService for score drift processing.
pub async fn start_kafka_consumer(
    service: LiveScoringService,
    brokers: &str,
    group_id: &str,
) -> Result<tokio::task::JoinHandle<()>> {
    let consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("group.id", group_id)
        .set("enable.auto.commit", "false")
        .set("auto.offset.reset", "latest")
        .set("session.timeout.ms", "30000")
        .set("heartbeat.interval.ms", "10000")
        .create()?;

    let topics = [
        "sysilo.integrations.events",
        "sysilo.governance.violations",
        "sysilo.ops.metrics",
    ];
    consumer.subscribe(&topics)?;

    tracing::info!(
        "Live scoring Kafka consumer subscribed to topics: {:?}",
        topics
    );

    let handle = tokio::spawn(async move {
        let mut stream = consumer.stream();

        while let Some(result) = stream.next().await {
            match result {
                Ok(msg) => {
                    let payload = match msg.payload_view::<str>() {
                        Some(Ok(text)) => text,
                        Some(Err(e)) => {
                            tracing::warn!("Failed to decode Kafka message payload: {}", e);
                            continue;
                        }
                        None => {
                            tracing::debug!("Empty Kafka message, skipping");
                            continue;
                        }
                    };

                    let topic = msg.topic();

                    match parse_event(topic, payload) {
                        Ok(Some((tenant_id, event))) => {
                            tracing::debug!(
                                "Processing score event: {:?} for tenant {}",
                                event.event_type(),
                                tenant_id
                            );

                            if let Err(e) = service.process_event(tenant_id, event).await {
                                tracing::error!("Failed to process score event: {}", e);
                            }
                        }
                        Ok(None) => {
                            // Event type not relevant to scoring, skip
                            tracing::trace!("Skipping irrelevant event from topic {}", topic);
                        }
                        Err(e) => {
                            tracing::warn!(
                                "Failed to parse event from topic {}: {}",
                                topic,
                                e
                            );
                        }
                    }

                    // Commit offset after processing
                    if let Err(e) = consumer.commit_message(&msg, CommitMode::Async) {
                        tracing::warn!("Failed to commit Kafka offset: {}", e);
                    }
                }
                Err(e) => {
                    tracing::error!("Kafka consumer error: {}", e);
                }
            }
        }

        tracing::warn!("Kafka consumer stream ended unexpectedly");
    });

    Ok(handle)
}

/// Parse a Kafka message from a given topic into a ScoreEvent
fn parse_event(topic: &str, payload: &str) -> Result<Option<(Uuid, ScoreEvent)>> {
    let envelope: KafkaEventEnvelope = serde_json::from_str(payload)?;
    let tenant_id = envelope.tenant_id;
    let p = &envelope.payload;

    let event = match topic {
        "sysilo.integrations.events" => match envelope.event_type.as_str() {
            "integration.created" | "integration.added" => {
                let integration_id = parse_uuid(p, "integration_id")?;
                let asset_id = parse_uuid(p, "asset_id")?;
                Some(ScoreEvent::IntegrationAdded {
                    integration_id,
                    asset_id,
                })
            }
            "integration.deleted" | "integration.removed" => {
                let integration_id = parse_uuid(p, "integration_id")?;
                let asset_id = parse_uuid(p, "asset_id")?;
                Some(ScoreEvent::IntegrationRemoved {
                    integration_id,
                    asset_id,
                })
            }
            "integration.failed" => {
                let integration_id = parse_uuid(p, "integration_id")?;
                let asset_id = parse_uuid(p, "asset_id")?;
                let failure_count = p
                    .get("failure_count")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(1) as i32;
                Some(ScoreEvent::IntegrationFailed {
                    integration_id,
                    asset_id,
                    failure_count,
                })
            }
            "connector.inactive" => {
                let connection_id = parse_uuid(p, "connection_id")?;
                let asset_id = parse_uuid(p, "asset_id")?;
                let inactive_days = p
                    .get("inactive_days")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0) as i32;
                Some(ScoreEvent::ConnectorInactive {
                    connection_id,
                    asset_id,
                    inactive_days,
                })
            }
            "connector.activated" => {
                let connection_id = parse_uuid(p, "connection_id")?;
                let asset_id = parse_uuid(p, "asset_id")?;
                Some(ScoreEvent::ConnectorActivated {
                    connection_id,
                    asset_id,
                })
            }
            _ => None,
        },
        "sysilo.governance.violations" => match envelope.event_type.as_str() {
            "violation.created" | "violation.detected" => {
                let asset_id = parse_uuid(p, "asset_id")?;
                let violation_id = parse_uuid(p, "violation_id")?;
                let severity = p
                    .get("severity")
                    .and_then(|v| v.as_str())
                    .unwrap_or("medium")
                    .to_string();
                Some(ScoreEvent::GovernanceViolation {
                    asset_id,
                    severity,
                    violation_id,
                })
            }
            "violation.resolved" | "violation.closed" => {
                let asset_id = parse_uuid(p, "asset_id")?;
                let violation_id = parse_uuid(p, "violation_id")?;
                Some(ScoreEvent::GovernanceResolved {
                    asset_id,
                    violation_id,
                })
            }
            _ => None,
        },
        "sysilo.ops.metrics" => match envelope.event_type.as_str() {
            "cost.changed" => {
                let asset_id = parse_uuid(p, "asset_id")?;
                let old_cost = parse_decimal(p, "old_cost")?;
                let new_cost = parse_decimal(p, "new_cost")?;
                Some(ScoreEvent::CostChange {
                    asset_id,
                    old_cost,
                    new_cost,
                })
            }
            "usage.spike" => {
                let asset_id = parse_uuid(p, "asset_id")?;
                let usage_multiplier = p
                    .get("usage_multiplier")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(2.0);
                Some(ScoreEvent::UsageSpike {
                    asset_id,
                    usage_multiplier,
                })
            }
            "usage.drop" => {
                let asset_id = parse_uuid(p, "asset_id")?;
                let usage_multiplier = p
                    .get("usage_multiplier")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.5);
                Some(ScoreEvent::UsageDrop {
                    asset_id,
                    usage_multiplier,
                })
            }
            _ => None,
        },
        _ => None,
    };

    Ok(event.map(|e| (tenant_id, e)))
}

/// Parse a UUID from a JSON value at a given key
fn parse_uuid(payload: &serde_json::Value, key: &str) -> Result<Uuid> {
    let val = payload
        .get(key)
        .ok_or_else(|| anyhow::anyhow!("Missing field '{}' in event payload", key))?;

    if let Some(s) = val.as_str() {
        Ok(Uuid::parse_str(s)?)
    } else {
        Err(anyhow::anyhow!(
            "Field '{}' is not a valid UUID string",
            key
        ))
    }
}

/// Parse a Decimal from a JSON value at a given key
fn parse_decimal(payload: &serde_json::Value, key: &str) -> Result<Decimal> {
    let val = payload
        .get(key)
        .ok_or_else(|| anyhow::anyhow!("Missing field '{}' in event payload", key))?;

    if let Some(s) = val.as_str() {
        Ok(s.parse::<Decimal>()?)
    } else if let Some(n) = val.as_f64() {
        Ok(Decimal::from_f64_retain(n).unwrap_or_default())
    } else if let Some(n) = val.as_i64() {
        Ok(Decimal::from(n))
    } else {
        Err(anyhow::anyhow!(
            "Field '{}' is not a valid decimal value",
            key
        ))
    }
}
