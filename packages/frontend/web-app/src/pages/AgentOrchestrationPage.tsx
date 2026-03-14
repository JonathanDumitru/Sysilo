import { useState } from 'react';

type AgentType = 'data' | 'sales' | 'support' | 'security' | 'custom';
type AgentStatus = 'active' | 'idle' | 'degraded' | 'offline';

interface EnterpriseAgent {
  id: string;
  name: string;
  type: AgentType;
  owner: string;
  status: AgentStatus;
  capabilities: string[];
  successRate: number;
  avgLatencyMs: number;
  invocations24h: number;
  memoryMB: number;
  cpuMillis: number;
  lastActiveAt: string;
}

const STATUS_COLORS: Record<AgentStatus, string> = {
  active: 'bg-green-500',
  idle: 'bg-gray-500',
  degraded: 'bg-yellow-500',
  offline: 'bg-red-500',
};

const TYPE_COLORS: Record<AgentType, string> = {
  data: 'text-cyan-400 bg-cyan-900/30',
  sales: 'text-green-400 bg-green-900/30',
  support: 'text-purple-400 bg-purple-900/30',
  security: 'text-red-400 bg-red-900/30',
  custom: 'text-gray-400 bg-gray-700',
};

const MOCK_AGENTS: EnterpriseAgent[] = [
  { id: 'a1', name: 'Pipeline Metrics Agent', type: 'data', owner: 'Data Team', status: 'active', capabilities: ['query_metrics', 'generate_reports', 'alert_triage'], successRate: 99.2, avgLatencyMs: 145, invocations24h: 12450, memoryMB: 256, cpuMillis: 500, lastActiveAt: '2 min ago' },
  { id: 'a2', name: 'Lead Scoring Agent', type: 'sales', owner: 'Revenue Ops', status: 'active', capabilities: ['score_leads', 'enrich_contacts', 'update_crm'], successRate: 97.8, avgLatencyMs: 320, invocations24h: 8900, memoryMB: 512, cpuMillis: 800, lastActiveAt: '30 sec ago' },
  { id: 'a3', name: 'Ticket Router Agent', type: 'support', owner: 'Support Engineering', status: 'active', capabilities: ['classify_tickets', 'route_priority', 'suggest_resolution'], successRate: 96.5, avgLatencyMs: 210, invocations24h: 5600, memoryMB: 384, cpuMillis: 600, lastActiveAt: '1 min ago' },
  { id: 'a4', name: 'Threat Detection Agent', type: 'security', owner: 'InfoSec', status: 'degraded', capabilities: ['scan_logs', 'detect_anomalies', 'trigger_alerts', 'quarantine'], successRate: 94.1, avgLatencyMs: 890, invocations24h: 24000, memoryMB: 1024, cpuMillis: 2000, lastActiveAt: '5 min ago' },
  { id: 'a5', name: 'Schema Evolution Agent', type: 'data', owner: 'Platform Team', status: 'active', capabilities: ['detect_schema_changes', 'propose_migrations', 'validate_compatibility'], successRate: 99.8, avgLatencyMs: 75, invocations24h: 340, memoryMB: 128, cpuMillis: 200, lastActiveAt: '15 min ago' },
  { id: 'a6', name: 'Compliance Checker Agent', type: 'security', owner: 'GRC Team', status: 'idle', capabilities: ['policy_evaluation', 'compliance_scoring', 'audit_reporting'], successRate: 100.0, avgLatencyMs: 550, invocations24h: 120, memoryMB: 256, cpuMillis: 400, lastActiveAt: '2 hours ago' },
  { id: 'a7', name: 'Custom ETL Orchestrator', type: 'custom', owner: 'Data Engineering', status: 'active', capabilities: ['orchestrate_pipelines', 'retry_failures', 'optimize_scheduling'], successRate: 98.5, avgLatencyMs: 180, invocations24h: 1800, memoryMB: 512, cpuMillis: 1000, lastActiveAt: '45 sec ago' },
];

export function AgentOrchestrationPage() {
  const [filterType, setFilterType] = useState<string>('all');
  const [filterStatus, setFilterStatus] = useState<string>('all');

  const filtered = MOCK_AGENTS
    .filter(a => filterType === 'all' || a.type === filterType)
    .filter(a => filterStatus === 'all' || a.status === filterStatus);

  const totalInvocations = MOCK_AGENTS.reduce((s, a) => s + a.invocations24h, 0);
  const avgSuccessRate = MOCK_AGENTS.reduce((s, a) => s + a.successRate, 0) / MOCK_AGENTS.length;
  const activeCount = MOCK_AGENTS.filter(a => a.status === 'active').length;
  const totalMemoryMB = MOCK_AGENTS.reduce((s, a) => s + a.memoryMB, 0);

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-white">Agent Orchestration Hub</h1>
          <p className="text-sm text-gray-400 mt-1">Enterprise-wide AI agent registry, monitoring, and governance</p>
        </div>
        <button className="px-4 py-2 bg-blue-600 hover:bg-blue-500 text-white rounded-lg text-sm font-medium transition-colors">
          Register Agent
        </button>
      </div>

      {/* Portfolio Summary */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        {[
          { label: 'Total Agents', value: MOCK_AGENTS.length, detail: `${activeCount} active` },
          { label: 'Invocations (24h)', value: totalInvocations.toLocaleString(), detail: 'across all agents' },
          { label: 'Avg Success Rate', value: `${avgSuccessRate.toFixed(1)}%`, detail: 'portfolio-wide' },
          { label: 'Resource Usage', value: `${(totalMemoryMB / 1024).toFixed(1)} GB`, detail: 'total memory allocated' },
        ].map(stat => (
          <div key={stat.label} className="bg-gray-800/50 border border-gray-700 rounded-xl p-4">
            <p className="text-xs text-gray-500 uppercase tracking-wider">{stat.label}</p>
            <p className="text-2xl font-bold text-white mt-1">{stat.value}</p>
            <p className="text-xs text-gray-500 mt-0.5">{stat.detail}</p>
          </div>
        ))}
      </div>

      {/* Filters */}
      <div className="flex gap-3">
        <select value={filterType} onChange={e => setFilterType(e.target.value)} className="px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg text-gray-300 text-sm">
          <option value="all">All Types</option>
          <option value="data">Data</option>
          <option value="sales">Sales</option>
          <option value="support">Support</option>
          <option value="security">Security</option>
          <option value="custom">Custom</option>
        </select>
        <select value={filterStatus} onChange={e => setFilterStatus(e.target.value)} className="px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg text-gray-300 text-sm">
          <option value="all">All Statuses</option>
          <option value="active">Active</option>
          <option value="idle">Idle</option>
          <option value="degraded">Degraded</option>
          <option value="offline">Offline</option>
        </select>
      </div>

      {/* Agent Cards */}
      <div className="space-y-3">
        {filtered.map(agent => (
          <div key={agent.id} className="bg-gray-800/50 border border-gray-700 rounded-xl p-5 hover:border-gray-600 transition-colors cursor-pointer">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-4">
                <div className={`w-2 h-2 rounded-full ${STATUS_COLORS[agent.status]}`} />
                <div>
                  <div className="flex items-center gap-2">
                    <h3 className="text-white font-semibold">{agent.name}</h3>
                    <span className={`text-xs px-2 py-0.5 rounded ${TYPE_COLORS[agent.type]}`}>{agent.type}</span>
                  </div>
                  <p className="text-xs text-gray-500">Owned by {agent.owner} &middot; Last active {agent.lastActiveAt}</p>
                </div>
              </div>
              <div className="flex items-center gap-6 text-sm">
                <div className="text-right">
                  <p className="text-gray-500 text-xs">Success Rate</p>
                  <p className={`font-mono ${agent.successRate >= 99 ? 'text-green-400' : agent.successRate >= 95 ? 'text-yellow-400' : 'text-red-400'}`}>{agent.successRate}%</p>
                </div>
                <div className="text-right">
                  <p className="text-gray-500 text-xs">Avg Latency</p>
                  <p className="text-gray-300 font-mono">{agent.avgLatencyMs}ms</p>
                </div>
                <div className="text-right">
                  <p className="text-gray-500 text-xs">Invocations (24h)</p>
                  <p className="text-gray-300 font-mono">{agent.invocations24h.toLocaleString()}</p>
                </div>
                <div className="text-right">
                  <p className="text-gray-500 text-xs">Resources</p>
                  <p className="text-gray-300 font-mono text-xs">{agent.memoryMB}MB / {agent.cpuMillis}m</p>
                </div>
              </div>
            </div>
            <div className="flex flex-wrap gap-1.5 mt-3 ml-6">
              {agent.capabilities.map(cap => (
                <span key={cap} className="text-xs px-2 py-0.5 bg-gray-700/50 text-gray-400 rounded">{cap}</span>
              ))}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
