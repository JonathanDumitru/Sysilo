import { useState } from 'react';

type SignalType = 'latency_spike' | 'schema_drift' | 'auth_failure' | 'data_quality_drop' | 'error_rate_surge' | 'throughput_drop';
type Severity = 'low' | 'medium' | 'high' | 'critical';
type Diagnosis = 'isolated_failure' | 'systemic_issue' | 'cascading_failure' | 'external_dependency' | 'false_positive';

interface DangerSignal {
  id: string;
  integrationName: string;
  signalType: SignalType;
  severity: Severity;
  details: string;
  detectedAt: string;
  acknowledged: boolean;
  resolved: boolean;
  autoRemediated: boolean;
}

interface ImmuneMemory {
  id: string;
  failureSignature: string;
  failureClass: string;
  successCount: number;
  failureCount: number;
  avgResolutionTimeMs: number;
  autoRemediate: boolean;
  lastSeen: string;
}

interface VaccinationRecord {
  id: string;
  failureSignature: string;
  countermeasure: string;
  appliedTenants: number;
  effectivenessScore: number;
  distributedAt: string;
}

const SIGNAL_ICONS: Record<SignalType, { label: string; color: string }> = {
  latency_spike: { label: 'Latency Spike', color: 'text-yellow-400 bg-yellow-900/30' },
  schema_drift: { label: 'Schema Drift', color: 'text-purple-400 bg-purple-900/30' },
  auth_failure: { label: 'Auth Failure', color: 'text-red-400 bg-red-900/30' },
  data_quality_drop: { label: 'Quality Drop', color: 'text-orange-400 bg-orange-900/30' },
  error_rate_surge: { label: 'Error Surge', color: 'text-red-400 bg-red-900/30' },
  throughput_drop: { label: 'Throughput Drop', color: 'text-blue-400 bg-blue-900/30' },
};

const SEVERITY_COLORS: Record<Severity, string> = {
  low: 'text-gray-400 bg-gray-700',
  medium: 'text-yellow-400 bg-yellow-900/30',
  high: 'text-orange-400 bg-orange-900/30',
  critical: 'text-red-400 bg-red-900/30',
};

const MOCK_SIGNALS: DangerSignal[] = [
  { id: 's1', integrationName: 'Salesforce CRM Sync', signalType: 'latency_spike', severity: 'medium', details: 'P99 latency increased from 450ms to 2,100ms over last 15 minutes', detectedAt: '3 min ago', acknowledged: false, resolved: false, autoRemediated: false },
  { id: 's2', integrationName: 'SAP ERP Pipeline', signalType: 'schema_drift', severity: 'high', details: 'Field "cost_center_v2" added to purchase_orders table, 3 downstream integrations affected', detectedAt: '12 min ago', acknowledged: true, resolved: false, autoRemediated: false },
  { id: 's3', integrationName: 'Stripe Payments', signalType: 'auth_failure', severity: 'critical', details: 'OAuth token refresh failed 5 consecutive times. API key may be revoked.', detectedAt: '5 min ago', acknowledged: false, resolved: false, autoRemediated: false },
  { id: 's4', integrationName: 'MongoDB Atlas', signalType: 'data_quality_drop', severity: 'low', details: 'Null rate for "email" field increased from 2% to 8% in last hour', detectedAt: '25 min ago', acknowledged: true, resolved: true, autoRemediated: true },
  { id: 's5', integrationName: 'Kafka Event Stream', signalType: 'throughput_drop', severity: 'medium', details: 'Consumer group lag increased to 45,000 messages. Processing rate dropped 60%', detectedAt: '8 min ago', acknowledged: false, resolved: false, autoRemediated: false },
  { id: 's6', integrationName: 'Bloomberg Feed', signalType: 'error_rate_surge', severity: 'high', details: 'Error rate spiked from 0.1% to 12.3%. Correlation with API provider status page incident.', detectedAt: '18 min ago', acknowledged: true, resolved: false, autoRemediated: false },
];

const MOCK_MEMORIES: ImmuneMemory[] = [
  { id: 'm1', failureSignature: 'salesforce_oauth_token_expired', failureClass: 'AuthExpiry', successCount: 47, failureCount: 2, avgResolutionTimeMs: 3200, autoRemediate: true, lastSeen: '3 days ago' },
  { id: 'm2', failureSignature: 'postgres_connection_pool_exhausted', failureClass: 'ConnectionTimeout', successCount: 23, failureCount: 1, avgResolutionTimeMs: 8500, autoRemediate: true, lastSeen: '1 week ago' },
  { id: 'm3', failureSignature: 'snowflake_warehouse_suspended', failureClass: 'ConnectionTimeout', successCount: 12, failureCount: 0, avgResolutionTimeMs: 15000, autoRemediate: true, lastSeen: '2 days ago' },
  { id: 'm4', failureSignature: 'api_rate_limit_429', failureClass: 'RateLimitBreach', successCount: 89, failureCount: 5, avgResolutionTimeMs: 60000, autoRemediate: true, lastSeen: '6 hours ago' },
  { id: 'm5', failureSignature: 'schema_field_removed', failureClass: 'SchemaChange', successCount: 8, failureCount: 3, avgResolutionTimeMs: 120000, autoRemediate: false, lastSeen: '12 days ago' },
];

const MOCK_VACCINATIONS: VaccinationRecord[] = [
  { id: 'v1', failureSignature: 'salesforce_api_v58_deprecation', countermeasure: 'Auto-upgrade API version in connector config from v58 to v60', appliedTenants: 42, effectivenessScore: 98.5, distributedAt: '5 days ago' },
  { id: 'v2', failureSignature: 'aws_us_east_1_latency_spike', countermeasure: 'Enable multi-region failover and increase connection timeout to 30s', appliedTenants: 28, effectivenessScore: 95.0, distributedAt: '2 weeks ago' },
  { id: 'v3', failureSignature: 'kafka_consumer_rebalance_storm', countermeasure: 'Apply static group membership and increase session timeout to 45s', appliedTenants: 15, effectivenessScore: 100.0, distributedAt: '1 month ago' },
];

export function ImmuneSystemPage() {
  const [activeTab, setActiveTab] = useState<'signals' | 'memory' | 'vaccinations'>('signals');

  const activeSignals = MOCK_SIGNALS.filter(s => !s.resolved).length;
  const criticalSignals = MOCK_SIGNALS.filter(s => s.severity === 'critical' && !s.resolved).length;
  const autoRemediationRate = (MOCK_MEMORIES.filter(m => m.autoRemediate).length / MOCK_MEMORIES.length * 100);
  const avgResolutionTime = MOCK_MEMORIES.reduce((s, m) => s + m.avgResolutionTimeMs, 0) / MOCK_MEMORIES.length;

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-2xl font-bold text-white">Digital Immune System</h1>
        <p className="text-sm text-gray-400 mt-1">Self-healing infrastructure with biological immune system-inspired anomaly detection and remediation</p>
      </div>

      {/* Resilience Score */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        {[
          { label: 'Active Danger Signals', value: activeSignals, color: activeSignals > 3 ? 'text-red-400' : 'text-yellow-400', detail: `${criticalSignals} critical` },
          { label: 'Immune Memories', value: MOCK_MEMORIES.length, color: 'text-blue-400', detail: 'known failure patterns' },
          { label: 'Auto-Remediation Rate', value: `${autoRemediationRate.toFixed(0)}%`, color: 'text-green-400', detail: 'of known failures' },
          { label: 'Avg Resolution Time', value: `${(avgResolutionTime / 1000).toFixed(1)}s`, color: 'text-white', detail: 'for auto-remediated' },
        ].map(stat => (
          <div key={stat.label} className="bg-gray-800/50 border border-gray-700 rounded-xl p-4">
            <p className="text-xs text-gray-500 uppercase tracking-wider">{stat.label}</p>
            <p className={`text-2xl font-bold mt-1 ${stat.color}`}>{stat.value}</p>
            <p className="text-xs text-gray-500 mt-0.5">{stat.detail}</p>
          </div>
        ))}
      </div>

      {/* Tabs */}
      <div className="flex gap-1 bg-gray-800/50 rounded-lg p-1 w-fit">
        {(['signals', 'memory', 'vaccinations'] as const).map(tab => (
          <button
            key={tab}
            onClick={() => setActiveTab(tab)}
            className={`px-4 py-2 rounded-md text-sm font-medium transition-colors ${
              activeTab === tab ? 'bg-gray-700 text-white' : 'text-gray-400 hover:text-gray-300'
            }`}
          >
            {tab === 'signals' ? `Danger Signals (${activeSignals})` : tab === 'memory' ? 'Immune Memory' : 'Vaccinations'}
          </button>
        ))}
      </div>

      {/* Tab Content */}
      {activeTab === 'signals' && (
        <div className="space-y-3">
          {MOCK_SIGNALS.map(signal => {
            const typeInfo = SIGNAL_ICONS[signal.signalType];
            return (
              <div key={signal.id} className={`bg-gray-800/50 border rounded-xl p-5 transition-colors ${signal.resolved ? 'border-gray-700/50 opacity-60' : signal.severity === 'critical' ? 'border-red-800 hover:border-red-700' : 'border-gray-700 hover:border-gray-600'}`}>
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-3">
                    <div className={`w-2 h-2 rounded-full ${signal.resolved ? 'bg-green-500' : signal.severity === 'critical' ? 'bg-red-500 animate-pulse' : 'bg-yellow-500'}`} />
                    <h3 className="text-white font-semibold">{signal.integrationName}</h3>
                    <span className={`text-xs px-2 py-0.5 rounded ${typeInfo.color}`}>{typeInfo.label}</span>
                    <span className={`text-xs px-2 py-0.5 rounded ${SEVERITY_COLORS[signal.severity]}`}>{signal.severity}</span>
                    {signal.autoRemediated && <span className="text-xs px-2 py-0.5 rounded bg-green-900/30 text-green-400">Auto-remediated</span>}
                    {signal.resolved && <span className="text-xs px-2 py-0.5 rounded bg-gray-700 text-gray-400">Resolved</span>}
                  </div>
                  <div className="flex items-center gap-2">
                    <span className="text-xs text-gray-500">{signal.detectedAt}</span>
                    {!signal.resolved && (
                      <button className="px-3 py-1.5 bg-blue-600 hover:bg-blue-500 text-white rounded-lg text-xs font-medium transition-colors">
                        {signal.acknowledged ? 'Remediate' : 'Acknowledge'}
                      </button>
                    )}
                  </div>
                </div>
                <p className="text-sm text-gray-400 mt-2 ml-5">{signal.details}</p>
              </div>
            );
          })}
        </div>
      )}

      {activeTab === 'memory' && (
        <div className="space-y-3">
          {MOCK_MEMORIES.map(memory => (
            <div key={memory.id} className="bg-gray-800/50 border border-gray-700 rounded-xl p-5 hover:border-gray-600 transition-colors">
              <div className="flex items-center justify-between">
                <div>
                  <div className="flex items-center gap-2 mb-1">
                    <h3 className="text-white font-semibold font-mono text-sm">{memory.failureSignature}</h3>
                    <span className="text-xs px-2 py-0.5 bg-gray-700 text-gray-400 rounded">{memory.failureClass}</span>
                    {memory.autoRemediate && <span className="text-xs px-2 py-0.5 bg-green-900/30 text-green-400 rounded">Auto-heal</span>}
                  </div>
                  <p className="text-xs text-gray-500">Last seen: {memory.lastSeen}</p>
                </div>
                <div className="flex items-center gap-6 text-sm">
                  <div className="text-right">
                    <p className="text-xs text-gray-500">Success / Fail</p>
                    <p className="font-mono text-green-400">{memory.successCount} <span className="text-gray-600">/</span> <span className="text-red-400">{memory.failureCount}</span></p>
                  </div>
                  <div className="text-right">
                    <p className="text-xs text-gray-500">Avg Resolution</p>
                    <p className="font-mono text-gray-300">{(memory.avgResolutionTimeMs / 1000).toFixed(1)}s</p>
                  </div>
                  <div className="text-right">
                    <p className="text-xs text-gray-500">Reliability</p>
                    <p className="font-mono text-gray-300">{((memory.successCount / (memory.successCount + memory.failureCount)) * 100).toFixed(1)}%</p>
                  </div>
                </div>
              </div>
            </div>
          ))}
        </div>
      )}

      {activeTab === 'vaccinations' && (
        <div className="space-y-3">
          {MOCK_VACCINATIONS.map(vax => (
            <div key={vax.id} className="bg-gray-800/50 border border-gray-700 rounded-xl p-5 hover:border-gray-600 transition-colors">
              <div className="flex items-center justify-between">
                <div>
                  <h3 className="text-white font-semibold font-mono text-sm mb-1">{vax.failureSignature}</h3>
                  <p className="text-sm text-gray-400">{vax.countermeasure}</p>
                  <p className="text-xs text-gray-500 mt-1">Distributed {vax.distributedAt}</p>
                </div>
                <div className="flex items-center gap-6 text-sm">
                  <div className="text-right">
                    <p className="text-xs text-gray-500">Applied To</p>
                    <p className="font-mono text-gray-300">{vax.appliedTenants} tenants</p>
                  </div>
                  <div className="text-right">
                    <p className="text-xs text-gray-500">Effectiveness</p>
                    <p className={`font-mono ${vax.effectivenessScore >= 95 ? 'text-green-400' : 'text-yellow-400'}`}>{vax.effectivenessScore}%</p>
                  </div>
                </div>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
