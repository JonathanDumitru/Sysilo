-- Phase 1 identity foundation: SSO/SCIM/session lifecycle support

ALTER TABLE users
	ADD COLUMN IF NOT EXISTS auth_source VARCHAR(50) NOT NULL DEFAULT 'local',
	ADD COLUMN IF NOT EXISTS idp_subject VARCHAR(255),
	ADD COLUMN IF NOT EXISTS session_version INTEGER NOT NULL DEFAULT 1,
	ADD COLUMN IF NOT EXISTS breakglass_eligible BOOLEAN NOT NULL DEFAULT FALSE,
	ADD COLUMN IF NOT EXISTS last_breakglass_login_at TIMESTAMP WITH TIME ZONE;

CREATE UNIQUE INDEX IF NOT EXISTS idx_users_tenant_idp_subject
	ON users(tenant_id, idp_subject)
	WHERE idp_subject IS NOT NULL;

CREATE TABLE IF NOT EXISTS refresh_tokens (
	id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
	tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
	user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
	token_hash VARCHAR(128) NOT NULL,
	replaced_by_hash VARCHAR(128),
	expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
	revoked_at TIMESTAMP WITH TIME ZONE,
	used_at TIMESTAMP WITH TIME ZONE,
	created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
	UNIQUE (tenant_id, token_hash)
);

CREATE INDEX IF NOT EXISTS idx_refresh_tokens_lookup
	ON refresh_tokens(tenant_id, user_id, expires_at);
