import {
  ConnectionConfig,
  ConnectorMetadata,
  DiscoveredResource,
  HealthCheckResult,
  ReadResult,
  Record,
  TaskConfig,
  WriteResult,
} from './types';

/**
 * Base interface that all connectors must implement
 */
export interface Connector {
  /**
   * Connector metadata
   */
  readonly metadata: ConnectorMetadata;

  /**
   * Initialize the connector with connection configuration
   */
  initialize(config: ConnectionConfig): Promise<void>;

  /**
   * Check if the connection is healthy
   */
  healthCheck(): Promise<HealthCheckResult>;

  /**
   * Discover available resources (tables, APIs, etc.)
   */
  discover?(): Promise<DiscoveredResource[]>;

  /**
   * Read data from the source
   */
  read?(config: TaskConfig): Promise<ReadResult>;

  /**
   * Write data to the target
   */
  write?(config: TaskConfig, records: Record[]): Promise<WriteResult>;

  /**
   * Clean up resources
   */
  close(): Promise<void>;
}

/**
 * Abstract base class for building connectors
 */
export abstract class BaseConnector implements Connector {
  abstract readonly metadata: ConnectorMetadata;
  protected config: ConnectionConfig | null = null;
  protected initialized = false;

  async initialize(config: ConnectionConfig): Promise<void> {
    // Validate config against schema
    const parsed = this.metadata.configSchema.safeParse(config);
    if (!parsed.success) {
      throw new Error(`Invalid configuration: ${parsed.error.message}`);
    }

    this.config = parsed.data;
    await this.onInitialize(config);
    this.initialized = true;
  }

  /**
   * Override this to perform initialization logic
   */
  protected abstract onInitialize(config: ConnectionConfig): Promise<void>;

  abstract healthCheck(): Promise<HealthCheckResult>;

  async close(): Promise<void> {
    if (this.initialized) {
      await this.onClose();
      this.initialized = false;
      this.config = null;
    }
  }

  /**
   * Override this to perform cleanup logic
   */
  protected async onClose(): Promise<void> {
    // Default: no-op
  }

  /**
   * Ensure the connector is initialized before operations
   */
  protected ensureInitialized(): void {
    if (!this.initialized || !this.config) {
      throw new Error('Connector not initialized. Call initialize() first.');
    }
  }
}

/**
 * Decorator for registering connectors
 */
export function connector(metadata: Partial<ConnectorMetadata>) {
  return function <T extends { new (...args: any[]): Connector }>(constructor: T) {
    return class extends constructor {
      metadata = {
        ...metadata,
        name: metadata.name || constructor.name,
        version: metadata.version || '0.1.0',
        description: metadata.description || '',
        author: metadata.author || 'Unknown',
        category: metadata.category || 'other',
        capabilities: metadata.capabilities || {
          supportsRead: false,
          supportsWrite: false,
          supportsDiscover: false,
          supportsHealthCheck: true,
          supportedOperations: [],
        },
        configSchema: metadata.configSchema,
      } as ConnectorMetadata;
    };
  };
}

/**
 * Registry for connectors
 */
export class ConnectorRegistry {
  private static connectors: Map<string, new () => Connector> = new Map();

  static register(name: string, connector: new () => Connector): void {
    this.connectors.set(name, connector);
  }

  static get(name: string): (new () => Connector) | undefined {
    return this.connectors.get(name);
  }

  static list(): string[] {
    return Array.from(this.connectors.keys());
  }

  static create(name: string): Connector {
    const ConnectorClass = this.connectors.get(name);
    if (!ConnectorClass) {
      throw new Error(`Connector not found: ${name}`);
    }
    return new ConnectorClass();
  }
}

export type ConnectorAuthType = 'credential' | 'oauth' | 'api_key';
export type SupportedConnectorType =
  | 'postgresql'
  | 'mysql'
  | 'snowflake'
  | 'oracle'
  | 'salesforce'
  | 'rest_api';

export interface SupportedConnectorSpec {
  connectorType: SupportedConnectorType;
  label: string;
  authType: ConnectorAuthType;
  configFields: readonly string[];
  requiresCredentialReplacementOnEdit: boolean;
}

export const SUPPORTED_CONNECTORS: readonly SupportedConnectorSpec[] = [
  {
    connectorType: 'postgresql',
    label: 'PostgreSQL',
    authType: 'credential',
    configFields: ['host', 'port', 'database', 'ssl_mode'],
    requiresCredentialReplacementOnEdit: true,
  },
  {
    connectorType: 'mysql',
    label: 'MySQL',
    authType: 'credential',
    configFields: ['host', 'port', 'database'],
    requiresCredentialReplacementOnEdit: true,
  },
  {
    connectorType: 'snowflake',
    label: 'Snowflake',
    authType: 'credential',
    configFields: ['account', 'warehouse', 'database', 'schema'],
    requiresCredentialReplacementOnEdit: true,
  },
  {
    connectorType: 'oracle',
    label: 'Oracle',
    authType: 'credential',
    configFields: ['host', 'port', 'service_name'],
    requiresCredentialReplacementOnEdit: true,
  },
  {
    connectorType: 'salesforce',
    label: 'Salesforce',
    authType: 'oauth',
    configFields: ['instance_url', 'api_version'],
    requiresCredentialReplacementOnEdit: true,
  },
  {
    connectorType: 'rest_api',
    label: 'REST API',
    authType: 'api_key',
    configFields: ['base_url', 'headers'],
    requiresCredentialReplacementOnEdit: true,
  },
];

export function getConnectorSpec(
  connectorType: SupportedConnectorType
): SupportedConnectorSpec {
  const spec = SUPPORTED_CONNECTORS.find((item) => item.connectorType === connectorType);
  if (!spec) {
    throw new Error(`Unsupported connector type: ${connectorType}`);
  }
  return spec;
}
