# Configuration reference

## Intent

Document service configuration sources, defaults, and required fields.

## YAML config files (Go services)

These services accept `--config` and otherwise search default paths in order.

| Service | Default search order | Format |
| --- | --- | --- |
| Agent | `./agent.yaml`, `./config/agent.yaml`, `/etc/sysilo/agent.yaml` | YAML |
| Agent gateway | `./agent-gateway.yaml`, `./config/agent-gateway.yaml`, `/etc/sysilo/agent-gateway.yaml` | YAML |
| API gateway | `./api-gateway.yaml`, `./config/api-gateway.yaml`, `/etc/sysilo/api-gateway.yaml` | YAML |

### Agent config (`agent.yaml`)

Required: `agent.id`, `agent.tenant_id`.

```yaml
agent:
  id: "agent-001"
  name: "sysilo-agent"
  tenant_id: "tenant-123"
  max_concurrent_tasks: 10
  labels:
    env: "dev"
    region: "local"

gateway:
  address: "localhost:9090"
  reconnect_interval_seconds: 5
  heartbeat_interval_seconds: 30

tls:
  enabled: false
  cert_file: ""
  key_file: ""
  ca_cert_file: ""
  server_name: ""

logging:
  level: "info"
  format: "json"
```

Environment overrides:

| Variable | Default | Notes |
| --- | --- | --- |
| `SYSILO_AGENT_ID` | none | Overrides `agent.id` |
| `SYSILO_TENANT_ID` | none | Overrides `agent.tenant_id` |
| `SYSILO_GATEWAY_ADDRESS` | `localhost:9090` | Overrides `gateway.address` |
| `SYSILO_LOG_LEVEL` | `info` | Overrides `logging.level` |

### Agent gateway config (`agent-gateway.yaml`)

```yaml
server:
  address: ":9090"
  max_connections_per_tenant: 100
  heartbeat_timeout_seconds: 90

tls:
  enabled: false
  cert_file: ""
  key_file: ""
  ca_cert_file: ""

logging:
  level: "info"
  format: "json"
```

Environment overrides:

| Variable | Default | Notes |
| --- | --- | --- |
| `SYSILO_GATEWAY_ADDRESS` | `:9090` | Overrides `server.address` |
| `SYSILO_LOG_LEVEL` | `info` | Overrides `logging.level` |

### API gateway config (`api-gateway.yaml`)

```yaml
server:
  address: ":8081"

auth:
  jwt_secret: "dev-secret-change-in-production"
  jwt_issuer: "sysilo"
  token_expiry_minutes: 60
  allowed_issuers: []

cors:
  allowed_origins:
    - "http://localhost:3000"
  allowed_methods: ["GET", "POST", "PUT", "DELETE", "OPTIONS"]
  allowed_headers: ["Accept", "Authorization", "Content-Type", "X-Tenant-ID"]
  exposed_headers: ["X-Request-ID"]
  allow_credentials: true
  max_age: 86400

rate_limit:
  enabled: true
  requests_per_minute: 1000
  burst_size: 50

logging:
  level: "info"
  format: "json"
```

Environment overrides:

| Variable | Default | Notes |
| --- | --- | --- |
| `SYSILO_API_ADDRESS` | `:8081` | Overrides `server.address` |
| `SYSILO_JWT_SECRET` | `dev-secret-change-in-production` | Overrides `auth.jwt_secret` |
| `SYSILO_LOG_LEVEL` | `info` | Overrides `logging.level` |

## Environment variables (Rust services)

### Integration service (`services/integration-service`)

| Variable | Default | Notes |
| --- | --- | --- |
| `SYSILO_SERVER_ADDRESS` | `0.0.0.0:8082` | Bind address |
| `DATABASE_URL` | `postgres://sysilo:sysilo_dev@localhost:5432/sysilo` | Postgres connection |
| `DATABASE_MAX_CONNECTIONS` | `10` | Connection pool size |
| `KAFKA_BROKERS` | `localhost:9092` | Kafka bootstrap servers |
| `KAFKA_GROUP_ID` | `integration-service` | Consumer group |
| `KAFKA_TASK_TOPIC` | `sysilo.tasks` | Task topic |
| `KAFKA_RESULT_TOPIC` | `sysilo.results` | Result topic |
| `CONSUMER_BOOTSTRAP_SERVERS` | `localhost:9092` | Kafka bootstrap servers for the result consumer |
| `CONSUMER_GROUP_ID` | `integration-service-consumers` | Result consumer group |
| `CONSUMER_ASSET_SERVICE_URL` | `http://localhost:8082` | Base URL for asset creation from discovery results (set to the Asset Service or API Gateway in local dev) |
| `CONSUMER_ENABLED` | `true` | Enable/disable the result consumer loop |
| `ENGINE_MAX_CONCURRENT_RUNS` | `100` | Engine concurrency |
| `ENGINE_DEFAULT_TIMEOUT_SECONDS` | `300` | Task timeout |

### Data service (`services/data-service`)

| Variable | Default | Notes |
| --- | --- | --- |
| `DATABASE_URL` | `postgres://sysilo:sysilo_dev@localhost:5432/sysilo` | Postgres connection |
| `SERVER_ADDRESS` | `0.0.0.0:8083` | Bind address |

### Asset service (`services/asset-service`)

| Variable | Default | Notes |
| --- | --- | --- |
| `NEO4J_URI` | `bolt://localhost:7687` | Neo4j Bolt URI |
| `NEO4J_USER` | `neo4j` | Neo4j username |
| `NEO4J_PASSWORD` | `password` | Neo4j password |
| `DATABASE_URL` | `postgres://sysilo:sysilo_dev@localhost:5432/sysilo` | Postgres connection |
| `SERVER_ADDRESS` | `0.0.0.0:8084` | Bind address |

Note: the local Docker Compose file sets the Neo4j password to `sysilo_dev`. For local dev, set `NEO4J_PASSWORD=sysilo_dev`.

### Operations service (`services/ops-service`)

| Variable | Default | Notes |
| --- | --- | --- |
| `DATABASE_URL` | `postgres://sysilo:sysilo_dev@localhost:5432/sysilo` | Postgres connection |
| `KAFKA_BROKERS` | `localhost:9092` | Kafka bootstrap servers |
| `SERVER_ADDRESS` | `0.0.0.0:8085` | Bind address |

### Governance service (`services/governance-service`)

| Variable | Default | Notes |
| --- | --- | --- |
| `DATABASE_URL` | `postgres://sysilo:sysilo_dev@localhost:5432/sysilo` | Postgres connection |
| `KAFKA_BROKERS` | `localhost:9092` | Kafka bootstrap servers |
| `SERVER_ADDRESS` | `0.0.0.0:8086` | Bind address |

## Logging

- Go services use `logging.level` (`info` by default) and `logging.format` (`json` by default).
- Rust services read log levels from the default tracing env filter (set `RUST_LOG` to override).
