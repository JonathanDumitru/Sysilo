const DEFAULT_GATEWAY_API_BASE_URL = 'http://localhost:8080';

type EnvRecord = Record<string, string | undefined>;

function getEnv(): EnvRecord {
  return import.meta.env as unknown as EnvRecord;
}

function normalizeBaseUrl(value: string): string {
  return value.replace(/\/+$/, '');
}

export function getGatewayApiBaseUrl(): string {
  const env = getEnv();
  const mode = import.meta.env.MODE;
  const gatewayBaseUrl = env.VITE_API_BASE_URL?.trim();
  const legacyApiUrl = env.VITE_API_URL?.trim();

  if (gatewayBaseUrl) {
    return normalizeBaseUrl(gatewayBaseUrl);
  }

  if (legacyApiUrl) {
    console.warn('[env] VITE_API_URL is deprecated. Use VITE_API_BASE_URL.');
    return normalizeBaseUrl(legacyApiUrl);
  }

  if (mode === 'production') {
    throw new Error('[env] Missing VITE_API_BASE_URL in production.');
  }

  if (mode !== 'test') {
    console.warn(
      `[env] Missing VITE_API_BASE_URL. Falling back to ${DEFAULT_GATEWAY_API_BASE_URL}.`
    );
  }

  return DEFAULT_GATEWAY_API_BASE_URL;
}

export function getAuthContextHeaders(): Record<string, string> {
  const env = getEnv();
  const tenantId = env.VITE_TENANT_ID?.trim() || 'dev-tenant';
  const teamId = env.VITE_TEAM_ID?.trim();
  const headers: Record<string, string> = {
    'X-Tenant-ID': tenantId,
  };

  if (teamId) {
    headers['X-Team-ID'] = teamId;
  }

  return headers;
}
