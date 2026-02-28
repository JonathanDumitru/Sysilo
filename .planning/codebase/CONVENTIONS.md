# Coding Conventions

**Analysis Date:** 2026-02-28

## Languages and Their Conventions

This codebase uses four languages with distinct conventions per language. Follow each language's own patterns exactly.

---

## TypeScript / React (Frontend)

### Naming Patterns

**Files:**
- React components: `PascalCase.tsx` — e.g., `PlanBadge.tsx`, `UpgradeModal.tsx`, `AIAssistButton.tsx`
- Pages: `PascalCasePage.tsx` suffix — e.g., `DashboardPage.tsx`, `PlaybookRunDetailPage.tsx`
- Hooks: `camelCase.ts` with `use` prefix — e.g., `usePlaybooks.ts`, `usePlan.ts`
- Services: `camelCase.ts`, noun describing domain — e.g., `billing.ts`, `playbooks.ts`, `connections.ts`
- Type files: `index.ts` in `src/types/`
- Barrel re-exports: `index.ts` in component folders — e.g., `src/components/ai/index.ts`

**Components:**
- Named exports only (no default exports for components) — e.g., `export function PlanBadge()`
- `App.tsx` is the sole default export: `export default App;`

**Hooks:**
- Custom hooks are named functions, exported directly — e.g., `export function usePlaybooks()`
- Query key factory objects are named `{domain}Keys` — e.g., `playbookKeys`, `planKeys`

**Interfaces:**
- PascalCase with descriptive suffix — `Request`, `Response`, `Props`
- Props interfaces defined inline in same file as component — e.g., `interface AIAssistButtonProps`

**Type Aliases:**
- PascalCase for union string types — e.g., `type TriggerType = 'manual' | 'scheduled' | ...`

### Code Style

**Formatting:**
- TypeScript strict mode: `tsconfig.json` with `"strict": true` implied
- ESLint with `@typescript-eslint` + `eslint-plugin-react-hooks` + `eslint-plugin-react-refresh`
- Run: `npm run lint` (eslint with `--max-warnings 0`)
- No prettier config found at project root; formatting enforced by ESLint

**Import Organization:**
1. External packages (React, libraries)
2. Internal services (`../services/...`)
3. Internal hooks (`../hooks/...`)
4. Internal components (`../components/...`)
5. Types (using `import type` or inline `type` keyword in import statement)

**Path Aliases:**
- `@/` maps to `src/` via Vite config: `resolve.alias['@'] = path.resolve(__dirname, './src')`
- Usage: `import { foo } from '@/components/...'` (alias available but not yet consistently used — relative paths `../` are more common)

### React Component Patterns

**Component Structure:**
```typescript
// 1. External imports
import { useState } from 'react';
import { Sparkles } from 'lucide-react';
// 2. Internal imports
import { AIChatPanel } from './AIChatPanel';

// 3. Props interface (inline)
interface AIAssistButtonProps {
  context?: string;
  className?: string;
}

// 4. Component as named function export
export function AIAssistButton({ context = 'general', className = '' }: AIAssistButtonProps) {
  // hooks at top
  const [isOpen, setIsOpen] = useState(false);

  // JSX return
  return (
    <>
      ...
    </>
  );
}
```

**Styling:**
- Tailwind CSS utility classes throughout; no CSS modules
- Dynamic class composition with template literals or `clsx` (available as dep)
- Tailwind `primary-*` custom color scale used extensively — e.g., `bg-primary-600`, `text-primary-800`

### Service Layer Pattern

Services in `src/services/` are plain async functions that call `apiFetch`. They export types alongside functions:

```typescript
// Types first (interfaces)
export interface CreatePlaybookRequest { ... }

// Then async functions
export async function createPlaybook(request: CreatePlaybookRequest): Promise<Playbook> {
  return apiFetch<Playbook>('/integrations/playbooks', {
    method: 'POST',
    body: JSON.stringify(request),
  });
}
```

### Hook (Data-Fetching) Pattern

Hooks in `src/hooks/` wrap TanStack Query. Pattern is consistent across all domains:

```typescript
// Query key factory object (exported, as const)
export const playbookKeys = {
  all: ['playbooks'] as const,
  list: () => [...playbookKeys.all, 'list'] as const,
  detail: (id: string) => [...playbookKeys.all, 'detail', id] as const,
};

// One hook per operation
export function usePlaybooks() {
  return useQuery({
    queryKey: playbookKeys.list(),
    queryFn: listPlaybooks,
  });
}

// Mutations always invalidate on success
export function useCreatePlaybook() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (request: CreatePlaybookRequest) => createPlaybook(request),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: playbookKeys.list() });
    },
  });
}
```

Key hook conventions:
- `enabled: !!id` guard on queries requiring an ID
- `staleTime` set on frequently-used queries (e.g., `staleTime: 30_000`, `60_000`)
- Polling via `refetchInterval` callback that returns `false` when not needed
- Re-export types from service at bottom: `export type { Playbook, ... }`

### Error Handling (Frontend)

- `ApiError` class in `src/services/api.ts` with `status: number` and `message: string`
- Components receive error states via TanStack Query's `error` return
- No global error boundary patterns detected — errors are handled locally per component

---

## Rust (Backend Services)

Services: `services/integration-service`, `services/governance-service`, `services/data-service`, `services/asset-service`, `services/ops-service`, `services/rationalization-service`

### Naming Patterns

**Files:**
- Modules: `mod.rs` for module entry points, then separate files for sub-concerns — e.g., `api.rs`, `executor.rs`, `result_handler.rs`
- All lowercase snake_case filenames

**Types:**
- Structs and enums: PascalCase — e.g., `PlaybookRun`, `RunStatus`, `TenantContext`
- Functions and methods: snake_case — e.g., `list_playbooks`, `create_playbook`
- Constants: SCREAMING_SNAKE_CASE — e.g., `TENANT_ID_HEADER`
- Module names: snake_case — e.g., `pub mod playbooks_api`

**Enum Variants:**
- PascalCase — e.g., `RunStatus::WaitingApproval`, `StepStatus::Pending`
- `#[serde(rename_all = "snake_case")]` applied consistently to all serialized enums

### Code Style

**Formatting:**
- `cargo fmt` via Makefile (`make fmt-rust`)
- `cargo clippy -- -D warnings` via Makefile (`make lint-rust`)
- No `rustfmt.toml` or `clippy.toml` — defaults used

**Derive Macros Pattern:**
Always use `#[derive]` in order: `Debug, Clone, Serialize, Deserialize` (and optionally `Default, PartialEq`):
```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    #[default]
    Pending,
    // ...
}
```

**Section Dividers:**
Large files use comment banners to delineate sections:
```rust
// =============================================================================
// Request/Response Types
// =============================================================================
```

**Doc Comments:**
- `///` for public items — structs, enums, and public functions
- `//` for inline comments explaining logic

### Axum Handler Pattern

All handlers follow this signature:
```rust
pub async fn list_playbooks(
    State(state): State<Arc<AppState>>,
    Extension(tenant): Extension<TenantContext>,
) -> Result<Json<PlaybookListResponse>, ApiError> {
    let tenant_id = tenant.tenant_id.to_string();

    let rows = state
        .storage
        .list_playbooks(&tenant_id, 100, 0)
        .await
        .map_err(|e| ApiError {
            error: "database_error".to_string(),
            message: e.to_string(),
        })?;

    // ... transform and return
    Ok(Json(response))
}
```

Key conventions:
- `Arc<AppState>` always first extractor
- `Extension(tenant): Extension<TenantContext>` for tenant-aware endpoints
- Errors mapped inline with `map_err(|e| ApiError { ... })` — no helper macros
- HTTP 201 via `(StatusCode::CREATED, Json(response))`; 202 via `StatusCode::ACCEPTED`
- 204 via `Ok(StatusCode::NO_CONTENT)` for deletes

### Error Handling (Rust)

`ApiError` struct in `src/api/mod.rs` is the canonical error type:
```rust
pub struct ApiError {
    pub error: String,   // machine-readable error code
    pub message: String, // human-readable description
    pub status: Option<StatusCode>, // skipped in serialization
    // billing-specific optional fields
    pub resource: Option<String>,
    pub current: Option<i64>,
    pub limit: Option<i64>,
    pub plan: Option<String>,
}
```

Constructor helpers:
- `ApiError::internal(error: &str, message: String)` — generic 500
- `ApiError::limit_reached(resource, current, limit, plan)` — 429
- `ApiError::upgrade_required(feature, plan)` — 403

For domain errors, use `thiserror::Error` derive on module-level enums:
```rust
#[derive(Debug, Error)]
pub enum EngineError {
    #[error("Integration not found: {0}")]
    IntegrationNotFound(Uuid),
    // ...
    #[error("Kafka error: {0}")]
    KafkaError(#[from] KafkaError),
}
```

### Logging (Rust)

Use structured `tracing` macros with key=value fields:
```rust
tracing::info!(
    run_id = %run_id,
    tenant_id = %tenant_id,
    "Playbook run started"
);

tracing::error!(
    run_id = %run_id,
    step_id = %step_id,
    error = %e,
    "Failed to dispatch step after approval"
);
```

Log levels:
- `info!` — successful operations with context
- `warn!` — degraded operation (e.g., Kafka unavailable, using fallback)
- `error!` — failed operation
- `debug!` — detailed tracing (token validation etc.)

---

## Go (Agent and Gateways)

Services: `agent/`, `services/api-gateway/`, `services/agent-gateway/`

### Naming Patterns

**Packages:**
- All lowercase, no underscores — e.g., `package executor`, `package middleware`, `package handlers`
- Module prefix: `github.com/sysilo/sysilo/...`

**Types:**
- Structs: PascalCase — e.g., `Executor`, `Handler`, `TaskResult`
- Interfaces: PascalCase, noun or verb+er — e.g., `TaskHandler`, `StepHandler`
- Constants: PascalCase with domain prefix — e.g., `TaskStatusPending`, `ContextKeyTenantID`
- Functions: PascalCase (exported), camelCase (unexported) — e.g., `New()`, `RegisterHandler()`, `executeTask()`

**Context keys:**
- Typed string alias to avoid collisions: `type contextKey string`
- Constants named `ContextKey{Name}` — e.g., `ContextKeyTenantID`, `ContextKeyUserID`

### Code Style

**Formatting:**
- `go fmt ./...` via Makefile (`make fmt-go`)
- `golangci-lint run` via Makefile (`make lint-go`)

**Constructor Pattern:**
- Constructor named `New(deps...) (*Type, error)` — e.g., `func New(logger *zap.Logger, cfg *config.Config) (*Executor, error)`
- Takes concrete dependencies, not interfaces (except `*zap.Logger`)

**Struct Tags:**
- JSON struct tags with `json:"field_name"` — snake_case field names
- Omitempty on optional fields: `json:"field,omitempty"`

### Logging (Go)

Use `go.uber.org/zap` with structured fields:
```go
e.logger.Info("Starting task execution",
    zap.String("task_id", task.ID),
    zap.String("type", task.Type),
    zap.String("integration_id", task.IntegrationID),
)

e.logger.Error("Task execution failed",
    zap.String("task_id", task.ID),
    zap.Error(err),
)
```

Logger is named per subsystem: `logger.Named("executor")`, `logger.Named("playbook")`

### HTTP Response Helpers (Go)

In `services/api-gateway/internal/handlers/handlers.go`, use shared helpers:
```go
func respondJSON(w http.ResponseWriter, status int, data interface{}) { ... }
func respondError(w http.ResponseWriter, status int, message string) { ... }
```

All handlers use these — do not call `json.NewEncoder` directly in handlers.

### Error Handling (Go)

- Return `error` from all fallible functions
- Wrap errors with `fmt.Errorf("context: %w", err)` for chain-able errors
- In HTTP handlers, call `respondError(w, http.StatusXXX, "message")` — not `http.Error` directly (except in middleware)

---

## Python (AI Service)

Service: `services/ai-service/`

### Naming Patterns

**Files:**
- Modules: `snake_case.py`
- Packages: `snake_case/` directories with `__init__.py`

**Classes:**
- PascalCase — e.g., `LLMClient`, `OpenAIClient`, `ChatRequest`, `ChatResponse`

**Functions:**
- snake_case — e.g., `get_llm_client()`, `init_llm_clients()`
- Private functions prefixed with `_` — e.g., `_execute_cypher()`, `_execute_sql()`

**Module-level instances:**
- Singletons prefixed with `_` — e.g., `_openai_client`, `_anthropic_client`

### Code Style

**Formatting:**
- `black` with `line-length = 100`, `target-version = ["py311"]`
- `ruff` with `line-length = 100`, rules: `E, F, I, N, W, UP`, ignoring `E501`
- `mypy` with `strict = true`, `ignore_missing_imports = true`

**Type Annotations:**
- Full type annotations required (mypy strict)
- Use `X | None` union syntax (Python 3.10+ style)
- `list[dict]` lowercase generics (Python 3.9+ style)

**Pydantic Models:**
- All API request/response types are Pydantic `BaseModel` subclasses
- Field constraints via `Field(...)` with validation — e.g., `Field(..., min_length=1, max_length=10000)`
- `default_factory=list` for mutable defaults

**Abstract Base Classes:**
```python
class LLMClient(ABC):
    @abstractmethod
    async def generate(self, messages: list[dict[str, str]], ...) -> str:
        """Generate a response from the LLM."""
        pass
```

### Logging (Python)

Use `structlog` with keyword arguments:
```python
logger = structlog.get_logger()
logger.info(
    "Processing chat request",
    conversation_id=str(conversation_id),
    context=request.context,
    message_length=len(request.message),
)
logger.error("Chat request failed", error=str(e))
```

### Error Handling (Python)

- FastAPI endpoints use `raise HTTPException(status_code=..., detail=str(e))` in broad `except Exception as e` blocks
- No custom exception hierarchy detected
- All endpoint bodies wrapped in `try/except Exception`

---

## Cross-Language Conventions

**Tenant isolation:**
- All storage operations require a `tenant_id` parameter (Rust services)
- `TenantContext` extracted from headers in Rust middleware
- Go middleware extracts from JWT claims or `X-Tenant-ID` header

**Snake case for serialized field names:**
- Rust: `#[serde(rename_all = "snake_case")]` on all enums and structs
- TypeScript: interface fields that match backend use snake_case (e.g., `trigger_type`, `step_type`, `var_type`)
- Go: `json:"field_name"` struct tags use snake_case
- Python: Pydantic field names are snake_case

**Datetime format:**
- Rust: `chrono::DateTime<Utc>`, serialized via `.to_rfc3339()` → ISO 8601 string
- TypeScript: string fields for dates (no Date objects)

---

*Convention analysis: 2026-02-28*
