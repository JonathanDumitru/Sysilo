-- Automation Playbooks schema
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
    playbook_id UUID NOT NULL REFERENCES playbooks(id) ON DELETE CASCADE,
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
