pub mod consumer;

use anyhow::Result;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use uuid::Uuid;

use crate::scoring::TimeQuadrant;

// ============================================================================
// Types
// ============================================================================

/// An event that can trigger TIME score recalculation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ScoreEvent {
    IntegrationAdded {
        integration_id: Uuid,
        asset_id: Uuid,
    },
    IntegrationRemoved {
        integration_id: Uuid,
        asset_id: Uuid,
    },
    IntegrationFailed {
        integration_id: Uuid,
        asset_id: Uuid,
        failure_count: i32,
    },
    ConnectorInactive {
        connection_id: Uuid,
        asset_id: Uuid,
        inactive_days: i32,
    },
    ConnectorActivated {
        connection_id: Uuid,
        asset_id: Uuid,
    },
    GovernanceViolation {
        asset_id: Uuid,
        severity: String,
        violation_id: Uuid,
    },
    GovernanceResolved {
        asset_id: Uuid,
        violation_id: Uuid,
    },
    CostChange {
        asset_id: Uuid,
        old_cost: Decimal,
        new_cost: Decimal,
    },
    UsageSpike {
        asset_id: Uuid,
        usage_multiplier: f64,
    },
    UsageDrop {
        asset_id: Uuid,
        usage_multiplier: f64,
    },
    ManualOverride {
        asset_id: Uuid,
        new_quadrant: TimeQuadrant,
        reason: String,
    },
}

impl ScoreEvent {
    /// Extract the asset_id from any event variant
    pub fn asset_id(&self) -> Uuid {
        match self {
            ScoreEvent::IntegrationAdded { asset_id, .. } => *asset_id,
            ScoreEvent::IntegrationRemoved { asset_id, .. } => *asset_id,
            ScoreEvent::IntegrationFailed { asset_id, .. } => *asset_id,
            ScoreEvent::ConnectorInactive { asset_id, .. } => *asset_id,
            ScoreEvent::ConnectorActivated { asset_id, .. } => *asset_id,
            ScoreEvent::GovernanceViolation { asset_id, .. } => *asset_id,
            ScoreEvent::GovernanceResolved { asset_id, .. } => *asset_id,
            ScoreEvent::CostChange { asset_id, .. } => *asset_id,
            ScoreEvent::UsageSpike { asset_id, .. } => *asset_id,
            ScoreEvent::UsageDrop { asset_id, .. } => *asset_id,
            ScoreEvent::ManualOverride { asset_id, .. } => *asset_id,
        }
    }

    /// Return the event type as a string for storage
    pub fn event_type(&self) -> &'static str {
        match self {
            ScoreEvent::IntegrationAdded { .. } => "integration_added",
            ScoreEvent::IntegrationRemoved { .. } => "integration_removed",
            ScoreEvent::IntegrationFailed { .. } => "integration_failed",
            ScoreEvent::ConnectorInactive { .. } => "connector_inactive",
            ScoreEvent::ConnectorActivated { .. } => "connector_activated",
            ScoreEvent::GovernanceViolation { .. } => "governance_violation",
            ScoreEvent::GovernanceResolved { .. } => "governance_resolved",
            ScoreEvent::CostChange { .. } => "cost_change",
            ScoreEvent::UsageSpike { .. } => "usage_spike",
            ScoreEvent::UsageDrop { .. } => "usage_drop",
            ScoreEvent::ManualOverride { .. } => "manual_override",
        }
    }
}

/// A drift applied to a score based on an event
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ScoreDrift {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub asset_id: Uuid,
    pub event_type: String,
    pub value_delta: Decimal,
    pub health_delta: Decimal,
    pub reason: String,
    pub applied_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub source_event: serde_json::Value,
}

/// Live TIME score with drift history
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct LiveTimeScore {
    pub asset_id: Uuid,
    pub tenant_id: Uuid,
    pub asset_name: String,

    // Base scores (from periodic analysis)
    pub base_value_score: Decimal,
    pub base_health_score: Decimal,
    pub base_quadrant: String,

    // Effective scores (base + accumulated drift)
    pub effective_value_score: Decimal,
    pub effective_health_score: Decimal,
    pub effective_quadrant: String,

    // Drift summary
    pub total_value_drift: Decimal,
    pub total_health_drift: Decimal,
    pub active_drift_count: i32,

    // Metadata
    pub last_event_at: Option<DateTime<Utc>>,
    pub quadrant_changed: bool,
    pub quadrant_change_at: Option<DateTime<Utc>>,

    pub updated_at: DateTime<Utc>,
}

/// Feed of recent score changes
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ScoreFeedEntry {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub asset_id: Uuid,
    pub asset_name: String,
    pub event_type: String,
    pub old_quadrant: String,
    pub new_quadrant: String,
    pub value_change: Decimal,
    pub health_change: Decimal,
    pub reason: String,
    pub timestamp: DateTime<Utc>,
}

/// Fleet-level rationalization summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LivePortfolioSummary {
    pub tenant_id: Uuid,
    pub total_assets: i32,
    pub quadrant_distribution: QuadrantDistribution,
    pub drifted_assets: i32,
    pub assets_trending_eliminate: i32,
    pub assets_trending_invest: i32,
    pub recent_changes: Vec<ScoreFeedEntry>,
    pub calculated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuadrantDistribution {
    pub tolerate: i32,
    pub invest: i32,
    pub migrate: i32,
    pub eliminate: i32,
}

/// Query filters for listing live scores
#[derive(Debug, Deserialize)]
pub struct LiveScoreFilters {
    pub tenant_id: Uuid,
    pub quadrant: Option<String>,
    pub drifted_only: Option<bool>,
    pub trending: Option<String>,
}

/// Query params for the score feed
#[derive(Debug, Deserialize)]
pub struct ScoreFeedQuery {
    pub tenant_id: Uuid,
    pub limit: Option<i64>,
    pub since: Option<DateTime<Utc>>,
}

// ============================================================================
// Service
// ============================================================================

#[derive(Clone)]
pub struct LiveScoringService {
    pool: PgPool,
}

impl LiveScoringService {
    /// Create a new LiveScoringService, creating the pool and required tables
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;

        let service = Self { pool };
        service.create_tables().await?;
        Ok(service)
    }

    /// Create from an existing pool
    pub fn from_pool(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Initialize tables (idempotent)
    pub async fn create_tables(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS live_time_scores (
                asset_id UUID PRIMARY KEY,
                tenant_id UUID NOT NULL,
                asset_name TEXT NOT NULL DEFAULT '',
                base_value_score DECIMAL(5,2) NOT NULL DEFAULT 5.0,
                base_health_score DECIMAL(5,2) NOT NULL DEFAULT 5.0,
                base_quadrant TEXT NOT NULL DEFAULT 'tolerate',
                effective_value_score DECIMAL(5,2) NOT NULL DEFAULT 5.0,
                effective_health_score DECIMAL(5,2) NOT NULL DEFAULT 5.0,
                effective_quadrant TEXT NOT NULL DEFAULT 'tolerate',
                total_value_drift DECIMAL(5,2) NOT NULL DEFAULT 0.0,
                total_health_drift DECIMAL(5,2) NOT NULL DEFAULT 0.0,
                active_drift_count INTEGER NOT NULL DEFAULT 0,
                last_event_at TIMESTAMPTZ,
                quadrant_changed BOOLEAN NOT NULL DEFAULT false,
                quadrant_change_at TIMESTAMPTZ,
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS score_drifts (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                tenant_id UUID NOT NULL,
                asset_id UUID NOT NULL,
                event_type TEXT NOT NULL,
                value_delta DECIMAL(5,2) NOT NULL DEFAULT 0.0,
                health_delta DECIMAL(5,2) NOT NULL DEFAULT 0.0,
                reason TEXT NOT NULL DEFAULT '',
                applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                expires_at TIMESTAMPTZ,
                source_event JSONB NOT NULL DEFAULT '{}',
                CONSTRAINT fk_score_drifts_asset FOREIGN KEY (asset_id) REFERENCES live_time_scores(asset_id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS score_feed (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                tenant_id UUID NOT NULL,
                asset_id UUID NOT NULL,
                asset_name TEXT NOT NULL DEFAULT '',
                event_type TEXT NOT NULL,
                old_quadrant TEXT NOT NULL,
                new_quadrant TEXT NOT NULL,
                value_change DECIMAL(5,2) NOT NULL DEFAULT 0.0,
                health_change DECIMAL(5,2) NOT NULL DEFAULT 0.0,
                reason TEXT NOT NULL DEFAULT '',
                timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create indexes for efficient querying
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_live_time_scores_tenant ON live_time_scores(tenant_id);
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_score_drifts_asset ON score_drifts(asset_id, applied_at DESC);
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_score_drifts_expires ON score_drifts(expires_at) WHERE expires_at IS NOT NULL;
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_score_feed_tenant ON score_feed(tenant_id, timestamp DESC);
            "#,
        )
        .execute(&self.pool)
        .await?;

        tracing::info!("Live scoring tables initialized");
        Ok(())
    }

    // ========================================================================
    // Event Processing
    // ========================================================================

    /// Process a score event, calculating drift and updating effective scores
    pub async fn process_event(&self, tenant_id: Uuid, event: ScoreEvent) -> Result<LiveTimeScore> {
        let asset_id = event.asset_id();
        let event_type = event.event_type().to_string();

        // Ensure the live score record exists (seed from time_assessments if available)
        self.ensure_live_score_exists(tenant_id, asset_id).await?;

        // Get current live score before applying drift
        let current = self.get_live_score(tenant_id, asset_id).await?;
        let old_quadrant = current.effective_quadrant.clone();

        // Handle manual override specially
        if let ScoreEvent::ManualOverride {
            new_quadrant,
            reason,
            ..
        } = &event
        {
            return self
                .apply_manual_override(tenant_id, asset_id, new_quadrant, reason, &event)
                .await;
        }

        // Calculate drift deltas from the event
        let (value_delta, health_delta, reason, expires_at) =
            self.calculate_drift(&event);

        // Create the drift record
        let source_event = serde_json::to_value(&event)?;
        sqlx::query(
            r#"
            INSERT INTO score_drifts (tenant_id, asset_id, event_type, value_delta, health_delta, reason, expires_at, source_event)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(tenant_id)
        .bind(asset_id)
        .bind(&event_type)
        .bind(value_delta)
        .bind(health_delta)
        .bind(&reason)
        .bind(expires_at)
        .bind(&source_event)
        .execute(&self.pool)
        .await?;

        // Recalculate effective scores from all active drifts
        self.recalculate_effective_scores(tenant_id, asset_id)
            .await?;

        // Reload the updated score
        let updated = self.get_live_score(tenant_id, asset_id).await?;

        // If quadrant changed, create a feed entry
        if updated.effective_quadrant != old_quadrant {
            self.create_feed_entry(
                tenant_id,
                asset_id,
                &updated.asset_name,
                &event_type,
                &old_quadrant,
                &updated.effective_quadrant,
                value_delta,
                health_delta,
                &reason,
            )
            .await?;

            // Mark quadrant as changed on the live score
            sqlx::query(
                r#"
                UPDATE live_time_scores
                SET quadrant_changed = true, quadrant_change_at = NOW()
                WHERE asset_id = $1 AND tenant_id = $2
                "#,
            )
            .bind(asset_id)
            .bind(tenant_id)
            .execute(&self.pool)
            .await?;

            // If drifted to Eliminate, log as notable event
            if updated.effective_quadrant == "eliminate" {
                tracing::warn!(
                    "Asset {} ({}) has drifted to Eliminate quadrant",
                    updated.asset_name,
                    asset_id
                );
                self.create_feed_entry(
                    tenant_id,
                    asset_id,
                    &updated.asset_name,
                    "trending_eliminate",
                    &old_quadrant,
                    "eliminate",
                    value_delta,
                    health_delta,
                    &format!(
                        "Asset has drifted to Eliminate quadrant from {}. Immediate attention recommended.",
                        old_quadrant
                    ),
                )
                .await?;
            }
        }

        // Return the final state
        self.get_live_score(tenant_id, asset_id).await
    }

    /// Calculate drift deltas from an event
    fn calculate_drift(
        &self,
        event: &ScoreEvent,
    ) -> (Decimal, Decimal, String, Option<DateTime<Utc>>) {
        match event {
            ScoreEvent::IntegrationAdded { .. } => (
                Decimal::new(5, 1), // +0.5 value
                Decimal::ZERO,
                "New integration added increases asset value".to_string(),
                None,
            ),
            ScoreEvent::IntegrationRemoved { .. } => (
                Decimal::new(-3, 1), // -0.3 value
                Decimal::ZERO,
                "Integration removed decreases asset value".to_string(),
                None,
            ),
            ScoreEvent::IntegrationFailed { failure_count, .. } => {
                // -0.5 per failure, max -3.0
                let delta = std::cmp::min(*failure_count, 6) as i64 * -5;
                let health_delta = Decimal::new(delta, 1);
                let expires_at = Utc::now() + chrono::Duration::days(7);
                (
                    Decimal::ZERO,
                    health_delta,
                    format!(
                        "Integration failures ({}) degrading health score",
                        failure_count
                    ),
                    Some(expires_at),
                )
            }
            ScoreEvent::ConnectorInactive { inactive_days, .. } => {
                // health -0.2 per 10 inactive days, value -0.1 per 30 days
                let health_delta = Decimal::new(-2, 1) * Decimal::from(*inactive_days / 10);
                let value_delta = Decimal::new(-1, 1) * Decimal::from(*inactive_days / 30);
                (
                    value_delta,
                    health_delta,
                    format!(
                        "Connector inactive for {} days, degrading scores",
                        inactive_days
                    ),
                    None, // Expires when connector becomes active
                )
            }
            ScoreEvent::ConnectorActivated { .. } => (
                Decimal::ZERO,
                Decimal::new(5, 1), // +0.5 health
                "Connector activated, health improving".to_string(),
                None,
            ),
            ScoreEvent::GovernanceViolation { severity, .. } => {
                let health_delta = match severity.to_lowercase().as_str() {
                    "critical" => Decimal::new(-10, 1), // -1.0
                    "high" => Decimal::new(-5, 1),      // -0.5
                    "medium" => Decimal::new(-2, 1),    // -0.2
                    _ => Decimal::new(-1, 1),           // -0.1 for low/unknown
                };
                (
                    Decimal::ZERO,
                    health_delta,
                    format!("Governance violation ({}) impacting health", severity),
                    None, // Expires when resolved
                )
            }
            ScoreEvent::GovernanceResolved { .. } => (
                Decimal::ZERO,
                Decimal::new(3, 1), // +0.3 health
                "Governance violation resolved, health recovering".to_string(),
                None,
            ),
            ScoreEvent::CostChange {
                old_cost, new_cost, ..
            } => {
                if *old_cost == Decimal::ZERO {
                    return (
                        Decimal::ZERO,
                        Decimal::ZERO,
                        "Cost changed from zero, no drift applied".to_string(),
                        None,
                    );
                }
                let cost_ratio = (*new_cost - *old_cost) / *old_cost;
                let value_delta = if cost_ratio > Decimal::ZERO {
                    // Cost increased: negative impact on value
                    // -(new-old)/old * 2.0, clamped
                    let raw = -cost_ratio * Decimal::new(20, 1);
                    std::cmp::max(raw, Decimal::new(-100, 1)) // clamp to -10.0
                } else {
                    // Cost decreased: positive
                    Decimal::new(5, 1) // +0.5
                };
                let direction = if cost_ratio > Decimal::ZERO {
                    "increased"
                } else {
                    "decreased"
                };
                (
                    value_delta,
                    Decimal::ZERO,
                    format!("Cost {} affecting value score", direction),
                    None,
                )
            }
            ScoreEvent::UsageSpike { .. } => (
                Decimal::new(10, 1), // +1.0 value
                Decimal::ZERO,
                "Usage spike indicates increased demand and value".to_string(),
                None,
            ),
            ScoreEvent::UsageDrop {
                usage_multiplier, ..
            } => {
                // -0.5 per 50% drop. usage_multiplier < 1.0 means a drop.
                // e.g., multiplier=0.5 means 50% drop => -0.5
                // multiplier=0.25 means 75% drop => -0.75
                let drop_pct = 1.0 - usage_multiplier;
                let raw_delta = -(drop_pct / 0.5) * 0.5;
                let value_delta =
                    Decimal::from_f64_retain(raw_delta).unwrap_or(Decimal::new(-5, 1));
                (
                    value_delta,
                    Decimal::ZERO,
                    format!(
                        "Usage dropped by {:.0}%, reducing value score",
                        drop_pct * 100.0
                    ),
                    None,
                )
            }
            ScoreEvent::ManualOverride { .. } => {
                // Handled separately in process_event
                (Decimal::ZERO, Decimal::ZERO, String::new(), None)
            }
        }
    }

    /// Apply a manual override, setting the quadrant directly
    async fn apply_manual_override(
        &self,
        tenant_id: Uuid,
        asset_id: Uuid,
        new_quadrant: &TimeQuadrant,
        reason: &str,
        event: &ScoreEvent,
    ) -> Result<LiveTimeScore> {
        let current = self.get_live_score(tenant_id, asset_id).await?;
        let old_quadrant = current.effective_quadrant.clone();
        let quadrant_str = new_quadrant.as_str();

        // Determine scores that correspond to the target quadrant center
        let (target_value, target_health) = match new_quadrant {
            TimeQuadrant::Invest => (Decimal::new(75, 1), Decimal::new(75, 1)),
            TimeQuadrant::Tolerate => (Decimal::new(25, 1), Decimal::new(75, 1)),
            TimeQuadrant::Migrate => (Decimal::new(75, 1), Decimal::new(25, 1)),
            TimeQuadrant::Eliminate => (Decimal::new(25, 1), Decimal::new(25, 1)),
        };

        // Record the override as a drift
        let source_event = serde_json::to_value(event)?;
        sqlx::query(
            r#"
            INSERT INTO score_drifts (tenant_id, asset_id, event_type, value_delta, health_delta, reason, source_event)
            VALUES ($1, $2, 'manual_override', $3, $4, $5, $6)
            "#,
        )
        .bind(tenant_id)
        .bind(asset_id)
        .bind(target_value - current.base_value_score)
        .bind(target_health - current.base_health_score)
        .bind(reason)
        .bind(&source_event)
        .execute(&self.pool)
        .await?;

        // Directly set the effective scores
        sqlx::query(
            r#"
            UPDATE live_time_scores SET
                effective_value_score = $1,
                effective_health_score = $2,
                effective_quadrant = $3,
                total_value_drift = $1 - base_value_score,
                total_health_drift = $2 - base_health_score,
                quadrant_changed = ($3 != base_quadrant),
                quadrant_change_at = CASE WHEN $3 != base_quadrant THEN NOW() ELSE quadrant_change_at END,
                last_event_at = NOW(),
                updated_at = NOW()
            WHERE asset_id = $4 AND tenant_id = $5
            "#,
        )
        .bind(target_value)
        .bind(target_health)
        .bind(quadrant_str)
        .bind(asset_id)
        .bind(tenant_id)
        .execute(&self.pool)
        .await?;

        // Create feed entry if quadrant changed
        if old_quadrant != quadrant_str {
            self.create_feed_entry(
                tenant_id,
                asset_id,
                &current.asset_name,
                "manual_override",
                &old_quadrant,
                quadrant_str,
                target_value - current.effective_value_score,
                target_health - current.effective_health_score,
                reason,
            )
            .await?;
        }

        self.get_live_score(tenant_id, asset_id).await
    }

    /// Ensure a live score record exists for an asset, seeding from time_assessments if possible
    async fn ensure_live_score_exists(&self, tenant_id: Uuid, asset_id: Uuid) -> Result<()> {
        let exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM live_time_scores WHERE asset_id = $1 AND tenant_id = $2)",
        )
        .bind(asset_id)
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await?;

        if exists {
            return Ok(());
        }

        // Try to seed from existing time assessment
        let assessment = sqlx::query_as::<_, (Option<Decimal>, Option<Decimal>, String)>(
            r#"
            SELECT business_value_score, technical_health_score, quadrant
            FROM time_assessments
            WHERE application_id = $1 AND tenant_id = $2
            "#,
        )
        .bind(asset_id)
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await?;

        // Try to get the asset name from applications table
        let asset_name = sqlx::query_scalar::<_, String>(
            "SELECT name FROM applications WHERE id = $1 AND tenant_id = $2",
        )
        .bind(asset_id)
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await?
        .unwrap_or_else(|| format!("Asset {}", asset_id));

        let (base_value, base_health, base_quadrant) = match assessment {
            Some((value, health, quadrant)) => (
                value.unwrap_or(Decimal::new(50, 1)),
                health.unwrap_or(Decimal::new(50, 1)),
                quadrant,
            ),
            None => (
                Decimal::new(50, 1),
                Decimal::new(50, 1),
                "tolerate".to_string(),
            ),
        };

        sqlx::query(
            r#"
            INSERT INTO live_time_scores (
                asset_id, tenant_id, asset_name,
                base_value_score, base_health_score, base_quadrant,
                effective_value_score, effective_health_score, effective_quadrant,
                total_value_drift, total_health_drift, active_drift_count,
                updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $4, $5, $6, 0, 0, 0, NOW())
            ON CONFLICT (asset_id) DO NOTHING
            "#,
        )
        .bind(asset_id)
        .bind(tenant_id)
        .bind(&asset_name)
        .bind(base_value)
        .bind(base_health)
        .bind(&base_quadrant)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Create a feed entry for a score change
    async fn create_feed_entry(
        &self,
        tenant_id: Uuid,
        asset_id: Uuid,
        asset_name: &str,
        event_type: &str,
        old_quadrant: &str,
        new_quadrant: &str,
        value_change: Decimal,
        health_change: Decimal,
        reason: &str,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO score_feed (tenant_id, asset_id, asset_name, event_type, old_quadrant, new_quadrant, value_change, health_change, reason)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(tenant_id)
        .bind(asset_id)
        .bind(asset_name)
        .bind(event_type)
        .bind(old_quadrant)
        .bind(new_quadrant)
        .bind(value_change)
        .bind(health_change)
        .bind(reason)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // ========================================================================
    // Queries
    // ========================================================================

    /// Get the live score for a single asset
    pub async fn get_live_score(
        &self,
        tenant_id: Uuid,
        asset_id: Uuid,
    ) -> Result<LiveTimeScore> {
        let score = sqlx::query_as::<_, LiveTimeScore>(
            r#"
            SELECT asset_id, tenant_id, asset_name,
                   base_value_score, base_health_score, base_quadrant,
                   effective_value_score, effective_health_score, effective_quadrant,
                   total_value_drift, total_health_drift, active_drift_count,
                   last_event_at, quadrant_changed, quadrant_change_at, updated_at
            FROM live_time_scores
            WHERE tenant_id = $1 AND asset_id = $2
            "#,
        )
        .bind(tenant_id)
        .bind(asset_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Live score not found for asset {}", asset_id))?;

        Ok(score)
    }

    /// List live scores with optional filters
    pub async fn get_live_scores(
        &self,
        filters: LiveScoreFilters,
    ) -> Result<Vec<LiveTimeScore>> {
        let scores = sqlx::query_as::<_, LiveTimeScore>(
            r#"
            SELECT asset_id, tenant_id, asset_name,
                   base_value_score, base_health_score, base_quadrant,
                   effective_value_score, effective_health_score, effective_quadrant,
                   total_value_drift, total_health_drift, active_drift_count,
                   last_event_at, quadrant_changed, quadrant_change_at, updated_at
            FROM live_time_scores
            WHERE tenant_id = $1
              AND ($2::text IS NULL OR effective_quadrant = $2)
              AND ($3::boolean IS NULL OR ($3 = true AND quadrant_changed = true))
              AND ($4::text IS NULL OR (
                  CASE
                      WHEN $4 = 'eliminate' THEN (total_value_drift < 0 AND total_health_drift < 0)
                      WHEN $4 = 'invest' THEN (total_value_drift > 0 AND total_health_drift > 0)
                      WHEN $4 = 'migrate' THEN (total_value_drift > 0 AND total_health_drift < 0)
                      WHEN $4 = 'tolerate' THEN (total_value_drift < 0 AND total_health_drift > 0)
                      ELSE true
                  END
              ))
            ORDER BY updated_at DESC
            "#,
        )
        .bind(filters.tenant_id)
        .bind(filters.quadrant.as_deref())
        .bind(filters.drifted_only)
        .bind(filters.trending.as_deref())
        .fetch_all(&self.pool)
        .await?;

        Ok(scores)
    }

    /// Get the recent score change feed
    pub async fn get_score_feed(
        &self,
        tenant_id: Uuid,
        limit: i64,
        since: Option<DateTime<Utc>>,
    ) -> Result<Vec<ScoreFeedEntry>> {
        let entries = sqlx::query_as::<_, ScoreFeedEntry>(
            r#"
            SELECT id, tenant_id, asset_id, asset_name, event_type,
                   old_quadrant, new_quadrant, value_change, health_change,
                   reason, timestamp
            FROM score_feed
            WHERE tenant_id = $1
              AND ($2::timestamptz IS NULL OR timestamp > $2)
            ORDER BY timestamp DESC
            LIMIT $3
            "#,
        )
        .bind(tenant_id)
        .bind(since)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(entries)
    }

    /// Get fleet-level portfolio summary
    pub async fn get_portfolio_summary(
        &self,
        tenant_id: Uuid,
    ) -> Result<LivePortfolioSummary> {
        let total_assets = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM live_time_scores WHERE tenant_id = $1",
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await? as i32;

        // Quadrant distribution
        let dist_rows = sqlx::query_as::<_, (String, i64)>(
            r#"
            SELECT effective_quadrant, COUNT(*)
            FROM live_time_scores
            WHERE tenant_id = $1
            GROUP BY effective_quadrant
            "#,
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await?;

        let mut distribution = QuadrantDistribution {
            tolerate: 0,
            invest: 0,
            migrate: 0,
            eliminate: 0,
        };
        for (quadrant, count) in &dist_rows {
            match quadrant.as_str() {
                "tolerate" => distribution.tolerate = *count as i32,
                "invest" => distribution.invest = *count as i32,
                "migrate" => distribution.migrate = *count as i32,
                "eliminate" => distribution.eliminate = *count as i32,
                _ => {}
            }
        }

        let drifted_assets = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM live_time_scores WHERE tenant_id = $1 AND quadrant_changed = true",
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await? as i32;

        // Assets trending toward Eliminate (both value and health drifting negative)
        let assets_trending_eliminate = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*) FROM live_time_scores
            WHERE tenant_id = $1 AND total_value_drift < 0 AND total_health_drift < 0
            "#,
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await? as i32;

        // Assets trending toward Invest (both value and health drifting positive)
        let assets_trending_invest = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*) FROM live_time_scores
            WHERE tenant_id = $1 AND total_value_drift > 0 AND total_health_drift > 0
            "#,
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await? as i32;

        let recent_changes = self.get_score_feed(tenant_id, 10, None).await?;

        Ok(LivePortfolioSummary {
            tenant_id,
            total_assets,
            quadrant_distribution: distribution,
            drifted_assets,
            assets_trending_eliminate,
            assets_trending_invest,
            recent_changes,
            calculated_at: Utc::now(),
        })
    }

    /// Get drift history for a specific asset
    pub async fn get_drift_history(
        &self,
        tenant_id: Uuid,
        asset_id: Uuid,
    ) -> Result<Vec<ScoreDrift>> {
        let drifts = sqlx::query_as::<_, ScoreDrift>(
            r#"
            SELECT id, tenant_id, asset_id, event_type, value_delta, health_delta,
                   reason, applied_at, expires_at, source_event
            FROM score_drifts
            WHERE tenant_id = $1 AND asset_id = $2
            ORDER BY applied_at DESC
            "#,
        )
        .bind(tenant_id)
        .bind(asset_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(drifts)
    }

    /// Recalculate effective scores from all active (non-expired) drifts
    pub async fn recalculate_effective_scores(
        &self,
        tenant_id: Uuid,
        asset_id: Uuid,
    ) -> Result<()> {
        // Sum all active drifts (non-expired)
        let drift_sums = sqlx::query_as::<_, (Decimal, Decimal, i64)>(
            r#"
            SELECT
                COALESCE(SUM(value_delta), 0),
                COALESCE(SUM(health_delta), 0),
                COUNT(*)
            FROM score_drifts
            WHERE tenant_id = $1
              AND asset_id = $2
              AND (expires_at IS NULL OR expires_at > NOW())
            "#,
        )
        .bind(tenant_id)
        .bind(asset_id)
        .fetch_one(&self.pool)
        .await?;

        let (total_value_drift, total_health_drift, active_count) = drift_sums;

        // Clamp effective scores to 0-10 range and recompute quadrant
        sqlx::query(
            r#"
            UPDATE live_time_scores SET
                total_value_drift = $1,
                total_health_drift = $2,
                active_drift_count = $3,
                effective_value_score = GREATEST(0, LEAST(10, base_value_score + $1)),
                effective_health_score = GREATEST(0, LEAST(10, base_health_score + $2)),
                effective_quadrant = CASE
                    WHEN GREATEST(0, LEAST(10, base_value_score + $1)) >= 5.0 THEN
                        CASE WHEN GREATEST(0, LEAST(10, base_health_score + $2)) >= 5.0 THEN 'invest' ELSE 'migrate' END
                    ELSE
                        CASE WHEN GREATEST(0, LEAST(10, base_health_score + $2)) >= 5.0 THEN 'tolerate' ELSE 'eliminate' END
                END,
                quadrant_changed = (
                    CASE
                        WHEN GREATEST(0, LEAST(10, base_value_score + $1)) >= 5.0 THEN
                            CASE WHEN GREATEST(0, LEAST(10, base_health_score + $2)) >= 5.0 THEN 'invest' ELSE 'migrate' END
                        ELSE
                            CASE WHEN GREATEST(0, LEAST(10, base_health_score + $2)) >= 5.0 THEN 'tolerate' ELSE 'eliminate' END
                    END
                ) != base_quadrant,
                last_event_at = NOW(),
                updated_at = NOW()
            WHERE asset_id = $4 AND tenant_id = $5
            "#,
        )
        .bind(total_value_drift)
        .bind(total_health_drift)
        .bind(active_count as i32)
        .bind(asset_id)
        .bind(tenant_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // ========================================================================
    // Drift Expiry & Background Tasks
    // ========================================================================

    /// Clean up expired drifts and recalculate affected assets
    pub async fn cleanup_expired_drifts(&self) -> Result<Vec<(Uuid, Uuid)>> {
        // Find assets with expired drifts before deleting
        let affected = sqlx::query_as::<_, (Uuid, Uuid)>(
            r#"
            SELECT DISTINCT tenant_id, asset_id
            FROM score_drifts
            WHERE expires_at IS NOT NULL AND expires_at <= NOW()
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        if affected.is_empty() {
            return Ok(affected);
        }

        // Delete expired drifts
        let deleted = sqlx::query(
            r#"
            DELETE FROM score_drifts
            WHERE expires_at IS NOT NULL AND expires_at <= NOW()
            "#,
        )
        .execute(&self.pool)
        .await?;

        tracing::info!(
            "Cleaned up {} expired drifts across {} assets",
            deleted.rows_affected(),
            affected.len()
        );

        // Recalculate effective scores for all affected assets
        for (tenant_id, asset_id) in &affected {
            if let Err(e) = self
                .recalculate_effective_scores(*tenant_id, *asset_id)
                .await
            {
                tracing::error!(
                    "Failed to recalculate scores for asset {}: {}",
                    asset_id,
                    e
                );
            }
        }

        Ok(affected)
    }

    /// Detect assets trending toward Eliminate and generate feed entries
    pub async fn detect_trending_eliminate(&self) -> Result<()> {
        // Assets where both value and health are drifting negative and haven't been
        // flagged as trending_eliminate in the feed within the last hour
        let trending = sqlx::query_as::<_, (Uuid, Uuid, String, String)>(
            r#"
            SELECT lts.tenant_id, lts.asset_id, lts.asset_name, lts.effective_quadrant
            FROM live_time_scores lts
            WHERE lts.total_value_drift < -1.0
              AND lts.total_health_drift < -1.0
              AND lts.effective_quadrant != 'eliminate'
              AND NOT EXISTS (
                  SELECT 1 FROM score_feed sf
                  WHERE sf.asset_id = lts.asset_id
                    AND sf.event_type = 'trending_eliminate'
                    AND sf.timestamp > NOW() - INTERVAL '1 hour'
              )
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        for (tenant_id, asset_id, asset_name, current_quadrant) in trending {
            self.create_feed_entry(
                tenant_id,
                asset_id,
                &asset_name,
                "trending_eliminate",
                &current_quadrant,
                &current_quadrant,
                Decimal::ZERO,
                Decimal::ZERO,
                "Asset is trending toward Eliminate quadrant based on accumulated negative drift",
            )
            .await?;

            tracing::info!(
                "Asset {} ({}) is trending toward Eliminate",
                asset_name,
                asset_id
            );
        }

        Ok(())
    }
}

// ============================================================================
// Background Task
// ============================================================================

/// Spawn the background task that periodically cleans up expired drifts
/// and detects assets trending toward Eliminate
pub fn spawn_background_task(service: LiveScoringService) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(300)); // 5 minutes
        loop {
            interval.tick().await;

            tracing::debug!("Running live scoring background task");

            // Clean up expired drifts
            match service.cleanup_expired_drifts().await {
                Ok(affected) => {
                    if !affected.is_empty() {
                        tracing::info!(
                            "Background task: recalculated {} assets after drift expiry",
                            affected.len()
                        );
                    }
                }
                Err(e) => {
                    tracing::error!("Background task: failed to cleanup expired drifts: {}", e);
                }
            }

            // Detect trending
            if let Err(e) = service.detect_trending_eliminate().await {
                tracing::error!(
                    "Background task: failed to detect trending eliminate: {}",
                    e
                );
            }
        }
    })
}
