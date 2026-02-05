// Agent types
export interface Agent {
  id: string;
  name: string;
  status: 'connected' | 'disconnected' | 'degraded';
  version: string;
  lastHeartbeat: string;
  capabilities: AgentCapabilities;
  labels: Record<string, string>;
}

export interface AgentCapabilities {
  supportedAdapters: string[];
  maxConcurrentTasks: number;
  supportsStreaming: boolean;
}

// Connection types
export interface Connection {
  id: string;
  name: string;
  connectionType: string;
  status: 'active' | 'inactive' | 'error';
  agentId?: string;
  lastTestedAt?: string;
  lastTestStatus?: 'success' | 'failure';
}

// Integration types
export interface Integration {
  id: string;
  name: string;
  description?: string;
  status: 'draft' | 'active' | 'paused' | 'archived';
  definition: IntegrationDefinition;
  schedule?: ScheduleConfig;
  version: number;
  createdAt: string;
  updatedAt: string;
}

export interface IntegrationDefinition {
  nodes: FlowNode[];
  edges: FlowEdge[];
}

export interface FlowNode {
  id: string;
  type: 'source' | 'transform' | 'target' | 'condition' | 'loop';
  position: { x: number; y: number };
  data: Record<string, unknown>;
}

export interface FlowEdge {
  id: string;
  source: string;
  target: string;
  sourceHandle?: string;
  targetHandle?: string;
}

export interface ScheduleConfig {
  enabled: boolean;
  cron: string;
  timezone: string;
}

// Integration Run types
export interface IntegrationRun {
  id: string;
  integrationId: string;
  status: 'pending' | 'running' | 'completed' | 'failed' | 'cancelled';
  triggerType: 'scheduled' | 'manual' | 'webhook' | 'api';
  startedAt?: string;
  completedAt?: string;
  errorMessage?: string;
  metrics: RunMetrics;
}

export interface RunMetrics {
  recordsRead: number;
  recordsWritten: number;
  bytesProcessed: number;
  durationMs: number;
}

// Data Hub types
export interface DataEntity {
  id: string;
  name: string;
  type: 'table' | 'view' | 'file' | 'stream';
  sourceSystem: string;
  schema: SchemaField[];
  classification?: DataClassification;
  lineage?: LineageInfo;
}

export interface SchemaField {
  name: string;
  dataType: string;
  nullable: boolean;
  description?: string;
  classification?: string;
}

export interface DataClassification {
  pii: boolean;
  pci: boolean;
  phi: boolean;
  custom: string[];
}

export interface LineageInfo {
  upstream: string[];
  downstream: string[];
}

// Asset Registry types
export interface Asset {
  id: string;
  name: string;
  assetType: 'system' | 'api' | 'database' | 'data_entity' | 'integration';
  description?: string;
  owner?: string;
  metadata: Record<string, unknown>;
  relationships: AssetRelationship[];
}

export interface AssetRelationship {
  targetId: string;
  relationshipType: string;
  direction: 'inbound' | 'outbound';
}

// API response types
export interface PaginatedResponse<T> {
  data: T[];
  total: number;
  page: number;
  pageSize: number;
}
