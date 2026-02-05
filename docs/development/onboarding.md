# Developer onboarding

## Intent

Provide a concrete, repeatable setup path for local development.

## Prerequisites

- Go 1.22 (required by `go.mod` for agent and gateways)
- Rust toolchain (stable) for Rust services
- Node.js (LTS recommended) for frontend and SDK
- Docker and Docker Compose for local dependencies
- `protoc` for protobuf generation
- `make`, `git`

## First-time setup

1. Install prerequisites above.
2. Initialize the repo scaffolding:
   ```bash
   make init
   ```
   This creates `bin/` and `config/` and installs Go tools used by `make`.
3. Start local infrastructure (Postgres, Neo4j, Redis, Kafka, MinIO):
   ```bash
   make dev-up
   ```
4. Run database migrations:
   ```bash
   make db-migrate
   ```
5. Create service configs as needed. See `docs/development/configuration.md`.
6. Build services:
   ```bash
   make build
   ```

## Run services

### Go services (agent, agent-gateway, api-gateway)

```bash
make run-agent
make run-agent-gateway
make run-api-gateway
```

Each reads a YAML config file. See `docs/development/configuration.md`.

### Rust services (integration-service, data-service, asset-service, ops-service, governance-service)

From each service directory:

```bash
cargo run
```

Environment variables are required for database and Kafka configuration. See `docs/development/configuration.md`.

## Frontend

```bash
cd packages/frontend/web-app
npm install
npm run dev
```

The dev server defaults to `http://localhost:3000`.

## Local discovery testing

There are two paths:

1. Mock discovery (dev only). The UI calls `POST /dev/mock-discovery` on the integration-service, bypassing Kafka and writing assets directly to the Asset Service. The modal uses mock mode by default. To exercise real discovery, set `USE_MOCK_DISCOVERY = false` in `packages/frontend/web-app/src/components/DiscoveryModal.tsx`.
2. Real discovery. The UI calls `POST /discovery/run`, which dispatches a `discovery` task to Kafka. This requires Kafka and an agent capable of discovery tasks.

Example mock request:

```bash
curl -X POST http://localhost:8082/dev/mock-discovery \
  -H 'Content-Type: application/json' \
  -H 'X-Tenant-ID: dev-tenant' \
  -d '{"connection_id":"00000000-0000-0000-0000-000000000001","asset_count":5}'
```

Notes:

- Mock discovery requires the Asset Service to be running and `CONSUMER_ASSET_SERVICE_URL` pointing at it (or the API gateway).
- The connection list in the modal is stubbed in `packages/frontend/web-app/src/services/discovery.ts` until the connections API is implemented.
- Ensure tenant IDs match between discovery and asset queries. The defaults live in `packages/frontend/web-app/src/services/discovery.ts` and `packages/frontend/web-app/src/services/assets.ts`.

## Connector SDK (TypeScript)

```bash
cd packages/sdk/typescript
npm install
npm run build
```

## Local dependency ports

- Postgres: `localhost:5432`
- Neo4j: `localhost:7474` (HTTP), `localhost:7687` (Bolt)
- Redis: `localhost:6379`
- Kafka: `localhost:9092`
- Kafka UI: `localhost:8080`
- MinIO: `localhost:9000` (API), `localhost:9001` (console)

## Common tasks

```bash
make test
make lint
make fmt
make proto
```

## Troubleshooting

- If `make db-migrate` fails, confirm Docker containers are healthy and `sysilo-postgres` is running.
- If a service fails to start, check its config/env vars and verify ports are free.
