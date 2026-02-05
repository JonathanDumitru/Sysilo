-- ============================================================================
-- Phase 6: Rationalization Engine Schema
-- Application portfolio management, scoring, and analysis
-- ============================================================================

-- Application portfolio entries (extends assets)
CREATE TABLE IF NOT EXISTS applications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    asset_id UUID REFERENCES assets(id),  -- Link to asset registry
    name VARCHAR(255) NOT NULL,
    description TEXT,
    vendor VARCHAR(255),
    version VARCHAR(100),

    -- Classification
    business_capability VARCHAR(255),
    business_unit VARCHAR(255),
    application_type VARCHAR(50),  -- custom, cots, saas, legacy
    criticality VARCHAR(20) NOT NULL DEFAULT 'medium',  -- critical, high, medium, low

    -- Lifecycle
    lifecycle_stage VARCHAR(30) NOT NULL DEFAULT 'production',  -- planning, development, production, sunset, retired
    go_live_date DATE,
    sunset_date DATE,

    -- Ownership
    business_owner_id UUID,
    technical_owner_id UUID,

    -- Costs (annual)
    license_cost DECIMAL(15, 2),
    infrastructure_cost DECIMAL(15, 2),
    support_cost DECIMAL(15, 2),
    development_cost DECIMAL(15, 2),
    total_cost DECIMAL(15, 2) GENERATED ALWAYS AS (
        COALESCE(license_cost, 0) + COALESCE(infrastructure_cost, 0) +
        COALESCE(support_cost, 0) + COALESCE(development_cost, 0)
    ) STORED,

    -- Technical details
    technology_stack JSONB,  -- {languages: [], frameworks: [], databases: []}
    hosting_model VARCHAR(50),  -- on-premise, cloud, hybrid

    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE(tenant_id, name)
);

CREATE INDEX idx_applications_tenant ON applications(tenant_id);
CREATE INDEX idx_applications_lifecycle ON applications(tenant_id, lifecycle_stage);
CREATE INDEX idx_applications_criticality ON applications(tenant_id, criticality);

-- Scoring dimensions configuration
CREATE TABLE IF NOT EXISTS scoring_dimensions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    name VARCHAR(100) NOT NULL,
    description TEXT,
    category VARCHAR(50) NOT NULL,  -- value, health, complexity, cost, fit
    weight DECIMAL(5, 2) NOT NULL DEFAULT 1.0,
    scoring_criteria JSONB NOT NULL,  -- {min: 0, max: 10, thresholds: {...}}
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE(tenant_id, name)
);

-- Default scoring dimensions
INSERT INTO scoring_dimensions (id, tenant_id, name, description, category, weight, scoring_criteria) VALUES
-- These will be copied per tenant on first use
('00000000-0000-0000-0000-000000000001', '00000000-0000-0000-0000-000000000000', 'Business Value', 'Revenue impact and strategic importance', 'value', 2.0, '{"min": 0, "max": 10}'),
('00000000-0000-0000-0000-000000000002', '00000000-0000-0000-0000-000000000000', 'User Satisfaction', 'End user satisfaction and adoption', 'value', 1.5, '{"min": 0, "max": 10}'),
('00000000-0000-0000-0000-000000000003', '00000000-0000-0000-0000-000000000000', 'Technical Health', 'Code quality, security, performance', 'health', 2.0, '{"min": 0, "max": 10}'),
('00000000-0000-0000-0000-000000000004', '00000000-0000-0000-0000-000000000000', 'Operational Health', 'Uptime, incident frequency, support burden', 'health', 1.5, '{"min": 0, "max": 10}'),
('00000000-0000-0000-0000-000000000005', '00000000-0000-0000-0000-000000000000', 'Integration Complexity', 'Number and quality of integrations', 'complexity', 1.0, '{"min": 0, "max": 10}'),
('00000000-0000-0000-0000-000000000006', '00000000-0000-0000-0000-000000000000', 'Technical Debt', 'Accumulated technical debt level', 'complexity', 1.5, '{"min": 0, "max": 10}'),
('00000000-0000-0000-0000-000000000007', '00000000-0000-0000-0000-000000000000', 'Total Cost of Ownership', 'Annual TCO relative to value', 'cost', 2.0, '{"min": 0, "max": 10}'),
('00000000-0000-0000-0000-000000000008', '00000000-0000-0000-0000-000000000000', 'Strategic Fit', 'Alignment with enterprise strategy', 'fit', 2.0, '{"min": 0, "max": 10}')
ON CONFLICT DO NOTHING;

-- Application scores (current snapshot)
CREATE TABLE IF NOT EXISTS application_scores (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    application_id UUID NOT NULL REFERENCES applications(id) ON DELETE CASCADE,
    dimension_id UUID NOT NULL REFERENCES scoring_dimensions(id),
    score DECIMAL(4, 2) NOT NULL CHECK (score >= 0 AND score <= 10),
    notes TEXT,
    evidence JSONB,  -- Links to supporting data
    scored_by UUID,
    scored_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE(application_id, dimension_id)
);

CREATE INDEX idx_app_scores_application ON application_scores(application_id);

-- Application score history (for trend analysis)
CREATE TABLE IF NOT EXISTS application_score_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    application_id UUID NOT NULL REFERENCES applications(id) ON DELETE CASCADE,
    dimension_id UUID NOT NULL REFERENCES scoring_dimensions(id),
    score DECIMAL(4, 2) NOT NULL,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_app_score_history_app_time ON application_score_history(application_id, recorded_at DESC);

-- TIME quadrant assignments (Tolerate, Invest, Migrate, Eliminate)
CREATE TABLE IF NOT EXISTS time_assessments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    application_id UUID NOT NULL REFERENCES applications(id) ON DELETE CASCADE,
    quadrant VARCHAR(20) NOT NULL CHECK (quadrant IN ('tolerate', 'invest', 'migrate', 'eliminate')),

    -- Calculated scores that led to this assessment
    business_value_score DECIMAL(4, 2),
    technical_health_score DECIMAL(4, 2),

    -- Override and reasoning
    is_override BOOLEAN NOT NULL DEFAULT false,
    override_reason TEXT,

    -- Recommended actions
    recommended_actions JSONB,  -- [{action, priority, timeline}]

    assessed_by UUID,
    assessed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE(application_id)
);

CREATE INDEX idx_time_assessments_quadrant ON time_assessments(tenant_id, quadrant);

-- What-if scenarios
CREATE TABLE IF NOT EXISTS rationalization_scenarios (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    scenario_type VARCHAR(50) NOT NULL,  -- consolidation, migration, retirement, modernization
    status VARCHAR(30) NOT NULL DEFAULT 'draft',  -- draft, analyzing, completed, approved, rejected

    -- Scope
    affected_applications UUID[] NOT NULL DEFAULT '{}',

    -- Analysis inputs
    assumptions JSONB,  -- Key assumptions for the scenario

    -- Analysis results
    current_state JSONB,  -- Snapshot of current metrics
    projected_state JSONB,  -- Projected metrics after changes

    -- Financial impact
    implementation_cost DECIMAL(15, 2),
    annual_savings DECIMAL(15, 2),
    payback_months INT,
    npv DECIMAL(15, 2),
    roi_percent DECIMAL(6, 2),

    -- Risk assessment
    risk_level VARCHAR(20),  -- low, medium, high, critical
    risk_factors JSONB,  -- [{factor, impact, mitigation}]

    -- Timeline
    estimated_duration_months INT,
    proposed_start_date DATE,

    created_by UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE(tenant_id, name)
);

CREATE INDEX idx_scenarios_tenant_status ON rationalization_scenarios(tenant_id, status);

-- Scenario comparison records
CREATE TABLE IF NOT EXISTS scenario_comparisons (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    name VARCHAR(255) NOT NULL,
    scenario_ids UUID[] NOT NULL,  -- Scenarios being compared
    comparison_metrics JSONB,  -- Side-by-side metrics
    recommendation TEXT,
    created_by UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Migration playbooks
CREATE TABLE IF NOT EXISTS migration_playbooks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    playbook_type VARCHAR(50) NOT NULL,  -- rehost, replatform, refactor, replace, retire

    -- Template structure
    phases JSONB NOT NULL,  -- [{name, description, tasks: [{name, description, checklist}]}]

    -- Estimates
    typical_duration_weeks INT,
    complexity_level VARCHAR(20),  -- simple, moderate, complex

    -- Resources
    required_roles JSONB,  -- [{role, fte_percent, duration_weeks}]

    is_template BOOLEAN NOT NULL DEFAULT false,  -- System-provided template

    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE(tenant_id, name)
);

-- Active migration projects (instances of playbooks)
CREATE TABLE IF NOT EXISTS migration_projects (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    scenario_id UUID REFERENCES rationalization_scenarios(id),
    playbook_id UUID REFERENCES migration_playbooks(id),
    application_id UUID NOT NULL REFERENCES applications(id),

    name VARCHAR(255) NOT NULL,
    status VARCHAR(30) NOT NULL DEFAULT 'planning',  -- planning, in_progress, blocked, completed, cancelled

    -- Progress tracking
    current_phase INT NOT NULL DEFAULT 0,
    progress_percent INT NOT NULL DEFAULT 0,
    task_status JSONB,  -- Tracks completion of each task

    -- Timeline
    planned_start DATE,
    planned_end DATE,
    actual_start DATE,
    actual_end DATE,

    -- Outcomes
    outcomes JSONB,  -- Actual results vs projected
    lessons_learned TEXT,

    project_lead_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_migration_projects_status ON migration_projects(tenant_id, status);
CREATE INDEX idx_migration_projects_app ON migration_projects(application_id);

-- Application dependencies (for impact analysis)
CREATE TABLE IF NOT EXISTS application_dependencies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    source_application_id UUID NOT NULL REFERENCES applications(id) ON DELETE CASCADE,
    target_application_id UUID NOT NULL REFERENCES applications(id) ON DELETE CASCADE,
    dependency_type VARCHAR(50) NOT NULL,  -- data, api, auth, shared_db, file
    criticality VARCHAR(20) NOT NULL DEFAULT 'medium',  -- critical, high, medium, low
    description TEXT,
    integration_id UUID,  -- Link to integration if exists
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE(source_application_id, target_application_id, dependency_type),
    CHECK (source_application_id != target_application_id)
);

CREATE INDEX idx_app_deps_source ON application_dependencies(source_application_id);
CREATE INDEX idx_app_deps_target ON application_dependencies(target_application_id);

-- AI-generated recommendations
CREATE TABLE IF NOT EXISTS ai_recommendations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    application_id UUID REFERENCES applications(id) ON DELETE CASCADE,
    scenario_id UUID REFERENCES rationalization_scenarios(id) ON DELETE CASCADE,

    recommendation_type VARCHAR(50) NOT NULL,  -- consolidation, migration, optimization, retirement
    title VARCHAR(500) NOT NULL,
    summary TEXT NOT NULL,
    detailed_analysis TEXT,

    -- Confidence and reasoning
    confidence_score DECIMAL(4, 2),
    reasoning JSONB,  -- AI explanation of the recommendation
    supporting_data JSONB,  -- Data points that informed the recommendation

    -- Impact
    estimated_savings DECIMAL(15, 2),
    estimated_effort VARCHAR(20),  -- low, medium, high
    risk_assessment VARCHAR(20),

    -- Status
    status VARCHAR(30) NOT NULL DEFAULT 'pending',  -- pending, accepted, rejected, implemented
    user_feedback TEXT,

    generated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    reviewed_by UUID,
    reviewed_at TIMESTAMPTZ
);

CREATE INDEX idx_ai_recommendations_app ON ai_recommendations(application_id);
CREATE INDEX idx_ai_recommendations_status ON ai_recommendations(tenant_id, status);
