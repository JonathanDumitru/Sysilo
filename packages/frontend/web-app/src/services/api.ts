import { getGatewayApiBaseUrl } from '../config/env';

export const GATEWAY_API_VERSION_PREFIX = '/api/v1';
export const GATEWAY_CONNECTIONS_BASE_PATH = `${GATEWAY_API_VERSION_PREFIX}/connections`;
export const GATEWAY_ROUTE_CONTRACT = Object.freeze({
  apiVersionPrefix: GATEWAY_API_VERSION_PREFIX,
  connectionsBasePath: GATEWAY_CONNECTIONS_BASE_PATH,
  connectionsTestPath: `${GATEWAY_CONNECTIONS_BASE_PATH}/{connectionID}/test`,
});

const API_BASE_URL = getGatewayApiBaseUrl();

export class ApiError extends Error {
  constructor(
    public status: number,
    message: string
  ) {
    super(message);
    this.name = 'ApiError';
  }
}

export async function apiFetch<T>(
  endpoint: string,
  options: RequestInit = {}
): Promise<T> {
  const normalizedEndpoint = endpoint.startsWith('/') ? endpoint : `/${endpoint}`;
  const url = `${API_BASE_URL}${normalizedEndpoint}`;

  const response = await fetch(url, {
    ...options,
    headers: {
      'Content-Type': 'application/json',
      ...options.headers,
    },
  });

  if (!response.ok) {
    const text = await response.text();
    throw new ApiError(response.status, text || response.statusText);
  }

  return response.json();
}

export const apiClient = {
  request: apiFetch,
};
