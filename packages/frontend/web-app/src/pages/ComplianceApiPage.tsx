import { useState } from 'react';

interface ComplianceScore {
  entityType: string;
  entityName: string;
  overallScore: number;
  policyAdherence: number;
  dataQuality: number;
  accessControl: number;
  auditCompleteness: number;
  riskLevel: 'low' | 'medium' | 'high' | 'critical';
  issuesCount: number;
}

interface PolicyDecision {
  id: string;
  requestingSystem: string;
  action: string;
  resource: string;
  decision: 'allow' | 'deny';
  matchedPolicies: string[];
  evaluationTimeMs: number;
  timestamp: string;
}

interface RegulatoryChange {
  id: string;
  regulation: string;
  title: string;
  changeType: string;
  status: 'pending_review' | 'approved' | 'applied';
  effectiveDate: string;
}

const MOCK_SCORES: ComplianceScore[] = [
  { entityType: 'Integration', entityName: 'Salesforce CRM Sync', overallScore: 95.2, policyAdherence: 98, dataQuality: 94, accessControl: 96, auditCompleteness: 90, riskLevel: 'low', issuesCount: 1 },
  { entityType: 'Integration', entityName: 'SAP ERP Pipeline', overallScore: 88.1, policyAdherence: 92, dataQuality: 85, accessControl: 90, auditCompleteness: 82, riskLevel: 'medium', issuesCount: 3 },
  { entityType: 'Data Product', entityName: 'Customer 360 Profile', overallScore: 97.5, policyAdherence: 100, dataQuality: 96, accessControl: 98, auditCompleteness: 94, riskLevel: 'low', issuesCount: 0 },
  { entityType: 'Agent', entityName: 'Threat Detection Agent', overallScore: 72.4, policyAdherence: 78, dataQuality: 70, accessControl: 68, auditCompleteness: 75, riskLevel: 'high', issuesCount: 7 },
  { entityType: 'Integration', entityName: 'Bloomberg Data Feed', overallScore: 91.8, policyAdherence: 95, dataQuality: 88, accessControl: 94, auditCompleteness: 88, riskLevel: 'low', issuesCount: 2 },
  { entityType: 'Data Product', entityName: 'Supply Chain Signals', overallScore: 85.3, policyAdherence: 88, dataQuality: 82, accessControl: 86, auditCompleteness: 84, riskLevel: 'medium', issuesCount: 4 },
];

const MOCK_DECISIONS: PolicyDecision[] = [
  { id: 'd1', requestingSystem: 'Sales Agent', action: 'read', resource: 'customer_data', decision: 'allow', matchedPolicies: ['data-access-policy'], evaluationTimeMs: 12, timestamp: '2 min ago' },
  { id: 'd2', requestingSystem: 'ETL Pipeline', action: 'export', resource: 'pii_records', decision: 'deny', matchedPolicies: ['pii-protection-policy', 'gdpr-compliance'], evaluationTimeMs: 8, timestamp: '5 min ago' },
  { id: 'd3', requestingSystem: 'Analytics Dashboard', action: 'query', resource: 'revenue_metrics', decision: 'allow', matchedPolicies: ['analytics-access-policy'], evaluationTimeMs: 5, timestamp: '12 min ago' },
  { id: 'd4', requestingSystem: 'Third-Party App', action: 'write', resource: 'user_profiles', decision: 'deny', matchedPolicies: ['external-write-restriction'], evaluationTimeMs: 15, timestamp: '18 min ago' },
  { id: 'd5', requestingSystem: 'Compliance Agent', action: 'audit', resource: 'all_resources', decision: 'allow', matchedPolicies: ['audit-access-policy'], evaluationTimeMs: 3, timestamp: '25 min ago' },
];

const MOCK_REG_CHANGES: RegulatoryChange[] = [
  { id: 'r1', regulation: 'GDPR', title: 'Updated data portability requirements for AI-processed data', changeType: 'amendment', status: 'pending_review', effectiveDate: '2026-06-01' },
  { id: 'r2', regulation: 'SOX', title: 'Enhanced internal controls for automated financial reporting', changeType: 'new_requirement', status: 'approved', effectiveDate: '2026-04-15' },
  { id: 'r3', regulation: 'HIPAA', title: 'Expanded breach notification requirements for AI agents', changeType: 'amendment', status: 'applied', effectiveDate: '2026-01-01' },
];

const RISK_COLORS: Record<string, string> = {
  low: 'text-green-400 bg-green-900/30',
  medium: 'text-yellow-400 bg-yellow-900/30',
  high: 'text-orange-400 bg-orange-900/30',
  critical: 'text-red-400 bg-red-900/30',
};

export function ComplianceApiPage() {
  const [activeTab, setActiveTab] = useState<'scores' | 'decisions' | 'regulatory'>('scores');

  const avgScore = MOCK_SCORES.reduce((s, c) => s + c.overallScore, 0) / MOCK_SCORES.length;
  const totalDecisions = 24567;
  const denyRate = 8.3;

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-2xl font-bold text-white">Compliance Center</h1>
        <p className="text-sm text-gray-400 mt-1">Governance API, compliance scoring, and regulatory change management</p>
      </div>

      {/* Summary */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        {[
          { label: 'Avg Compliance Score', value: `${avgScore.toFixed(1)}%`, color: avgScore >= 90 ? 'text-green-400' : 'text-yellow-400' },
          { label: 'Policy Decisions (30d)', value: totalDecisions.toLocaleString(), color: 'text-white' },
          { label: 'Deny Rate', value: `${denyRate}%`, color: denyRate > 10 ? 'text-red-400' : 'text-green-400' },
          { label: 'Pending Regulatory Changes', value: MOCK_REG_CHANGES.filter(r => r.status === 'pending_review').length, color: 'text-yellow-400' },
        ].map(stat => (
          <div key={stat.label} className="bg-gray-800/50 border border-gray-700 rounded-xl p-4">
            <p className="text-xs text-gray-500 uppercase tracking-wider">{stat.label}</p>
            <p className={`text-2xl font-bold mt-1 ${stat.color}`}>{stat.value}</p>
          </div>
        ))}
      </div>

      {/* Tabs */}
      <div className="flex gap-1 bg-gray-800/50 rounded-lg p-1 w-fit">
        {(['scores', 'decisions', 'regulatory'] as const).map(tab => (
          <button
            key={tab}
            onClick={() => setActiveTab(tab)}
            className={`px-4 py-2 rounded-md text-sm font-medium transition-colors ${
              activeTab === tab ? 'bg-gray-700 text-white' : 'text-gray-400 hover:text-gray-300'
            }`}
          >
            {tab === 'scores' ? 'Compliance Scores' : tab === 'decisions' ? 'Policy Decisions' : 'Regulatory Changes'}
          </button>
        ))}
      </div>

      {/* Tab Content */}
      {activeTab === 'scores' && (
        <div className="space-y-3">
          {MOCK_SCORES.map(score => (
            <div key={score.entityName} className="bg-gray-800/50 border border-gray-700 rounded-xl p-5 hover:border-gray-600 transition-colors">
              <div className="flex items-center justify-between mb-3">
                <div className="flex items-center gap-3">
                  <h3 className="text-white font-semibold">{score.entityName}</h3>
                  <span className="text-xs px-2 py-0.5 bg-gray-700 text-gray-400 rounded">{score.entityType}</span>
                  <span className={`text-xs px-2 py-0.5 rounded ${RISK_COLORS[score.riskLevel]}`}>{score.riskLevel} risk</span>
                </div>
                <div className="flex items-center gap-2">
                  <span className={`text-2xl font-bold ${score.overallScore >= 90 ? 'text-green-400' : score.overallScore >= 70 ? 'text-yellow-400' : 'text-red-400'}`}>
                    {score.overallScore}%
                  </span>
                  {score.issuesCount > 0 && (
                    <span className="text-xs px-2 py-0.5 bg-red-900/30 text-red-400 rounded">{score.issuesCount} issues</span>
                  )}
                </div>
              </div>
              <div className="grid grid-cols-4 gap-4">
                {[
                  { label: 'Policy Adherence', value: score.policyAdherence },
                  { label: 'Data Quality', value: score.dataQuality },
                  { label: 'Access Control', value: score.accessControl },
                  { label: 'Audit Completeness', value: score.auditCompleteness },
                ].map(metric => (
                  <div key={metric.label}>
                    <p className="text-xs text-gray-500">{metric.label}</p>
                    <div className="flex items-center gap-2 mt-1">
                      <div className="flex-1 h-1.5 bg-gray-700 rounded-full overflow-hidden">
                        <div
                          className={`h-full rounded-full ${metric.value >= 90 ? 'bg-green-500' : metric.value >= 70 ? 'bg-yellow-500' : 'bg-red-500'}`}
                          style={{ width: `${metric.value}%` }}
                        />
                      </div>
                      <span className="text-xs text-gray-400 font-mono w-8">{metric.value}%</span>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          ))}
        </div>
      )}

      {activeTab === 'decisions' && (
        <div className="bg-gray-800/50 border border-gray-700 rounded-xl overflow-hidden">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-gray-700">
                <th className="text-left text-gray-500 text-xs uppercase py-3 px-4">System</th>
                <th className="text-left text-gray-500 text-xs uppercase py-3 px-4">Action</th>
                <th className="text-left text-gray-500 text-xs uppercase py-3 px-4">Resource</th>
                <th className="text-left text-gray-500 text-xs uppercase py-3 px-4">Decision</th>
                <th className="text-left text-gray-500 text-xs uppercase py-3 px-4">Policies</th>
                <th className="text-right text-gray-500 text-xs uppercase py-3 px-4">Eval Time</th>
                <th className="text-right text-gray-500 text-xs uppercase py-3 px-4">When</th>
              </tr>
            </thead>
            <tbody>
              {MOCK_DECISIONS.map(d => (
                <tr key={d.id} className="border-b border-gray-700/50 hover:bg-gray-700/20">
                  <td className="py-3 px-4 text-gray-300">{d.requestingSystem}</td>
                  <td className="py-3 px-4 text-gray-400">{d.action}</td>
                  <td className="py-3 px-4 text-gray-400 font-mono text-xs">{d.resource}</td>
                  <td className="py-3 px-4">
                    <span className={`text-xs px-2 py-0.5 rounded ${d.decision === 'allow' ? 'text-green-400 bg-green-900/30' : 'text-red-400 bg-red-900/30'}`}>
                      {d.decision}
                    </span>
                  </td>
                  <td className="py-3 px-4">
                    <div className="flex gap-1">
                      {d.matchedPolicies.map(p => (
                        <span key={p} className="text-xs px-1.5 py-0.5 bg-gray-700 text-gray-400 rounded">{p}</span>
                      ))}
                    </div>
                  </td>
                  <td className="py-3 px-4 text-right text-gray-400 font-mono">{d.evaluationTimeMs}ms</td>
                  <td className="py-3 px-4 text-right text-gray-500 text-xs">{d.timestamp}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {activeTab === 'regulatory' && (
        <div className="space-y-3">
          {MOCK_REG_CHANGES.map(change => (
            <div key={change.id} className="bg-gray-800/50 border border-gray-700 rounded-xl p-5 hover:border-gray-600 transition-colors">
              <div className="flex items-center justify-between">
                <div>
                  <div className="flex items-center gap-2 mb-1">
                    <span className="text-xs px-2 py-0.5 bg-blue-900/30 text-blue-400 rounded border border-blue-800">{change.regulation}</span>
                    <span className="text-xs px-2 py-0.5 bg-gray-700 text-gray-400 rounded">{change.changeType}</span>
                    <span className={`text-xs px-2 py-0.5 rounded ${
                      change.status === 'pending_review' ? 'bg-yellow-900/30 text-yellow-400' :
                      change.status === 'approved' ? 'bg-green-900/30 text-green-400' :
                      'bg-gray-700 text-gray-400'
                    }`}>{change.status.replace('_', ' ')}</span>
                  </div>
                  <h3 className="text-white font-medium">{change.title}</h3>
                  <p className="text-xs text-gray-500 mt-1">Effective: {change.effectiveDate}</p>
                </div>
                {change.status === 'pending_review' && (
                  <button className="px-3 py-1.5 bg-blue-600 hover:bg-blue-500 text-white rounded-lg text-xs font-medium transition-colors">
                    Review & Apply
                  </button>
                )}
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
