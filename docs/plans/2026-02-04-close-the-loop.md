# Close the Loop: Agent → Asset Registry Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Complete the end-to-end data flow so discovered assets from agents appear in the Asset Registry UI.

**Architecture:** Add a Kafka consumer to integration-service that processes task results from agents, extracts discovered assets, and forwards them to asset-service via HTTP. Update frontend to fetch real data from asset-service API.

**Tech Stack:** Rust (rdkafka, reqwest), TypeScript (React Query), existing asset-service REST API

---

## Overview

```
Agent → Agent-Gateway → Kafka (sysilo.results) → [NEW: Consumer] → Asset-Service API → [NEW: Frontend hooks] → UI
```

**Two gaps to close:**
1. Kafka consumer in integration-service → asset-service HTTP calls
2. Frontend API client → React Query hooks → AssetRegistryPage

---

## Task 1: Add Kafka Consumer Types to Integration Service

**Files:**
- Modify: `services/integration-service/src/kafka/mod.rs`

**Step 1: Add consumer dependencies check**

Verify rdkafka in Cargo.toml supports consumer (it does - tokio feature includes StreamConsumer).

**Step 2: Add ResultMessage deserialization struct**

Add to `services/integration-service/src/kafka/mod.rs` after line 22:

```rust
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::Message;
use serde::Deserialize;
use futures::StreamExt;

/// Task result received from agents via Kafka
#[derive(Debug, Deserialize)]
pub struct TaskResult {
    pub task_id: String,
    pub agent_id: String,
    pub integration_id: String,
    pub tenant_id: String,
    pub status: String,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: chrono::DateTime<chrono::Utc>,
    pub output: Option<serde_json::Value>,
    pub error: Option<TaskError>,
    pub metrics: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct TaskError {
    pub code: String,
    pub message: String,
    pub details: Option<String>,
    pub retryable: bool,
}

/// Discovered asset from agent discovery task
#[derive(Debug, Deserialize)]
pub struct DiscoveredAsset {
    pub name: String,
    pub asset_type: String,
    pub description: Option<String>,
    pub vendor: Option<String>,
    pub version: Option<String>,
    pub metadata: Option<serde_json::Value>,
}
```

**Step 3: Run cargo check**

Run: `cd services/integration-service && cargo check`
Expected: Compiles with warnings about unused imports (expected at this stage)

**Step 4: Commit**

```bash
git add services/integration-service/src/kafka/mod.rs
git commit -m "feat(integration): add Kafka consumer types for task results"
```

---

## Task 2: Create Result Consumer Module

**Files:**
- Create: `services/integration-service/src/consumer/mod.rs`
- Modify: `services/integration-service/src/main.rs`

**Step 1: Create consumer module**

Create `services/integration-service/src/consumer/mod.rs`:

```rust
use anyhow::Result;
use futures::StreamExt;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::Message;
use tracing::{info, warn, error};

use crate::kafka::{topics, TaskResult, DiscoveredAsset};

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

        let response = self.http_client
            .post(&url)
            .json(&body)
            .send()
            .await?;

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
```

**Step 2: Add reqwest dependency**

Add to `services/integration-service/Cargo.toml` dependencies:

```toml
reqwest = { version = "0.11", features = ["json"] }
```

**Step 3: Register module in main.rs**

Add after line 20 in `services/integration-service/src/main.rs`:

```rust
mod consumer;
```

**Step 4: Run cargo check**

Run: `cd services/integration-service && cargo check`
Expected: Compiles successfully

**Step 5: Commit**

```bash
git add services/integration-service/
git commit -m "feat(integration): add Kafka result consumer module"
```

---

## Task 3: Start Consumer in Main

**Files:**
- Modify: `services/integration-service/src/main.rs`
- Modify: `services/integration-service/src/config/mod.rs`

**Step 1: Add consumer config to Config struct**

In `services/integration-service/src/config/mod.rs`, add to the Config struct:

```rust
pub struct Config {
    // ... existing fields ...
    pub consumer: ConsumerConfig,
}

#[derive(Debug, Clone)]
pub struct ConsumerConfig {
    pub bootstrap_servers: String,
    pub group_id: String,
    pub asset_service_url: String,
    pub enabled: bool,
}

impl Default for ConsumerConfig {
    fn default() -> Self {
        Self {
            bootstrap_servers: "localhost:9092".to_string(),
            group_id: "integration-service-consumers".to_string(),
            asset_service_url: "http://localhost:8082".to_string(),
            enabled: true,
        }
    }
}
```

**Step 2: Spawn consumer task in main**

In `services/integration-service/src/main.rs`, add after engine initialization (around line 56):

```rust
    // Start result consumer in background
    if config.consumer.enabled {
        let consumer_config = consumer::ConsumerConfig {
            bootstrap_servers: config.consumer.bootstrap_servers.clone(),
            group_id: config.consumer.group_id.clone(),
            asset_service_url: config.consumer.asset_service_url.clone(),
        };

        tokio::spawn(async move {
            let consumer = match consumer::ResultConsumer::new(&consumer_config) {
                Ok(c) => c,
                Err(e) => {
                    error!("Failed to create result consumer: {}", e);
                    return;
                }
            };

            if let Err(e) = consumer.run().await {
                error!("Result consumer error: {}", e);
            }
        });

        info!("Result consumer started");
    }
```

Add the import at the top:
```rust
use tracing::error;
```

**Step 3: Run cargo check**

Run: `cd services/integration-service && cargo check`
Expected: Compiles successfully

**Step 4: Commit**

```bash
git add services/integration-service/
git commit -m "feat(integration): spawn Kafka consumer on service startup"
```

---

## Task 4: Create Frontend API Client

**Files:**
- Create: `packages/frontend/web-app/src/services/api.ts`
- Create: `packages/frontend/web-app/src/services/assets.ts`

**Step 1: Create base API client**

Create `packages/frontend/web-app/src/services/api.ts`:

```typescript
const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:8082';

export class ApiError extends Error {
  constructor(public status: number, message: string) {
    super(message);
    this.name = 'ApiError';
  }
}

export async function apiFetch<T>(
  endpoint: string,
  options: RequestInit = {}
): Promise<T> {
  const url = `${API_BASE_URL}${endpoint}`;

  const response = await fetch(url, {
    ...options,
    headers: {
      'Content-Type': 'application/json',
      ...options.headers,
    },
  });

  if (!response.ok) {
    const text = await response.text();
    throw new ApiError(response.status, text || response.statusText);
  }

  return response.json();
}
```

**Step 2: Create assets service**

Create `packages/frontend/web-app/src/services/assets.ts`:

```typescript
import { apiFetch } from './api';

// Default tenant ID for development
const DEV_TENANT_ID = '00000000-0000-0000-0000-000000000001';

export interface Asset {
  id: string;
  tenant_id: string;
  name: string;
  asset_type: string;
  status: string;
  description?: string;
  owner?: string;
  team?: string;
  vendor?: string;
  version?: string;
  documentation_url?: string;
  repository_url?: string;
  metadata?: Record<string, unknown>;
  tags: string[];
  created_at: string;
  updated_at: string;
}

export interface ListAssetsResponse {
  assets: Asset[];
  total: number;
}

export interface ListAssetsParams {
  tenant_id?: string;
  asset_type?: string;
  status?: string;
  limit?: number;
  offset?: number;
}

export async function listAssets(params: ListAssetsParams = {}): Promise<ListAssetsResponse> {
  const searchParams = new URLSearchParams();
  searchParams.set('tenant_id', params.tenant_id || DEV_TENANT_ID);

  if (params.asset_type) searchParams.set('asset_type', params.asset_type);
  if (params.status) searchParams.set('status', params.status);
  if (params.limit) searchParams.set('limit', params.limit.toString());
  if (params.offset) searchParams.set('offset', params.offset.toString());

  return apiFetch<ListAssetsResponse>(`/assets?${searchParams.toString()}`);
}

export async function getAsset(id: string, tenantId?: string): Promise<Asset> {
  const searchParams = new URLSearchParams();
  searchParams.set('tenant_id', tenantId || DEV_TENANT_ID);

  return apiFetch<Asset>(`/assets/${id}?${searchParams.toString()}`);
}

export async function searchAssets(query: string, tenantId?: string): Promise<Asset[]> {
  const searchParams = new URLSearchParams();
  searchParams.set('tenant_id', tenantId || DEV_TENANT_ID);
  searchParams.set('q', query);

  return apiFetch<Asset[]>(`/assets/search?${searchParams.toString()}`);
}
```

**Step 3: Run TypeScript check**

Run: `cd packages/frontend/web-app && npx tsc --noEmit`
Expected: No errors

**Step 4: Commit**

```bash
git add packages/frontend/web-app/src/services/
git commit -m "feat(frontend): add API client and assets service"
```

---

## Task 5: Create React Query Hooks

**Files:**
- Create: `packages/frontend/web-app/src/hooks/useAssets.ts`

**Step 1: Create useAssets hook**

Create `packages/frontend/web-app/src/hooks/useAssets.ts`:

```typescript
import { useQuery } from '@tanstack/react-query';
import { listAssets, getAsset, searchAssets, type ListAssetsParams, type Asset } from '../services/assets';

export function useAssets(params: ListAssetsParams = {}) {
  return useQuery({
    queryKey: ['assets', params],
    queryFn: () => listAssets(params),
  });
}

export function useAsset(id: string, tenantId?: string) {
  return useQuery({
    queryKey: ['asset', id, tenantId],
    queryFn: () => getAsset(id, tenantId),
    enabled: !!id,
  });
}

export function useAssetSearch(query: string, tenantId?: string) {
  return useQuery({
    queryKey: ['assets', 'search', query, tenantId],
    queryFn: () => searchAssets(query, tenantId),
    enabled: query.length >= 2,
  });
}

export type { Asset, ListAssetsParams };
```

**Step 2: Run TypeScript check**

Run: `cd packages/frontend/web-app && npx tsc --noEmit`
Expected: No errors

**Step 3: Commit**

```bash
git add packages/frontend/web-app/src/hooks/
git commit -m "feat(frontend): add React Query hooks for assets"
```

---

## Task 6: Update Asset Registry Page

**Files:**
- Modify: `packages/frontend/web-app/src/pages/AssetRegistryPage.tsx`

**Step 1: Replace hardcoded data with hook**

Replace entire contents of `packages/frontend/web-app/src/pages/AssetRegistryPage.tsx`:

```typescript
import { useState } from 'react';
import { Search, Filter, Network, Server, Database, Workflow, Loader2, AlertCircle } from 'lucide-react';
import { useAssets, useAssetSearch, type Asset } from '../hooks/useAssets';

export function AssetRegistryPage() {
  const [searchQuery, setSearchQuery] = useState('');
  const [typeFilter, setTypeFilter] = useState<string | undefined>();

  // Use search hook if query exists, otherwise use list
  const assetsQuery = useAssets({
    asset_type: typeFilter,
    limit: 50
  });
  const searchResults = useAssetSearch(searchQuery);

  // Use search results if searching, otherwise use all assets
  const isSearching = searchQuery.length >= 2;
  const assets = isSearching
    ? searchResults.data || []
    : assetsQuery.data?.assets || [];
  const isLoading = isSearching ? searchResults.isLoading : assetsQuery.isLoading;
  const error = isSearching ? searchResults.error : assetsQuery.error;

  const typeIcons: Record<string, React.ElementType> = {
    application: Server,
    service: Server,
    database: Database,
    api: Workflow,
    datastore: Database,
    integration: Workflow,
    infrastructure: Server,
    platform: Server,
    tool: Server,
  };

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-gray-900">Asset Registry</h1>
        <p className="text-gray-500">Inventory and relationships across your technology landscape</p>
      </div>

      {/* Search and filters */}
      <div className="flex items-center gap-4">
        <div className="flex items-center gap-2 bg-white border border-gray-200 rounded-lg px-3 py-2 flex-1 max-w-md">
          <Search className="w-4 h-4 text-gray-400" />
          <input
            type="text"
            placeholder="Search assets..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="bg-transparent border-none outline-none text-sm flex-1"
          />
        </div>
        <select
          value={typeFilter || ''}
          onChange={(e) => setTypeFilter(e.target.value || undefined)}
          className="px-3 py-2 bg-white border border-gray-200 rounded-lg text-sm text-gray-600"
        >
          <option value="">All Types</option>
          <option value="application">Application</option>
          <option value="service">Service</option>
          <option value="database">Database</option>
          <option value="api">API</option>
          <option value="integration">Integration</option>
        </select>
        <button className="flex items-center gap-2 px-3 py-2 bg-white border border-gray-200 rounded-lg text-sm text-gray-600 hover:bg-gray-50">
          <Filter className="w-4 h-4" />
          Filters
        </button>
        <button className="flex items-center gap-2 px-3 py-2 bg-white border border-gray-200 rounded-lg text-sm text-gray-600 hover:bg-gray-50">
          <Network className="w-4 h-4" />
          Graph View
        </button>
      </div>

      {/* Loading state */}
      {isLoading && (
        <div className="flex items-center justify-center py-12">
          <Loader2 className="w-8 h-8 text-primary-500 animate-spin" />
          <span className="ml-2 text-gray-500">Loading assets...</span>
        </div>
      )}

      {/* Error state */}
      {error && (
        <div className="flex items-center gap-3 p-4 bg-red-50 border border-red-200 rounded-lg">
          <AlertCircle className="w-5 h-5 text-red-500" />
          <div>
            <p className="font-medium text-red-800">Failed to load assets</p>
            <p className="text-sm text-red-600">{error.message}</p>
          </div>
        </div>
      )}

      {/* Empty state */}
      {!isLoading && !error && assets.length === 0 && (
        <div className="text-center py-12">
          <Database className="w-12 h-12 text-gray-300 mx-auto mb-4" />
          <h3 className="text-lg font-medium text-gray-900">No assets found</h3>
          <p className="text-gray-500 mt-1">
            {isSearching
              ? 'Try a different search term'
              : 'Assets will appear here once discovered by agents'}
          </p>
        </div>
      )}

      {/* Asset grid */}
      {!isLoading && !error && assets.length > 0 && (
        <div className="grid grid-cols-3 gap-4">
          {assets.map((asset: Asset) => {
            const Icon = typeIcons[asset.asset_type.toLowerCase()] || Server;
            return (
              <div
                key={asset.id}
                className="bg-white rounded-xl p-5 shadow-sm border border-gray-100 hover:border-primary-200 cursor-pointer transition-colors"
              >
                <div className="flex items-start gap-4">
                  <div className="p-3 bg-gray-100 rounded-lg">
                    <Icon className="w-6 h-6 text-gray-600" />
                  </div>
                  <div className="flex-1 min-w-0">
                    <h3 className="text-lg font-semibold text-gray-900 truncate">{asset.name}</h3>
                    <p className="text-sm text-gray-500 truncate">{asset.description || 'No description'}</p>
                    <span className="inline-block mt-2 text-xs font-medium px-2 py-0.5 bg-gray-100 text-gray-600 rounded">
                      {asset.asset_type}
                    </span>
                  </div>
                </div>
                <div className="mt-4 pt-4 border-t border-gray-50 flex items-center justify-between text-sm">
                  <span className="text-gray-500">
                    {asset.vendor && <span className="font-medium text-gray-700">{asset.vendor}</span>}
                    {asset.version && <span className="ml-1">v{asset.version}</span>}
                  </span>
                  <span className={`px-2 py-0.5 rounded text-xs font-medium ${
                    asset.status === 'active' ? 'bg-green-100 text-green-700' :
                    asset.status === 'deprecated' ? 'bg-yellow-100 text-yellow-700' :
                    'bg-gray-100 text-gray-600'
                  }`}>
                    {asset.status}
                  </span>
                </div>
                {asset.tags.length > 0 && (
                  <div className="mt-3 flex flex-wrap gap-1">
                    {asset.tags.slice(0, 3).map((tag) => (
                      <span key={tag} className="text-xs px-2 py-0.5 bg-primary-50 text-primary-700 rounded">
                        {tag}
                      </span>
                    ))}
                    {asset.tags.length > 3 && (
                      <span className="text-xs text-gray-400">+{asset.tags.length - 3}</span>
                    )}
                  </div>
                )}
              </div>
            );
          })}
        </div>
      )}

      {/* Results count */}
      {!isLoading && !error && assets.length > 0 && (
        <div className="text-sm text-gray-500">
          Showing {assets.length} {isSearching ? 'results' : `of ${assetsQuery.data?.total || 0} assets`}
        </div>
      )}
    </div>
  );
}
```

**Step 2: Run TypeScript check**

Run: `cd packages/frontend/web-app && npx tsc --noEmit`
Expected: No errors

**Step 3: Commit**

```bash
git add packages/frontend/web-app/src/pages/AssetRegistryPage.tsx
git commit -m "feat(frontend): connect Asset Registry page to real API"
```

---

## Task 7: Add CORS Configuration to Asset Service

**Files:**
- Modify: `services/asset-service/src/main.rs`

**Step 1: Verify CORS is configured**

Check that asset-service has permissive CORS for local dev. Look for `CorsLayer::permissive()` in main.rs.

If not present, add to the router:
```rust
.layer(CorsLayer::permissive())
```

**Step 2: Commit if changes made**

```bash
git add services/asset-service/
git commit -m "feat(asset-service): enable CORS for frontend"
```

---

## Task 8: End-to-End Verification

**Step 1: Start infrastructure**

Run: `cd infra/docker && docker-compose up -d`
Expected: All containers start (postgres, neo4j, kafka, redis, minio)

**Step 2: Start asset-service**

Run: `cd services/asset-service && cargo run`
Expected: "Listening on 0.0.0.0:8082"

**Step 3: Start integration-service**

Run: `cd services/integration-service && cargo run`
Expected: "Listening on..." and "Result consumer started"

**Step 4: Start frontend**

Run: `cd packages/frontend/web-app && npm run dev`
Expected: Vite dev server starts

**Step 5: Verify empty state**

Open: http://localhost:5173/assets
Expected: "No assets found" message with "Assets will appear here once discovered by agents"

**Step 6: Create test asset via API**

```bash
curl -X POST http://localhost:8082/assets \
  -H "Content-Type: application/json" \
  -d '{
    "tenant_id": "00000000-0000-0000-0000-000000000001",
    "name": "Test Database",
    "asset_type": "database",
    "description": "A test PostgreSQL database",
    "vendor": "PostgreSQL",
    "version": "16.0",
    "status": "active",
    "tags": ["test", "database"]
  }'
```

**Step 7: Verify asset appears in UI**

Refresh: http://localhost:5173/assets
Expected: "Test Database" card appears in the grid

**Step 8: Final commit**

```bash
git add -A
git commit -m "feat: complete end-to-end asset discovery flow"
```

---

## Summary

After completing all tasks:
- ✅ Kafka consumer in integration-service processes `sysilo.results` topic
- ✅ Discovered assets are forwarded to asset-service via HTTP
- ✅ Frontend fetches real data from asset-service API
- ✅ Asset Registry page shows loading, error, and empty states
- ✅ Search and type filtering work against real API

**Next steps (not in this plan):**
- Add agent discovery task triggering
- Implement asset detail view
- Add graph visualization
- Implement relationship management
