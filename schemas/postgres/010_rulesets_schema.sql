-- Rulesets Schema
-- Migration: 010
-- Rulesets group related policies together for collective management and evaluation

-- ============================================================================
-- RULESETS
-- ============================================================================

-- Ruleset definitions
CREATE TABLE rulesets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    scope VARCHAR(50) NOT NULL DEFAULT 'all',
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE(tenant_id, name),
    CONSTRAINT valid_ruleset_scope CHECK (scope IN ('integration', 'connection', 'agent', 'data_entity', 'user', 'api', 'all'))
);

CREATE INDEX idx_rulesets_tenant ON rulesets(tenant_id);
CREATE INDEX idx_rulesets_scope ON rulesets(tenant_id, scope);
CREATE INDEX idx_rulesets_enabled ON rulesets(tenant_id, enabled);

-- Ruleset-to-policy associations (many-to-many)
CREATE TABLE ruleset_policies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ruleset_id UUID NOT NULL REFERENCES rulesets(id) ON DELETE CASCADE,
    policy_id UUID NOT NULL REFERENCES policies(id) ON DELETE CASCADE,
    position INT NOT NULL DEFAULT 0,
    added_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE(ruleset_id, policy_id)
);

CREATE INDEX idx_ruleset_policies_ruleset ON ruleset_policies(ruleset_id);
CREATE INDEX idx_ruleset_policies_policy ON ruleset_policies(policy_id);

-- ============================================================================
-- TRIGGERS
-- ============================================================================

CREATE TRIGGER update_rulesets_timestamp
    BEFORE UPDATE ON rulesets
    FOR EACH ROW EXECUTE FUNCTION update_timestamp();
