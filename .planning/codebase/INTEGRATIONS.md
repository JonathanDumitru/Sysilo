# Sysilo External Integrations

Analysis scope: concrete integrations and integration-ready scaffolding in `/Users/dev/Documents/Software/Web/Sysilo`.

## 1) AI provider integrations
- OpenAI SDK integration in AI service (`services/ai-service/pyproject.toml`, `services/ai-service/src/ai_service/llm/clients.py`).
- Anthropic SDK integration in AI service (`services/ai-service/pyproject.toml`, `services/ai-service/src/ai_service/llm/clients.py`).
- Runtime keys/config come from env-backed settings (`services/ai-service/src/ai_service/config.py`).
- Embeddings are explicitly OpenAI-backed (`services/ai-service/src/ai_service/llm/clients.py`).

## 2) Billing integration
- Stripe integration points are defined in API Gateway billing handlers (`services/api-gateway/internal/handlers/billing_handlers.go`).
- Uses `STRIPE_SECRET_KEY` and `STRIPE_WEBHOOK_SECRET` configuration (`services/api-gateway/internal/handlers/billing_handlers.go`).
- Webhook event handling is implemented for core subscription/invoice events (`services/api-gateway/internal/handlers/billing_handlers.go`).
- Current status is partial/scaffolded: checkout/portal include "integration pending" behavior when key is unset (`services/api-gateway/internal/handlers/billing_handlers.go`).

## 3) Databases and state systems
- PostgreSQL integrated across Go/Rust/Python services (`services/api-gateway/internal/config/config.go`, `services/integration-service/src/config/mod.rs`, `services/ai-service/src/ai_service/config.py`).
- Neo4j integrated in asset-service and configured in AI service settings (`services/asset-service/src/main.rs`, `services/asset-service/Cargo.toml`, `services/ai-service/src/ai_service/config.py`).
- Redis integrated in API gateway and AI service (`services/api-gateway/go.mod`, `services/ai-service/pyproject.toml`).
- Local infra dependencies are provisioned via compose (`infra/docker/docker-compose.yml`).

## 4) Event streaming and async integration
- Kafka integration in agent-gateway (Sarama) for task result/log forwarding (`services/agent-gateway/go.mod`, `services/agent-gateway/internal/kafka/consumer.go`, `services/agent-gateway/internal/kafka/producer.go`).
- Kafka integration in Rust services via `rdkafka` (`services/integration-service/Cargo.toml`, `services/data-service/Cargo.toml`, `services/governance-service/Cargo.toml`).
- Topic and broker config are env-driven (`services/integration-service/src/config/mod.rs`, `services/agent-gateway/internal/config/config.go`).

## 5) Agent/protocol integration surface
- gRPC service contract for remote agents is defined in protobuf (`proto/agent/v1/agent.proto`).
- Agent gateway implements stream handling for registration, heartbeat, task/result/log exchange (`services/agent-gateway/internal/tunnel/server.go`).
- Agent runtime consumes gateway contract via shared proto module (`agent/go.mod`, `agent/internal/tunnel/client.go`).

## 6) Outbound notification integrations (Operations service)
- Supported notification channels include Slack, Webhook, PagerDuty, Teams, Email/OpsGenie enum entries (`services/ops-service/src/notifications/mod.rs`).
- Slack and Teams use incoming webhook URLs (`services/ops-service/src/notifications/mod.rs`).
- PagerDuty targets Events API v2 enqueue endpoint (`services/ops-service/src/notifications/mod.rs`).
- Generic webhook posts JSON payloads to arbitrary URLs (`services/ops-service/src/notifications/mod.rs`).
- Email path is currently placeholder/stub despite `lettre` dependency (`services/ops-service/src/notifications/mod.rs`, `services/ops-service/Cargo.toml`).
- OpsGenie appears declared but not implemented in dispatch match paths (`services/ops-service/src/notifications/mod.rs`).

## 7) Storage and connector-adjacent integrations
- MinIO/S3-compatible object storage is provisioned in infra (`infra/docker/docker-compose.yml`).
- UI and docs expose S3/Salesforce/Snowflake/MySQL/PostgreSQL/AWS/Azure as connector targets/sources (`packages/frontend/web-app/src/components/studio/NodeToolbox.tsx`, `services/integration-service/src/api/mod.rs`, `docs/integration/integration-studio.md`).
- `connectors/` has no active connector source files checked in at present; practical implementation entrypoints are the TypeScript SDK and integration service APIs (`packages/sdk/typescript/src/index.ts`, `services/integration-service/src/connections/api.rs`).

## 8) Auth/integration security context
- API gateway uses JWT-based auth middleware and configurable secret/issuer (`services/api-gateway/internal/middleware/middleware.go`, `services/api-gateway/internal/config/config.go`).
- Multi-tenant context is propagated through tenant-aware handlers and middleware (`services/api-gateway/internal/middleware/middleware.go`, `services/api-gateway/internal/handlers/handlers.go`).
- Agent gateway notes mTLS-oriented authorization model as production intent (`services/agent-gateway/internal/tunnel/server.go`).

## 9) Practical integration status summary
- Fully wired integrations: PostgreSQL, Neo4j, Redis, Kafka, OpenAI/Anthropic clients, gRPC agent protocol.
- Partially wired/scaffolded integrations: Stripe checkout/portal execution, notification email/OpsGenie, S3/connector runtime implementations.
- Local developer integration environment is consistently documented and runnable through compose + make (`Makefile`, `infra/docker/docker-compose.yml`, `docs/development/configuration.md`).
