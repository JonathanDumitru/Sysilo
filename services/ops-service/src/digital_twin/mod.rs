use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration as StdDuration;
use tokio::time;
use tracing::{error, info, warn};
use uuid::Uuid;

// ============================================================================
// Types
// ============================================================================

/// Status of a digital twin
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TwinStatus {
    /// Twin is gathering historical data to build baseline
    Learning,
    /// Twin has a baseline and is actively monitoring
    Active,
    /// Twin has detected ongoing anomalies
    Degraded,
    /// Twin has not received data in over 7 days
    Stale,
}

impl TwinStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TwinStatus::Learning => "learning",
            TwinStatus::Active => "active",
            TwinStatus::Degraded => "degraded",
            TwinStatus::Stale => "stale",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "learning" => TwinStatus::Learning,
            "active" => TwinStatus::Active,
            "degraded" => TwinStatus::Degraded,
            "stale" => TwinStatus::Stale,
            _ => TwinStatus::Learning,
        }
    }
}

/// Baseline statistics learned from historical data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwinBaseline {
    // Throughput (records per run)
    pub avg_throughput: f64,
    pub stddev_throughput: f64,
    pub min_throughput: f64,
    pub max_throughput: f64,

    // Latency (ms per run)
    pub avg_latency_ms: f64,
    pub stddev_latency_ms: f64,
    pub p50_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,

    // Error rate (0.0-1.0)
    pub avg_error_rate: f64,
    pub stddev_error_rate: f64,

    // Data shape
    pub expected_field_count: i32,
    pub expected_schema_hash: String,

    // Temporal patterns
    pub typical_run_frequency_secs: f64,
    pub typical_run_times: Vec<u32>,

    // Learning metadata
    pub sample_count: i64,
    pub learning_window_days: i32,
}

impl Default for TwinBaseline {
    fn default() -> Self {
        Self {
            avg_throughput: 0.0,
            stddev_throughput: 0.0,
            min_throughput: 0.0,
            max_throughput: 0.0,
            avg_latency_ms: 0.0,
            stddev_latency_ms: 0.0,
            p50_latency_ms: 0.0,
            p95_latency_ms: 0.0,
            p99_latency_ms: 0.0,
            avg_error_rate: 0.0,
            stddev_error_rate: 0.0,
            expected_field_count: 0,
            expected_schema_hash: String::new(),
            typical_run_frequency_secs: 0.0,
            typical_run_times: Vec::new(),
            sample_count: 0,
            learning_window_days: 30,
        }
    }
}

/// Current state of the integration as observed by the twin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwinState {
    pub last_run_at: Option<DateTime<Utc>>,
    pub last_throughput: Option<f64>,
    pub last_latency_ms: Option<f64>,
    pub last_error_rate: Option<f64>,
    pub last_schema_hash: Option<String>,
    pub health_score: f64,
    pub consecutive_failures: i32,
    pub runs_since_anomaly: i32,
}

impl Default for TwinState {
    fn default() -> Self {
        Self {
            last_run_at: None,
            last_throughput: None,
            last_latency_ms: None,
            last_error_rate: None,
            last_schema_hash: None,
            health_score: 100.0,
            consecutive_failures: 0,
            runs_since_anomaly: 0,
        }
    }
}

/// Types of anomalies the twin can detect
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AnomalyType {
    ThroughputDrop,
    ThroughputSpike,
    LatencySpike,
    ErrorRateSpike,
    SchemaChange,
    MissedSchedule,
    DataVolumeDrift,
}

impl AnomalyType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AnomalyType::ThroughputDrop => "throughput_drop",
            AnomalyType::ThroughputSpike => "throughput_spike",
            AnomalyType::LatencySpike => "latency_spike",
            AnomalyType::ErrorRateSpike => "error_rate_spike",
            AnomalyType::SchemaChange => "schema_change",
            AnomalyType::MissedSchedule => "missed_schedule",
            AnomalyType::DataVolumeDrift => "data_volume_drift",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "throughput_drop" => AnomalyType::ThroughputDrop,
            "throughput_spike" => AnomalyType::ThroughputSpike,
            "latency_spike" => AnomalyType::LatencySpike,
            "error_rate_spike" => AnomalyType::ErrorRateSpike,
            "schema_change" => AnomalyType::SchemaChange,
            "missed_schedule" => AnomalyType::MissedSchedule,
            "data_volume_drift" => AnomalyType::DataVolumeDrift,
            _ => AnomalyType::DataVolumeDrift,
        }
    }
}

/// An anomaly detected by the twin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwinAnomaly {
    pub id: Uuid,
    pub twin_id: Uuid,
    pub anomaly_type: String,
    pub severity: String,
    pub detected_at: DateTime<Utc>,
    pub expected_value: f64,
    pub actual_value: f64,
    pub deviation_sigma: f64,
    pub message: String,
    pub resolved: bool,
    pub resolved_at: Option<DateTime<Utc>>,
}

/// Types of predictions the twin can make
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PredictionType {
    FailureProbability,
    LatencyForecast,
    ThroughputForecast,
    SchemaBreakRisk,
}

impl PredictionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            PredictionType::FailureProbability => "failure_probability",
            PredictionType::LatencyForecast => "latency_forecast",
            PredictionType::ThroughputForecast => "throughput_forecast",
            PredictionType::SchemaBreakRisk => "schema_break_risk",
        }
    }
}

/// A prediction generated by the twin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwinPrediction {
    pub id: Uuid,
    pub twin_id: Uuid,
    pub prediction_type: String,
    pub predicted_at: DateTime<Utc>,
    pub prediction_horizon_secs: i64,
    pub predicted_value: f64,
    pub confidence: f64,
    pub message: String,
}

/// The digital twin model for an integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationTwin {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub integration_id: Uuid,
    pub integration_name: String,
    pub status: String,
    pub baseline: TwinBaseline,
    pub current_state: TwinState,
    pub anomalies: Vec<TwinAnomaly>,
    pub predictions: Vec<TwinPrediction>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_learning_at: Option<DateTime<Utc>>,
}

/// Database row for the twin (flat for sqlx)
#[derive(Debug, Clone, sqlx::FromRow)]
struct TwinRow {
    id: Uuid,
    tenant_id: Uuid,
    integration_id: Uuid,
    integration_name: String,
    status: String,
    baseline: serde_json::Value,
    current_state: serde_json::Value,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    last_learning_at: Option<DateTime<Utc>>,
}

/// Database row for anomalies
#[derive(Debug, Clone, sqlx::FromRow)]
struct AnomalyRow {
    id: Uuid,
    twin_id: Uuid,
    anomaly_type: String,
    severity: String,
    detected_at: DateTime<Utc>,
    expected_value: f64,
    actual_value: f64,
    deviation_sigma: f64,
    message: String,
    resolved: bool,
    resolved_at: Option<DateTime<Utc>>,
}

/// Database row for predictions
#[derive(Debug, Clone, sqlx::FromRow)]
struct PredictionRow {
    id: Uuid,
    twin_id: Uuid,
    prediction_type: String,
    predicted_at: DateTime<Utc>,
    prediction_horizon_secs: i64,
    predicted_value: f64,
    confidence: f64,
    message: String,
}

/// Request to create a twin
#[derive(Debug, Clone, Deserialize)]
pub struct CreateTwinRequest {
    pub integration_id: Uuid,
    pub integration_name: String,
}

/// Request to update twin state from an integration run
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateRunRequest {
    pub throughput: Option<f64>,
    pub latency_ms: Option<f64>,
    pub error_rate: Option<f64>,
    pub schema_hash: Option<String>,
    pub field_count: Option<i32>,
}

/// Parameters for what-if simulation
#[derive(Debug, Clone, Deserialize)]
pub struct SimulationParams {
    /// Proposed throughput multiplier (e.g. 2.0 = double throughput)
    pub throughput_multiplier: Option<f64>,
    /// Proposed latency change in ms
    pub latency_delta_ms: Option<f64>,
    /// Proposed new error rate
    pub new_error_rate: Option<f64>,
    /// Proposed schema change (new hash)
    pub new_schema_hash: Option<String>,
    /// Proposed new run frequency in seconds
    pub new_run_frequency_secs: Option<f64>,
}

/// Result of a what-if simulation
#[derive(Debug, Clone, Serialize)]
pub struct SimulationResult {
    pub current_health_score: f64,
    pub predicted_health_score: f64,
    pub risk_level: String,
    pub predicted_anomalies: Vec<String>,
    pub impact_summary: String,
}

/// Anomaly query filters
#[derive(Debug, Clone, Deserialize)]
pub struct AnomalyFilters {
    pub anomaly_type: Option<String>,
    pub severity: Option<String>,
    pub resolved: Option<bool>,
    pub limit: Option<i64>,
}

/// Prediction query filters
#[derive(Debug, Clone, Deserialize)]
pub struct PredictionFilters {
    pub prediction_type: Option<String>,
    pub limit: Option<i64>,
}

/// Fleet health summary
#[derive(Debug, Clone, Serialize)]
pub struct FleetHealth {
    pub total_twins: i64,
    pub active_twins: i64,
    pub learning_twins: i64,
    pub degraded_twins: i64,
    pub stale_twins: i64,
    pub avg_health_score: f64,
    pub min_health_score: f64,
    pub total_open_anomalies: i64,
    pub critical_anomalies: i64,
    pub overall_status: String,
}

// ============================================================================
// Service
// ============================================================================

/// Service for managing integration digital twins
pub struct DigitalTwinService {
    pool: PgPool,
}

impl DigitalTwinService {
    /// Create a new DigitalTwinService with database connection and table creation
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;

        let service = Self { pool };
        service.create_tables().await?;

        Ok(service)
    }

    /// Create the required database tables
    async fn create_tables(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS integration_twins (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                tenant_id UUID NOT NULL,
                integration_id UUID NOT NULL,
                integration_name TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'learning',
                baseline JSONB NOT NULL DEFAULT '{}',
                current_state JSONB NOT NULL DEFAULT '{}',
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                last_learning_at TIMESTAMPTZ,
                UNIQUE(tenant_id, integration_id)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS twin_anomalies (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                twin_id UUID NOT NULL REFERENCES integration_twins(id) ON DELETE CASCADE,
                anomaly_type TEXT NOT NULL,
                severity TEXT NOT NULL DEFAULT 'medium',
                detected_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                expected_value DOUBLE PRECISION NOT NULL DEFAULT 0.0,
                actual_value DOUBLE PRECISION NOT NULL DEFAULT 0.0,
                deviation_sigma DOUBLE PRECISION NOT NULL DEFAULT 0.0,
                message TEXT NOT NULL DEFAULT '',
                resolved BOOLEAN NOT NULL DEFAULT false,
                resolved_at TIMESTAMPTZ
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS twin_predictions (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                twin_id UUID NOT NULL REFERENCES integration_twins(id) ON DELETE CASCADE,
                prediction_type TEXT NOT NULL,
                predicted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                prediction_horizon_secs BIGINT NOT NULL DEFAULT 3600,
                predicted_value DOUBLE PRECISION NOT NULL DEFAULT 0.0,
                confidence DOUBLE PRECISION NOT NULL DEFAULT 0.0,
                message TEXT NOT NULL DEFAULT ''
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create indexes
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_twins_tenant ON integration_twins(tenant_id);
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_twin_anomalies_twin ON twin_anomalies(twin_id, resolved);
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_twin_predictions_twin ON twin_predictions(twin_id, predicted_at);
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Create a new digital twin for an integration (starts in learning state)
    pub async fn create_twin(
        &self,
        tenant_id: Uuid,
        req: CreateTwinRequest,
    ) -> Result<IntegrationTwin> {
        let baseline = TwinBaseline::default();
        let state = TwinState::default();

        let row = sqlx::query_as::<_, TwinRow>(
            r#"
            INSERT INTO integration_twins
                (tenant_id, integration_id, integration_name, status, baseline, current_state)
            VALUES ($1, $2, $3, 'learning', $4, $5)
            RETURNING id, tenant_id, integration_id, integration_name, status,
                      baseline, current_state, created_at, updated_at, last_learning_at
            "#,
        )
        .bind(tenant_id)
        .bind(req.integration_id)
        .bind(&req.integration_name)
        .bind(serde_json::to_value(&baseline)?)
        .bind(serde_json::to_value(&state)?)
        .fetch_one(&self.pool)
        .await?;

        Ok(self.row_to_twin(row, vec![], vec![]))
    }

    /// Get a twin by tenant_id and integration_id, with anomalies and predictions
    pub async fn get_twin(
        &self,
        tenant_id: Uuid,
        integration_id: Uuid,
    ) -> Result<Option<IntegrationTwin>> {
        let row = sqlx::query_as::<_, TwinRow>(
            r#"
            SELECT id, tenant_id, integration_id, integration_name, status,
                   baseline, current_state, created_at, updated_at, last_learning_at
            FROM integration_twins
            WHERE tenant_id = $1 AND integration_id = $2
            "#,
        )
        .bind(tenant_id)
        .bind(integration_id)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => {
                let anomalies = self.fetch_anomalies_for_twin(row.id, 50).await?;
                let predictions = self.fetch_predictions_for_twin(row.id, 20).await?;
                Ok(Some(self.row_to_twin(row, anomalies, predictions)))
            }
            None => Ok(None),
        }
    }

    /// List all twins for a tenant
    pub async fn list_twins(&self, tenant_id: Uuid) -> Result<Vec<IntegrationTwin>> {
        let rows = sqlx::query_as::<_, TwinRow>(
            r#"
            SELECT id, tenant_id, integration_id, integration_name, status,
                   baseline, current_state, created_at, updated_at, last_learning_at
            FROM integration_twins
            WHERE tenant_id = $1
            ORDER BY integration_name
            "#,
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await?;

        let mut twins = Vec::with_capacity(rows.len());
        for row in rows {
            let anomalies = self.fetch_anomalies_for_twin(row.id, 10).await?;
            let predictions = self.fetch_predictions_for_twin(row.id, 5).await?;
            twins.push(self.row_to_twin(row, anomalies, predictions));
        }
        Ok(twins)
    }

    /// Learn baseline from historical metrics data for a twin.
    /// Queries the metrics table for the integration's data over the learning window
    /// and computes statistical baselines.
    pub async fn learn_baseline(&self, tenant_id: Uuid, twin_id: Uuid) -> Result<IntegrationTwin> {
        let row = sqlx::query_as::<_, TwinRow>(
            r#"
            SELECT id, tenant_id, integration_id, integration_name, status,
                   baseline, current_state, created_at, updated_at, last_learning_at
            FROM integration_twins
            WHERE id = $1 AND tenant_id = $2
            "#,
        )
        .bind(twin_id)
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Twin not found"))?;

        let learning_window_days = 30i32;
        let start_time = Utc::now() - Duration::days(learning_window_days as i64);
        let integration_id = row.integration_id;

        // Query throughput metrics
        let throughput_stats = self
            .query_metric_stats(tenant_id, integration_id, "throughput", start_time)
            .await?;

        // Query latency metrics
        let latency_stats = self
            .query_metric_stats(tenant_id, integration_id, "latency_ms", start_time)
            .await?;

        // Query error rate metrics
        let error_stats = self
            .query_metric_stats(tenant_id, integration_id, "error_rate", start_time)
            .await?;

        // Query latency percentiles
        let latency_percentiles = self
            .query_percentiles(tenant_id, integration_id, "latency_ms", start_time)
            .await?;

        // Detect temporal patterns (which hours of day runs typically happen)
        let run_hours = self
            .query_run_hours(tenant_id, integration_id, start_time)
            .await?;

        // Compute typical run frequency from timestamps
        let run_frequency = self
            .query_run_frequency(tenant_id, integration_id, start_time)
            .await?;

        // Get latest schema hash
        let latest_schema_hash = self
            .query_latest_tag_value(tenant_id, integration_id, "schema_hash", start_time)
            .await?
            .unwrap_or_default();

        // Get latest field count
        let field_count: i32 = self
            .query_latest_tag_value(tenant_id, integration_id, "field_count", start_time)
            .await?
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);

        let sample_count = throughput_stats.count.max(latency_stats.count).max(error_stats.count);

        let baseline = TwinBaseline {
            avg_throughput: throughput_stats.avg,
            stddev_throughput: throughput_stats.stddev,
            min_throughput: throughput_stats.min,
            max_throughput: throughput_stats.max,
            avg_latency_ms: latency_stats.avg,
            stddev_latency_ms: latency_stats.stddev,
            p50_latency_ms: latency_percentiles.0,
            p95_latency_ms: latency_percentiles.1,
            p99_latency_ms: latency_percentiles.2,
            avg_error_rate: error_stats.avg,
            stddev_error_rate: error_stats.stddev,
            expected_field_count: field_count,
            expected_schema_hash: latest_schema_hash,
            typical_run_frequency_secs: run_frequency,
            typical_run_times: run_hours,
            sample_count,
            learning_window_days,
        };

        let new_status = if sample_count >= 10 {
            "active"
        } else {
            "learning"
        };

        let updated_row = sqlx::query_as::<_, TwinRow>(
            r#"
            UPDATE integration_twins SET
                baseline = $1,
                status = $2,
                last_learning_at = NOW(),
                updated_at = NOW()
            WHERE id = $3 AND tenant_id = $4
            RETURNING id, tenant_id, integration_id, integration_name, status,
                      baseline, current_state, created_at, updated_at, last_learning_at
            "#,
        )
        .bind(serde_json::to_value(&baseline)?)
        .bind(new_status)
        .bind(twin_id)
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await?;

        let anomalies = self.fetch_anomalies_for_twin(twin_id, 50).await?;
        let predictions = self.fetch_predictions_for_twin(twin_id, 20).await?;
        Ok(self.row_to_twin(updated_row, anomalies, predictions))
    }

    /// Update twin state after an integration run. Detects anomalies and generates predictions.
    pub async fn update_state(
        &self,
        tenant_id: Uuid,
        integration_id: Uuid,
        run: UpdateRunRequest,
    ) -> Result<IntegrationTwin> {
        let row = sqlx::query_as::<_, TwinRow>(
            r#"
            SELECT id, tenant_id, integration_id, integration_name, status,
                   baseline, current_state, created_at, updated_at, last_learning_at
            FROM integration_twins
            WHERE tenant_id = $1 AND integration_id = $2
            "#,
        )
        .bind(tenant_id)
        .bind(integration_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Twin not found for integration"))?;

        let twin_id = row.id;
        let baseline: TwinBaseline =
            serde_json::from_value(row.baseline.clone()).unwrap_or_default();
        let mut state: TwinState =
            serde_json::from_value(row.current_state.clone()).unwrap_or_default();

        // Update current state with latest run data
        state.last_run_at = Some(Utc::now());
        if let Some(t) = run.throughput {
            state.last_throughput = Some(t);
        }
        if let Some(l) = run.latency_ms {
            state.last_latency_ms = Some(l);
        }
        if let Some(e) = run.error_rate {
            state.last_error_rate = Some(e);
        }
        if let Some(ref h) = run.schema_hash {
            state.last_schema_hash = Some(h.clone());
        }

        // Track consecutive failures
        if run.error_rate.unwrap_or(0.0) >= 1.0 {
            state.consecutive_failures += 1;
        } else {
            state.consecutive_failures = 0;
        }

        // Compute health score as a weighted combination of deviations
        let health_score =
            self.compute_health_score(&baseline, &state);
        state.health_score = health_score;

        // Detect anomalies
        let detected_anomalies =
            self.detect_anomalies_internal(&baseline, &state, twin_id);
        let anomaly_count = detected_anomalies.len();

        // Insert anomalies into database
        for anomaly in &detected_anomalies {
            sqlx::query(
                r#"
                INSERT INTO twin_anomalies
                    (twin_id, anomaly_type, severity, expected_value, actual_value,
                     deviation_sigma, message)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                "#,
            )
            .bind(twin_id)
            .bind(&anomaly.anomaly_type)
            .bind(&anomaly.severity)
            .bind(anomaly.expected_value)
            .bind(anomaly.actual_value)
            .bind(anomaly.deviation_sigma)
            .bind(&anomaly.message)
            .execute(&self.pool)
            .await?;
        }

        if anomaly_count > 0 {
            state.runs_since_anomaly = 0;
        } else {
            state.runs_since_anomaly += 1;
        }

        // Generate predictions
        let predictions = self.generate_predictions(&baseline, &state, twin_id);
        // Clear old predictions and insert new ones
        sqlx::query("DELETE FROM twin_predictions WHERE twin_id = $1")
            .bind(twin_id)
            .execute(&self.pool)
            .await?;

        for pred in &predictions {
            sqlx::query(
                r#"
                INSERT INTO twin_predictions
                    (twin_id, prediction_type, prediction_horizon_secs,
                     predicted_value, confidence, message)
                VALUES ($1, $2, $3, $4, $5, $6)
                "#,
            )
            .bind(twin_id)
            .bind(&pred.prediction_type)
            .bind(pred.prediction_horizon_secs)
            .bind(pred.predicted_value)
            .bind(pred.confidence)
            .bind(&pred.message)
            .execute(&self.pool)
            .await?;
        }

        // Determine new status
        let new_status = if anomaly_count > 0 && health_score < 50.0 {
            "degraded"
        } else if row.status == "degraded" && anomaly_count == 0 {
            "active"
        } else {
            &row.status
        };

        // Persist updated state
        let updated_row = sqlx::query_as::<_, TwinRow>(
            r#"
            UPDATE integration_twins SET
                current_state = $1,
                status = $2,
                updated_at = NOW()
            WHERE id = $3 AND tenant_id = $4
            RETURNING id, tenant_id, integration_id, integration_name, status,
                      baseline, current_state, created_at, updated_at, last_learning_at
            "#,
        )
        .bind(serde_json::to_value(&state)?)
        .bind(new_status)
        .bind(twin_id)
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await?;

        let anomalies = self.fetch_anomalies_for_twin(twin_id, 50).await?;
        let preds = self.fetch_predictions_for_twin(twin_id, 20).await?;
        Ok(self.row_to_twin(updated_row, anomalies, preds))
    }

    /// Run a what-if simulation on a twin
    pub async fn simulate_change(
        &self,
        tenant_id: Uuid,
        integration_id: Uuid,
        params: SimulationParams,
    ) -> Result<SimulationResult> {
        let twin = self
            .get_twin(tenant_id, integration_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Twin not found"))?;

        let baseline = &twin.baseline;
        let current_state = &twin.current_state;

        // Create a simulated state by applying proposed changes
        let mut sim_state = current_state.clone();

        if let Some(multiplier) = params.throughput_multiplier {
            let base_throughput = sim_state.last_throughput.unwrap_or(baseline.avg_throughput);
            sim_state.last_throughput = Some(base_throughput * multiplier);
        }

        if let Some(delta) = params.latency_delta_ms {
            let base_latency = sim_state.last_latency_ms.unwrap_or(baseline.avg_latency_ms);
            sim_state.last_latency_ms = Some((base_latency + delta).max(0.0));
        }

        if let Some(err_rate) = params.new_error_rate {
            sim_state.last_error_rate = Some(err_rate.clamp(0.0, 1.0));
        }

        if let Some(ref hash) = params.new_schema_hash {
            sim_state.last_schema_hash = Some(hash.clone());
        }

        let predicted_health = self.compute_health_score(baseline, &sim_state);
        let current_health = current_state.health_score;

        // Detect what anomalies the simulated state would trigger
        let sim_anomalies =
            self.detect_anomalies_internal(baseline, &sim_state, twin.id);
        let predicted_anomaly_descriptions: Vec<String> =
            sim_anomalies.iter().map(|a| a.message.clone()).collect();

        let risk_level = if predicted_health >= 80.0 {
            "low".to_string()
        } else if predicted_health >= 50.0 {
            "medium".to_string()
        } else if predicted_health >= 20.0 {
            "high".to_string()
        } else {
            "critical".to_string()
        };

        let health_delta = predicted_health - current_health;
        let impact_summary = format!(
            "Health score would change from {:.1} to {:.1} ({:+.1}). {} anomalies predicted. Risk level: {}.",
            current_health,
            predicted_health,
            health_delta,
            sim_anomalies.len(),
            risk_level
        );

        Ok(SimulationResult {
            current_health_score: current_health,
            predicted_health_score: predicted_health,
            risk_level,
            predicted_anomalies: predicted_anomaly_descriptions,
            impact_summary,
        })
    }

    /// List anomalies across all twins for a tenant
    pub async fn get_anomalies(
        &self,
        tenant_id: Uuid,
        filters: AnomalyFilters,
    ) -> Result<Vec<TwinAnomaly>> {
        let limit = filters.limit.unwrap_or(100).min(1000);

        let rows = sqlx::query_as::<_, AnomalyRow>(
            r#"
            SELECT ta.id, ta.twin_id, ta.anomaly_type, ta.severity, ta.detected_at,
                   ta.expected_value, ta.actual_value, ta.deviation_sigma,
                   ta.message, ta.resolved, ta.resolved_at
            FROM twin_anomalies ta
            JOIN integration_twins it ON it.id = ta.twin_id
            WHERE it.tenant_id = $1
              AND ($2::text IS NULL OR ta.anomaly_type = $2)
              AND ($3::text IS NULL OR ta.severity = $3)
              AND ($4::boolean IS NULL OR ta.resolved = $4)
            ORDER BY ta.detected_at DESC
            LIMIT $5
            "#,
        )
        .bind(tenant_id)
        .bind(&filters.anomaly_type)
        .bind(&filters.severity)
        .bind(filters.resolved)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(Self::anomaly_from_row).collect())
    }

    /// List predictions across all twins for a tenant
    pub async fn get_predictions(
        &self,
        tenant_id: Uuid,
        filters: PredictionFilters,
    ) -> Result<Vec<TwinPrediction>> {
        let limit = filters.limit.unwrap_or(100).min(1000);

        let rows = sqlx::query_as::<_, PredictionRow>(
            r#"
            SELECT tp.id, tp.twin_id, tp.prediction_type, tp.predicted_at,
                   tp.prediction_horizon_secs, tp.predicted_value, tp.confidence,
                   tp.message
            FROM twin_predictions tp
            JOIN integration_twins it ON it.id = tp.twin_id
            WHERE it.tenant_id = $1
              AND ($2::text IS NULL OR tp.prediction_type = $2)
            ORDER BY tp.predicted_at DESC
            LIMIT $3
            "#,
        )
        .bind(tenant_id)
        .bind(&filters.prediction_type)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(Self::prediction_from_row).collect())
    }

    /// Get aggregate fleet health across all twins for a tenant
    pub async fn get_fleet_health(&self, tenant_id: Uuid) -> Result<FleetHealth> {
        // Get twin counts by status
        let total: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM integration_twins WHERE tenant_id = $1",
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await?;

        let active: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM integration_twins WHERE tenant_id = $1 AND status = 'active'",
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await?;

        let learning: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM integration_twins WHERE tenant_id = $1 AND status = 'learning'",
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await?;

        let degraded: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM integration_twins WHERE tenant_id = $1 AND status = 'degraded'",
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await?;

        let stale: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM integration_twins WHERE tenant_id = $1 AND status = 'stale'",
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await?;

        // Get health score stats from current_state JSONB
        let health_stats: (Option<f64>, Option<f64>) = sqlx::query_as(
            r#"
            SELECT
                AVG((current_state->>'health_score')::float),
                MIN((current_state->>'health_score')::float)
            FROM integration_twins
            WHERE tenant_id = $1 AND status IN ('active', 'degraded')
            "#,
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await?;

        // Count open anomalies
        let open_anomalies: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*)
            FROM twin_anomalies ta
            JOIN integration_twins it ON it.id = ta.twin_id
            WHERE it.tenant_id = $1 AND ta.resolved = false
            "#,
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await?;

        let critical_anomalies: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*)
            FROM twin_anomalies ta
            JOIN integration_twins it ON it.id = ta.twin_id
            WHERE it.tenant_id = $1 AND ta.resolved = false AND ta.severity = 'critical'
            "#,
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await?;

        let avg_health = health_stats.0.unwrap_or(100.0);
        let min_health = health_stats.1.unwrap_or(100.0);

        let overall_status = if critical_anomalies.0 > 0 || degraded.0 > 0 {
            "degraded"
        } else if avg_health >= 80.0 {
            "healthy"
        } else if avg_health >= 50.0 {
            "warning"
        } else {
            "critical"
        };

        Ok(FleetHealth {
            total_twins: total.0,
            active_twins: active.0,
            learning_twins: learning.0,
            degraded_twins: degraded.0,
            stale_twins: stale.0,
            avg_health_score: avg_health,
            min_health_score: min_health,
            total_open_anomalies: open_anomalies.0,
            critical_anomalies: critical_anomalies.0,
            overall_status: overall_status.to_string(),
        })
    }

    // ========================================================================
    // Internal helpers
    // ========================================================================

    /// Compute health score as weighted combination of deviations from baseline
    fn compute_health_score(&self, baseline: &TwinBaseline, state: &TwinState) -> f64 {
        if baseline.sample_count == 0 {
            return 100.0;
        }

        let mut score = 100.0f64;

        // Throughput deviation (weight: 25)
        if let Some(throughput) = state.last_throughput {
            if baseline.stddev_throughput > 0.0 {
                let deviation =
                    ((throughput - baseline.avg_throughput) / baseline.stddev_throughput).abs();
                let penalty = (deviation * 8.0).min(25.0);
                score -= penalty;
            }
        }

        // Latency deviation (weight: 25)
        if let Some(latency) = state.last_latency_ms {
            if baseline.stddev_latency_ms > 0.0 {
                let deviation =
                    ((latency - baseline.avg_latency_ms) / baseline.stddev_latency_ms).max(0.0);
                let penalty = (deviation * 8.0).min(25.0);
                score -= penalty;
            }
        }

        // Error rate deviation (weight: 30)
        if let Some(error_rate) = state.last_error_rate {
            if baseline.stddev_error_rate > 0.0 {
                let deviation =
                    ((error_rate - baseline.avg_error_rate) / baseline.stddev_error_rate).max(0.0);
                let penalty = (deviation * 10.0).min(30.0);
                score -= penalty;
            } else if error_rate > baseline.avg_error_rate + 0.01 {
                // No stddev but error rate increased
                let penalty = ((error_rate - baseline.avg_error_rate) * 100.0).min(30.0);
                score -= penalty;
            }
        }

        // Schema consistency (weight: 10)
        if let Some(ref hash) = state.last_schema_hash {
            if !baseline.expected_schema_hash.is_empty() && hash != &baseline.expected_schema_hash {
                score -= 10.0;
            }
        }

        // Consecutive failures penalty (weight: 10)
        let failure_penalty = (state.consecutive_failures as f64 * 5.0).min(10.0);
        score -= failure_penalty;

        score.clamp(0.0, 100.0)
    }

    /// Detect anomalies by comparing current state against baseline
    fn detect_anomalies_internal(
        &self,
        baseline: &TwinBaseline,
        state: &TwinState,
        twin_id: Uuid,
    ) -> Vec<TwinAnomaly> {
        let mut anomalies = Vec::new();

        if baseline.sample_count == 0 {
            return anomalies;
        }

        // Throughput drop: current < avg - 2*stddev
        if let Some(throughput) = state.last_throughput {
            if baseline.stddev_throughput > 0.0 {
                let lower_bound = baseline.avg_throughput - 2.0 * baseline.stddev_throughput;
                if throughput < lower_bound {
                    let sigma = (baseline.avg_throughput - throughput) / baseline.stddev_throughput;
                    let severity = if sigma > 4.0 {
                        "critical"
                    } else if sigma > 3.0 {
                        "high"
                    } else {
                        "medium"
                    };
                    anomalies.push(TwinAnomaly {
                        id: Uuid::new_v4(),
                        twin_id,
                        anomaly_type: AnomalyType::ThroughputDrop.as_str().to_string(),
                        severity: severity.to_string(),
                        detected_at: Utc::now(),
                        expected_value: baseline.avg_throughput,
                        actual_value: throughput,
                        deviation_sigma: sigma,
                        message: format!(
                            "Throughput dropped to {:.1} (expected {:.1} +/- {:.1}, {:.1} sigma below mean)",
                            throughput, baseline.avg_throughput, baseline.stddev_throughput, sigma
                        ),
                        resolved: false,
                        resolved_at: None,
                    });
                }

                // Throughput spike: current > avg + 3*stddev
                let upper_bound = baseline.avg_throughput + 3.0 * baseline.stddev_throughput;
                if throughput > upper_bound {
                    let sigma = (throughput - baseline.avg_throughput) / baseline.stddev_throughput;
                    let severity = if sigma > 5.0 {
                        "high"
                    } else {
                        "medium"
                    };
                    anomalies.push(TwinAnomaly {
                        id: Uuid::new_v4(),
                        twin_id,
                        anomaly_type: AnomalyType::ThroughputSpike.as_str().to_string(),
                        severity: severity.to_string(),
                        detected_at: Utc::now(),
                        expected_value: baseline.avg_throughput,
                        actual_value: throughput,
                        deviation_sigma: sigma,
                        message: format!(
                            "Throughput spiked to {:.1} (expected {:.1} +/- {:.1}, {:.1} sigma above mean)",
                            throughput, baseline.avg_throughput, baseline.stddev_throughput, sigma
                        ),
                        resolved: false,
                        resolved_at: None,
                    });
                }
            }
        }

        // Latency spike: current > p95
        if let Some(latency) = state.last_latency_ms {
            if baseline.p95_latency_ms > 0.0 && latency > baseline.p95_latency_ms {
                let sigma = if baseline.stddev_latency_ms > 0.0 {
                    (latency - baseline.avg_latency_ms) / baseline.stddev_latency_ms
                } else {
                    2.0
                };
                let severity = if latency > baseline.p99_latency_ms {
                    "critical"
                } else {
                    "high"
                };
                anomalies.push(TwinAnomaly {
                    id: Uuid::new_v4(),
                    twin_id,
                    anomaly_type: AnomalyType::LatencySpike.as_str().to_string(),
                    severity: severity.to_string(),
                    detected_at: Utc::now(),
                    expected_value: baseline.p95_latency_ms,
                    actual_value: latency,
                    deviation_sigma: sigma,
                    message: format!(
                        "Latency at {:.1}ms exceeds p95 baseline of {:.1}ms ({:.1} sigma)",
                        latency, baseline.p95_latency_ms, sigma
                    ),
                    resolved: false,
                    resolved_at: None,
                });
            }
        }

        // Error rate spike: current > avg + 2*stddev
        if let Some(error_rate) = state.last_error_rate {
            let threshold = if baseline.stddev_error_rate > 0.0 {
                baseline.avg_error_rate + 2.0 * baseline.stddev_error_rate
            } else {
                baseline.avg_error_rate + 0.05
            };

            if error_rate > threshold && error_rate > 0.01 {
                let sigma = if baseline.stddev_error_rate > 0.0 {
                    (error_rate - baseline.avg_error_rate) / baseline.stddev_error_rate
                } else {
                    3.0
                };
                let severity = if error_rate > 0.5 {
                    "critical"
                } else if error_rate > 0.2 {
                    "high"
                } else {
                    "medium"
                };
                anomalies.push(TwinAnomaly {
                    id: Uuid::new_v4(),
                    twin_id,
                    anomaly_type: AnomalyType::ErrorRateSpike.as_str().to_string(),
                    severity: severity.to_string(),
                    detected_at: Utc::now(),
                    expected_value: baseline.avg_error_rate,
                    actual_value: error_rate,
                    deviation_sigma: sigma,
                    message: format!(
                        "Error rate at {:.2}% exceeds baseline of {:.2}% ({:.1} sigma)",
                        error_rate * 100.0,
                        baseline.avg_error_rate * 100.0,
                        sigma
                    ),
                    resolved: false,
                    resolved_at: None,
                });
            }
        }

        // Schema change: hash mismatch
        if let Some(ref hash) = state.last_schema_hash {
            if !baseline.expected_schema_hash.is_empty() && hash != &baseline.expected_schema_hash {
                anomalies.push(TwinAnomaly {
                    id: Uuid::new_v4(),
                    twin_id,
                    anomaly_type: AnomalyType::SchemaChange.as_str().to_string(),
                    severity: "high".to_string(),
                    detected_at: Utc::now(),
                    expected_value: 0.0,
                    actual_value: 1.0,
                    deviation_sigma: 0.0,
                    message: format!(
                        "Schema hash changed from '{}' to '{}'",
                        baseline.expected_schema_hash, hash
                    ),
                    resolved: false,
                    resolved_at: None,
                });
            }
        }

        // Missed schedule: time since last run > 2x typical frequency
        if let Some(last_run) = state.last_run_at {
            if baseline.typical_run_frequency_secs > 0.0 {
                let elapsed = (Utc::now() - last_run).num_seconds() as f64;
                let threshold = baseline.typical_run_frequency_secs * 2.0;
                if elapsed > threshold {
                    let ratio = elapsed / baseline.typical_run_frequency_secs;
                    anomalies.push(TwinAnomaly {
                        id: Uuid::new_v4(),
                        twin_id,
                        anomaly_type: AnomalyType::MissedSchedule.as_str().to_string(),
                        severity: if ratio > 5.0 { "critical" } else { "high" }.to_string(),
                        detected_at: Utc::now(),
                        expected_value: baseline.typical_run_frequency_secs,
                        actual_value: elapsed,
                        deviation_sigma: ratio,
                        message: format!(
                            "No run for {:.0}s (expected every {:.0}s, {:.1}x overdue)",
                            elapsed, baseline.typical_run_frequency_secs, ratio
                        ),
                        resolved: false,
                        resolved_at: None,
                    });
                }
            }
        }

        anomalies
    }

    /// Generate statistical predictions based on current state and baseline
    fn generate_predictions(
        &self,
        baseline: &TwinBaseline,
        state: &TwinState,
        twin_id: Uuid,
    ) -> Vec<TwinPrediction> {
        let mut predictions = Vec::new();

        if baseline.sample_count == 0 {
            return predictions;
        }

        let horizon_1h = 3600i64;
        let horizon_24h = 86400i64;

        // Failure probability: based on error rate trend and consecutive failures
        let failure_prob = {
            let base_prob = state.last_error_rate.unwrap_or(0.0);
            let failure_factor = (state.consecutive_failures as f64 * 0.15).min(0.6);
            let health_factor = ((100.0 - state.health_score) / 200.0).min(0.3);
            (base_prob + failure_factor + health_factor).clamp(0.0, 1.0)
        };

        let confidence = if baseline.sample_count > 100 {
            0.85
        } else if baseline.sample_count > 30 {
            0.7
        } else {
            0.5
        };

        predictions.push(TwinPrediction {
            id: Uuid::new_v4(),
            twin_id,
            prediction_type: PredictionType::FailureProbability.as_str().to_string(),
            predicted_at: Utc::now(),
            prediction_horizon_secs: horizon_24h,
            predicted_value: failure_prob,
            confidence,
            message: format!(
                "Estimated {:.1}% probability of failure in next 24h (consecutive failures: {}, health: {:.1})",
                failure_prob * 100.0, state.consecutive_failures, state.health_score
            ),
        });

        // Latency forecast: exponential moving average projection
        if let Some(latency) = state.last_latency_ms {
            let alpha = 0.3; // EMA smoothing factor
            let ema_latency = alpha * latency + (1.0 - alpha) * baseline.avg_latency_ms;
            let trend = latency - baseline.avg_latency_ms;
            let forecast = ema_latency + trend * 0.5; // project half the trend forward

            predictions.push(TwinPrediction {
                id: Uuid::new_v4(),
                twin_id,
                prediction_type: PredictionType::LatencyForecast.as_str().to_string(),
                predicted_at: Utc::now(),
                prediction_horizon_secs: horizon_1h,
                predicted_value: forecast.max(0.0),
                confidence: confidence * 0.9,
                message: format!(
                    "Predicted latency of {:.1}ms in next hour (current: {:.1}ms, baseline avg: {:.1}ms)",
                    forecast.max(0.0), latency, baseline.avg_latency_ms
                ),
            });
        }

        // Throughput forecast: exponential moving average
        if let Some(throughput) = state.last_throughput {
            let alpha = 0.3;
            let ema_throughput = alpha * throughput + (1.0 - alpha) * baseline.avg_throughput;
            let trend = throughput - baseline.avg_throughput;
            let forecast = ema_throughput + trend * 0.3;

            predictions.push(TwinPrediction {
                id: Uuid::new_v4(),
                twin_id,
                prediction_type: PredictionType::ThroughputForecast.as_str().to_string(),
                predicted_at: Utc::now(),
                prediction_horizon_secs: horizon_1h,
                predicted_value: forecast.max(0.0),
                confidence: confidence * 0.85,
                message: format!(
                    "Predicted throughput of {:.1} records/run in next hour (current: {:.1}, baseline: {:.1})",
                    forecast.max(0.0), throughput, baseline.avg_throughput
                ),
            });
        }

        // Schema break risk: based on whether schema has recently changed
        let schema_risk = if let Some(ref hash) = state.last_schema_hash {
            if !baseline.expected_schema_hash.is_empty() && hash != &baseline.expected_schema_hash {
                0.6 // schema is currently different, high risk
            } else {
                0.05 // schema matches baseline
            }
        } else {
            0.1 // no schema data, low but uncertain risk
        };

        predictions.push(TwinPrediction {
            id: Uuid::new_v4(),
            twin_id,
            prediction_type: PredictionType::SchemaBreakRisk.as_str().to_string(),
            predicted_at: Utc::now(),
            prediction_horizon_secs: horizon_24h,
            predicted_value: schema_risk,
            confidence: confidence * 0.7,
            message: format!(
                "Schema break risk: {:.1}% in next 24h",
                schema_risk * 100.0
            ),
        });

        predictions
    }

    // ========================================================================
    // Database query helpers
    // ========================================================================

    /// Query aggregate statistics for a metric
    async fn query_metric_stats(
        &self,
        tenant_id: Uuid,
        integration_id: Uuid,
        metric_name: &str,
        start_time: DateTime<Utc>,
    ) -> Result<MetricStatsResult> {
        let row: (Option<f64>, Option<f64>, Option<f64>, Option<f64>, i64) = sqlx::query_as(
            r#"
            SELECT
                AVG(metric_value),
                COALESCE(STDDEV(metric_value), 0),
                MIN(metric_value),
                MAX(metric_value),
                COUNT(*)
            FROM metrics
            WHERE tenant_id = $1
              AND resource_id = $2
              AND metric_name = $3
              AND recorded_at >= $4
            "#,
        )
        .bind(tenant_id)
        .bind(integration_id)
        .bind(metric_name)
        .bind(start_time)
        .fetch_one(&self.pool)
        .await?;

        Ok(MetricStatsResult {
            avg: row.0.unwrap_or(0.0),
            stddev: row.1.unwrap_or(0.0),
            min: row.2.unwrap_or(0.0),
            max: row.3.unwrap_or(0.0),
            count: row.4,
        })
    }

    /// Query latency percentiles (p50, p95, p99)
    async fn query_percentiles(
        &self,
        tenant_id: Uuid,
        integration_id: Uuid,
        metric_name: &str,
        start_time: DateTime<Utc>,
    ) -> Result<(f64, f64, f64)> {
        let row: (Option<f64>, Option<f64>, Option<f64>) = sqlx::query_as(
            r#"
            SELECT
                PERCENTILE_CONT(0.50) WITHIN GROUP (ORDER BY metric_value),
                PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY metric_value),
                PERCENTILE_CONT(0.99) WITHIN GROUP (ORDER BY metric_value)
            FROM metrics
            WHERE tenant_id = $1
              AND resource_id = $2
              AND metric_name = $3
              AND recorded_at >= $4
            "#,
        )
        .bind(tenant_id)
        .bind(integration_id)
        .bind(metric_name)
        .bind(start_time)
        .fetch_one(&self.pool)
        .await?;

        Ok((
            row.0.unwrap_or(0.0),
            row.1.unwrap_or(0.0),
            row.2.unwrap_or(0.0),
        ))
    }

    /// Query which hours of day runs typically happen
    async fn query_run_hours(
        &self,
        tenant_id: Uuid,
        integration_id: Uuid,
        start_time: DateTime<Utc>,
    ) -> Result<Vec<u32>> {
        let rows: Vec<(f64,)> = sqlx::query_as(
            r#"
            SELECT DISTINCT EXTRACT(HOUR FROM recorded_at)::float8 as hour
            FROM metrics
            WHERE tenant_id = $1
              AND resource_id = $2
              AND metric_name = 'throughput'
              AND recorded_at >= $3
            ORDER BY hour
            "#,
        )
        .bind(tenant_id)
        .bind(integration_id)
        .bind(start_time)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.0 as u32).collect())
    }

    /// Compute average frequency between metric data points (in seconds)
    async fn query_run_frequency(
        &self,
        tenant_id: Uuid,
        integration_id: Uuid,
        start_time: DateTime<Utc>,
    ) -> Result<f64> {
        // Get the average interval between consecutive records
        let row: (Option<f64>,) = sqlx::query_as(
            r#"
            WITH ordered AS (
                SELECT recorded_at,
                       LAG(recorded_at) OVER (ORDER BY recorded_at) as prev_at
                FROM metrics
                WHERE tenant_id = $1
                  AND resource_id = $2
                  AND metric_name = 'throughput'
                  AND recorded_at >= $3
            )
            SELECT AVG(EXTRACT(EPOCH FROM (recorded_at - prev_at)))::float8
            FROM ordered
            WHERE prev_at IS NOT NULL
            "#,
        )
        .bind(tenant_id)
        .bind(integration_id)
        .bind(start_time)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.0.unwrap_or(0.0))
    }

    /// Query the latest tag value from metrics for a specific key
    async fn query_latest_tag_value(
        &self,
        tenant_id: Uuid,
        integration_id: Uuid,
        tag_key: &str,
        start_time: DateTime<Utc>,
    ) -> Result<Option<String>> {
        let row: Option<(Option<serde_json::Value>,)> = sqlx::query_as(
            r#"
            SELECT tags
            FROM metrics
            WHERE tenant_id = $1
              AND resource_id = $2
              AND recorded_at >= $3
              AND tags ? $4
            ORDER BY recorded_at DESC
            LIMIT 1
            "#,
        )
        .bind(tenant_id)
        .bind(integration_id)
        .bind(start_time)
        .bind(tag_key)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some((Some(tags),)) => {
                let value = tags.get(tag_key).and_then(|v| v.as_str()).map(String::from);
                Ok(value)
            }
            _ => Ok(None),
        }
    }

    /// Fetch anomalies for a specific twin
    async fn fetch_anomalies_for_twin(
        &self,
        twin_id: Uuid,
        limit: i64,
    ) -> Result<Vec<TwinAnomaly>> {
        let rows = sqlx::query_as::<_, AnomalyRow>(
            r#"
            SELECT id, twin_id, anomaly_type, severity, detected_at,
                   expected_value, actual_value, deviation_sigma,
                   message, resolved, resolved_at
            FROM twin_anomalies
            WHERE twin_id = $1
            ORDER BY detected_at DESC
            LIMIT $2
            "#,
        )
        .bind(twin_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(Self::anomaly_from_row).collect())
    }

    /// Fetch predictions for a specific twin
    async fn fetch_predictions_for_twin(
        &self,
        twin_id: Uuid,
        limit: i64,
    ) -> Result<Vec<TwinPrediction>> {
        let rows = sqlx::query_as::<_, PredictionRow>(
            r#"
            SELECT id, twin_id, prediction_type, predicted_at,
                   prediction_horizon_secs, predicted_value, confidence, message
            FROM twin_predictions
            WHERE twin_id = $1
            ORDER BY predicted_at DESC
            LIMIT $2
            "#,
        )
        .bind(twin_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(Self::prediction_from_row).collect())
    }

    /// Convert a TwinRow + anomalies + predictions into an IntegrationTwin
    fn row_to_twin(
        &self,
        row: TwinRow,
        anomalies: Vec<TwinAnomaly>,
        predictions: Vec<TwinPrediction>,
    ) -> IntegrationTwin {
        let baseline: TwinBaseline =
            serde_json::from_value(row.baseline).unwrap_or_default();
        let current_state: TwinState =
            serde_json::from_value(row.current_state).unwrap_or_default();

        IntegrationTwin {
            id: row.id,
            tenant_id: row.tenant_id,
            integration_id: row.integration_id,
            integration_name: row.integration_name,
            status: row.status,
            baseline,
            current_state,
            anomalies,
            predictions,
            created_at: row.created_at,
            updated_at: row.updated_at,
            last_learning_at: row.last_learning_at,
        }
    }

    fn anomaly_from_row(row: AnomalyRow) -> TwinAnomaly {
        TwinAnomaly {
            id: row.id,
            twin_id: row.twin_id,
            anomaly_type: row.anomaly_type,
            severity: row.severity,
            detected_at: row.detected_at,
            expected_value: row.expected_value,
            actual_value: row.actual_value,
            deviation_sigma: row.deviation_sigma,
            message: row.message,
            resolved: row.resolved,
            resolved_at: row.resolved_at,
        }
    }

    fn prediction_from_row(row: PredictionRow) -> TwinPrediction {
        TwinPrediction {
            id: row.id,
            twin_id: row.twin_id,
            prediction_type: row.prediction_type,
            predicted_at: row.predicted_at,
            prediction_horizon_secs: row.prediction_horizon_secs,
            predicted_value: row.predicted_value,
            confidence: row.confidence,
            message: row.message,
        }
    }

    // ========================================================================
    // Methods used by the background learner
    // ========================================================================

    /// Fetch all twins in learning state that may have enough data to compute baseline
    pub async fn fetch_learning_twins(&self) -> Result<Vec<(Uuid, Uuid, Uuid)>> {
        let rows: Vec<(Uuid, Uuid, Uuid)> = sqlx::query_as(
            r#"
            SELECT id, tenant_id, integration_id
            FROM integration_twins
            WHERE status = 'learning'
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Fetch all active twins for prediction refresh
    pub async fn fetch_active_twins(&self) -> Result<Vec<(Uuid, Uuid, Uuid)>> {
        let rows: Vec<(Uuid, Uuid, Uuid)> = sqlx::query_as(
            r#"
            SELECT id, tenant_id, integration_id
            FROM integration_twins
            WHERE status IN ('active', 'degraded')
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Mark stale twins (no data in 7 days)
    pub async fn mark_stale_twins(&self) -> Result<i64> {
        let cutoff = Utc::now() - Duration::days(7);
        let result = sqlx::query(
            r#"
            UPDATE integration_twins SET
                status = 'stale',
                updated_at = NOW()
            WHERE status IN ('active', 'degraded')
              AND (current_state->>'last_run_at')::timestamptz < $1
            "#,
        )
        .bind(cutoff)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() as i64)
    }

    /// Check if an integration has enough metrics data for baseline learning
    pub async fn has_enough_data(
        &self,
        tenant_id: Uuid,
        integration_id: Uuid,
    ) -> Result<bool> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*)
            FROM metrics
            WHERE tenant_id = $1 AND resource_id = $2
            "#,
        )
        .bind(tenant_id)
        .bind(integration_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(count.0 >= 10)
    }
}

/// Internal stats result
struct MetricStatsResult {
    avg: f64,
    stddev: f64,
    min: f64,
    max: f64,
    count: i64,
}

// ============================================================================
// Background Learner
// ============================================================================

/// Background task that periodically maintains digital twins:
/// - Learns baselines for twins in learning state
/// - Updates predictions for active twins
/// - Marks stale twins
pub struct TwinLearner {
    twins: Arc<DigitalTwinService>,
    interval: StdDuration,
}

impl TwinLearner {
    pub fn new(twins: Arc<DigitalTwinService>, interval_seconds: u64) -> Self {
        Self {
            twins,
            interval: StdDuration::from_secs(interval_seconds),
        }
    }

    /// Spawn the background learning task
    pub fn start(self) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            info!(
                interval_secs = self.interval.as_secs(),
                "Digital twin learner started"
            );

            let mut ticker = time::interval(self.interval);

            loop {
                ticker.tick().await;

                match self.run_learning_cycle().await {
                    Ok((learned, refreshed, stale)) => {
                        if learned > 0 || refreshed > 0 || stale > 0 {
                            info!(
                                baselines_learned = learned,
                                predictions_refreshed = refreshed,
                                marked_stale = stale,
                                "Twin learner cycle complete"
                            );
                        }
                    }
                    Err(e) => {
                        error!(error = %e, "Twin learner cycle failed");
                    }
                }
            }
        })
    }

    async fn run_learning_cycle(&self) -> Result<(usize, usize, i64)> {
        let mut learned = 0usize;
        let mut refreshed = 0usize;

        // 1. Check learning twins for sufficient data and compute baselines
        let learning_twins = self.twins.fetch_learning_twins().await?;
        for (twin_id, tenant_id, integration_id) in &learning_twins {
            match self.twins.has_enough_data(*tenant_id, *integration_id).await {
                Ok(true) => {
                    match self.twins.learn_baseline(*tenant_id, *twin_id).await {
                        Ok(twin) => {
                            if twin.status == "active" {
                                info!(
                                    twin_id = %twin_id,
                                    integration = %twin.integration_name,
                                    "Baseline learned, twin now active"
                                );
                                learned += 1;
                            }
                        }
                        Err(e) => {
                            warn!(
                                twin_id = %twin_id,
                                error = %e,
                                "Failed to learn baseline"
                            );
                        }
                    }
                }
                Ok(false) => {
                    // Not enough data yet, skip
                }
                Err(e) => {
                    warn!(
                        twin_id = %twin_id,
                        error = %e,
                        "Failed to check data availability"
                    );
                }
            }
        }

        // 2. Refresh predictions for active twins
        let active_twins = self.twins.fetch_active_twins().await?;
        for (twin_id, tenant_id, _integration_id) in &active_twins {
            // Re-learn baseline periodically (every cycle for now, could be less frequent)
            match self.twins.learn_baseline(*tenant_id, *twin_id).await {
                Ok(_) => {
                    refreshed += 1;
                }
                Err(e) => {
                    warn!(
                        twin_id = %twin_id,
                        error = %e,
                        "Failed to refresh twin baseline"
                    );
                }
            }
        }

        // 3. Mark stale twins
        let stale_count = self.twins.mark_stale_twins().await?;
        if stale_count > 0 {
            info!(count = stale_count, "Marked twins as stale");
        }

        Ok((learned, refreshed, stale_count))
    }
}
