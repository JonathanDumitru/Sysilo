# Agent Discovery Task Triggering Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Enable users to trigger asset discovery from the UI, completing the full loop from user action → agent task → discovered assets in registry.

**Architecture:** Add dedicated discovery API endpoint to integration-service that creates discovery tasks and dispatches them to agents via Kafka. Frontend gets a "Discover Assets" button that calls this API and shows results in real-time.

**Tech Stack:** Rust (integration-service), React/TypeScript (frontend), Kafka (messaging)

---

## Task 1: Add Discovery API Types to Integration Service

**Files:**
- Modify: `services/integration-service/src/api/mod.rs`

**Step 1: Add request/response types for discovery**

Add after the existing `RunResponse` struct:

```rust
/// Request to start a discovery run
#[derive(Debug, Deserialize)]
pub struct DiscoveryRequest {
    /// Connection to discover against
    pub connection_id: Uuid,
    /// Type of discovery (full or incremental)
    #[serde(default)]
    pub discovery_type: DiscoveryType,
    /// Optional resource type filters
    #[serde(default)]
    pub resource_types: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiscoveryType {
    #[default]
    Full,
    Incremental,
}

/// Response from starting discovery
#[derive(Debug, Serialize)]
pub struct DiscoveryResponse {
    pub run_id: Uuid,
    pub task_id: Uuid,
    pub status: String,
    pub message: String,
}
```

**Step 2: Verify it compiles**

Run: `cd services/integration-service && cargo check`
Expected: Compiles without errors

**Step 3: Commit**

```bash
git add services/integration-service/src/api/mod.rs
git commit -m "feat(integration): add discovery API types"
```

---

## Task 2: Add Discovery Endpoint Handler

**Files:**
- Modify: `services/integration-service/src/api/mod.rs`
- Modify: `services/integration-service/src/main.rs`

**Step 1: Add the discovery handler function**

Add after `cancel_run` function in `api/mod.rs`:

```rust
/// Start a discovery run against a connection
pub async fn run_discovery(
    State(state): State<Arc<AppState>>,
    Extension(tenant): Extension<TenantContext>,
    Json(req): Json<DiscoveryRequest>,
) -> Result<(StatusCode, Json<DiscoveryResponse>), ApiError> {
    let tenant_id = tenant.tenant_id.to_string();
    let run_id = Uuid::new_v4();
    let task_id = Uuid::new_v4();

    // Create a discovery task
    let task = crate::engine::Task {
        id: task_id,
        run_id,
        integration_id: Uuid::nil(), // No integration - direct discovery
        tenant_id: tenant_id.clone(),
        task_type: "discovery".to_string(),
        config: serde_json::json!({
            "connection_id": req.connection_id,
            "discovery_type": format!("{:?}", req.discovery_type).to_lowercase(),
            "resource_types": req.resource_types,
        }),
        priority: 2,
        timeout_seconds: 300, // 5 minute timeout for discovery
        sequence: 0,
        depends_on: vec![],
    };

    // Send to Kafka if producer available
    if let Some(ref producer) = state.engine.kafka_producer() {
        producer.send_task(&task).await.map_err(|e| ApiError {
            error: "dispatch_error".to_string(),
            message: e.to_string(),
        })?;

        tracing::info!(
            run_id = %run_id,
            task_id = %task_id,
            connection_id = %req.connection_id,
            "Discovery task dispatched"
        );
    } else {
        tracing::warn!(
            run_id = %run_id,
            task_id = %task_id,
            "No Kafka producer - discovery task logged only"
        );
    }

    Ok((
        StatusCode::ACCEPTED,
        Json(DiscoveryResponse {
            run_id,
            task_id,
            status: "pending".to_string(),
            message: "Discovery task dispatched to agent".to_string(),
        }),
    ))
}
```

**Step 2: Add kafka_producer accessor to Engine**

In `services/integration-service/src/engine/mod.rs`, add method to Engine impl:

```rust
/// Get reference to Kafka producer (if available)
pub fn kafka_producer(&self) -> Option<&TaskProducer> {
    self.kafka_producer.as_ref()
}
```

**Step 3: Add route in main.rs**

In the protected_routes section, add:

```rust
.route("/discovery/run", post(api::run_discovery))
```

**Step 4: Verify it compiles**

Run: `cd services/integration-service && cargo check`
Expected: Compiles without errors

**Step 5: Commit**

```bash
git add services/integration-service/src/api/mod.rs services/integration-service/src/engine/mod.rs services/integration-service/src/main.rs
git commit -m "feat(integration): add POST /discovery/run endpoint"
```

---

## Task 3: Add Frontend Discovery Service

**Files:**
- Create: `packages/frontend/web-app/src/services/discovery.ts`

**Step 1: Create the discovery service**

```typescript
import { apiFetch } from './api.js';

export interface DiscoveryRequest {
  connection_id: string;
  discovery_type?: 'full' | 'incremental';
  resource_types?: string[];
}

export interface DiscoveryResponse {
  run_id: string;
  task_id: string;
  status: string;
  message: string;
}

export interface Connection {
  id: string;
  name: string;
  connector_type: string;
  status: string;
}

const DEV_TENANT_ID = 'dev-tenant';

/**
 * Start a discovery run against a connection
 */
export async function runDiscovery(request: DiscoveryRequest): Promise<DiscoveryResponse> {
  return apiFetch<DiscoveryResponse>('/discovery/run', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'X-Tenant-ID': DEV_TENANT_ID,
    },
    body: JSON.stringify(request),
  });
}

/**
 * List available connections for discovery
 * TODO: Replace with real API when connections service is ready
 */
export async function listConnections(): Promise<Connection[]> {
  // Stub data until connections API is implemented
  return [
    { id: '00000000-0000-0000-0000-000000000001', name: 'Production PostgreSQL', connector_type: 'postgresql', status: 'active' },
    { id: '00000000-0000-0000-0000-000000000002', name: 'Salesforce CRM', connector_type: 'salesforce', status: 'active' },
    { id: '00000000-0000-0000-0000-000000000003', name: 'AWS S3 Data Lake', connector_type: 's3', status: 'active' },
  ];
}
```

**Step 2: Verify TypeScript compiles**

Run: `cd packages/frontend/web-app && npm run typecheck`
Expected: No errors

**Step 3: Commit**

```bash
git add packages/frontend/web-app/src/services/discovery.ts
git commit -m "feat(frontend): add discovery service"
```

---

## Task 4: Add Discovery React Query Hooks

**Files:**
- Create: `packages/frontend/web-app/src/hooks/useDiscovery.ts`

**Step 1: Create the hooks**

```typescript
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { listConnections, runDiscovery, type DiscoveryRequest } from '../services/discovery.js';

/**
 * Hook to list available connections
 */
export function useConnections() {
  return useQuery({
    queryKey: ['connections'],
    queryFn: listConnections,
    staleTime: 30_000, // 30 seconds
  });
}

/**
 * Hook to trigger a discovery run
 */
export function useRunDiscovery() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (request: DiscoveryRequest) => runDiscovery(request),
    onSuccess: () => {
      // Invalidate assets query to trigger refresh
      // Assets will appear as they're discovered
      queryClient.invalidateQueries({ queryKey: ['assets'] });
    },
  });
}
```

**Step 2: Verify TypeScript compiles**

Run: `cd packages/frontend/web-app && npm run typecheck`
Expected: No errors

**Step 3: Commit**

```bash
git add packages/frontend/web-app/src/hooks/useDiscovery.ts
git commit -m "feat(frontend): add discovery hooks"
```

---

## Task 5: Add Discovery Modal Component

**Files:**
- Create: `packages/frontend/web-app/src/components/DiscoveryModal.tsx`

**Step 1: Create the modal component**

```typescript
import { useState } from 'react';
import { X, Search, Database, Server, Globe, Loader2, CheckCircle } from 'lucide-react';
import { useConnections, useRunDiscovery } from '../hooks/useDiscovery.js';

interface DiscoveryModalProps {
  isOpen: boolean;
  onClose: () => void;
}

const connectorIcons: Record<string, React.ElementType> = {
  postgresql: Database,
  mysql: Database,
  salesforce: Globe,
  s3: Server,
  default: Server,
};

export function DiscoveryModal({ isOpen, onClose }: DiscoveryModalProps) {
  const [selectedConnections, setSelectedConnections] = useState<string[]>([]);
  const { data: connections, isLoading: loadingConnections } = useConnections();
  const { mutate: runDiscovery, isPending, isSuccess } = useRunDiscovery();

  if (!isOpen) return null;

  const handleToggleConnection = (id: string) => {
    setSelectedConnections((prev) =>
      prev.includes(id) ? prev.filter((c) => c !== id) : [...prev, id]
    );
  };

  const handleStartDiscovery = () => {
    selectedConnections.forEach((connectionId) => {
      runDiscovery({ connection_id: connectionId });
    });
  };

  const handleClose = () => {
    setSelectedConnections([]);
    onClose();
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      {/* Backdrop */}
      <div className="absolute inset-0 bg-black/50" onClick={handleClose} />

      {/* Modal */}
      <div className="relative bg-white rounded-xl shadow-xl w-full max-w-lg mx-4">
        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b border-gray-100">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-primary-50 rounded-lg">
              <Search className="w-5 h-5 text-primary-600" />
            </div>
            <div>
              <h2 className="text-lg font-semibold text-gray-900">Discover Assets</h2>
              <p className="text-sm text-gray-500">Select connections to scan for assets</p>
            </div>
          </div>
          <button
            onClick={handleClose}
            className="p-2 text-gray-400 hover:text-gray-600 rounded-lg hover:bg-gray-100"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Content */}
        <div className="p-4 max-h-80 overflow-y-auto">
          {loadingConnections ? (
            <div className="flex items-center justify-center py-8">
              <Loader2 className="w-6 h-6 text-primary-500 animate-spin" />
            </div>
          ) : isSuccess ? (
            <div className="flex flex-col items-center justify-center py-8 text-center">
              <CheckCircle className="w-12 h-12 text-green-500 mb-3" />
              <h3 className="text-lg font-medium text-gray-900">Discovery Started</h3>
              <p className="text-sm text-gray-500 mt-1">
                Assets will appear in the registry as they're discovered
              </p>
            </div>
          ) : (
            <div className="space-y-2">
              {connections?.map((connection) => {
                const Icon = connectorIcons[connection.connector_type] ?? connectorIcons.default;
                const isSelected = selectedConnections.includes(connection.id);

                return (
                  <button
                    key={connection.id}
                    onClick={() => handleToggleConnection(connection.id)}
                    className={`w-full flex items-center gap-3 p-3 rounded-lg border transition-colors ${
                      isSelected
                        ? 'border-primary-500 bg-primary-50'
                        : 'border-gray-200 hover:border-gray-300 hover:bg-gray-50'
                    }`}
                  >
                    <div
                      className={`p-2 rounded-lg ${
                        isSelected ? 'bg-primary-100' : 'bg-gray-100'
                      }`}
                    >
                      <Icon
                        className={`w-5 h-5 ${
                          isSelected ? 'text-primary-600' : 'text-gray-600'
                        }`}
                      />
                    </div>
                    <div className="flex-1 text-left">
                      <div className="font-medium text-gray-900">{connection.name}</div>
                      <div className="text-sm text-gray-500">{connection.connector_type}</div>
                    </div>
                    <div
                      className={`w-5 h-5 rounded-full border-2 flex items-center justify-center ${
                        isSelected ? 'border-primary-500 bg-primary-500' : 'border-gray-300'
                      }`}
                    >
                      {isSelected && <CheckCircle className="w-3 h-3 text-white" />}
                    </div>
                  </button>
                );
              })}
            </div>
          )}
        </div>

        {/* Footer */}
        {!isSuccess && (
          <div className="flex items-center justify-end gap-3 p-4 border-t border-gray-100">
            <button
              onClick={handleClose}
              className="px-4 py-2 text-sm font-medium text-gray-700 hover:bg-gray-100 rounded-lg"
            >
              Cancel
            </button>
            <button
              onClick={handleStartDiscovery}
              disabled={selectedConnections.length === 0 || isPending}
              className="flex items-center gap-2 px-4 py-2 text-sm font-medium text-white bg-primary-600 hover:bg-primary-700 rounded-lg disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {isPending ? (
                <>
                  <Loader2 className="w-4 h-4 animate-spin" />
                  Starting...
                </>
              ) : (
                <>
                  <Search className="w-4 h-4" />
                  Start Discovery ({selectedConnections.length})
                </>
              )}
            </button>
          </div>
        )}
      </div>
    </div>
  );
}
```

**Step 2: Verify TypeScript compiles**

Run: `cd packages/frontend/web-app && npm run typecheck`
Expected: No errors

**Step 3: Commit**

```bash
git add packages/frontend/web-app/src/components/DiscoveryModal.tsx
git commit -m "feat(frontend): add DiscoveryModal component"
```

---

## Task 6: Integrate Discovery Button into Asset Registry Page

**Files:**
- Modify: `packages/frontend/web-app/src/pages/AssetRegistryPage.tsx`

**Step 1: Add discovery button and modal integration**

At the top, add import:

```typescript
import { Radar } from 'lucide-react';
import { DiscoveryModal } from '../components/DiscoveryModal.js';
```

Add state for modal inside the component:

```typescript
const [isDiscoveryOpen, setIsDiscoveryOpen] = useState(false);
```

After the "Graph View" button, add the Discover button:

```typescript
<button
  onClick={() => setIsDiscoveryOpen(true)}
  className="flex items-center gap-2 px-4 py-2 bg-primary-600 text-white rounded-lg text-sm font-medium hover:bg-primary-700"
>
  <Radar className="w-4 h-4" />
  Discover Assets
</button>
```

At the end of the component, before the closing `</div>`, add the modal:

```typescript
<DiscoveryModal
  isOpen={isDiscoveryOpen}
  onClose={() => setIsDiscoveryOpen(false)}
/>
```

**Step 2: Verify TypeScript compiles**

Run: `cd packages/frontend/web-app && npm run typecheck`
Expected: No errors

**Step 3: Commit**

```bash
git add packages/frontend/web-app/src/pages/AssetRegistryPage.tsx
git commit -m "feat(frontend): add discovery button to Asset Registry"
```

---

## Task 7: Add Auto-Refresh for Asset List

**Files:**
- Modify: `packages/frontend/web-app/src/hooks/useAssets.ts`

**Step 1: Add refetch interval when discovery is active**

Update the `useAssets` hook to support polling:

```typescript
export function useAssets(params: AssetQueryParams = {}, enablePolling = false) {
  return useQuery({
    queryKey: ['assets', params],
    queryFn: () => listAssets(params),
    staleTime: 10_000, // 10 seconds
    refetchInterval: enablePolling ? 5_000 : false, // Poll every 5s when enabled
  });
}
```

**Step 2: Verify TypeScript compiles**

Run: `cd packages/frontend/web-app && npm run typecheck`
Expected: No errors

**Step 3: Commit**

```bash
git add packages/frontend/web-app/src/hooks/useAssets.ts
git commit -m "feat(frontend): add polling support to useAssets"
```

---

## Task 8: Integration Verification

**Step 1: Build everything**

```bash
# Frontend
cd packages/frontend/web-app && npm run build

# Integration service
cd services/integration-service && cargo build
```

Expected: Both build successfully

**Step 2: Manual verification checklist**

With services running:
- [ ] Navigate to Asset Registry page
- [ ] Click "Discover Assets" button
- [ ] Modal opens showing available connections
- [ ] Select one or more connections
- [ ] Click "Start Discovery"
- [ ] See success message
- [ ] New assets appear in registry (requires agent + Kafka running)

**Step 3: Commit verification notes**

```bash
git add -A
git commit -m "docs: complete discovery triggering implementation"
```

---

## Summary

This implementation adds:
1. **Backend:** `POST /discovery/run` endpoint in integration-service
2. **Frontend:** Discovery service, hooks, modal, and button integration
3. **Flow:** User → UI → API → Kafka → Agent → Results → Consumer → Asset Service → UI

The flow reuses the existing consumer we built in the previous "Close the Loop" work, so discovered assets automatically appear in the registry.
