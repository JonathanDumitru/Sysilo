-- Discovery Runs: tracks discovery task lifecycle
-- =============================================================================

CREATE TABLE IF NOT EXISTS discovery_runs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    connection_id UUID NOT NULL REFERENCES connections(id),
    connection_name VARCHAR(255) NOT NULL,
    task_id UUID,  -- links to the Kafka task for result matching
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    -- status: pending → scanning → completed | failed
    assets_found INTEGER NOT NULL DEFAULT 0,
    error_message TEXT,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_discovery_runs_tenant ON discovery_runs(tenant_id);
CREATE INDEX idx_discovery_runs_status ON discovery_runs(status);
CREATE INDEX idx_discovery_runs_started ON discovery_runs(started_at DESC);
CREATE UNIQUE INDEX idx_discovery_runs_task_id ON discovery_runs(task_id) WHERE task_id IS NOT NULL;
