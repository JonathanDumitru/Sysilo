import { z } from 'zod';

/**
 * Schema field definition
 */
export interface SchemaField {
  name: string;
  type: FieldType;
  nullable: boolean;
  description?: string;
}

export type FieldType =
  | 'string'
  | 'number'
  | 'integer'
  | 'boolean'
  | 'date'
  | 'datetime'
  | 'json'
  | 'binary';

/**
 * Data schema definition
 */
export interface Schema {
  fields: SchemaField[];
}

/**
 * A single record/row of data
 */
export type Record = { [key: string]: unknown };

/**
 * Batch of records
 */
export interface RecordBatch {
  schema: Schema;
  records: Record[];
  metadata?: {
    cursor?: string;
    hasMore?: boolean;
    totalCount?: number;
  };
}

/**
 * Connection configuration (varies by connector type)
 */
export type ConnectionConfig = { [key: string]: unknown };

/**
 * Task configuration passed to the connector
 */
export interface TaskConfig {
  connectionId: string;
  connection: ConnectionConfig;
  operation: string;
  parameters: { [key: string]: unknown };
}

/**
 * Result of a read operation
 */
export interface ReadResult {
  data: RecordBatch;
  metrics: OperationMetrics;
}

/**
 * Result of a write operation
 */
export interface WriteResult {
  recordsWritten: number;
  metrics: OperationMetrics;
}

/**
 * Operation metrics
 */
export interface OperationMetrics {
  recordsProcessed: number;
  bytesProcessed: number;
  durationMs: number;
}

/**
 * Discovered resource from a connector
 */
export interface DiscoveredResource {
  name: string;
  type: string;
  schema?: Schema;
  metadata?: { [key: string]: unknown };
}

/**
 * Connector capability definition
 */
export interface ConnectorCapabilities {
  supportsRead: boolean;
  supportsWrite: boolean;
  supportsDiscover: boolean;
  supportsHealthCheck: boolean;
  supportedOperations: string[];
}

/**
 * Connector metadata
 */
export interface ConnectorMetadata {
  name: string;
  version: string;
  description: string;
  author: string;
  icon?: string;
  category: ConnectorCategory;
  capabilities: ConnectorCapabilities;
  configSchema: z.ZodType<ConnectionConfig>;
}

export type ConnectorCategory =
  | 'database'
  | 'saas'
  | 'file'
  | 'api'
  | 'messaging'
  | 'other';

/**
 * Health check result
 */
export interface HealthCheckResult {
  healthy: boolean;
  message?: string;
  latencyMs: number;
  details?: { [key: string]: unknown };
}
