use std::sync::Arc;

use anyhow::Result;
use futures::StreamExt;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::Message;
use tracing::{error, info, warn};

use crate::kafka::{topics, DiscoveredAsset, TaskProducer, TaskResult};
use crate::storage::Storage;

/// Configuration for the result consumer
#[derive(Debug, Clone)]
pub struct ConsumerConfig {
    pub bootstrap_servers: String,
    pub group_id: String,
    pub asset_service_url: String,
}

impl Default for ConsumerConfig {
    fn default() -> Self {
        Self {
            bootstrap_servers: "localhost:9092".to_string(),
            group_id: "integration-service-consumers".to_string(),
            asset_service_url: "http://localhost:8082".to_string(),
        }
    }
}

/// Result consumer that processes task results and forwards assets
pub struct ResultConsumer {
    consumer: StreamConsumer,
    asset_service_url: String,
    http_client: reqwest::Client,
    storage: Option<Arc<Storage>>,
    kafka_producer: Option<Arc<TaskProducer>>,
}

impl ResultConsumer {
    /// Create a new result consumer
    pub fn new(
        config: &ConsumerConfig,
        storage: Option<Arc<Storage>>,
        kafka_producer: Option<Arc<TaskProducer>>,
    ) -> Result<Self> {
        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", &config.bootstrap_servers)
            .set("group.id", &config.group_id)
            .set("enable.auto.commit", "true")
            .set("auto.offset.reset", "earliest")
            .create()?;

        consumer.subscribe(&[topics::RESULTS])?;

        info!(
            "Result consumer subscribed to {} (group: {})",
            topics::RESULTS,
            config.group_id
        );

        Ok(Self {
            consumer,
            asset_service_url: config.asset_service_url.clone(),
            http_client: reqwest::Client::new(),
            storage,
            kafka_producer,
        })
    }

    /// Start consuming and processing results
    pub async fn run(&self) -> Result<()> {
        info!("Starting result consumer loop");

        let mut stream = self.consumer.stream();

        while let Some(message_result) = stream.next().await {
            match message_result {
                Ok(message) => {
                    if let Some(payload) = message.payload() {
                        if let Err(e) = self.process_message(payload).await {
                            error!("Failed to process message: {}", e);
                        }
                    }
                }
                Err(e) => {
                    warn!("Kafka consumer error: {}", e);
                }
            }
        }

        Ok(())
    }

    async fn process_message(&self, payload: &[u8]) -> Result<()> {
        let result: TaskResult = serde_json::from_slice(payload)?;

        info!(
            task_id = %result.task_id,
            tenant_id = %result.tenant_id,
            status = %result.status,
            "Processing task result"
        );

        // Check if this is a playbook step result
        if let Some(ref output) = result.output {
            if output.get("step_id").is_some() {
                return self.process_playbook_result(&result).await;
            }
        }

        // Process discovery results
        if let Some(output) = &result.output {
            if let Some(assets) = output.get("discovered_assets") {
                let discovered: Vec<DiscoveredAsset> = serde_json::from_value(assets.clone())?;
                let asset_count = discovered.len() as i32;

                // Create assets in the registry
                for asset in discovered {
                    self.create_asset(&result.tenant_id, asset).await?;
                }

                // Update discovery run status via task_id matching
                if let Some(storage) = &self.storage {
                    let task_uuid: uuid::Uuid = result.task_id.parse().unwrap_or_default();
                    if task_uuid != uuid::Uuid::nil() {
                        let (status, error_msg) = if result.status == "success" {
                            ("completed", None)
                        } else {
                            ("failed", result.error.as_ref().map(|e| e.message.as_str()))
                        };

                        if let Err(e) = storage
                            .update_discovery_run_by_task_id(task_uuid, status, asset_count, error_msg)
                            .await
                        {
                            error!(
                                task_id = %result.task_id,
                                error = %e,
                                "Failed to update discovery run status"
                            );
                        }
                    }
                }

                info!(
                    task_id = %result.task_id,
                    assets_created = asset_count,
                    "Processed discovery results"
                );

                return Ok(());
            }
        }

        // Non-discovery, non-playbook results
        if result.status != "success" {
            return Ok(());
        }

        Ok(())
    }

    async fn process_playbook_result(&self, result: &TaskResult) -> Result<()> {
        let (storage, producer) = match (&self.storage, &self.kafka_producer) {
            (Some(s), Some(p)) => (s, p),
            _ => {
                warn!("Cannot process playbook result - storage or producer not available");
                return Ok(());
            }
        };

        let step_id = result
            .output
            .as_ref()
            .and_then(|o| o.get("step_id"))
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        // The Task struct uses run_id field which maps to TaskResult's run_id equivalent.
        // TaskResult doesn't have a direct run_id field, but we set integration_id = playbook_id
        // in the executor. The run_id was set on the Task, which should come back via headers
        // or we can look it up from output. For now, parse from output or integration_id field.
        let run_id_str = result
            .output
            .as_ref()
            .and_then(|o| o.get("run_id"))
            .and_then(|v| v.as_str());

        let run_id = if let Some(rid) = run_id_str {
            rid.parse::<uuid::Uuid>().unwrap_or_default()
        } else {
            result
                .integration_id
                .parse::<uuid::Uuid>()
                .unwrap_or_default()
        };

        let success = result.status == "success";
        let output = result.output.clone();
        let error_msg = result.error.as_ref().map(|e| e.message.clone());

        info!(
            run_id = %run_id,
            step_id = %step_id,
            success = success,
            "Routing playbook step result to handler"
        );

        if let Err(e) =
            crate::playbooks::result_handler::PlaybookResultHandler::handle_step_result(
                producer,
                storage,
                run_id,
                step_id,
                success,
                output,
                error_msg,
            )
            .await
        {
            error!(
                run_id = %run_id,
                step_id = %step_id,
                error = %e,
                "Failed to handle playbook step result"
            );
        }

        Ok(())
    }

    async fn create_asset(&self, tenant_id: &str, asset: DiscoveredAsset) -> Result<()> {
        let url = format!("{}/assets", self.asset_service_url);

        let body = serde_json::json!({
            "tenant_id": tenant_id,
            "name": asset.name,
            "asset_type": asset.asset_type,
            "description": asset.description,
            "vendor": asset.vendor,
            "version": asset.version,
            "metadata": asset.metadata,
            "status": "active"
        });

        let response = self.http_client.post(&url).json(&body).send().await?;

        if response.status().is_success() {
            info!(name = %asset.name, "Created asset in registry");
        } else {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            error!(
                name = %asset.name,
                status = %status,
                error = %text,
                "Failed to create asset"
            );
        }

        Ok(())
    }
}
