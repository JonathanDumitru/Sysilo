use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, postgres::PgPoolOptions};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use sha2::{Sha256, Digest};

/// Actor type for audit entries
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActorType {
    User,
    System,
    Agent,
    Service,
    Anonymous,
}

impl ActorType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ActorType::User => "user",
            ActorType::System => "system",
            ActorType::Agent => "agent",
            ActorType::Service => "service",
            ActorType::Anonymous => "anonymous",
        }
    }
}

/// An immutable audit log entry
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AuditEntry {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub actor_id: Option<Uuid>,
    pub actor_type: String,
    pub actor_name: Option<String>,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<Uuid>,
    pub resource_name: Option<String>,
    pub before_state: Option<serde_json::Value>,
    pub after_state: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub hash: String,
}

/// Request to create an audit entry
#[derive(Debug, Clone, Deserialize)]
pub struct CreateAuditEntryRequest {
    pub actor_id: Option<Uuid>,
    pub actor_type: ActorType,
    pub actor_name: Option<String>,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<Uuid>,
    pub resource_name: Option<String>,
    pub before_state: Option<serde_json::Value>,
    pub after_state: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

/// Query parameters for audit log
#[derive(Debug, Clone, Deserialize)]
pub struct AuditQueryParams {
    pub actor_id: Option<Uuid>,
    pub actor_type: Option<String>,
    pub action: Option<String>,
    pub resource_type: Option<String>,
    pub resource_id: Option<Uuid>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Audit statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditStats {
    pub total_entries: i64,
    pub entries_today: i64,
    pub unique_actors: i64,
    pub top_actions: Vec<ActionCount>,
    pub top_resources: Vec<ResourceCount>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionCount {
    pub action: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceCount {
    pub resource_type: String,
    pub count: i64,
}

/// Verification result for hash chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainVerificationResult {
    pub valid: bool,
    pub entries_checked: i64,
    pub first_invalid_id: Option<Uuid>,
    pub error: Option<String>,
}

/// Service for managing audit logs
pub struct AuditService {
    pool: PgPool,
}

impl AuditService {
    /// Create a new audit service
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

    /// Create an audit entry with hash chain
    pub async fn log(
        &self,
        tenant_id: Uuid,
        req: CreateAuditEntryRequest,
    ) -> Result<AuditEntry> {
        // Get the hash of the previous entry for this tenant
        let prev_hash = self.get_last_hash(tenant_id).await?;

        // Generate hash for this entry
        let entry_id = Uuid::new_v4();
        let timestamp = Utc::now();
        let hash = self.compute_hash(
            &entry_id,
            tenant_id,
            &req,
            &timestamp,
            &prev_hash,
        );

        let entry = sqlx::query_as::<_, AuditEntry>(
            r#"
            INSERT INTO audit_log
                (id, tenant_id, actor_id, actor_type, actor_name, action,
                 resource_type, resource_id, resource_name, before_state,
                 after_state, metadata, ip_address, user_agent, timestamp, hash)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
            RETURNING id, tenant_id, actor_id, actor_type, actor_name, action,
                      resource_type, resource_id, resource_name, before_state,
                      after_state, metadata, ip_address, user_agent, timestamp, hash
            "#
        )
        .bind(entry_id)
        .bind(tenant_id)
        .bind(req.actor_id)
        .bind(req.actor_type.as_str())
        .bind(&req.actor_name)
        .bind(&req.action)
        .bind(&req.resource_type)
        .bind(req.resource_id)
        .bind(&req.resource_name)
        .bind(&req.before_state)
        .bind(&req.after_state)
        .bind(&req.metadata)
        .bind(&req.ip_address)
        .bind(&req.user_agent)
        .bind(timestamp)
        .bind(&hash)
        .fetch_one(&self.pool)
        .await?;

        Ok(entry)
    }

    /// Get the hash of the last entry for a tenant
    async fn get_last_hash(&self, tenant_id: Uuid) -> Result<String> {
        let result: Option<(String,)> = sqlx::query_as(
            r#"
            SELECT hash FROM audit_log
            WHERE tenant_id = $1
            ORDER BY timestamp DESC
            LIMIT 1
            "#
        )
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.map(|r| r.0).unwrap_or_else(|| "genesis".to_string()))
    }

    /// Compute hash for an audit entry
    fn compute_hash(
        &self,
        entry_id: &Uuid,
        tenant_id: Uuid,
        req: &CreateAuditEntryRequest,
        timestamp: &DateTime<Utc>,
        prev_hash: &str,
    ) -> String {
        let mut hasher = Sha256::new();

        // Include all important fields in the hash
        hasher.update(entry_id.as_bytes());
        hasher.update(tenant_id.as_bytes());
        hasher.update(req.action.as_bytes());
        hasher.update(req.resource_type.as_bytes());
        hasher.update(timestamp.to_rfc3339().as_bytes());
        hasher.update(prev_hash.as_bytes());

        // Include optional fields
        if let Some(actor_id) = &req.actor_id {
            hasher.update(actor_id.as_bytes());
        }
        if let Some(resource_id) = &req.resource_id {
            hasher.update(resource_id.as_bytes());
        }
        if let Some(before) = &req.before_state {
            hasher.update(before.to_string().as_bytes());
        }
        if let Some(after) = &req.after_state {
            hasher.update(after.to_string().as_bytes());
        }

        let result = hasher.finalize();
        hex::encode(result)
    }

    /// Query audit log
    pub async fn query(
        &self,
        tenant_id: Uuid,
        params: AuditQueryParams,
    ) -> Result<(Vec<AuditEntry>, i64)> {
        let limit = params.limit.unwrap_or(100).min(1000);
        let offset = params.offset.unwrap_or(0);

        let entries = sqlx::query_as::<_, AuditEntry>(
            r#"
            SELECT id, tenant_id, actor_id, actor_type, actor_name, action,
                   resource_type, resource_id, resource_name, before_state,
                   after_state, metadata, ip_address, user_agent, timestamp, hash
            FROM audit_log
            WHERE tenant_id = $1
              AND ($2::uuid IS NULL OR actor_id = $2)
              AND ($3::text IS NULL OR actor_type = $3)
              AND ($4::text IS NULL OR action = $4)
              AND ($5::text IS NULL OR resource_type = $5)
              AND ($6::uuid IS NULL OR resource_id = $6)
              AND ($7::timestamptz IS NULL OR timestamp >= $7)
              AND ($8::timestamptz IS NULL OR timestamp <= $8)
            ORDER BY timestamp DESC
            LIMIT $9 OFFSET $10
            "#
        )
        .bind(tenant_id)
        .bind(params.actor_id)
        .bind(&params.actor_type)
        .bind(&params.action)
        .bind(&params.resource_type)
        .bind(params.resource_id)
        .bind(params.start_time)
        .bind(params.end_time)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        let total: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*)
            FROM audit_log
            WHERE tenant_id = $1
              AND ($2::uuid IS NULL OR actor_id = $2)
              AND ($3::text IS NULL OR actor_type = $3)
              AND ($4::text IS NULL OR action = $4)
              AND ($5::text IS NULL OR resource_type = $5)
              AND ($6::uuid IS NULL OR resource_id = $6)
              AND ($7::timestamptz IS NULL OR timestamp >= $7)
              AND ($8::timestamptz IS NULL OR timestamp <= $8)
            "#
        )
        .bind(tenant_id)
        .bind(params.actor_id)
        .bind(&params.actor_type)
        .bind(&params.action)
        .bind(&params.resource_type)
        .bind(params.resource_id)
        .bind(params.start_time)
        .bind(params.end_time)
        .fetch_one(&self.pool)
        .await?;

        Ok((entries, total.0))
    }

    /// Get a single audit entry
    pub async fn get_entry(&self, tenant_id: Uuid, id: Uuid) -> Result<Option<AuditEntry>> {
        let entry = sqlx::query_as::<_, AuditEntry>(
            r#"
            SELECT id, tenant_id, actor_id, actor_type, actor_name, action,
                   resource_type, resource_id, resource_name, before_state,
                   after_state, metadata, ip_address, user_agent, timestamp, hash
            FROM audit_log
            WHERE tenant_id = $1 AND id = $2
            "#
        )
        .bind(tenant_id)
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(entry)
    }

    /// Verify the hash chain for a tenant
    pub async fn verify_chain(&self, tenant_id: Uuid) -> Result<ChainVerificationResult> {
        let entries = sqlx::query_as::<_, AuditEntry>(
            r#"
            SELECT id, tenant_id, actor_id, actor_type, actor_name, action,
                   resource_type, resource_id, resource_name, before_state,
                   after_state, metadata, ip_address, user_agent, timestamp, hash
            FROM audit_log
            WHERE tenant_id = $1
            ORDER BY timestamp ASC
            "#
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await?;

        let mut prev_hash = "genesis".to_string();
        let mut entries_checked = 0i64;

        for entry in &entries {
            // Reconstruct the request from the entry
            let req = CreateAuditEntryRequest {
                actor_id: entry.actor_id,
                actor_type: match entry.actor_type.as_str() {
                    "user" => ActorType::User,
                    "system" => ActorType::System,
                    "agent" => ActorType::Agent,
                    "service" => ActorType::Service,
                    _ => ActorType::Anonymous,
                },
                actor_name: entry.actor_name.clone(),
                action: entry.action.clone(),
                resource_type: entry.resource_type.clone(),
                resource_id: entry.resource_id,
                resource_name: entry.resource_name.clone(),
                before_state: entry.before_state.clone(),
                after_state: entry.after_state.clone(),
                metadata: entry.metadata.clone(),
                ip_address: entry.ip_address.clone(),
                user_agent: entry.user_agent.clone(),
            };

            let computed_hash = self.compute_hash(
                &entry.id,
                entry.tenant_id,
                &req,
                &entry.timestamp,
                &prev_hash,
            );

            if computed_hash != entry.hash {
                return Ok(ChainVerificationResult {
                    valid: false,
                    entries_checked,
                    first_invalid_id: Some(entry.id),
                    error: Some(format!(
                        "Hash mismatch at entry {}: expected {}, found {}",
                        entry.id, computed_hash, entry.hash
                    )),
                });
            }

            prev_hash = entry.hash.clone();
            entries_checked += 1;
        }

        Ok(ChainVerificationResult {
            valid: true,
            entries_checked,
            first_invalid_id: None,
            error: None,
        })
    }

    /// Get audit statistics
    pub async fn get_stats(&self, tenant_id: Uuid) -> Result<AuditStats> {
        let total: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM audit_log WHERE tenant_id = $1"
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await?;

        let today: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM audit_log WHERE tenant_id = $1 AND timestamp >= CURRENT_DATE"
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await?;

        let unique_actors: (i64,) = sqlx::query_as(
            "SELECT COUNT(DISTINCT actor_id) FROM audit_log WHERE tenant_id = $1 AND actor_id IS NOT NULL"
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await?;

        let top_actions: Vec<(String, i64)> = sqlx::query_as(
            r#"
            SELECT action, COUNT(*) as cnt
            FROM audit_log
            WHERE tenant_id = $1
            GROUP BY action
            ORDER BY cnt DESC
            LIMIT 10
            "#
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await?;

        let top_resources: Vec<(String, i64)> = sqlx::query_as(
            r#"
            SELECT resource_type, COUNT(*) as cnt
            FROM audit_log
            WHERE tenant_id = $1
            GROUP BY resource_type
            ORDER BY cnt DESC
            LIMIT 10
            "#
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(AuditStats {
            total_entries: total.0,
            entries_today: today.0,
            unique_actors: unique_actors.0,
            top_actions: top_actions.into_iter()
                .map(|(action, count)| ActionCount { action, count })
                .collect(),
            top_resources: top_resources.into_iter()
                .map(|(resource_type, count)| ResourceCount { resource_type, count })
                .collect(),
        })
    }

    /// Export audit log entries as JSON for compliance
    pub async fn export(
        &self,
        tenant_id: Uuid,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<Vec<AuditEntry>> {
        let entries = sqlx::query_as::<_, AuditEntry>(
            r#"
            SELECT id, tenant_id, actor_id, actor_type, actor_name, action,
                   resource_type, resource_id, resource_name, before_state,
                   after_state, metadata, ip_address, user_agent, timestamp, hash
            FROM audit_log
            WHERE tenant_id = $1
              AND timestamp >= $2
              AND timestamp <= $3
            ORDER BY timestamp ASC
            "#
        )
        .bind(tenant_id)
        .bind(start_time)
        .bind(end_time)
        .fetch_all(&self.pool)
        .await?;

        Ok(entries)
    }
}
