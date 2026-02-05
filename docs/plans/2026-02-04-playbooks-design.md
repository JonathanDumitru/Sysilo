# Playbooks Design

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:writing-plans to create the implementation plan from this design.

**Goal:** General-purpose automation system for operational runbooks, migration playbooks, and integration orchestration.

**Architecture:** Visual drag-and-drop builder using React Flow, execution via existing Kafka task dispatch in integration-service.

**Tech Stack:** React Flow (frontend), Rust/Axum (backend), PostgreSQL (storage), Kafka (task dispatch)

---

## Data Model

### Playbook

Reusable automation template stored in PostgreSQL:

```rust
Playbook {
    id: Uuid,
    tenant_id: Uuid,
    name: String,                    // "Deploy to Production"
    description: Option<String>,
    trigger_type: TriggerType,       // Manual, Scheduled, Webhook, Event
    steps: Vec<Step>,                // Ordered list of steps (stored as JSON)
    variables: Vec<Variable>,        // Input parameters
    created_at: DateTime,
    updated_at: DateTime,
}

enum TriggerType {
    Manual,
    Scheduled,
    Webhook,
    Event,
}

struct Variable {
    name: String,
    var_type: String,                // "string", "number", "boolean"
    required: bool,
    default_value: Option<String>,
}
```

### Step

Defines one action in the playbook:

```rust
struct Step {
    id: String,                      // Unique within playbook
    step_type: StepType,
    name: String,                    // "Run data sync"
    config: serde_json::Value,       // Type-specific configuration
    on_success: Vec<String>,         // Step IDs to run next
    on_failure: Vec<String>,         // Step IDs on error (or halt)
}

enum StepType {
    Integration,                     // Run an existing integration
    Webhook,                         // HTTP POST to configured URL
    Wait,                            // Sleep for configured duration
    Condition,                       // Evaluate expression, branch
    Approval,                        // Pause for manual approval
}
```

### PlaybookRun

Tracks each execution:

```rust
PlaybookRun {
    id: Uuid,
    playbook_id: Uuid,
    tenant_id: Uuid,
    status: RunStatus,
    variables: serde_json::Value,    // Resolved input values
    step_states: Vec<StepState>,     // Progress of each step
    started_at: DateTime,
    completed_at: Option<DateTime>,
}

enum RunStatus {
    Pending,
    Running,
    WaitingApproval,
    Completed,
    Failed,
    Cancelled,
}

struct StepState {
    step_id: String,
    status: StepStatus,
    started_at: Option<DateTime>,
    completed_at: Option<DateTime>,
    output: Option<serde_json::Value>,
    error: Option<String>,
}
```

---

## Execution Flow

### Starting a Playbook Run

1. User clicks "Run Playbook" → `POST /playbooks/{id}/run` with variables
2. Integration-service creates `PlaybookRun` record with status `running`
3. Service identifies first step(s) (those with no dependencies) and dispatches them as Kafka tasks

### Step Execution via Existing Task System

Each step becomes a task on `sysilo.tasks` topic:

```json
{
    "task_type": "playbook_step",
    "config": {
        "run_id": "uuid",
        "step_id": "step-1",
        "step_type": "integration",
        "step_config": { "integration_id": "uuid" }
    }
}
```

Agents process steps based on type:

| Step Type | Agent Behavior |
|-----------|----------------|
| `integration` | Trigger existing integration run, wait for completion |
| `webhook` | HTTP POST/GET to configured URL with optional body |
| `wait` | Sleep for configured duration |
| `condition` | Evaluate expression, return which branch to take |
| `approval` | Pause run, set status to `waiting_approval` |

### Result Handling

Agent publishes result to `sysilo.results`. Integration-service consumer:

1. Updates `step_states` in `PlaybookRun`
2. Checks `on_success` or `on_failure` for next steps
3. Dispatches next steps as new Kafka tasks
4. When no more steps → mark run `completed` or `failed`

### Approval Gates

When a run hits an approval step:

1. Run status becomes `waiting_approval`
2. Frontend polls (or uses WebSocket) to show approval UI
3. User approves → `POST /runs/{id}/approve` → resume execution
4. User rejects → `POST /runs/{id}/reject` → mark run failed

---

## API Endpoints

### Playbook CRUD

```
GET    /playbooks              → List playbooks for tenant
POST   /playbooks              → Create new playbook
GET    /playbooks/:id          → Get playbook with steps
PUT    /playbooks/:id          → Update playbook
DELETE /playbooks/:id          → Delete playbook
```

### Execution Endpoints

```
POST   /playbooks/:id/run      → Start a new run (body: { variables })
GET    /playbooks/:id/runs     → List runs for this playbook
GET    /runs/:id               → Get run with step states (extend existing)
POST   /runs/:id/cancel        → Cancel a running playbook (existing)
POST   /runs/:id/approve       → Approve pending approval step
POST   /runs/:id/reject        → Reject and fail the approval step
```

### Example: Create Playbook

```json
POST /playbooks
{
    "name": "Deploy to Production",
    "description": "Full deployment workflow with approval gate",
    "trigger_type": "manual",
    "variables": [
        { "name": "version", "var_type": "string", "required": true }
    ],
    "steps": [
        {
            "id": "backup",
            "step_type": "integration",
            "name": "Backup Database",
            "config": { "integration_id": "uuid" },
            "on_success": ["deploy"],
            "on_failure": []
        },
        {
            "id": "deploy",
            "step_type": "webhook",
            "name": "Trigger Deploy",
            "config": { "url": "https://deploy.example.com/trigger", "method": "POST" },
            "on_success": ["approve"],
            "on_failure": ["rollback"]
        },
        {
            "id": "approve",
            "step_type": "approval",
            "name": "Verify Deployment",
            "config": { "message": "Check staging looks good before continuing" },
            "on_success": ["notify"],
            "on_failure": ["rollback"]
        },
        {
            "id": "notify",
            "step_type": "webhook",
            "name": "Notify Slack",
            "config": { "url": "https://hooks.slack.com/...", "method": "POST" },
            "on_success": [],
            "on_failure": []
        },
        {
            "id": "rollback",
            "step_type": "integration",
            "name": "Rollback",
            "config": { "integration_id": "uuid" },
            "on_success": [],
            "on_failure": []
        }
    ]
}
```

---

## Frontend Components

### Routes

```
/playbooks              → PlaybooksListPage
/playbooks/new          → PlaybookEditorPage (create)
/playbooks/:id          → PlaybookEditorPage (edit)
/playbooks/:id/runs     → PlaybookRunsPage (history)
/playbooks/:id/runs/:runId → PlaybookRunDetailPage (live view)
```

### PlaybookEditorPage

Visual builder using React Flow, reusing patterns from Integration Studio:

**New node types:**
- `IntegrationStepNode` - Dropdown to select integration
- `WebhookStepNode` - URL, method, headers, body config
- `WaitStepNode` - Duration input (seconds/minutes/hours)
- `ConditionStepNode` - Expression editor, two output handles (true/false)
- `ApprovalStepNode` - Message text and optional assignee

**Layout:**
- Left: Toolbox with draggable step types
- Center: React Flow canvas
- Right: Config panel for selected node
- Header: Name input, Save button, "Run Now" button

### PlaybookRunDetailPage

Same graph as editor but read-only, with step nodes colored by status:

| Status | Color |
|--------|-------|
| Pending | Gray |
| Running | Blue + pulse animation |
| Completed | Green |
| Failed | Red |
| Waiting Approval | Yellow + Approve/Reject buttons |

**Sidebar shows:**
- Run variables
- Timeline of step completions
- Error details for failed steps

### PlaybooksListPage

Simple table view:

| Column | Content |
|--------|---------|
| Name | Playbook name (link to editor) |
| Trigger | manual / scheduled / webhook |
| Last Run | Timestamp or "Never" |
| Status | Last run status badge |
| Actions | Run, Edit, Delete |

Header has "New Playbook" button.

---

## Database Schema

```sql
CREATE TABLE playbooks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    trigger_type VARCHAR(50) NOT NULL DEFAULT 'manual',
    steps JSONB NOT NULL DEFAULT '[]',
    variables JSONB NOT NULL DEFAULT '[]',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_playbooks_tenant ON playbooks(tenant_id);

CREATE TABLE playbook_runs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    playbook_id UUID NOT NULL REFERENCES playbooks(id),
    tenant_id UUID NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    variables JSONB NOT NULL DEFAULT '{}',
    step_states JSONB NOT NULL DEFAULT '[]',
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

CREATE INDEX idx_playbook_runs_playbook ON playbook_runs(playbook_id);
CREATE INDEX idx_playbook_runs_tenant ON playbook_runs(tenant_id);
CREATE INDEX idx_playbook_runs_status ON playbook_runs(status);
```

---

## Implementation Phases

### Phase 1: Backend Foundation
- Add database tables for playbooks and runs
- Implement CRUD endpoints for playbooks
- Add run creation endpoint

### Phase 2: Execution Engine
- Extend result consumer to handle playbook steps
- Implement step dispatching logic
- Add approval/reject endpoints

### Phase 3: Frontend - List & Editor
- PlaybooksListPage with table
- PlaybookEditorPage with React Flow
- Step node components (5 types)
- Save/load playbook API integration

### Phase 4: Frontend - Execution
- PlaybookRunDetailPage with live status
- Approval UI in run view
- Run history page
