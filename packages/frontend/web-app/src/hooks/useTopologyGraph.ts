import { useState, useMemo, useCallback } from 'react';
import type { Node, Edge } from '@xyflow/react';

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export type GraphLens = 'health' | 'governance' | 'time' | 'lineage';

export type TopologyNodeType = 'integration' | 'connection' | 'asset' | 'agent';
export type NodeStatus = 'healthy' | 'warning' | 'critical' | 'inactive';
export type TimeQuadrant = 'tolerate' | 'invest' | 'migrate' | 'eliminate';
export type GovernanceStatus = 'compliant' | 'violation' | 'uncovered';

export interface TopologyNodeData extends Record<string, unknown> {
  id: string;
  type: TopologyNodeType;
  name: string;
  status: NodeStatus;
  timeQuadrant?: TimeQuadrant;
  governanceStatus?: GovernanceStatus;
  lastActivity?: string;
  description?: string;
  errorCount?: number;
  timeScore?: number;
}

// ---------------------------------------------------------------------------
// Mock data — swap this for real API calls later
// ---------------------------------------------------------------------------

const MOCK_TOPOLOGY: TopologyNodeData[] = [
  // Integrations
  { id: 'int-1', type: 'integration', name: 'Salesforce Sync', status: 'healthy', timeQuadrant: 'invest', governanceStatus: 'compliant', lastActivity: '2 min ago', description: 'Bi-directional CRM sync', errorCount: 0, timeScore: 82 },
  { id: 'int-2', type: 'integration', name: 'HubSpot Import', status: 'warning', timeQuadrant: 'tolerate', governanceStatus: 'compliant', lastActivity: '15 min ago', description: 'Contact data import pipeline', errorCount: 3, timeScore: 55 },
  { id: 'int-3', type: 'integration', name: 'Stripe Billing', status: 'healthy', timeQuadrant: 'invest', governanceStatus: 'compliant', lastActivity: '5 min ago', description: 'Payment and invoice processing', errorCount: 0, timeScore: 91 },
  { id: 'int-4', type: 'integration', name: 'Legacy ERP', status: 'critical', timeQuadrant: 'eliminate', governanceStatus: 'violation', lastActivity: '2 hours ago', description: 'Legacy enterprise resource planning', errorCount: 12, timeScore: 18 },
  { id: 'int-5', type: 'integration', name: 'Slack Notifications', status: 'healthy', timeQuadrant: 'tolerate', governanceStatus: 'compliant', lastActivity: '1 min ago', description: 'Alert and notification delivery', errorCount: 0, timeScore: 60 },

  // Connections
  { id: 'conn-1', type: 'connection', name: 'PostgreSQL Primary', status: 'healthy', timeQuadrant: 'invest', governanceStatus: 'compliant', lastActivity: '30s ago', description: 'Primary transactional database', errorCount: 0, timeScore: 95 },
  { id: 'conn-2', type: 'connection', name: 'Snowflake DWH', status: 'healthy', timeQuadrant: 'invest', governanceStatus: 'compliant', lastActivity: '1 min ago', description: 'Data warehouse for analytics', errorCount: 0, timeScore: 88 },
  { id: 'conn-3', type: 'connection', name: 'Redis Cache', status: 'warning', timeQuadrant: 'migrate', governanceStatus: 'uncovered', lastActivity: '10 min ago', description: 'Session and cache layer', errorCount: 2, timeScore: 40 },
  { id: 'conn-4', type: 'connection', name: 'S3 Data Lake', status: 'healthy', timeQuadrant: 'invest', governanceStatus: 'compliant', lastActivity: '5 min ago', description: 'Raw data storage', errorCount: 0, timeScore: 85 },

  // Assets
  { id: 'asset-1', type: 'asset', name: 'Customer Dataset', status: 'healthy', timeQuadrant: 'invest', governanceStatus: 'compliant', lastActivity: '3 min ago', description: 'Unified customer profile data', errorCount: 0, timeScore: 90 },
  { id: 'asset-2', type: 'asset', name: 'Invoice Records', status: 'healthy', timeQuadrant: 'tolerate', governanceStatus: 'compliant', lastActivity: '20 min ago', description: 'Historical invoice data', errorCount: 0, timeScore: 65 },
  { id: 'asset-3', type: 'asset', name: 'Legacy Reports', status: 'inactive', timeQuadrant: 'eliminate', governanceStatus: 'violation', lastActivity: '3 days ago', description: 'Deprecated reporting tables', errorCount: 0, timeScore: 12 },
  { id: 'asset-4', type: 'asset', name: 'Product Catalog', status: 'healthy', timeQuadrant: 'invest', governanceStatus: 'compliant', lastActivity: '1 min ago', description: 'Master product data', errorCount: 0, timeScore: 87 },

  // Agents
  { id: 'agent-1', type: 'agent', name: 'prod-agent-01', status: 'healthy', timeQuadrant: 'invest', governanceStatus: 'compliant', lastActivity: '10s ago', description: 'Primary production agent — AWS us-east-1', errorCount: 0, timeScore: 96 },
  { id: 'agent-2', type: 'agent', name: 'prod-agent-02', status: 'healthy', timeQuadrant: 'invest', governanceStatus: 'compliant', lastActivity: '15s ago', description: 'Secondary production agent — AWS us-west-2', errorCount: 0, timeScore: 94 },
  { id: 'agent-3', type: 'agent', name: 'on-prem-agent', status: 'warning', timeQuadrant: 'migrate', governanceStatus: 'uncovered', lastActivity: '5 min ago', description: 'On-premises data center agent', errorCount: 1, timeScore: 38 },
  { id: 'agent-4', type: 'agent', name: 'dev-agent', status: 'inactive', timeQuadrant: 'tolerate', governanceStatus: 'uncovered', lastActivity: '1 day ago', description: 'Development / testing agent', errorCount: 0, timeScore: 50 },
];

const MOCK_EDGES_RAW: { source: string; target: string }[] = [
  // Agents connect to integrations they run
  { source: 'agent-1', target: 'int-1' },
  { source: 'agent-1', target: 'int-3' },
  { source: 'agent-1', target: 'int-5' },
  { source: 'agent-2', target: 'int-2' },
  { source: 'agent-3', target: 'int-4' },
  // Integrations connect to connections they use
  { source: 'int-1', target: 'conn-1' },
  { source: 'int-1', target: 'conn-2' },
  { source: 'int-2', target: 'conn-1' },
  { source: 'int-3', target: 'conn-1' },
  { source: 'int-4', target: 'conn-3' },
  { source: 'int-5', target: 'conn-3' },
  // Connections feed assets
  { source: 'conn-1', target: 'asset-1' },
  { source: 'conn-1', target: 'asset-2' },
  { source: 'conn-2', target: 'asset-1' },
  { source: 'conn-2', target: 'asset-4' },
  { source: 'conn-3', target: 'asset-3' },
  { source: 'conn-4', target: 'asset-4' },
  { source: 'conn-4', target: 'asset-1' },
];

// ---------------------------------------------------------------------------
// Color maps per lens
// ---------------------------------------------------------------------------

const STATUS_BORDER: Record<NodeStatus, string> = {
  healthy: 'rgba(63,185,80,0.35)',
  warning: 'rgba(210,153,34,0.4)',
  critical: 'rgba(248,81,73,0.5)',
  inactive: 'rgba(139,148,158,0.25)',
};

const TIME_BORDER: Record<TimeQuadrant, string> = {
  invest: 'rgba(63,185,80,0.4)',
  tolerate: 'rgba(88,166,255,0.4)',
  migrate: 'rgba(210,153,34,0.4)',
  eliminate: 'rgba(248,81,73,0.45)',
};

const GOVERNANCE_BORDER: Record<GovernanceStatus, string> = {
  compliant: 'rgba(121,192,255,0.4)',
  violation: 'rgba(248,81,73,0.5)',
  uncovered: 'rgba(139,148,158,0.25)',
};

// ---------------------------------------------------------------------------
// Force-directed layout (simple spring model)
// ---------------------------------------------------------------------------

function forceLayout(
  nodeData: TopologyNodeData[],
  rawEdges: { source: string; target: string }[],
  width = 1200,
  height = 700,
): { x: number; y: number }[] {
  const n = nodeData.length;
  // Initial positions in a circle
  const positions = nodeData.map((_, i) => ({
    x: width / 2 + (width * 0.35) * Math.cos((2 * Math.PI * i) / n),
    y: height / 2 + (height * 0.35) * Math.sin((2 * Math.PI * i) / n),
  }));

  const idxMap = new Map(nodeData.map((d, i) => [d.id, i]));
  const iterations = 80;
  const repulsion = 8000;
  const attraction = 0.005;
  const idealLength = 200;

  for (let iter = 0; iter < iterations; iter++) {
    const forces = positions.map(() => ({ fx: 0, fy: 0 }));
    const cooling = 1 - iter / iterations;

    // Repulsion between all pairs
    for (let i = 0; i < n; i++) {
      for (let j = i + 1; j < n; j++) {
        const dx = positions[i].x - positions[j].x;
        const dy = positions[i].y - positions[j].y;
        const dist = Math.max(Math.sqrt(dx * dx + dy * dy), 1);
        const force = (repulsion / (dist * dist)) * cooling;
        const fx = (dx / dist) * force;
        const fy = (dy / dist) * force;
        forces[i].fx += fx;
        forces[i].fy += fy;
        forces[j].fx -= fx;
        forces[j].fy -= fy;
      }
    }

    // Attraction along edges
    for (const edge of rawEdges) {
      const si = idxMap.get(edge.source);
      const ti = idxMap.get(edge.target);
      if (si === undefined || ti === undefined) continue;
      const dx = positions[ti].x - positions[si].x;
      const dy = positions[ti].y - positions[si].y;
      const dist = Math.max(Math.sqrt(dx * dx + dy * dy), 1);
      const force = attraction * (dist - idealLength) * cooling;
      const fx = (dx / dist) * force;
      const fy = (dy / dist) * force;
      forces[si].fx += fx;
      forces[si].fy += fy;
      forces[ti].fx -= fx;
      forces[ti].fy -= fy;
    }

    // Apply forces
    const maxMove = 20 * cooling;
    for (let i = 0; i < n; i++) {
      const mag = Math.sqrt(forces[i].fx ** 2 + forces[i].fy ** 2);
      const scale = mag > maxMove ? maxMove / mag : 1;
      positions[i].x += forces[i].fx * scale;
      positions[i].y += forces[i].fy * scale;
      // Clamp inside bounds
      positions[i].x = Math.max(50, Math.min(width - 50, positions[i].x));
      positions[i].y = Math.max(50, Math.min(height - 50, positions[i].y));
    }
  }

  return positions;
}

// ---------------------------------------------------------------------------
// Edge styling per lens
// ---------------------------------------------------------------------------

function edgeStyle(lens: GraphLens): React.CSSProperties {
  return {
    stroke: lens === 'lineage' ? '#4B5563' : '#374151',
    strokeWidth: lens === 'lineage' ? 2 : 1.5,
  };
}

function borderForLens(lens: GraphLens, data: TopologyNodeData): string {
  switch (lens) {
    case 'health':
      return STATUS_BORDER[data.status];
    case 'time':
      return TIME_BORDER[data.timeQuadrant ?? 'tolerate'];
    case 'governance':
      return GOVERNANCE_BORDER[data.governanceStatus ?? 'uncovered'];
    case 'lineage':
      return 'rgba(88,166,255,0.3)';
  }
}

// ---------------------------------------------------------------------------
// Hook
// ---------------------------------------------------------------------------

export interface TopologyGraphResult {
  nodes: Node[];
  edges: Edge[];
  isLoading: boolean;
  activeLens: GraphLens;
  setLens: (lens: GraphLens) => void;
  topologyData: TopologyNodeData[];
  stats: {
    totalAssets: number;
    activeIntegrations: number;
    runningAgents: number;
    openAlerts: number;
  };
}

export function useTopologyGraph(): TopologyGraphResult {
  const [activeLens, setActiveLens] = useState<GraphLens>('health');

  const positions = useMemo(
    () => forceLayout(MOCK_TOPOLOGY, MOCK_EDGES_RAW),
    [],
  );

  const nodes = useMemo<Node[]>(() => {
    return MOCK_TOPOLOGY.map((data, i) => ({
      id: data.id,
      type: 'topology',
      position: positions[i],
      data: {
        ...data,
        _borderColor: borderForLens(activeLens, data),
        _lens: activeLens,
      },
    }));
  }, [activeLens, positions]);

  const edges = useMemo<Edge[]>(() => {
    return MOCK_EDGES_RAW.map((raw, i) => ({
      id: `edge-${i}`,
      source: raw.source,
      target: raw.target,
      type: activeLens === 'lineage' ? 'default' : 'default',
      animated: activeLens === 'lineage',
      style: edgeStyle(activeLens),
      markerEnd: activeLens === 'lineage' ? { type: 'arrowclosed' as const, color: '#4B5563' } : undefined,
    }));
  }, [activeLens]);

  const stats = useMemo(() => {
    const integrations = MOCK_TOPOLOGY.filter((n) => n.type === 'integration');
    const agents = MOCK_TOPOLOGY.filter((n) => n.type === 'agent');
    const assets = MOCK_TOPOLOGY.filter((n) => n.type === 'asset');
    return {
      totalAssets: assets.length,
      activeIntegrations: integrations.filter((n) => n.status !== 'inactive').length,
      runningAgents: agents.filter((n) => n.status === 'healthy' || n.status === 'warning').length,
      openAlerts: MOCK_TOPOLOGY.reduce((acc, n) => acc + (n.errorCount ?? 0), 0),
    };
  }, []);

  const setLens = useCallback((lens: GraphLens) => {
    setActiveLens(lens);
  }, []);

  return {
    nodes,
    edges,
    isLoading: false,
    activeLens,
    setLens,
    topologyData: MOCK_TOPOLOGY,
    stats,
  };
}
