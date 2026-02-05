# Sysilo

Enterprise Integration and Data Unification Platform

## Overview

Sysilo eliminates integration chaos, unifies siloed data, and enables continuous app rationalization for enterprise IT teams.

## Repository Structure

```
sysilo/
├── agent/                   # Go - On-prem agent
├── services/
│   ├── api-gateway/         # Go - API gateway + auth
│   ├── agent-gateway/       # Go - Agent tunnel termination
│   ├── integration-service/ # Rust - Integration execution
│   ├── data-service/        # Rust - Data pipeline execution
│   ├── asset-service/       # Rust - Asset registry
│   ├── ops-service/         # Rust - Operations/monitoring
│   ├── ai-service/          # Python - AI/ML inference
│   └── governance-service/  # Rust - Policy engine
├── packages/
│   ├── frontend/            # React + TypeScript
│   └── sdk/                 # Connector SDKs
├── proto/                   # Protobuf/gRPC definitions
├── schemas/                 # Database schemas
├── connectors/              # Pre-built connectors
└── infra/                   # Infrastructure as code
```

## Tech Stack

| Layer | Technology |
|-------|------------|
| Frontend | React + TypeScript |
| API | Go |
| Services | Rust |
| Event Bus | Kafka |
| Primary DB | PostgreSQL |
| Graph DB | Neo4j |
| Cache | Redis |
| AI/ML | Python |
| Agent | Go |

## Quick Start

```bash
# Start local development environment
make dev-up

# Build all services
make build

# Run tests
make test
```

## Development

- Onboarding: `docs/development/onboarding.md`
- Configuration: `docs/development/configuration.md`
- Architecture: `docs/architecture/`
