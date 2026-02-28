-- Sysilo Billing Schema
-- Enterprise monetization: plans, usage tracking, and Stripe integration

-- =============================================================================
-- PLANS
-- =============================================================================

CREATE TABLE IF NOT EXISTS plans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(50) NOT NULL UNIQUE,
    display_name VARCHAR(100) NOT NULL,
    description TEXT,
    price_cents INT NOT NULL DEFAULT 0,
    billing_interval VARCHAR(20) NOT NULL DEFAULT 'monthly'
        CHECK (billing_interval IN ('monthly', 'yearly')),
    stripe_price_id VARCHAR(255),
    is_active BOOLEAN NOT NULL DEFAULT true,
    limits JSONB NOT NULL DEFAULT '{}',
    features JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_plans_name ON plans(name);
CREATE INDEX idx_plans_active ON plans(is_active);

-- =============================================================================
-- USAGE COUNTERS (per tenant, per billing period)
-- =============================================================================

CREATE TABLE IF NOT EXISTS usage_counters (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    period_start TIMESTAMPTZ NOT NULL,
    period_end TIMESTAMPTZ NOT NULL,
    integration_runs INT NOT NULL DEFAULT 0,
    active_users INT NOT NULL DEFAULT 0,
    data_bytes_processed BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    reset_at TIMESTAMPTZ,
    UNIQUE(tenant_id, period_start)
);

CREATE INDEX idx_usage_counters_tenant ON usage_counters(tenant_id);
CREATE INDEX idx_usage_counters_period ON usage_counters(tenant_id, period_start DESC);

-- =============================================================================
-- ALTER TENANTS — billing fields
-- =============================================================================

ALTER TABLE tenants
    ADD COLUMN IF NOT EXISTS plan_id UUID REFERENCES plans(id) ON DELETE SET NULL,
    ADD COLUMN IF NOT EXISTS plan_status VARCHAR(30) NOT NULL DEFAULT 'trial'
        CHECK (plan_status IN ('active', 'trial', 'past_due', 'cancelled', 'suspended')),
    ADD COLUMN IF NOT EXISTS trial_ends_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS billing_email VARCHAR(255),
    ADD COLUMN IF NOT EXISTS stripe_customer_id VARCHAR(255),
    ADD COLUMN IF NOT EXISTS stripe_subscription_id VARCHAR(255);

CREATE INDEX idx_tenants_plan ON tenants(plan_id);
CREATE INDEX idx_tenants_plan_status ON tenants(plan_status);
CREATE INDEX idx_tenants_trial_ends ON tenants(trial_ends_at)
    WHERE plan_status = 'trial';
CREATE INDEX idx_tenants_stripe_customer ON tenants(stripe_customer_id)
    WHERE stripe_customer_id IS NOT NULL;

-- =============================================================================
-- SEED DATA — Plan tiers
-- =============================================================================

INSERT INTO plans (id, name, display_name, description, price_cents, billing_interval, is_active, limits, features) VALUES
(
    '10000000-0000-0000-0000-000000000001',
    'trial',
    'Trial',
    '14-day trial with full Business-tier access',
    0,
    'monthly',
    true,
    '{
        "max_users": 15,
        "max_integrations": 50,
        "max_connections": 25,
        "max_playbooks": 25,
        "max_runs_per_month": 5000,
        "max_agents": 5,
        "audit_retention_days": 90
    }',
    '{
        "governance_enabled": true,
        "governance_level": "basic",
        "compliance_enabled": false,
        "rationalization_enabled": false,
        "ai_enabled": true,
        "ai_level": "basic",
        "advanced_ops_enabled": true,
        "ops_level": "basic"
    }'
),
(
    '10000000-0000-0000-0000-000000000002',
    'team',
    'Team',
    'For small teams getting started with integration',
    49900,
    'monthly',
    true,
    '{
        "max_users": 5,
        "max_integrations": 10,
        "max_connections": 5,
        "max_playbooks": 5,
        "max_runs_per_month": 500,
        "max_agents": 1,
        "audit_retention_days": 30
    }',
    '{
        "governance_enabled": false,
        "compliance_enabled": false,
        "rationalization_enabled": false,
        "ai_enabled": false,
        "ai_level": "none",
        "advanced_ops_enabled": false,
        "ops_level": "none"
    }'
),
(
    '10000000-0000-0000-0000-000000000003',
    'business',
    'Business',
    'For growing teams that need governance and AI',
    149900,
    'monthly',
    true,
    '{
        "max_users": 15,
        "max_integrations": 50,
        "max_connections": 25,
        "max_playbooks": 25,
        "max_runs_per_month": 5000,
        "max_agents": 5,
        "audit_retention_days": 90
    }',
    '{
        "governance_enabled": true,
        "governance_level": "basic",
        "compliance_enabled": false,
        "rationalization_enabled": false,
        "ai_enabled": true,
        "ai_level": "basic",
        "advanced_ops_enabled": true,
        "ops_level": "basic"
    }'
),
(
    '10000000-0000-0000-0000-000000000004',
    'enterprise',
    'Enterprise',
    'Full platform access with unlimited resources',
    0,
    'monthly',
    true,
    '{
        "max_users": -1,
        "max_integrations": -1,
        "max_connections": -1,
        "max_playbooks": -1,
        "max_runs_per_month": -1,
        "max_agents": -1,
        "audit_retention_days": -1
    }',
    '{
        "governance_enabled": true,
        "governance_level": "full",
        "compliance_enabled": true,
        "rationalization_enabled": true,
        "ai_enabled": true,
        "ai_level": "full",
        "advanced_ops_enabled": true,
        "ops_level": "full"
    }'
);

-- Update dev tenant to trial plan
UPDATE tenants
SET plan_id = '10000000-0000-0000-0000-000000000001',
    plan_status = 'trial',
    trial_ends_at = NOW() + INTERVAL '14 days'
WHERE id = '00000000-0000-0000-0000-000000000001';
