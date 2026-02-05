-- Asset Registry Schema
-- Migration: 004

-- Assets table (PostgreSQL mirror of Neo4j data for tenant-aware queries)
CREATE TABLE assets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    asset_type VARCHAR(50) NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    description TEXT,
    owner VARCHAR(255),
    team VARCHAR(255),
    vendor VARCHAR(255),
    version VARCHAR(100),
    documentation_url TEXT,
    repository_url TEXT,
    metadata JSONB,
    tags TEXT[] NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE(tenant_id, name)
);

CREATE INDEX idx_assets_tenant ON assets(tenant_id);
CREATE INDEX idx_assets_type ON assets(tenant_id, asset_type);
CREATE INDEX idx_assets_status ON assets(tenant_id, status);
CREATE INDEX idx_assets_owner ON assets(tenant_id, owner);
CREATE INDEX idx_assets_team ON assets(tenant_id, team);
CREATE INDEX idx_assets_tags ON assets USING GIN(tags);
CREATE INDEX idx_assets_name_search ON assets(tenant_id, LOWER(name));

-- Update timestamp trigger
CREATE TRIGGER update_assets_timestamp
    BEFORE UPDATE ON assets
    FOR EACH ROW EXECUTE FUNCTION update_timestamp();
