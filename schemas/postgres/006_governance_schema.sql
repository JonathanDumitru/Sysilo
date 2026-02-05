-- Governance Center Schema
-- Migration: 006

-- ============================================================================
-- STANDARDS
-- ============================================================================

-- Standards library
CREATE TABLE standards (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    category VARCHAR(100) NOT NULL,
    description TEXT,
    rules JSONB NOT NULL,
    examples JSONB,
    version INT NOT NULL DEFAULT 1,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE(tenant_id, name, version)
);

CREATE INDEX idx_standards_tenant ON standards(tenant_id);
CREATE INDEX idx_standards_category ON standards(tenant_id, category);
CREATE INDEX idx_standards_active ON standards(tenant_id, is_active);

-- ============================================================================
-- POLICIES
-- ============================================================================

-- Policy definitions
CREATE TABLE policies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    rego_policy TEXT NOT NULL,
    scope VARCHAR(50) NOT NULL,  -- integration, connection, agent, data_entity, etc.
    enforcement_mode VARCHAR(20) NOT NULL DEFAULT 'warn',
    severity VARCHAR(20) NOT NULL DEFAULT 'medium',
    enabled BOOLEAN NOT NULL DEFAULT true,
    version INT NOT NULL DEFAULT 1,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE(tenant_id, name),
    CONSTRAINT valid_scope CHECK (scope IN ('integration', 'connection', 'agent', 'data_entity', 'user', 'api', 'all')),
    CONSTRAINT valid_enforcement_mode CHECK (enforcement_mode IN ('enforce', 'warn', 'audit')),
    CONSTRAINT valid_policy_severity CHECK (severity IN ('critical', 'high', 'medium', 'low', 'info'))
);

CREATE INDEX idx_policies_tenant ON policies(tenant_id);
CREATE INDEX idx_policies_scope ON policies(tenant_id, scope);
CREATE INDEX idx_policies_enabled ON policies(tenant_id, enabled);

-- Policy violations
CREATE TABLE policy_violations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    policy_id UUID NOT NULL REFERENCES policies(id) ON DELETE CASCADE,
    resource_type VARCHAR(50) NOT NULL,
    resource_id UUID NOT NULL,
    details JSONB NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'open',
    resolved_by UUID REFERENCES users(id),
    resolved_at TIMESTAMPTZ,
    resolution_note TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT valid_violation_status CHECK (status IN ('open', 'acknowledged', 'resolved', 'waived'))
);

CREATE INDEX idx_policy_violations_tenant ON policy_violations(tenant_id);
CREATE INDEX idx_policy_violations_policy ON policy_violations(policy_id);
CREATE INDEX idx_policy_violations_resource ON policy_violations(tenant_id, resource_type, resource_id);
CREATE INDEX idx_policy_violations_status ON policy_violations(tenant_id, status);
CREATE INDEX idx_policy_violations_time ON policy_violations(tenant_id, created_at DESC);

-- Policy waivers (exceptions)
CREATE TABLE policy_waivers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    policy_id UUID NOT NULL REFERENCES policies(id) ON DELETE CASCADE,
    resource_type VARCHAR(50),
    resource_id UUID,
    reason TEXT NOT NULL,
    approved_by UUID NOT NULL REFERENCES users(id),
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_policy_waivers_policy ON policy_waivers(policy_id);
CREATE INDEX idx_policy_waivers_resource ON policy_waivers(tenant_id, resource_type, resource_id);

-- ============================================================================
-- APPROVAL WORKFLOWS
-- ============================================================================

-- Approval workflow definitions
CREATE TABLE approval_workflows (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    trigger_conditions JSONB NOT NULL,  -- When to trigger this workflow
    stages JSONB NOT NULL,              -- [{name, approvers, required_count, timeout_hours}]
    auto_approve_conditions JSONB,      -- Conditions for automatic approval
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE(tenant_id, name)
);

CREATE INDEX idx_approval_workflows_tenant ON approval_workflows(tenant_id);
CREATE INDEX idx_approval_workflows_enabled ON approval_workflows(tenant_id, enabled);

-- Approval requests
CREATE TABLE approval_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    workflow_id UUID NOT NULL REFERENCES approval_workflows(id) ON DELETE CASCADE,
    resource_type VARCHAR(50) NOT NULL,
    resource_id UUID NOT NULL,
    resource_snapshot JSONB,            -- Snapshot of resource at request time
    requester_id UUID NOT NULL REFERENCES users(id),
    current_stage INT NOT NULL DEFAULT 0,
    status VARCHAR(30) NOT NULL DEFAULT 'pending',
    auto_approved BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,

    CONSTRAINT valid_request_status CHECK (status IN ('pending', 'approved', 'rejected', 'cancelled', 'expired'))
);

CREATE INDEX idx_approval_requests_tenant ON approval_requests(tenant_id);
CREATE INDEX idx_approval_requests_workflow ON approval_requests(workflow_id);
CREATE INDEX idx_approval_requests_resource ON approval_requests(tenant_id, resource_type, resource_id);
CREATE INDEX idx_approval_requests_requester ON approval_requests(requester_id);
CREATE INDEX idx_approval_requests_status ON approval_requests(tenant_id, status);
CREATE INDEX idx_approval_requests_time ON approval_requests(tenant_id, created_at DESC);

-- Approval decisions
CREATE TABLE approval_decisions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    request_id UUID NOT NULL REFERENCES approval_requests(id) ON DELETE CASCADE,
    stage INT NOT NULL,
    approver_id UUID NOT NULL REFERENCES users(id),
    decision VARCHAR(20) NOT NULL,
    comment TEXT,
    decided_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT valid_decision CHECK (decision IN ('approved', 'rejected', 'delegated'))
);

CREATE INDEX idx_approval_decisions_request ON approval_decisions(request_id);
CREATE INDEX idx_approval_decisions_approver ON approval_decisions(approver_id);

-- Approval delegation
CREATE TABLE approval_delegations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    from_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    to_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    starts_at TIMESTAMPTZ NOT NULL,
    ends_at TIMESTAMPTZ NOT NULL,
    reason TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_approval_delegations_from ON approval_delegations(from_user_id, starts_at, ends_at);

-- ============================================================================
-- AUDIT LOG
-- ============================================================================

-- Immutable audit log
-- IMPORTANT: Do NOT grant UPDATE or DELETE permissions on this table
CREATE TABLE audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    actor_id UUID,
    actor_type VARCHAR(50) NOT NULL,  -- user, system, agent, api_key
    actor_name VARCHAR(255),          -- Denormalized for historical reference
    action VARCHAR(100) NOT NULL,
    resource_type VARCHAR(50) NOT NULL,
    resource_id UUID,
    resource_name VARCHAR(255),       -- Denormalized for historical reference
    before_state JSONB,
    after_state JSONB,
    change_summary TEXT,
    metadata JSONB,
    ip_address INET,
    user_agent TEXT,
    request_id VARCHAR(100),          -- Correlation ID
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    hash VARCHAR(64) NOT NULL         -- SHA-256 for tamper detection
);

-- Indexes optimized for audit queries
CREATE INDEX idx_audit_log_tenant_time ON audit_log(tenant_id, timestamp DESC);
CREATE INDEX idx_audit_log_actor ON audit_log(tenant_id, actor_id, timestamp DESC);
CREATE INDEX idx_audit_log_resource ON audit_log(tenant_id, resource_type, resource_id, timestamp DESC);
CREATE INDEX idx_audit_log_action ON audit_log(tenant_id, action, timestamp DESC);

-- Prevent modifications to audit log
CREATE OR REPLACE FUNCTION prevent_audit_modification()
RETURNS TRIGGER AS $$
BEGIN
    RAISE EXCEPTION 'Audit log entries cannot be modified or deleted';
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER prevent_audit_update
    BEFORE UPDATE ON audit_log
    FOR EACH ROW EXECUTE FUNCTION prevent_audit_modification();

CREATE TRIGGER prevent_audit_delete
    BEFORE DELETE ON audit_log
    FOR EACH ROW EXECUTE FUNCTION prevent_audit_modification();

-- ============================================================================
-- COMPLIANCE
-- ============================================================================

-- Compliance frameworks (shared across tenants)
CREATE TABLE compliance_frameworks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL UNIQUE,
    description TEXT,
    version VARCHAR(20),
    controls JSONB NOT NULL,  -- [{control_id, name, description, category, evidence_requirements}]
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Seed common frameworks
INSERT INTO compliance_frameworks (name, description, version, controls) VALUES
('SOC2', 'Service Organization Control 2', '2017', '[]'::jsonb),
('GDPR', 'General Data Protection Regulation', '2018', '[]'::jsonb),
('HIPAA', 'Health Insurance Portability and Accountability Act', '1996', '[]'::jsonb),
('ISO27001', 'Information Security Management', '2022', '[]'::jsonb);

-- Tenant compliance status
CREATE TABLE compliance_status (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    framework_id UUID NOT NULL REFERENCES compliance_frameworks(id) ON DELETE CASCADE,
    control_id VARCHAR(50) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'not_assessed',
    evidence_refs UUID[],
    notes TEXT,
    assessed_by UUID REFERENCES users(id),
    last_assessed TIMESTAMPTZ,
    next_review TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE(tenant_id, framework_id, control_id),
    CONSTRAINT valid_compliance_status CHECK (status IN ('compliant', 'non_compliant', 'partial', 'not_applicable', 'not_assessed'))
);

CREATE INDEX idx_compliance_status_tenant ON compliance_status(tenant_id);
CREATE INDEX idx_compliance_status_framework ON compliance_status(tenant_id, framework_id);
CREATE INDEX idx_compliance_status_status ON compliance_status(tenant_id, status);

-- Compliance evidence
CREATE TABLE compliance_evidence (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    framework_id UUID NOT NULL REFERENCES compliance_frameworks(id) ON DELETE CASCADE,
    control_id VARCHAR(50) NOT NULL,
    evidence_type VARCHAR(50) NOT NULL,  -- document, screenshot, log, attestation
    title VARCHAR(255) NOT NULL,
    description TEXT,
    file_path TEXT,
    collected_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    collected_by UUID REFERENCES users(id),
    valid_until TIMESTAMPTZ,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_compliance_evidence_control ON compliance_evidence(tenant_id, framework_id, control_id);

-- ============================================================================
-- DATA CLASSIFICATION
-- ============================================================================

-- Data classification labels
CREATE TABLE data_classifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    description TEXT,
    sensitivity_level INT NOT NULL DEFAULT 1,  -- 1=public, 2=internal, 3=confidential, 4=restricted
    handling_requirements JSONB,
    retention_days INT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE(tenant_id, name)
);

CREATE INDEX idx_data_classifications_tenant ON data_classifications(tenant_id);

-- Data classification assignments
CREATE TABLE data_classification_assignments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    classification_id UUID NOT NULL REFERENCES data_classifications(id) ON DELETE CASCADE,
    resource_type VARCHAR(50) NOT NULL,
    resource_id UUID NOT NULL,
    assigned_by UUID REFERENCES users(id),
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    verified_at TIMESTAMPTZ,

    UNIQUE(tenant_id, resource_type, resource_id)
);

CREATE INDEX idx_classification_assignments_resource ON data_classification_assignments(tenant_id, resource_type, resource_id);

-- ============================================================================
-- TRIGGERS
-- ============================================================================

CREATE TRIGGER update_standards_timestamp
    BEFORE UPDATE ON standards
    FOR EACH ROW EXECUTE FUNCTION update_timestamp();

CREATE TRIGGER update_policies_timestamp
    BEFORE UPDATE ON policies
    FOR EACH ROW EXECUTE FUNCTION update_timestamp();

CREATE TRIGGER update_approval_workflows_timestamp
    BEFORE UPDATE ON approval_workflows
    FOR EACH ROW EXECUTE FUNCTION update_timestamp();

CREATE TRIGGER update_approval_requests_timestamp
    BEFORE UPDATE ON approval_requests
    FOR EACH ROW EXECUTE FUNCTION update_timestamp();

CREATE TRIGGER update_compliance_frameworks_timestamp
    BEFORE UPDATE ON compliance_frameworks
    FOR EACH ROW EXECUTE FUNCTION update_timestamp();

CREATE TRIGGER update_compliance_status_timestamp
    BEFORE UPDATE ON compliance_status
    FOR EACH ROW EXECUTE FUNCTION update_timestamp();

CREATE TRIGGER update_data_classifications_timestamp
    BEFORE UPDATE ON data_classifications
    FOR EACH ROW EXECUTE FUNCTION update_timestamp();
