# Technology Stack

**Analysis Date:** 2026-02-28

## Languages

**Primary:**
- Go 1.22 - API gateway (`services/api-gateway`), agent gateway (`services/agent-gateway`), and agent binary (`agent/`)
- Rust (edition 2021) - Five backend services: asset-service, data-service, governance-service, integration-service, ops-service, rationalization-service (`services/*/`)
- TypeScript 5.3 - Frontend web application (`packages/frontend/web-app/`) and connector SDK (`packages/sdk/typescript/`)
- Python 3.11+ - AI/LLM service (`services/ai-service/`)

**Secondary:**
- Protocol Buffers (proto3) - Agent-to-gateway gRPC contract (`proto/agent/v1/agent.proto`)
- SQL - PostgreSQL schema migrations (`schemas/postgres/`)
- Cypher - Neo4j graph schema (`schemas/neo4j/`)

## Runtime

**Environments:**
- Go: 1.22 (standard `go run`/binary)
- Rust: Tokio async runtime 1.35 (`tokio = { version = "1.35", features = ["full"] }`)
- Node.js: Not pinned; frontend served via Vite dev server on port 3000
- Python: >=3.11 with Uvicorn ASGI server

**Package Managers:**
- Go: `go mod` (lockfile: `go.sum` per module)
- Rust: Cargo (lockfile: `Cargo.lock` per service)
- Node: npm (lockfile: `package-lock.json`)
- Python: pip/hatch via `pyproject.toml` (build backend: hatchling)

## Frameworks

**Backend HTTP:**
- Axum 0.7 with Tower middleware - All Rust services (e.g., `services/integration-service/Cargo.toml`)
- chi v5 - Go HTTP router for `services/api-gateway`
- FastAPI 0.109+ with Uvicorn - Python AI service (`services/ai-service/`)

**Frontend:**
- React 18.2 - UI framework (`packages/frontend/web-app/`)
- React Router DOM 6.21 - Client-side routing
- TanStack React Query 5.17 - Server state and data fetching
- Zustand 4.4 - Client-side state management
- React Hook Form 7.49 - Form handling
- `@xyflow/react` 12.0 - Visual flow editor (integration/playbook DAG builder)
- Tailwind CSS 3.4 - Utility-first styling

**AI/ML:**
- LangChain 0.1 with `langchain-openai` and `langchain-anthropic` - LLM orchestration (`services/ai-service/`)
- OpenAI SDK 1.10 - GPT-4 chat and `text-embedding-3-small` embeddings
- Anthropic SDK 0.18 - Claude 3 Sonnet (fallback/alternative LLM)

**Inter-service Communication:**
- gRPC (google.golang.org/grpc v1.60) - Agent-to-gateway streaming (`proto/agent/v1/agent.proto`)
- Apache Kafka (IBM/sarama v1.42 in Go; rdkafka 0.36 in Rust) - Async event streaming
- HTTP/REST - Service-to-service calls (reqwest 0.11 in Rust; httpx 0.26 in Python)

**Testing:**
- Vitest 1.2 - TypeScript/frontend tests (`packages/frontend/web-app/`, `packages/sdk/typescript/`)
- Go `testing` package with `go test` - Go service tests
- Rust `cargo test` + tokio-test 0.4 - Rust async tests
- pytest 7.4 + pytest-asyncio 0.23 - Python AI service tests

**Build/Dev:**
- Vite 5.0 with `@vitejs/plugin-react` - Frontend build and dev server
- Docker Compose (Confluent Platform 7.5.0, postgres:16, neo4j:5, redis:7, minio) - Local dev infra (`infra/docker/docker-compose.yml`)
- Make - Top-level build orchestration (`Makefile`)
- protoc with `protoc-gen-go` and `protoc-gen-go-grpc` - Proto code generation

## Key Dependencies

**Critical:**
- `sqlx 0.7` (Rust, postgres feature) - Type-safe async PostgreSQL in all Rust services
- `neo4rs 0.7` (Rust) - Neo4j driver; used only in `services/asset-service/`
- `golang-jwt/jwt v5` (Go) - JWT auth in `services/api-gateway/`
- `redis/go-redis v9` (Go) - Redis caching/rate-limit in `services/api-gateway/`
- `regorus 0.2` (Rust) - OPA/Rego policy engine in `services/governance-service/`
- `arrow 50.0` (Rust) - Apache Arrow for data profiling in `services/data-service/`
- `zod 3.22` (TypeScript) - Runtime schema validation in `packages/sdk/typescript/`

**Infrastructure:**
- `rdkafka 0.36` - Kafka producer/consumer in Rust services (integration, governance, data)
- `IBM/sarama v1.42` - Kafka in Go agent-gateway
- `tower-http 0.5` (CORS, trace) - HTTP middleware across all Rust services
- `lettre 0.11` - Email sending in `services/ops-service/`
- `sha2 0.10` + `hex 0.4` - Audit hashing in governance and ops services
- `rust_decimal 1.33` - Monetary values in `services/rationalization-service/`

## Configuration

**Environment:**
- Go services: YAML config file with env variable overrides (`SYSILO_*` prefix)
  - API gateway: `SYSILO_API_ADDRESS`, `SYSILO_JWT_SECRET`, `SYSILO_DB_*`, `SYSILO_REDIS_ADDRESS`
  - Agent: `SYSILO_AGENT_ID`, `SYSILO_TENANT_ID`, `SYSILO_GATEWAY_ADDRESS`
- Python AI service: `pydantic-settings` loading from `.env` file
  - Required: `OPENAI_API_KEY` or `ANTHROPIC_API_KEY`
  - Connection: `DATABASE_URL`, `REDIS_URL`, `NEO4J_URI`, `NEO4J_USER`, `NEO4J_PASSWORD`
- Rust services: `config 0.14` crate for layered config
- Billing: `STRIPE_SECRET_KEY`, `STRIPE_WEBHOOK_SECRET` (read via `os.Getenv` in api-gateway)

**Build:**
- `tsconfig.json` and `tsconfig.node.json` - TypeScript compiler options (`packages/frontend/web-app/`)
- `vite.config.ts` - Build config, path alias `@` → `./src`, dev proxy `/api` → `http://localhost:8081`
- `tailwind.config.js` + `postcss.config.js` - CSS tooling
- `Makefile` - Orchestrates `go build`, `cargo build --release`, proto generation

## Platform Requirements

**Development:**
- Docker + Docker Compose (for all backing services)
- Go 1.22+
- Rust toolchain (cargo, rustup)
- Node.js (npm)
- Python 3.11+
- protoc + protoc-gen-go + protoc-gen-go-grpc (for proto changes)
- golangci-lint (Go linting)

**Production:**
- No cloud provider locked in (Terraform and Kubernetes dirs exist but are empty: `infra/terraform/`, `infra/kubernetes/`)
- Services ship as compiled binaries (Go) or Cargo release builds (Rust) or Docker containers

---

*Stack analysis: 2026-02-28*
