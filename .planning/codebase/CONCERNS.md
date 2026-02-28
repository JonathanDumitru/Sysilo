# Codebase Concerns

**Analysis Date:** 2026-02-28

## Tech Debt

**Agent-to-API-Gateway Communication Not Wired:**
- Issue: `RunIntegration` creates a DB run record but never dispatches it to the agent-gateway. `CancelRun` marks DB state cancelled but never sends the cancellation downstream.
- Files: `services/api-gateway/internal/handlers/handlers.go:458`, `services/api-gateway/internal/handlers/handlers.go:513`
- Impact: Triggering an integration via the API does nothing — agents never receive work. Cancel has no effect on in-flight agent tasks.
- Fix approach: Implement HTTP or gRPC client call from api-gateway to agent-gateway (or Kafka producer) when creating a run. Wire up `Services.AgentGateway` config field which is defined but never instantiated as a client.

**TestConnection Always Returns Success:**
- Issue: `TestConnection` handler simulates success without actually routing a test task to the agent.
- Files: `services/api-gateway/internal/handlers/handlers.go:276`
- Impact: Users receive false confirmation that connections work. Failed connections appear healthy.
- Fix approach: Dispatch a `health_check` task to the agent that owns the connection and await the result.

**Rate Limiting Is a Stub:**
- Issue: The `RateLimit` middleware is fully stubbed — the config is read but no limiting logic executes. Redis is imported and configured but unused in this middleware.
- Files: `services/api-gateway/internal/middleware/middleware.go:244-258`
- Impact: No request throttling in production. A single misbehaving tenant can saturate the API.
- Fix approach: Implement sliding-window rate limiting using the already-configured Redis client (`RedisConfig`), keyed by `tenant_id`.

**data-service Ingestion Is a `todo!()` Stub:**
- Issue: All four core `IngestionService` methods (`start_job`, `get_job_status`, `cancel_job`, `process_batch`) call `todo!()` which will panic at runtime if invoked.
- Files: `services/data-service/src/ingestion/mod.rs:138,147,156,172`
- Impact: Any API call routing to ingestion will crash the data-service process.
- Fix approach: Implement real job execution or return a structured error (`NotImplemented`) until the feature is ready; replace `todo!()` with `Err(anyhow::anyhow!("not implemented"))`.

**WaitGroup Never Incremented in Executor Shutdown:**
- Issue: `Executor.Shutdown` calls `e.wg.Wait()` but `wg.Add(1)` and `wg.Done()` are never called anywhere in the executor. The WaitGroup is always zero so shutdown completes immediately without waiting for running tasks.
- Files: `agent/internal/executor/executor.go:341-366`
- Impact: Graceful shutdown does not actually drain in-flight tasks; tasks are cancelled but the shutdown goroutine returns before they finish, potentially corrupting task results.
- Fix approach: Add `e.wg.Add(1)` at the start of each task goroutine and `defer e.wg.Done()` before returning from `executeTask`.

**Stripe Billing Integration Is Incomplete:**
- Issue: `CreateCheckoutSession` returns the Stripe request configuration JSON rather than actually calling the Stripe API. `CreatePortalSession` also returns a stub response when `STRIPE_SECRET_KEY` is set but does nothing. Webhook handlers `handleSubscriptionUpdated` and `handleSubscriptionDeleted` log events but perform no state changes.
- Files: `services/api-gateway/internal/handlers/billing_handlers.go:113-116`, `billing_handlers.go:144-147`, `billing_handlers.go:281-302`
- Impact: No actual billing is processed. Plan upgrades can't complete in production.
- Fix approach: Use the Stripe Go SDK (`github.com/stripe/stripe-go`) to create real checkout and portal sessions; implement subscription state updates in the webhook handlers.

**Stripe Webhook Signature Not Verified:**
- Issue: Webhook handler reads and processes the event body without verifying the `Stripe-Signature` header. The verification code is commented out.
- Files: `services/api-gateway/internal/handlers/billing_handlers.go:176-177`
- Impact: Any external actor can forge Stripe events (fake subscription upgrades, cancellations) by sending requests to the webhook endpoint.
- Fix approach: Uncomment and implement signature verification using `STRIPE_WEBHOOK_SECRET` with Stripe's Go SDK `webhook.ConstructEvent`.

**Hardcoded Dev Tenant IDs in Frontend:**
- Issue: Frontend service modules hardcode development tenant IDs (`'dev-tenant'`, `'00000000-0000-0000-0000-000000000001'`) as `X-Tenant-ID` header values with no auth token attached to requests.
- Files: `packages/frontend/web-app/src/services/connections.ts:3`, `packages/frontend/web-app/src/services/assets.ts:4`, `packages/frontend/web-app/src/services/discovery.ts:19`
- Impact: The frontend has no real authentication flow. All API calls bypass JWT auth and use a hardcoded tenant, making multi-tenancy non-functional from the UI.
- Fix approach: Implement an auth context (JWT storage, token refresh) and inject `Authorization: Bearer <token>` headers from authenticated user state. Remove `DEV_TENANT_ID` constants.

**Majority of Frontend Pages Use Mock Data:**
- Issue: 13 pages render entirely from local static arrays, not live API data.
- Files: `packages/frontend/web-app/src/pages/AlertsPage.tsx`, `ApprovalsPage.tsx`, `AuditLogPage.tsx`, `GovernanceDashboardPage.tsx`, `IncidentsPage.tsx`, `OperationsDashboardPage.tsx`, `PlaybooksPage.tsx`, `PoliciesPage.tsx`, `ProjectsPage.tsx`, `RationalizationDashboardPage.tsx`, `ScenariosPage.tsx`, `StandardsPage.tsx`, `ApplicationPortfolioPage.tsx`
- Impact: These pages cannot be used in production. Data shown is disconnected from actual system state.
- Fix approach: Wire each page to its corresponding backend service using the existing API client pattern (`apiFetch` + React Query hooks). Services exist in backend but aren't called.

**Run State Kept Only In Memory (Integration Engine):**
- Issue: `Engine.active_runs` is an in-memory `HashMap`. There is no persistence to PostgreSQL.
- Files: `services/integration-service/src/engine/mod.rs:131-133`
- Impact: All in-flight run state is lost on service restart. Run history is not queryable. The api-gateway `integration_runs` table exists but integration-service never writes to it.
- Fix approach: Persist run state via the `Storage` layer on create/update/complete. Use the `storage/mod.rs` pattern already established in the integration service.

---

## Known Bugs

**CORS `Access-Control-Max-Age` Header Encodes Wrong Value:**
- Symptoms: Preflight responses send garbled `Access-Control-Max-Age` values (e.g., `∞` or `?` instead of `86400`).
- Files: `services/api-gateway/internal/middleware/middleware.go:80`
- Trigger: Any CORS preflight (`OPTIONS`) request.
- Root cause: `string(rune(cfg.MaxAge))` converts the integer `86400` as a Unicode code point, producing a Unicode character, not the decimal string `"86400"`.
- Fix: `w.Header().Set("Access-Control-Max-Age", strconv.Itoa(cfg.MaxAge))`

**Kafka Consumer Uses Auto-Commit (At-Most-Once Delivery):**
- Symptoms: If integration-service crashes mid-processing, the message offset is already committed and the result is silently dropped.
- Files: `services/integration-service/src/consumer/mod.rs:50`
- Trigger: Service crash or panic between receiving a Kafka message and completing `process_message`.
- Fix approach: Set `enable.auto.commit` to `false` and call `consumer.commit_message()` only after successful processing.

---

## Security Considerations

**Agent Registration Has No Authentication:**
- Risk: Any process that can reach the agent-gateway gRPC port can register as an agent for any tenant and receive tasks, including tasks containing database credentials.
- Files: `services/agent-gateway/internal/tunnel/server.go:78-82`
- Current mitigation: Code comment suggests mTLS or API verification; neither is implemented.
- Recommendations: Implement pre-shared agent token validation against the api-gateway on registration, or enable mTLS with per-tenant certificates. The TODO comment on line 80 acknowledges this gap.

**gRPC Server Has No TLS:**
- Risk: All agent-to-gateway communication (including task configs that contain credentials) is transmitted in plaintext.
- Files: `services/agent-gateway/cmd/agent-gateway/main.go:89-91`
- Current mitigation: None — the TODO comment at line 89 marks this as required for production.
- Recommendations: Configure `grpc.Creds(credentials.NewTLS(tlsConfig))` server option before production deployment.

**CORS Wildcard `*` Configured in Multiple Rust Services:**
- Risk: All Rust microservices (asset, data, ops, integration, governance) use `CorsLayer::permissive()` which sets `Access-Control-Allow-Origin: *` and allows all methods/headers.
- Files: `services/asset-service/src/main.rs:93`, `services/data-service/src/main.rs:80`, `services/ops-service/src/main.rs:102`, `services/integration-service/src/main.rs:173`, `services/governance-service/src/main.rs:120`
- Current mitigation: Services are not directly browser-accessible in production (api-gateway proxy intent), but this is not enforced.
- Recommendations: Replace `CorsLayer::permissive()` with an allowlist matching the api-gateway origin; restrict methods to those actually used.

**AI Service CORS Wildcard:**
- Risk: `allow_origins=["*"]` on the AI service allows any origin to make credentialed requests.
- Files: `services/ai-service/src/ai_service/main.py:70`
- Current mitigation: Code comment acknowledges this needs production configuration.
- Recommendations: Restrict to known frontend origins.

**AI Service Executes Arbitrary SQL/Cypher Queries:**
- Risk: The `/chat/query` endpoint accepts a natural language prompt, generates a query with an LLM, and optionally executes it directly against PostgreSQL or Neo4j. The generated query is not sandboxed, reviewed, or limited in scope.
- Files: `services/ai-service/src/ai_service/api/chat.py:144-180`
- Current mitigation: `execute: bool` flag must be explicitly set. No query allow-listing or read-only enforcement.
- Recommendations: Always use read-only database connections for AI-executed queries; add an explicit SQL/Cypher allow-list or only allow SELECT/MATCH statements; require elevated tenant permissions for query execution.

**Default JWT Secret Is a Known Plaintext Value:**
- Risk: The default config ships `JWTSecret: "dev-secret-change-in-production"`. If `SYSILO_JWT_SECRET` env var is not set, this value is used. Any token signed with this secret is valid.
- Files: `services/api-gateway/internal/config/config.go:113`
- Current mitigation: Documentation says to change it. No startup guard enforces this.
- Recommendations: Add startup validation that rejects the default secret in non-development environments (`if cfg.Auth.JWTSecret == "dev-secret-change-in-production" && env != "dev" { fatal }`)

**PostgreSQL Adapter Credentials Travel in Task Payload:**
- Risk: Database credentials (host, user, password) are embedded as plaintext in the task `config` JSON that flows through Kafka, is stored in the `tasks` table, and is transmitted over the gRPC tunnel.
- Files: `agent/internal/adapters/postgresql/adapter.go:75-82`, `agent/internal/executor/executor.go:24-31`
- Current mitigation: `Connection` model has `credentials_encrypted []byte` column but it is never populated from the API — `CreateConnectionInput` receives `Config map[string]interface{}` which includes plaintext credentials.
- Recommendations: Store credentials encrypted at rest (AES-256-GCM via a KMS key); only pass a connection reference ID in task configs; decrypt credentials in the agent just before use.

**X-Tenant-ID Header Accepted as Fallback for Unauthenticated Requests:**
- Risk: `TenantContext` middleware falls back to accepting `X-Tenant-ID` from the raw HTTP header when no JWT tenant claim is present. This allows service-to-service callers (or external callers who bypass JWT) to assert any tenant identity.
- Files: `services/api-gateway/internal/middleware/middleware.go:162-165`
- Current mitigation: JWT auth middleware runs before `TenantContext` on protected routes. Unprotected routes would accept the header.
- Recommendations: Remove the `X-Tenant-ID` header fallback, or restrict it to requests from known internal IP ranges / with a separate internal service token.

---

## Performance Bottlenecks

**No Connection Pooling for Agent's PostgreSQL Adapter:**
- Problem: `postgresql/adapter.go` opens a new `*sql.DB` per task execution and closes it when the task finishes. `sql.Open` does not actually open a connection but the subsequent `Ping` does — this incurs a new TCP connection per task.
- Files: `agent/internal/adapters/postgresql/adapter.go:46-50`, `agent/internal/adapters/postgresql/adapter.go:109-137`
- Cause: Adapter design creates connection objects inside `Execute()` which is called per task.
- Improvement path: Maintain a pool of `*sql.DB` keyed by connection DSN, with LRU eviction. Reuse connections across tasks to the same database.

**No Redis Client Instantiated in API Gateway:**
- Problem: Redis is configured (`RedisConfig`) and the `go-redis` dependency is in `go.mod`, but no Redis client is created in `main.go` or used anywhere. Rate limiting is a stub, session caching is absent.
- Files: `services/api-gateway/internal/config/config.go:49-53`, `services/api-gateway/cmd/api-gateway/main.go`
- Cause: Rate limiting was declared and deferred; Redis client was never wired in.
- Improvement path: Initialize `redis.NewClient` in `main.go` and pass it to `RateLimit` and any future caching middleware.

**Plan Feature Check Deserializes JSON on Every Request:**
- Problem: `hasFeature()` calls `json.Unmarshal` on the `plan_features` JSON blob on every request that hits `PlanGate`. With concurrent traffic, this creates repeated allocations per request.
- Files: `services/api-gateway/internal/middleware/plan_gate.go:87-110`
- Cause: `json.RawMessage` is stored in context and unmarshalled in the gate function rather than being pre-parsed at middleware load time.
- Improvement path: Parse `json.RawMessage` into `map[string]interface{}` once in `SetPlanContext` and store the parsed map in context instead.

---

## Fragile Areas

**Integration Engine State is In-Memory Only:**
- Files: `services/integration-service/src/engine/mod.rs:130-137`
- Why fragile: Service restart loses all run state. `active_runs` HashMap grows without bound unless `cleanup_completed_runs` is called periodically (no periodic call is wired).
- Safe modification: Any change to how runs progress must also add database persistence before this can be relied upon.
- Test coverage: No tests for engine state management.

**Playbook Step Type Serialization Workaround:**
- Files: `agent/internal/adapters/playbook/handler.go:139-146`
- Why fragile: `extractStepType` contains a special-case hack noting that "The Rust side serializes it as JSON, so we might get `\"webhook\"` or just `webhook`". This double-encoding is a cross-language contract that is not validated at compile time.
- Safe modification: Any change to how Rust serializes `StepType` (e.g., renaming variants, changing `serde` attributes) will silently break step routing unless `extractStepType` is updated simultaneously.
- Test coverage: No tests for the Go-side deserialization behavior.

**Playbook Only Has Webhook Step Handler:**
- Files: `agent/internal/adapters/playbook/handler.go:48`
- Why fragile: Only `"webhook"` step type is registered. All other step types return an `unknown_step_type` error without panicking, but playbooks with any other step type (`integration`, `condition`, `approval`, etc.) will silently fail.
- Safe modification: Adding new step types requires registering a new `StepHandler` in `NewHandler`; forgetting to do so produces silent failures.
- Test coverage: None for step routing.

**Dynamic SQL Update Queries in Go:**
- Files: `services/api-gateway/internal/db/connections.go:155-214`, `services/api-gateway/internal/db/integrations.go` (same pattern)
- Why fragile: The `Update` methods build `SET` clauses dynamically by appending to a slice and using `fmt.Sprintf` with `argNum` counters. A mistake in the argument counter ordering would silently corrupt data by updating the wrong columns.
- Safe modification: Any new optional field added to `UpdateXInput` must carefully manage the `argNum` index. The `joinStrings` helper reimplements `strings.Join` needlessly.
- Test coverage: None.

**AI Query Execution Has No Tenant Isolation:**
- Files: `services/ai-service/src/ai_service/api/chat.py:157-180`
- Why fragile: `_execute_cypher` and `_execute_sql` run queries without any tenant scoping. A generated query with no `WHERE tenant_id = X` clause will read all tenants' data.
- Safe modification: Any feature that enables AI query execution must inject tenant context into the query or use row-level security.
- Test coverage: None.

---

## Scaling Limits

**In-Memory Agent Registry:**
- Current capacity: Unbounded agents per process, but state is lost on restart.
- Limit: A single agent-gateway instance serves all tenants. No clustering or horizontal scaling is supported — a second agent-gateway instance would have a separate registry and agents registered to one instance are invisible to the other.
- Scaling path: Move registry to Redis (pub/sub for heartbeats, hash for agent state) to enable multiple agent-gateway replicas.
- Files: `services/agent-gateway/internal/registry/registry.go`

**Single Kafka Broker in Docker Compose:**
- Current capacity: Single broker with replication factor 1.
- Limit: Any broker restart loses uncommitted messages. Not suitable for production.
- Scaling path: Increase `KAFKA_OFFSETS_TOPIC_REPLICATION_FACTOR` and add broker replicas in production Kafka config.
- Files: `infra/docker/docker-compose.yml:77-88`

---

## Dependencies at Risk

**Zookeeper-based Kafka (Legacy Mode):**
- Risk: The docker-compose uses Confluent Platform 7.5 with Zookeeper mode. Kafka 3.x deprecates Zookeeper in favor of KRaft. This is a dev environment issue but may affect production assumptions.
- Impact: Future Kafka upgrades will require KRaft migration.
- Files: `infra/docker/docker-compose.yml:57-88`
- Migration plan: Switch to KRaft-mode Kafka in next infrastructure iteration.

**`lib/pq` PostgreSQL Driver (Deprecated):**
- Risk: `agent/internal/adapters/postgresql/adapter.go` and `services/api-gateway/internal/db/db.go` use `github.com/lib/pq` which is in maintenance mode. The recommended replacement is `github.com/jackc/pgx`.
- Impact: No critical security issues currently, but `lib/pq` will not receive new features; `pgx` has better performance and active development.
- Files: `agent/internal/adapters/postgresql/adapter.go:6`, `services/api-gateway/internal/db/db.go:9`
- Migration plan: Replace with `pgx/v5` stdlib adapter; the query interface is compatible.

**Hardcoded Anthropic Model Version:**
- Risk: `claude-3-sonnet-20240229` is hardcoded in the Anthropic client implementation. This model version may be deprecated.
- Files: `services/ai-service/src/ai_service/llm/clients.py:139,165`
- Impact: AI service will fail to generate responses if Anthropic deprecates this specific model version.
- Migration plan: Promote model name to a config setting (`anthropic_model: str = "claude-3-sonnet-20240229"`) in `Settings` so it can be updated without a code change.

---

## Missing Critical Features

**No Authentication Flow in Frontend:**
- Problem: The frontend has no login page, token storage, or auth context. Every API call uses a hardcoded `dev-tenant` ID. JWT middleware on the backend expects a valid `Authorization: Bearer <token>` header.
- Blocks: Multi-tenant usage, any production deployment, feature-gating by plan tier.

**No Integration Scheduling:**
- Problem: The `Integration` model has a `schedule` JSON field and the config accepts schedule configuration, but no scheduler (cron, job queue) is implemented in any service.
- Blocks: Automated integration runs. All runs must be triggered manually.

**No Trial Expiration Enforcement:**
- Problem: `GetExpiredTrials` query exists in `plans.go` to find expired trial tenants but nothing calls it. No background job enforces trial expiration.
- Files: `services/api-gateway/internal/db/plans.go:256-274`
- Blocks: Monetization flow. Trial tenants keep full access indefinitely.

**No Dockerfile for Go Services:**
- Problem: Only the Python AI service has a `Dockerfile`. The Go services (`api-gateway`, `agent-gateway`) and Rust services have no Dockerfiles, only appearing in `docker-compose.yml` as infrastructure dependencies (not as built services).
- Blocks: Containerized production deployment of core services.

---

## Test Coverage Gaps

**No Tests in Go Services:**
- What's not tested: All of `services/api-gateway` and `services/agent-gateway` have zero test files (`*_test.go`). This includes auth middleware, plan gating, all HTTP handlers, and database queries.
- Files: All files under `services/api-gateway/`, `services/agent-gateway/`
- Risk: Regressions in authentication, billing, or tenant isolation go undetected.
- Priority: High

**No Tests for Agent Executor Logic:**
- What's not tested: Task dispatch, timeout handling, cancellation, and the semaphore-based concurrency limit in `agent/internal/executor/executor.go`.
- Files: `agent/internal/executor/executor.go`
- Risk: The WaitGroup bug (never incremented) was introduced and not caught because there are no tests.
- Priority: High

**No Tests for Billing Handlers:**
- What's not tested: Stripe webhook processing, plan upgrade flow, subscription lifecycle events.
- Files: `services/api-gateway/internal/handlers/billing_handlers.go`
- Risk: Billing state corruption in production with no automated regression detection.
- Priority: High

**Integration Service Tests Are Limited:**
- What's not tested: `engine/mod.rs` (run state management, task generation, result handling), `consumer/mod.rs` (message processing, asset creation), `storage/mod.rs` (SQL queries).
- Files: `services/integration-service/src/engine/mod.rs`, `services/integration-service/src/consumer/mod.rs`
- Risk: Task dispatch chain and run lifecycle have no automated verification.
- Priority: High

---

*Concerns audit: 2026-02-28*
