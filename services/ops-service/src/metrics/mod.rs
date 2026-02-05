use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, postgres::PgPoolOptions};
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};

/// A metric data point
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Metric {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub resource_type: String,
    pub resource_id: Uuid,
    pub metric_name: String,
    pub metric_value: f64,
    pub tags: Option<serde_json::Value>,
    pub recorded_at: DateTime<Utc>,
}

/// Request to ingest metrics
#[derive(Debug, Clone, Deserialize)]
pub struct IngestMetricsRequest {
    pub metrics: Vec<MetricInput>,
}

/// A single metric input for ingestion
#[derive(Debug, Clone, Deserialize)]
pub struct MetricInput {
    pub resource_type: String,
    pub resource_id: Uuid,
    pub metric_name: String,
    pub metric_value: f64,
    pub tags: Option<serde_json::Value>,
    pub recorded_at: Option<DateTime<Utc>>,
}

/// Aggregation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregationResult {
    pub metric_name: String,
    pub resource_type: Option<String>,
    pub resource_id: Option<Uuid>,
    pub min_value: f64,
    pub max_value: f64,
    pub avg_value: f64,
    pub count: i64,
    pub bucket_start: DateTime<Utc>,
}

/// Time bucket for aggregations
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimeBucket {
    Minute,
    Hour,
    Day,
}

impl TimeBucket {
    pub fn to_interval(&self) -> &'static str {
        match self {
            TimeBucket::Minute => "1 minute",
            TimeBucket::Hour => "1 hour",
            TimeBucket::Day => "1 day",
        }
    }
}

/// Service for managing metrics
pub struct MetricsService {
    pool: PgPool,
}

impl MetricsService {
    /// Create a new metrics service
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;

        Ok(Self { pool })
    }

    /// Health check
    pub async fn health_check(&self) -> Result<()> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Ingest a batch of metrics
    pub async fn ingest_metrics(
        &self,
        tenant_id: Uuid,
        metrics: Vec<MetricInput>,
    ) -> Result<i64> {
        let mut count = 0i64;

        for metric in metrics {
            let recorded_at = metric.recorded_at.unwrap_or_else(Utc::now);

            sqlx::query(
                r#"
                INSERT INTO metrics (tenant_id, resource_type, resource_id, metric_name, metric_value, tags, recorded_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                "#
            )
            .bind(tenant_id)
            .bind(&metric.resource_type)
            .bind(metric.resource_id)
            .bind(&metric.metric_name)
            .bind(metric.metric_value)
            .bind(&metric.tags)
            .bind(recorded_at)
            .execute(&self.pool)
            .await?;

            count += 1;
        }

        Ok(count)
    }

    /// Query metrics with filters
    pub async fn query_metrics(
        &self,
        tenant_id: Uuid,
        resource_type: Option<String>,
        resource_id: Option<Uuid>,
        metric_name: Option<String>,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
        limit: i64,
    ) -> Result<Vec<Metric>> {
        let start = start_time.unwrap_or_else(|| Utc::now() - Duration::hours(1));
        let end = end_time.unwrap_or_else(Utc::now);

        let metrics = sqlx::query_as::<_, Metric>(
            r#"
            SELECT id, tenant_id, resource_type, resource_id, metric_name, metric_value, tags, recorded_at
            FROM metrics
            WHERE tenant_id = $1
              AND recorded_at >= $2
              AND recorded_at <= $3
              AND ($4::text IS NULL OR resource_type = $4)
              AND ($5::uuid IS NULL OR resource_id = $5)
              AND ($6::text IS NULL OR metric_name = $6)
            ORDER BY recorded_at DESC
            LIMIT $7
            "#
        )
        .bind(tenant_id)
        .bind(start)
        .bind(end)
        .bind(resource_type)
        .bind(resource_id)
        .bind(metric_name)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(metrics)
    }

    /// Get aggregated metrics
    pub async fn get_aggregations(
        &self,
        tenant_id: Uuid,
        metric_name: String,
        resource_type: Option<String>,
        resource_id: Option<Uuid>,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
        bucket: TimeBucket,
    ) -> Result<Vec<AggregationResult>> {
        let start = start_time.unwrap_or_else(|| Utc::now() - Duration::hours(24));
        let end = end_time.unwrap_or_else(Utc::now);
        let interval = bucket.to_interval();

        // Build dynamic query based on grouping
        let (group_by_resource_type, group_by_resource_id) = match (&resource_type, &resource_id) {
            (None, None) => (false, false),
            (Some(_), None) => (true, false),
            (Some(_), Some(_)) => (true, true),
            (None, Some(_)) => (false, true),
        };

        let results = sqlx::query_as::<_, (String, Option<String>, Option<Uuid>, f64, f64, f64, i64, DateTime<Utc>)>(
            r#"
            SELECT
                metric_name,
                CASE WHEN $7 THEN resource_type ELSE NULL END as rt,
                CASE WHEN $8 THEN resource_id ELSE NULL END as rid,
                MIN(metric_value) as min_val,
                MAX(metric_value) as max_val,
                AVG(metric_value) as avg_val,
                COUNT(*) as cnt,
                date_trunc($9, recorded_at) as bucket_start
            FROM metrics
            WHERE tenant_id = $1
              AND metric_name = $2
              AND recorded_at >= $3
              AND recorded_at <= $4
              AND ($5::text IS NULL OR resource_type = $5)
              AND ($6::uuid IS NULL OR resource_id = $6)
            GROUP BY metric_name,
                     CASE WHEN $7 THEN resource_type ELSE NULL END,
                     CASE WHEN $8 THEN resource_id ELSE NULL END,
                     date_trunc($9, recorded_at)
            ORDER BY bucket_start
            "#
        )
        .bind(tenant_id)
        .bind(&metric_name)
        .bind(start)
        .bind(end)
        .bind(&resource_type)
        .bind(resource_id)
        .bind(group_by_resource_type)
        .bind(group_by_resource_id)
        .bind(interval)
        .fetch_all(&self.pool)
        .await?;

        let aggregations = results
            .into_iter()
            .map(|(name, rt, rid, min_val, max_val, avg_val, cnt, bucket)| {
                AggregationResult {
                    metric_name: name,
                    resource_type: rt,
                    resource_id: rid,
                    min_value: min_val,
                    max_value: max_val,
                    avg_value: avg_val,
                    count: cnt,
                    bucket_start: bucket,
                }
            })
            .collect();

        Ok(aggregations)
    }

    /// Get the latest value for a metric
    pub async fn get_latest_value(
        &self,
        tenant_id: Uuid,
        resource_type: &str,
        resource_id: Uuid,
        metric_name: &str,
    ) -> Result<Option<f64>> {
        let result: Option<(f64,)> = sqlx::query_as(
            r#"
            SELECT metric_value
            FROM metrics
            WHERE tenant_id = $1
              AND resource_type = $2
              AND resource_id = $3
              AND metric_name = $4
            ORDER BY recorded_at DESC
            LIMIT 1
            "#
        )
        .bind(tenant_id)
        .bind(resource_type)
        .bind(resource_id)
        .bind(metric_name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.map(|r| r.0))
    }

    /// Clean up old metrics (retention)
    pub async fn cleanup_old_metrics(
        &self,
        tenant_id: Uuid,
        retention_days: i64,
    ) -> Result<i64> {
        let cutoff = Utc::now() - Duration::days(retention_days);

        let result = sqlx::query(
            "DELETE FROM metrics WHERE tenant_id = $1 AND recorded_at < $2"
        )
        .bind(tenant_id)
        .bind(cutoff)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() as i64)
    }
}
