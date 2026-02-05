-- Operations Center Schema
-- Migration: 005

-- ============================================================================
-- METRICS
-- ============================================================================

-- Metrics storage (time-series data)
CREATE TABLE metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    resource_type VARCHAR(50) NOT NULL,
    resource_id UUID NOT NULL,
    metric_name VARCHAR(100) NOT NULL,
    metric_value DOUBLE PRECISION NOT NULL,
    tags JSONB,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for time-series queries
CREATE INDEX idx_metrics_tenant_time ON metrics(tenant_id, recorded_at DESC);
CREATE INDEX idx_metrics_resource ON metrics(tenant_id, resource_type, resource_id);
CREATE INDEX idx_metrics_name ON metrics(tenant_id, metric_name, recorded_at DESC);
CREATE INDEX idx_metrics_tags ON metrics USING GIN(tags);

-- Consider partitioning by time for large deployments:
-- CREATE TABLE metrics (...) PARTITION BY RANGE (recorded_at);

-- ============================================================================
-- ALERTS
-- ============================================================================

-- Alert rules
CREATE TABLE alert_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    metric_name VARCHAR(100) NOT NULL,
    condition VARCHAR(20) NOT NULL,  -- gt, lt, eq, gte, lte, ne
    threshold DOUBLE PRECISION NOT NULL,
    duration_seconds INT NOT NULL DEFAULT 60,
    severity VARCHAR(20) NOT NULL DEFAULT 'medium',
    channels UUID[] NOT NULL DEFAULT '{}',
    labels JSONB,
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE(tenant_id, name),
    CONSTRAINT valid_condition CHECK (condition IN ('gt', 'lt', 'eq', 'gte', 'lte', 'ne')),
    CONSTRAINT valid_severity CHECK (severity IN ('critical', 'high', 'medium', 'low', 'info'))
);

CREATE INDEX idx_alert_rules_tenant ON alert_rules(tenant_id);
CREATE INDEX idx_alert_rules_enabled ON alert_rules(tenant_id, enabled);
CREATE INDEX idx_alert_rules_metric ON alert_rules(tenant_id, metric_name);

-- Alert instances (triggered alerts)
CREATE TABLE alert_instances (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    rule_id UUID NOT NULL REFERENCES alert_rules(id) ON DELETE CASCADE,
    status VARCHAR(20) NOT NULL DEFAULT 'firing',
    triggered_value DOUBLE PRECISION,
    triggered_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    resolved_at TIMESTAMPTZ,
    acknowledged_at TIMESTAMPTZ,
    acknowledged_by UUID REFERENCES users(id),
    incident_id UUID,
    metadata JSONB,

    CONSTRAINT valid_status CHECK (status IN ('firing', 'resolved', 'acknowledged', 'silenced'))
);

CREATE INDEX idx_alert_instances_tenant ON alert_instances(tenant_id);
CREATE INDEX idx_alert_instances_rule ON alert_instances(rule_id);
CREATE INDEX idx_alert_instances_status ON alert_instances(tenant_id, status);
CREATE INDEX idx_alert_instances_time ON alert_instances(tenant_id, triggered_at DESC);

-- Alert silences (suppress alerts temporarily)
CREATE TABLE alert_silences (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    rule_id UUID REFERENCES alert_rules(id) ON DELETE CASCADE,
    matchers JSONB,  -- Label matchers for flexible silencing
    reason TEXT NOT NULL,
    created_by UUID NOT NULL REFERENCES users(id),
    starts_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ends_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_alert_silences_active ON alert_silences(tenant_id, starts_at, ends_at);

-- ============================================================================
-- INCIDENTS
-- ============================================================================

-- Incidents
CREATE TABLE incidents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    title VARCHAR(500) NOT NULL,
    description TEXT,
    severity VARCHAR(20) NOT NULL DEFAULT 'medium',
    status VARCHAR(30) NOT NULL DEFAULT 'open',
    priority INT NOT NULL DEFAULT 3,
    assignee_id UUID REFERENCES users(id),
    source VARCHAR(50),  -- manual, alert, system
    source_ref UUID,     -- Reference to alert_instance or other source
    labels JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    acknowledged_at TIMESTAMPTZ,
    resolved_at TIMESTAMPTZ,
    closed_at TIMESTAMPTZ,

    CONSTRAINT valid_incident_severity CHECK (severity IN ('critical', 'high', 'medium', 'low', 'info')),
    CONSTRAINT valid_incident_status CHECK (status IN ('open', 'acknowledged', 'investigating', 'resolved', 'closed'))
);

CREATE INDEX idx_incidents_tenant ON incidents(tenant_id);
CREATE INDEX idx_incidents_status ON incidents(tenant_id, status);
CREATE INDEX idx_incidents_severity ON incidents(tenant_id, severity);
CREATE INDEX idx_incidents_assignee ON incidents(assignee_id);
CREATE INDEX idx_incidents_time ON incidents(tenant_id, created_at DESC);

-- Add foreign key from alert_instances to incidents
ALTER TABLE alert_instances
    ADD CONSTRAINT fk_alert_incident
    FOREIGN KEY (incident_id) REFERENCES incidents(id) ON DELETE SET NULL;

-- Incident timeline events
CREATE TABLE incident_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    incident_id UUID NOT NULL REFERENCES incidents(id) ON DELETE CASCADE,
    event_type VARCHAR(50) NOT NULL,
    content TEXT,
    metadata JSONB,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_incident_events_incident ON incident_events(incident_id);
CREATE INDEX idx_incident_events_time ON incident_events(incident_id, created_at DESC);

-- Incident related alerts (many-to-many)
CREATE TABLE incident_alerts (
    incident_id UUID NOT NULL REFERENCES incidents(id) ON DELETE CASCADE,
    alert_instance_id UUID NOT NULL REFERENCES alert_instances(id) ON DELETE CASCADE,
    linked_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    PRIMARY KEY (incident_id, alert_instance_id)
);

-- ============================================================================
-- NOTIFICATIONS
-- ============================================================================

-- Notification channels
CREATE TABLE notification_channels (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    channel_type VARCHAR(50) NOT NULL,
    config JSONB NOT NULL,
    default_for_severity VARCHAR(20)[],
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE(tenant_id, name),
    CONSTRAINT valid_channel_type CHECK (channel_type IN ('email', 'slack', 'webhook', 'pagerduty', 'teams', 'opsgenie'))
);

CREATE INDEX idx_notification_channels_tenant ON notification_channels(tenant_id);
CREATE INDEX idx_notification_channels_type ON notification_channels(tenant_id, channel_type);

-- Notification delivery log
CREATE TABLE notification_deliveries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    channel_id UUID NOT NULL REFERENCES notification_channels(id) ON DELETE CASCADE,
    alert_instance_id UUID REFERENCES alert_instances(id) ON DELETE SET NULL,
    incident_id UUID REFERENCES incidents(id) ON DELETE SET NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    attempt_count INT NOT NULL DEFAULT 0,
    last_attempt_at TIMESTAMPTZ,
    delivered_at TIMESTAMPTZ,
    error_message TEXT,
    payload JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT valid_delivery_status CHECK (status IN ('pending', 'sent', 'delivered', 'failed', 'skipped'))
);

CREATE INDEX idx_notification_deliveries_channel ON notification_deliveries(channel_id);
CREATE INDEX idx_notification_deliveries_status ON notification_deliveries(tenant_id, status);
CREATE INDEX idx_notification_deliveries_time ON notification_deliveries(tenant_id, created_at DESC);

-- ============================================================================
-- ON-CALL SCHEDULES
-- ============================================================================

-- On-call schedules
CREATE TABLE oncall_schedules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    timezone VARCHAR(50) NOT NULL DEFAULT 'UTC',
    rotation_type VARCHAR(20) NOT NULL DEFAULT 'weekly',
    handoff_time TIME NOT NULL DEFAULT '09:00:00',
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE(tenant_id, name),
    CONSTRAINT valid_rotation_type CHECK (rotation_type IN ('daily', 'weekly', 'custom'))
);

-- On-call participants
CREATE TABLE oncall_participants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    schedule_id UUID NOT NULL REFERENCES oncall_schedules(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    position INT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE(schedule_id, user_id)
);

CREATE INDEX idx_oncall_participants_schedule ON oncall_participants(schedule_id);

-- On-call overrides
CREATE TABLE oncall_overrides (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    schedule_id UUID NOT NULL REFERENCES oncall_schedules(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    starts_at TIMESTAMPTZ NOT NULL,
    ends_at TIMESTAMPTZ NOT NULL,
    reason TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_oncall_overrides_schedule ON oncall_overrides(schedule_id, starts_at, ends_at);

-- ============================================================================
-- TRIGGERS
-- ============================================================================

CREATE TRIGGER update_alert_rules_timestamp
    BEFORE UPDATE ON alert_rules
    FOR EACH ROW EXECUTE FUNCTION update_timestamp();

CREATE TRIGGER update_incidents_timestamp
    BEFORE UPDATE ON incidents
    FOR EACH ROW EXECUTE FUNCTION update_timestamp();

CREATE TRIGGER update_notification_channels_timestamp
    BEFORE UPDATE ON notification_channels
    FOR EACH ROW EXECUTE FUNCTION update_timestamp();

CREATE TRIGGER update_oncall_schedules_timestamp
    BEFORE UPDATE ON oncall_schedules
    FOR EACH ROW EXECUTE FUNCTION update_timestamp();
