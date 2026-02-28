# Testing Patterns

**Analysis Date:** 2026-02-28

## Overview

Testing is language-specific and sparse. Most test coverage lives in Rust services via inline `#[cfg(test)]` modules. Go and Python test infrastructure exists but no test files have been written yet. The frontend has Vitest configured but no test files exist.

---

## Rust Testing

### Framework

**Runner:** Rust's built-in `cargo test`
- Config: no `rust-test` config files; defaults used
- Dev dependency: `tokio-test = "0.4"` available in `integration-service` and `governance-service`

**Run Commands:**
```bash
# Run tests for a specific service
cd services/integration-service && cargo test

# Run with output shown
cd services/integration-service && cargo test -- --nocapture

# Via Makefile
make test-integration-service
```

### Test File Organization

Tests are co-located in the same file as the production code, inside `#[cfg(test)]` modules at the bottom of each module file:

```
services/integration-service/src/
├── playbooks/
│   ├── mod.rs          # Contains #[cfg(test)] mod tests { ... }
│   └── executor.rs     # Contains #[cfg(test)] mod tests { ... }
├── connections/
│   └── mod.rs          # Contains #[cfg(test)] mod tests { ... }
services/data-service/src/
└── ingestion/
    └── mod.rs          # Contains #[cfg(test)] mod tests { ... }
```

Files with tests confirmed:
- `services/integration-service/src/playbooks/mod.rs`
- `services/integration-service/src/playbooks/executor.rs`
- `services/integration-service/src/connections/mod.rs`
- `services/data-service/src/ingestion/mod.rs`

### Test Structure

**Suite Organization:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Helper factory functions at top of test module
    fn make_step(id: &str, on_success: Vec<&str>, on_failure: Vec<&str>) -> Step {
        Step {
            id: id.to_string(),
            step_type: StepType::Integration,
            name: format!("Step {}", id),
            config: serde_json::json!({}),
            on_success: on_success.into_iter().map(String::from).collect(),
            on_failure: on_failure.into_iter().map(String::from).collect(),
        }
    }

    #[test]
    fn test_find_starting_steps_simple_chain() {
        // arrange
        let steps = vec![
            make_step("a", vec!["b"], vec![]),
            make_step("b", vec!["c"], vec![]),
            make_step("c", vec![], vec![]),
        ];
        // act
        let starting = PlaybookExecutor::find_starting_steps(&steps);
        // assert
        assert_eq!(starting.len(), 1);
        assert_eq!(starting[0].id, "a");
    }
}
```

**Key Patterns:**
- `use super::*;` to import the module under test
- Helper factory functions (not macros) build test fixtures inline
- Test function names: `test_{function_name}_{scenario}` — e.g., `test_find_starting_steps_simple_chain`, `test_validate_config_postgresql_valid`
- Synchronous tests only — no `#[tokio::test]` found; async code is tested via pure function extraction

### Assertion Patterns

```rust
// Equality
assert_eq!(starting.len(), 1);
assert_eq!(json, "\"rest_api\"");

// Contains check
assert!(err.contains("port"));
assert!(json.contains("integration"));

// Pattern match for enums
assert!(matches!(status, RunStatus::Pending));
assert!(matches!(detect_type(&ints), DataType::Int64));

// Result unwrap pattern for expected-success paths
let result = convert_value(&serde_json::json!("42"), &DataType::Int64).unwrap();
assert_eq!(result, serde_json::json!(42));

// Result for expected-error paths
let err = validate_config(&ConnectorType::Postgresql, &config).unwrap_err();
assert!(err.contains("port"));

// is_ok() / is_err()
assert!(validate_config(&ConnectorType::Postgresql, &config).is_ok());
```

### What is Tested in Rust

**Unit tests exist for:**
- Serialization: enum serialization to/from JSON strings (`test_step_type_serialization`, `test_connector_type_serialization`)
- Pure logic: `PlaybookExecutor::find_starting_steps()` — DAG traversal algorithm
- Validation: `validate_config()` — connector type field validation
- Type defaults: `test_run_status_default()`
- Data conversion: `convert_value()`, `detect_type()` in `data-service`

**No tests exist for:**
- HTTP handlers (no integration test harness)
- Database operations (no test database setup)
- Kafka interactions
- Middleware/auth

### What is NOT Tested (Gaps)

- All HTTP handler functions in `src/api/mod.rs`, `src/playbooks/api.rs`, `src/connections/api.rs`
- Storage layer (`src/storage/mod.rs`)
- Engine orchestration (`src/engine/mod.rs`)
- Middleware tenant context extraction (`src/middleware/mod.rs`)
- Kafka producer/consumer (`src/kafka/`, `src/consumer/`)

---

## Go Testing

### Framework

**Runner:** `go test`
- No separate test config; uses standard Go testing
- Run: `go test -v ./...` (from each service directory)
- Via Makefile: `make test-agent`, `make test-agent-gateway`, `make test-api-gateway`

**No test files exist yet** in `agent/`, `services/api-gateway/`, or `services/agent-gateway/`. The Makefile targets are defined but empty.

### Anticipated Pattern (for new tests)

When writing Go tests, follow the Go standard layout:
- Test files: `{file}_test.go` co-located with the file under test
- Package: `package {package_name}` (white-box) or `package {package_name}_test` (black-box)
- Test functions: `func Test{MethodName}(t *testing.T)`

```go
// Example for services/api-gateway/internal/handlers/handlers_test.go
package handlers_test

import (
    "net/http"
    "net/http/httptest"
    "testing"
)

func TestHealth(t *testing.T) {
    // arrange
    handler := New(nil, nil) // or use test doubles
    req := httptest.NewRequest(http.MethodGet, "/health", nil)
    w := httptest.NewRecorder()

    // act
    handler.Health(w, req)

    // assert
    if w.Code != http.StatusOK {
        t.Errorf("expected 200, got %d", w.Code)
    }
}
```

---

## Python Testing

### Framework

**Runner:** `pytest` 7.4.0+
- Config in `services/ai-service/pyproject.toml`:
  ```toml
  [tool.pytest.ini_options]
  asyncio_mode = "auto"
  testpaths = ["tests"]
  ```
- Plugins: `pytest-asyncio = "^0.23.0"` (asyncio_mode = "auto" means all async tests run automatically)
- Coverage: `pytest-cov = "^4.1.0"` available

**Run Commands:**
```bash
cd services/ai-service
# Run all tests
pytest

# With coverage
pytest --cov=ai_service --cov-report=term-missing

# Verbose
pytest -v
```

**No test files exist yet** in `services/ai-service/`. The `tests/` directory is specified in config but contains no files.

### Anticipated Pattern (for new tests)

```python
# services/ai-service/tests/test_chat.py
import pytest
from fastapi.testclient import TestClient
from unittest.mock import AsyncMock, patch

from ai_service.main import app

client = TestClient(app)

@pytest.mark.asyncio
async def test_chat_success():
    with patch("ai_service.api.chat.get_llm_client") as mock_client:
        mock_client.return_value.generate = AsyncMock(return_value="Hello!")
        response = client.post("/chat", json={
            "message": "Hello",
            "context": "general",
        })
    assert response.status_code == 200
    assert "message" in response.json()
```

---

## Frontend Testing

### Framework

**Runner:** Vitest 1.2.1
- Config: No `vitest.config.ts` found; Vitest runs via Vite config
- Script: `"test": "vitest"` in `packages/frontend/web-app/package.json`

**Run Commands:**
```bash
cd packages/frontend/web-app
npm run test          # Watch mode
npx vitest run        # Single run
npx vitest --coverage # With coverage (v8 provider)
```

**No test files exist** in `packages/frontend/web-app/src/`. Vitest is configured but unused.

### Anticipated Pattern (for new tests)

Co-locate test files with the code they test:
- Components: `src/components/billing/PlanBadge.test.tsx`
- Hooks: `src/hooks/usePlaybooks.test.ts`
- Services: `src/services/playbooks.test.ts`

```typescript
// Example: src/services/billing.test.ts
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { getCurrentPlan } from './billing';

// Mock the apiFetch module
vi.mock('./api', () => ({
  apiFetch: vi.fn(),
}));

import { apiFetch } from './api';

describe('getCurrentPlan', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('returns tenant plan on success', async () => {
    const mockPlan = { tenant_id: 'uuid', plan_status: 'active' };
    (apiFetch as ReturnType<typeof vi.fn>).mockResolvedValue(mockPlan);

    const result = await getCurrentPlan();
    expect(result).toEqual(mockPlan);
    expect(apiFetch).toHaveBeenCalledWith('/api/v1/plan');
  });
});
```

**React component testing pattern** (when added):
```typescript
// src/components/billing/PlanBadge.test.tsx
import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { PlanBadge } from './PlanBadge';

// Mock the hook
vi.mock('../../hooks/usePlan', () => ({
  usePlan: () => ({
    plan: { name: 'team', display_name: 'Team' },
    planStatus: 'active',
  }),
}));

describe('PlanBadge', () => {
  it('renders the plan display name', () => {
    const queryClient = new QueryClient();
    render(
      <QueryClientProvider client={queryClient}>
        <PlanBadge />
      </QueryClientProvider>
    );
    expect(screen.getByText('Team')).toBeInTheDocument();
  });
});
```

Note: `@testing-library/react` is not yet installed — add it alongside test files.

---

## Coverage

**Requirements:** None enforced (no coverage thresholds configured anywhere)

**Current coverage by area:**

| Area | Test Files | Coverage |
|------|-----------|----------|
| Rust pure logic (executor, validation, serialization) | 4 files | Low (spot tests only) |
| Rust HTTP handlers | 0 | None |
| Rust storage layer | 0 | None |
| Go services | 0 | None |
| Python AI service | 0 | None |
| TypeScript frontend | 0 | None |

---

## Test Infrastructure Notes

**No shared test fixtures or factories** exist across services. Each `#[cfg(test)]` module defines its own local helper functions.

**No mock/stub frameworks** are used in existing Rust tests — tests exercise pure functions only.

**No database test harness** exists. Integration tests against a real database would require a running PostgreSQL instance (available via `make dev-up`).

**For Go integration tests**, `net/http/httptest` from the standard library is the pattern to use (chi router is compatible).

---

*Testing analysis: 2026-02-28*
