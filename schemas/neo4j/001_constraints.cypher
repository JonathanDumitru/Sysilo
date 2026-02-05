// Neo4j Schema Constraints and Indexes for Asset Registry
// Run these queries in Neo4j Browser or via cypher-shell

// ============================================================================
// CONSTRAINTS
// ============================================================================

// Unique constraint on Asset id within tenant
CREATE CONSTRAINT asset_id_unique IF NOT EXISTS
FOR (a:Asset)
REQUIRE (a.tenant_id, a.id) IS UNIQUE;

// Unique constraint on Asset name within tenant
CREATE CONSTRAINT asset_name_unique IF NOT EXISTS
FOR (a:Asset)
REQUIRE (a.tenant_id, a.name) IS UNIQUE;

// ============================================================================
// INDEXES
// ============================================================================

// Index for tenant lookups
CREATE INDEX asset_tenant_idx IF NOT EXISTS
FOR (a:Asset)
ON (a.tenant_id);

// Index for asset type filtering
CREATE INDEX asset_type_idx IF NOT EXISTS
FOR (a:Asset)
ON (a.asset_type);

// Index for status filtering
CREATE INDEX asset_status_idx IF NOT EXISTS
FOR (a:Asset)
ON (a.status);

// Index for owner lookups
CREATE INDEX asset_owner_idx IF NOT EXISTS
FOR (a:Asset)
ON (a.owner);

// Index for team lookups
CREATE INDEX asset_team_idx IF NOT EXISTS
FOR (a:Asset)
ON (a.team);

// Full-text index for search
CREATE FULLTEXT INDEX asset_search_idx IF NOT EXISTS
FOR (a:Asset)
ON EACH [a.name, a.description];

// ============================================================================
// RELATIONSHIP INDEXES
// ============================================================================

// Index relationship IDs for deletion
CREATE INDEX rel_id_idx IF NOT EXISTS
FOR ()-[r:DEPENDS_ON]-()
ON (r.id);

CREATE INDEX rel_id_integrates_idx IF NOT EXISTS
FOR ()-[r:INTEGRATES]-()
ON (r.id);

CREATE INDEX rel_id_reads_idx IF NOT EXISTS
FOR ()-[r:READS_FROM]-()
ON (r.id);

CREATE INDEX rel_id_writes_idx IF NOT EXISTS
FOR ()-[r:WRITES_TO]-()
ON (r.id);

CREATE INDEX rel_id_calls_idx IF NOT EXISTS
FOR ()-[r:CALLS]-()
ON (r.id);

CREATE INDEX rel_id_hosts_idx IF NOT EXISTS
FOR ()-[r:HOSTS]-()
ON (r.id);

// ============================================================================
// SAMPLE QUERIES FOR VERIFICATION
// ============================================================================

// Show all constraints
// SHOW CONSTRAINTS;

// Show all indexes
// SHOW INDEXES;

// Test data creation (comment out in production)
// CREATE (app:Asset {
//     id: 'test-001',
//     tenant_id: 'tenant-001',
//     name: 'Test Application',
//     asset_type: 'Application',
//     status: 'active',
//     description: 'A test application for verification',
//     owner: 'engineering',
//     team: 'platform',
//     created_at: datetime(),
//     updated_at: datetime()
// });
