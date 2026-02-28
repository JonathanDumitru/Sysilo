# External Integrations

**Analysis Date:** 2026-02-28

## APIs & External Services

**AI / LLM Providers:**
- OpenAI - Chat completions (GPT-4 Turbo) and embeddings (`text-embedding-3-small`)
  - SDK/Client: `openai>=1.10.0` (`services/ai-service/src/ai_service/llm/clients.py`)
  - Auth: `OPENAI_API_KEY` env var
  - Default model: `gpt-4-turbo-preview`; configurable via `DEFAULT_MODEL` env var
- Anthropic - Claude 3 Sonnet for chat completions (fallback/alternative provider)
  - SDK/Client: `anthropic>=0.18.0` (`services/ai-service/src/ai_service/llm/clients.py`)
  - Auth: `ANTHROPIC_API_KEY` env var
  - Note: Embeddings not supported; always falls back to OpenAI for embeddings

**Billing:**
- Stripe - Subscription billing, checkout sessions, customer portal
  - SDK/Client: Direct HTTP calls to Stripe API (no Stripe Go SDK; raw `net/http` in `services/api-gateway/internal/handlers/billing_handlers.go`)
  - Auth: `STRIPE_SECRET_KEY` env var
  - Webhook secret: `STRIPE_WEBHOOK_SECRET` env var
  - Webhook events handled: `checkout.session.completed`, `customer.subscription.updated`, `customer.subscription.deleted`, `invoice.payment_failed`, `invoice.paid`
  - Note: Stripe API calls are stubbed in dev mode when `STRIPE_SECRET_KEY` is empty

## Data Storage

**Databases:**
- PostgreSQL 16 - Primary relational database for all services
  - Connection: Go services use DSN `host/port/user/password/dbname/sslmode` from YAML + env overrides (`SYSILO_DB_*`)
  - Python AI service: `DATABASE_URL` env var (`postgresql+asyncpg://...`)
  - Client (Go): `database/sql` with `lib/pq` driver
  - Client (Rust): `sqlx 0.7` with async postgres feature
  - Client (Python): `SQLAlchemy 2.0` + `asyncpg 0.29`
  - Schemas: 9 migration files in `schemas/postgres/` (001_initial through 009_billing)
  - Extensions used: `uuid-ossp`, `pgcrypto`

- Neo4j 5 Community - Graph database for technology asset relationships
  - Connection: Bolt protocol on port 7687; env vars `NEO4J_URI`, `NEO4J_USER`, `NEO4J_PASSWORD`
  - Client (Rust): `neo4rs 0.7` in `services/asset-service/`
  - Client (Python): `neo4j>=5.15.0` in `services/ai-service/`
  - Schema: `schemas/neo4j/001_constraints.cypher`
  - Used by: asset-service (graph traversal, impact analysis, relationship management) and ai-service (context for AI recommendations)
  - Docker: `neo4j:5-community` with APOC plugin enabled

**Message Streaming:**
- Apache Kafka (Confluent Platform 7.5.0) - Event streaming between services
  - Connection: `localhost:9092` (external), `kafka:29092` (internal Docker)
  - Zookeeper: required (Confluent cp-zookeeper:7.5.0)
  - Client (Go agent-gateway): `IBM/sarama v1.42`
  - Client (Rust services): `rdkafka 0.36`
  - Used by: agent-gateway (task dispatch to agents), integration-service (task results), data-service (streaming ingestion), governance-service (policy events)
  - Dev tooling: `provectuslabs/kafka-ui` on port 8080

**File Storage:**
- MinIO (S3-compatible) - Object storage for binary/file data
  - API port: 9000; Console port: 9001
  - Docker: `minio/minio:latest`
  - Auth env vars: `MINIO_ROOT_USER`, `MINIO_ROOT_PASSWORD`
  - Note: No application-level S3/MinIO SDK found yet; infra is provisioned but client integration may be pending

**Caching:**
- Redis 7 - Caching and rate limiting
  - Connection (Go): `redis/go-redis v9.4` in `services/api-gateway/`; env `SYSILO_REDIS_ADDRESS`, `SYSILO_REDIS_PASSWORD`
  - Connection (Python): `redis>=5.0.0` in `services/ai-service/`; env `REDIS_URL`
  - Used for: response caching in AI service (TTL 300s), rate-limit enforcement in API gateway (TODO: not fully implemented yet)

## Authentication & Identity

**Auth Provider:**
- Custom JWT - Self-managed authentication; no external identity provider
  - Implementation: HMAC-signed JWTs validated in `services/api-gateway/internal/middleware/middleware.go`
  - Library: `golang-jwt/jwt v5` in api-gateway
  - JWT secret: `SYSILO_JWT_SECRET` env var
  - Token claims: `sub` (user ID), `tenant_id`, `roles` (array)
  - Token expiry: configurable, default 60 minutes
  - Multi-tenancy: tenant isolation enforced via `X-Tenant-ID` context header

## Monitoring & Observability

**Error Tracking:**
- Not detected - No Sentry, Datadog, or similar SDK integrated

**Logging:**
- Go services: `go.uber.org/zap v1.26` structured JSON logging
- Rust services: `tracing 0.1` + `tracing-subscriber 0.3` with JSON format and env-filter
- Python AI service: `structlog>=24.1` + `python-json-logger>=2.0`
- All services emit structured JSON logs; format controlled by `LOG_LEVEL` / `SYSILO_LOG_LEVEL`

**Metrics/Tracing:**
- Not detected at instrumentation level - no OpenTelemetry, Prometheus, or Jaeger SDKs found

## CI/CD & Deployment

**Hosting:**
- No cloud provider locked in; `infra/terraform/` and `infra/kubernetes/` directories exist but are empty

**CI Pipeline:**
- Not detected - `.github/workflows/` directory is empty

**Build System:**
- GNU Make (`Makefile`) orchestrates all build, test, lint, and proto generation tasks
- Key targets: `dev-up`, `build`, `test`, `lint`, `proto`, `db-migrate`

## Webhooks & Callbacks

**Incoming:**
- Stripe Webhooks: `POST /webhooks/stripe` (handled in `services/api-gateway/internal/handlers/billing_handlers.go`)
  - Events: `checkout.session.completed`, `customer.subscription.updated`, `customer.subscription.deleted`, `invoice.payment_failed`, `invoice.paid`
  - Signature verification via `STRIPE_WEBHOOK_SECRET` (stubbed - `TODO` comment in code)

**Outgoing (Notification Channels):**
- Slack: Incoming webhook URL (configured per channel); sends attachment-style messages (`services/ops-service/src/notifications/mod.rs`)
- Microsoft Teams: Incoming webhook URL; sends MessageCard format
- PagerDuty: Events API v2 at `https://events.pagerduty.com/v2/enqueue`; uses routing key
- Generic Webhook: Arbitrary HTTP POST endpoint with JSON payload
- Email: `lettre 0.11` (`tokio1-native-tls` feature) in ops-service; implementation is a placeholder stub
- OpsGenie: Channel type defined in enum but no implementation found

## Internal Service Communication

**gRPC (bidirectional streaming):**
- Agent ↔ Agent-Gateway: `proto/agent/v1/agent.proto` defines `AgentService`
  - `rpc Connect(stream AgentMessage) returns (stream GatewayMessage)` - persistent bidirectional stream
  - `rpc ReportTaskResult(TaskResult) returns (TaskResultAck)` - unary result reporting
  - Task types: `QUERY`, `API_CALL`, `FILE_TRANSFER`, `DISCOVERY`, `HEALTH_CHECK`

**HTTP (internal REST):**
- Frontend → API Gateway: proxied via Vite dev server (`/api` → `http://localhost:8081`); production URL via `VITE_API_URL` env var
- API Gateway → Integration Service: `SYSILO_INTEGRATION_SERVICE_ADDRESS` env var (default `localhost:8085`)
- API Gateway → Agent Gateway: `SYSILO_AGENT_GATEWAY_ADDRESS` env var (default `localhost:8082`)
- Rationalization Service → AI Service: HTTP calls for AI analysis (`reqwest 0.11`)
- Integration Service → Asset Service: Kafka consumer posts results to asset-service URL (configured via `consumer.asset_service_url`)

## Environment Configuration

**Required env vars by service:**

api-gateway:
- `SYSILO_JWT_SECRET` - JWT signing secret
- `SYSILO_DB_HOST`, `SYSILO_DB_PORT`, `SYSILO_DB_USER`, `SYSILO_DB_PASSWORD`, `SYSILO_DB_NAME`
- `SYSILO_REDIS_ADDRESS`
- `STRIPE_SECRET_KEY`, `STRIPE_WEBHOOK_SECRET` (optional; billing disabled if absent)

agent:
- `SYSILO_AGENT_ID` - Unique agent identifier
- `SYSILO_TENANT_ID` - Tenant this agent belongs to
- `SYSILO_GATEWAY_ADDRESS` - gRPC gateway address

ai-service:
- `OPENAI_API_KEY` and/or `ANTHROPIC_API_KEY` (at least one required)
- `DATABASE_URL`, `REDIS_URL`, `NEO4J_URI`, `NEO4J_USER`, `NEO4J_PASSWORD`

**Secrets location:**
- `.env` file (gitignored) for local development
- YAML config files in `config/` directory (gitignored) for service-level config

---

*Integration audit: 2026-02-28*
