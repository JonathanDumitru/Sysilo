# Sysilo Platform - Build Orchestration
# =====================================

.PHONY: all build test lint clean dev-up dev-down proto help

# Default target
all: build

# =====================================
# Development Environment
# =====================================

## Start local development environment (databases, kafka, etc.)
dev-up:
	@echo "Starting development environment..."
	docker compose -f infra/docker/docker-compose.yml up -d
	@echo "Waiting for services to be healthy..."
	@sleep 5
	@echo "Development environment is ready!"
	@echo "  PostgreSQL: localhost:5432"
	@echo "  Neo4j:      localhost:7474 (browser) / localhost:7687 (bolt)"
	@echo "  Redis:      localhost:6379"
	@echo "  Kafka:      localhost:9092"
	@echo "  Kafka UI:   localhost:8080"
	@echo "  MinIO:      localhost:9000 (api) / localhost:9001 (console)"

## Stop local development environment
dev-down:
	@echo "Stopping development environment..."
	docker compose -f infra/docker/docker-compose.yml down

## Stop and remove all data volumes
dev-clean:
	@echo "Cleaning development environment..."
	docker compose -f infra/docker/docker-compose.yml down -v

## View logs from development environment
dev-logs:
	docker compose -f infra/docker/docker-compose.yml logs -f

# =====================================
# Build
# =====================================

## Build all services
build: build-agent build-agent-gateway build-api-gateway build-integration-service

## Build the agent
build-agent:
	@echo "Building agent..."
	cd agent && go build -o ../bin/agent ./cmd/agent

## Build the agent gateway
build-agent-gateway:
	@echo "Building agent-gateway..."
	cd services/agent-gateway && go build -o ../../bin/agent-gateway ./cmd/agent-gateway

## Build the API gateway
build-api-gateway:
	@echo "Building api-gateway..."
	cd services/api-gateway && go build -o ../../bin/api-gateway ./cmd/api-gateway

## Build the integration service
build-integration-service:
	@echo "Building integration-service..."
	cd services/integration-service && cargo build --release
	cp services/integration-service/target/release/integration-service bin/

# =====================================
# Protocol Buffers
# =====================================

## Generate protobuf code
proto:
	@echo "Generating protobuf code..."
	@mkdir -p proto/gen/go proto/gen/rust
	protoc --proto_path=proto \
		--go_out=proto/gen/go --go_opt=paths=source_relative \
		--go-grpc_out=proto/gen/go --go-grpc_opt=paths=source_relative \
		proto/agent/v1/agent.proto
	@echo "Protobuf code generated"

# =====================================
# Test
# =====================================

## Run all tests
test: test-agent test-agent-gateway test-api-gateway test-integration-service

## Test the agent
test-agent:
	@echo "Testing agent..."
	cd agent && go test -v ./...

## Test the agent gateway
test-agent-gateway:
	@echo "Testing agent-gateway..."
	cd services/agent-gateway && go test -v ./...

## Test the API gateway
test-api-gateway:
	@echo "Testing api-gateway..."
	cd services/api-gateway && go test -v ./...

## Test the integration service
test-integration-service:
	@echo "Testing integration-service..."
	cd services/integration-service && cargo test

# =====================================
# Lint
# =====================================

## Run all linters
lint: lint-go lint-rust

## Lint Go code
lint-go:
	@echo "Linting Go code..."
	cd agent && golangci-lint run
	cd services/agent-gateway && golangci-lint run
	cd services/api-gateway && golangci-lint run

## Lint Rust code
lint-rust:
	@echo "Linting Rust code..."
	cd services/integration-service && cargo clippy -- -D warnings

## Format all code
fmt: fmt-go fmt-rust

## Format Go code
fmt-go:
	@echo "Formatting Go code..."
	cd agent && go fmt ./...
	cd services/agent-gateway && go fmt ./...
	cd services/api-gateway && go fmt ./...

## Format Rust code
fmt-rust:
	@echo "Formatting Rust code..."
	cd services/integration-service && cargo fmt

# =====================================
# Run Services
# =====================================

## Run the agent (requires dev environment)
run-agent:
	@echo "Running agent..."
	./bin/agent --config=config/agent.yaml

## Run the agent gateway
run-agent-gateway:
	@echo "Running agent-gateway..."
	./bin/agent-gateway --config=config/agent-gateway.yaml

## Run the API gateway
run-api-gateway:
	@echo "Running api-gateway..."
	./bin/api-gateway --config=config/api-gateway.yaml

## Run the integration service
run-integration-service:
	@echo "Running integration-service..."
	./bin/integration-service

# =====================================
# Database
# =====================================

## Run database migrations
db-migrate:
	@echo "Running database migrations..."
	@docker exec -i sysilo-postgres psql -U sysilo -d sysilo < schemas/postgres/001_initial_schema.sql

## Reset database (drop and recreate)
db-reset:
	@echo "Resetting database..."
	@docker exec -i sysilo-postgres psql -U sysilo -c "DROP SCHEMA public CASCADE; CREATE SCHEMA public;"
	@make db-migrate

# =====================================
# Clean
# =====================================

## Clean all build artifacts
clean:
	@echo "Cleaning build artifacts..."
	rm -rf bin/
	cd services/integration-service && cargo clean
	@echo "Clean complete"

# =====================================
# Setup
# =====================================

## Install development dependencies
setup:
	@echo "Installing development dependencies..."
	@echo "Installing Go tools..."
	go install github.com/golangci/golangci-lint/cmd/golangci-lint@latest
	go install google.golang.org/protobuf/cmd/protoc-gen-go@latest
	go install google.golang.org/grpc/cmd/protoc-gen-go-grpc@latest
	@echo "Checking Rust installation..."
	@which cargo > /dev/null || (echo "Please install Rust: https://rustup.rs" && exit 1)
	@echo "Setup complete!"

## Initialize the project (first time setup)
init: setup
	@mkdir -p bin config
	@echo "Project initialized!"

# =====================================
# Help
# =====================================

## Show this help message
help:
	@echo "Sysilo Platform - Build System"
	@echo ""
	@echo "Usage: make [target]"
	@echo ""
	@echo "Development Environment:"
	@echo "  dev-up              Start local development environment"
	@echo "  dev-down            Stop local development environment"
	@echo "  dev-clean           Stop and remove all data volumes"
	@echo "  dev-logs            View logs from development environment"
	@echo ""
	@echo "Build:"
	@echo "  build               Build all services"
	@echo "  build-agent         Build the agent"
	@echo "  build-agent-gateway Build the agent gateway"
	@echo "  build-api-gateway   Build the API gateway"
	@echo "  build-integration-service Build the integration service"
	@echo ""
	@echo "Test:"
	@echo "  test                Run all tests"
	@echo "  lint                Run all linters"
	@echo "  fmt                 Format all code"
	@echo ""
	@echo "Database:"
	@echo "  db-migrate          Run database migrations"
	@echo "  db-reset            Reset database (drop and recreate)"
	@echo ""
	@echo "Other:"
	@echo "  proto               Generate protobuf code"
	@echo "  clean               Clean all build artifacts"
	@echo "  setup               Install development dependencies"
	@echo "  init                Initialize the project (first time setup)"
	@echo "  help                Show this help message"
