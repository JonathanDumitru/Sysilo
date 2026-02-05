use anyhow::Result;
use futures::StreamExt;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::Message;
use tracing::{error, info, warn};

use crate::kafka::{topics, DiscoveredAsset, TaskResult};

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
}

impl ResultConsumer {
    /// Create a new result consumer
    pub fn new(config: &ConsumerConfig) -> Result<Self> {
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

        // Only process successful discovery tasks
        if result.status != "success" {
            return Ok(());
        }

        // Extract discovered assets from output
        if let Some(output) = &result.output {
            if let Some(assets) = output.get("discovered_assets") {
                let discovered: Vec<DiscoveredAsset> = serde_json::from_value(assets.clone())?;

                for asset in discovered {
                    self.create_asset(&result.tenant_id, asset).await?;
                }
            }
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
