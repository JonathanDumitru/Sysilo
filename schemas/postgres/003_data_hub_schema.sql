-- Data Hub Schema: Catalog, Lineage, and Quality
-- Migration: 003

-- ============================================================================
-- CATALOG
-- ============================================================================

-- Entity types enum
CREATE TYPE entity_type AS ENUM ('table', 'view', 'file', 'api', 'stream', 'dataset');

-- Catalog entities (data assets)
CREATE TABLE catalog_entities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    entity_type entity_type NOT NULL,
    source_system VARCHAR(255) NOT NULL,
    description TEXT,
    metadata JSONB,
    schema_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE(tenant_id, source_system, name)
);

CREATE INDEX idx_catalog_entities_tenant ON catalog_entities(tenant_id);
CREATE INDEX idx_catalog_entities_type ON catalog_entities(tenant_id, entity_type);
CREATE INDEX idx_catalog_entities_source ON catalog_entities(tenant_id, source_system);
CREATE INDEX idx_catalog_entities_name ON catalog_entities(tenant_id, name);

-- Entity schemas with versioning
CREATE TABLE entity_schemas (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_id UUID NOT NULL REFERENCES catalog_entities(id) ON DELETE CASCADE,
    version INT NOT NULL DEFAULT 1,
    fields JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE(entity_id, version)
);

CREATE INDEX idx_entity_schemas_entity ON entity_schemas(entity_id);

-- Add foreign key for schema reference
ALTER TABLE catalog_entities
    ADD CONSTRAINT fk_catalog_schema
    FOREIGN KEY (schema_id) REFERENCES entity_schemas(id);

-- Entity tags for classification
CREATE TABLE entity_tags (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    entity_id UUID NOT NULL REFERENCES catalog_entities(id) ON DELETE CASCADE,
    tag_key VARCHAR(100) NOT NULL,
    tag_value VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE(entity_id, tag_key, tag_value)
);

CREATE INDEX idx_entity_tags_entity ON entity_tags(entity_id);
CREATE INDEX idx_entity_tags_key ON entity_tags(tenant_id, tag_key);

-- ============================================================================
-- LINEAGE
-- ============================================================================

-- Lineage edges representing data flow
CREATE TABLE lineage_edges (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    source_entity_id UUID NOT NULL REFERENCES catalog_entities(id) ON DELETE CASCADE,
    target_entity_id UUID NOT NULL REFERENCES catalog_entities(id) ON DELETE CASCADE,
    transformation_type VARCHAR(50) NOT NULL,
    transformation_logic TEXT,
    integration_id UUID REFERENCES integrations(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE(tenant_id, source_entity_id, target_entity_id, transformation_type)
);

CREATE INDEX idx_lineage_edges_tenant ON lineage_edges(tenant_id);
CREATE INDEX idx_lineage_edges_source ON lineage_edges(source_entity_id);
CREATE INDEX idx_lineage_edges_target ON lineage_edges(target_entity_id);
CREATE INDEX idx_lineage_edges_integration ON lineage_edges(integration_id);

-- Column-level lineage (more granular)
CREATE TABLE column_lineage (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    edge_id UUID NOT NULL REFERENCES lineage_edges(id) ON DELETE CASCADE,
    source_column VARCHAR(255) NOT NULL,
    target_column VARCHAR(255) NOT NULL,
    transformation_expression TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_column_lineage_edge ON column_lineage(edge_id);

-- ============================================================================
-- QUALITY
-- ============================================================================

-- Quality rules
CREATE TABLE quality_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    entity_id UUID NOT NULL REFERENCES catalog_entities(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    rule_type VARCHAR(50) NOT NULL,
    expression TEXT NOT NULL,
    severity VARCHAR(20) NOT NULL DEFAULT 'medium',
    description TEXT,
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE(tenant_id, entity_id, name)
);

CREATE INDEX idx_quality_rules_tenant ON quality_rules(tenant_id);
CREATE INDEX idx_quality_rules_entity ON quality_rules(entity_id);
CREATE INDEX idx_quality_rules_enabled ON quality_rules(tenant_id, enabled);

-- Quality check results
CREATE TABLE quality_check_results (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    rule_id UUID NOT NULL REFERENCES quality_rules(id) ON DELETE CASCADE,
    entity_id UUID NOT NULL REFERENCES catalog_entities(id) ON DELETE CASCADE,
    passed BOOLEAN NOT NULL,
    records_checked BIGINT NOT NULL DEFAULT 0,
    records_failed BIGINT NOT NULL DEFAULT 0,
    failure_rate DOUBLE PRECISION NOT NULL DEFAULT 0,
    sample_failures JSONB,
    executed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_quality_results_rule ON quality_check_results(rule_id);
CREATE INDEX idx_quality_results_entity ON quality_check_results(entity_id);
CREATE INDEX idx_quality_results_time ON quality_check_results(executed_at DESC);

-- Quality score snapshots (aggregated)
CREATE TABLE quality_score_snapshots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    entity_id UUID NOT NULL REFERENCES catalog_entities(id) ON DELETE CASCADE,
    overall_score DOUBLE PRECISION NOT NULL,
    completeness_score DOUBLE PRECISION,
    uniqueness_score DOUBLE PRECISION,
    validity_score DOUBLE PRECISION,
    consistency_score DOUBLE PRECISION,
    timeliness_score DOUBLE PRECISION,
    rules_passed INT NOT NULL DEFAULT 0,
    rules_failed INT NOT NULL DEFAULT 0,
    snapshot_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_quality_snapshots_entity ON quality_score_snapshots(entity_id);
CREATE INDEX idx_quality_snapshots_time ON quality_score_snapshots(snapshot_at DESC);

-- ============================================================================
-- INGESTION
-- ============================================================================

-- Ingestion jobs
CREATE TABLE ingestion_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    entity_id UUID NOT NULL REFERENCES catalog_entities(id) ON DELETE CASCADE,
    connection_id UUID NOT NULL REFERENCES connections(id),
    mode VARCHAR(20) NOT NULL,
    source_query TEXT NOT NULL,
    watermark_column VARCHAR(255),
    watermark_value TEXT,
    batch_size INT NOT NULL DEFAULT 10000,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    records_processed BIGINT NOT NULL DEFAULT 0,
    records_failed BIGINT NOT NULL DEFAULT 0,
    bytes_transferred BIGINT NOT NULL DEFAULT 0,
    error_message TEXT,
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_ingestion_jobs_tenant ON ingestion_jobs(tenant_id);
CREATE INDEX idx_ingestion_jobs_entity ON ingestion_jobs(entity_id);
CREATE INDEX idx_ingestion_jobs_status ON ingestion_jobs(status);
CREATE INDEX idx_ingestion_jobs_time ON ingestion_jobs(created_at DESC);

-- ============================================================================
-- TRIGGERS
-- ============================================================================

-- Update timestamps
CREATE TRIGGER update_catalog_entities_timestamp
    BEFORE UPDATE ON catalog_entities
    FOR EACH ROW EXECUTE FUNCTION update_timestamp();

CREATE TRIGGER update_quality_rules_timestamp
    BEFORE UPDATE ON quality_rules
    FOR EACH ROW EXECUTE FUNCTION update_timestamp();
