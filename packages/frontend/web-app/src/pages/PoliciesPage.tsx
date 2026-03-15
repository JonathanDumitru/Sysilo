import { useState } from 'react';
import {
  Plus,
  Search,
  Scale,
  CheckCircle,
  XCircle,
  MoreVertical,
  X,
  Code,
  Eye,
} from 'lucide-react';

// Mock data
const policies = [
  {
    id: '1',
    name: 'Integration Naming Convention',
    description: 'Enforces naming conventions for all integrations',
    scope: 'integration',
    enforcement: 'enforce',
    severity: 'low',
    enabled: true,
    violations: { open: 2, resolved: 15 },
    lastEvaluated: '5 min ago',
    rego: `package policy

deny[msg] {
  not regex.match("^[a-z][a-z0-9-]*$", input.name)
  msg := "Integration name must be lowercase with hyphens only"
}`,
  },
  {
    id: '2',
    name: 'Connection Security Requirements',
    description: 'Ensures all connections use secure protocols and authentication',
    scope: 'connection',
    enforcement: 'enforce',
    severity: 'critical',
    enabled: true,
    violations: { open: 1, resolved: 8 },
    lastEvaluated: '2 min ago',
    rego: `package policy

deny[msg] {
  input.protocol != "https"
  msg := "Connection must use HTTPS protocol"
}

deny[msg] {
  not input.authentication
  msg := "Connection must have authentication configured"
}`,
  },
  {
    id: '3',
    name: 'Agent Deployment Policy',
    description: 'Controls where agents can be deployed',
    scope: 'agent',
    enforcement: 'warn',
    severity: 'medium',
    enabled: true,
    violations: { open: 0, resolved: 3 },
    lastEvaluated: '10 min ago',
    rego: `package policy

deny[msg] {
  input.environment == "production"
  not input.approved
  msg := "Production agents require approval"
}`,
  },
  {
    id: '4',
    name: 'Data Export Restrictions',
    description: 'Restricts export of sensitive data types',
    scope: 'data_entity',
    enforcement: 'enforce',
    severity: 'high',
    enabled: false,
    violations: { open: 0, resolved: 0 },
    lastEvaluated: 'Never',
    rego: `package policy

deny[msg] {
  input.classification == "pii"
  not input.encrypted
  msg := "PII data must be encrypted before export"
}`,
  },
];

const violations = [
  {
    id: '1',
    policyId: '1',
    policyName: 'Integration Naming Convention',
    resourceType: 'integration',
    resourceId: 'Test_Integration_123',
    details: 'Name contains uppercase letters and underscores',
    severity: 'low',
    status: 'open',
    created: '2024-01-15T10:30:00Z',
  },
  {
    id: '2',
    policyId: '2',
    policyName: 'Connection Security Requirements',
    resourceType: 'connection',
    resourceId: 'legacy-api-connection',
    details: 'Connection uses HTTP instead of HTTPS',
    severity: 'critical',
    status: 'open',
    created: '2024-01-15T09:15:00Z',
  },
  {
    id: '3',
    policyId: '1',
    policyName: 'Integration Naming Convention',
    resourceType: 'integration',
    resourceId: 'MyIntegration',
    details: 'Name contains uppercase letters',
    severity: 'low',
    status: 'resolved',
    created: '2024-01-14T15:00:00Z',
    resolved: '2024-01-14T16:30:00Z',
  },
];

type TabType = 'policies' | 'violations';

export function PoliciesPage() {
  const [activeTab, setActiveTab] = useState<TabType>('policies');
  const [searchQuery, setSearchQuery] = useState('');
  const [scopeFilter, setScopeFilter] = useState<string>('all');
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [selectedPolicy, setSelectedPolicy] = useState<typeof policies[0] | null>(null);

  const getSeverityColor = (severity: string) => {
    switch (severity) {
      case 'critical':
        return 'bg-red-900/40 text-red-400';
      case 'high':
        return 'bg-orange-900/40 text-orange-400';
      case 'medium':
        return 'bg-yellow-900/40 text-yellow-400';
      case 'low':
        return 'bg-blue-900/40 text-blue-400';
      default:
        return 'bg-surface-overlay text-gray-300';
    }
  };

  const getEnforcementColor = (enforcement: string) => {
    switch (enforcement) {
      case 'enforce':
        return 'bg-red-900/30 text-red-400 border-red-800';
      case 'warn':
        return 'bg-yellow-900/30 text-yellow-400 border-yellow-800';
      case 'audit':
        return 'bg-surface-overlay/50 text-gray-300 border-surface-border';
      default:
        return 'bg-surface-overlay/50 text-gray-300 border-surface-border';
    }
  };

  const getScopeLabel = (scope: string) => {
    switch (scope) {
      case 'integration':
        return 'Integration';
      case 'connection':
        return 'Connection';
      case 'agent':
        return 'Agent';
      case 'data_entity':
        return 'Data Entity';
      case 'all':
        return 'All Resources';
      default:
        return scope;
    }
  };

  const formatDate = (dateStr: string) => {
    const date = new Date(dateStr);
    return date.toLocaleString();
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-white">Policies</h1>
          <p className="text-gray-500">Define and enforce governance policies using Rego</p>
        </div>
        <button
          onClick={() => setShowCreateModal(true)}
          className="flex items-center gap-2 px-4 py-2 bg-primary-600 text-white rounded-lg text-sm font-medium hover:bg-primary-700"
        >
          <Plus className="w-4 h-4" />
          Create Policy
        </button>
      </div>

      {/* Tabs */}
      <div className="border-b border-surface-border">
        <nav className="flex gap-6">
          <button
            onClick={() => setActiveTab('policies')}
            className={`py-3 text-sm font-medium border-b-2 transition-colors ${
              activeTab === 'policies'
                ? 'border-primary-600 text-primary-600'
                : 'border-transparent text-gray-500 hover:text-gray-200'
            }`}
          >
            Policies
            <span className="ml-2 px-2 py-0.5 text-xs rounded-full bg-surface-overlay text-gray-300">
              {policies.length}
            </span>
          </button>
          <button
            onClick={() => setActiveTab('violations')}
            className={`py-3 text-sm font-medium border-b-2 transition-colors ${
              activeTab === 'violations'
                ? 'border-primary-600 text-primary-600'
                : 'border-transparent text-gray-500 hover:text-gray-200'
            }`}
          >
            Violations
            <span className="ml-2 px-2 py-0.5 text-xs rounded-full bg-red-900/40 text-red-400">
              {violations.filter((v) => v.status === 'open').length}
            </span>
          </button>
        </nav>
      </div>

      {/* Filters */}
      <div className="flex items-center gap-4">
        <div className="relative flex-1 max-w-md">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400" />
          <input
            type="text"
            placeholder="Search policies..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-full pl-10 pr-4 py-2 glass-input text-sm"
          />
        </div>
        <select
          value={scopeFilter}
          onChange={(e) => setScopeFilter(e.target.value)}
          className="px-3 py-2 glass-input text-sm"
        >
          <option value="all">All Scopes</option>
          <option value="integration">Integration</option>
          <option value="connection">Connection</option>
          <option value="agent">Agent</option>
          <option value="data_entity">Data Entity</option>
        </select>
      </div>

      {/* Policies Tab */}
      {activeTab === 'policies' && (
        <div className="grid gap-4">
          {policies.map((policy) => (
            <div
              key={policy.id}
              className="glass-panel p-6"
            >
              <div className="flex items-start justify-between">
                <div className="flex items-start gap-4">
                  <div
                    className={`p-2 rounded-lg ${
                      policy.enabled ? 'bg-primary-900/30' : 'bg-surface-overlay'
                    }`}
                  >
                    <Scale
                      className={`w-5 h-5 ${
                        policy.enabled ? 'text-primary-600' : 'text-gray-400'
                      }`}
                    />
                  </div>
                  <div className="flex-1">
                    <div className="flex items-center gap-3 mb-1">
                      <h3 className="text-lg font-semibold text-white">{policy.name}</h3>
                      {!policy.enabled && (
                        <span className="text-xs font-medium px-2 py-0.5 rounded-full bg-surface-overlay text-gray-500">
                          Disabled
                        </span>
                      )}
                    </div>
                    <p className="text-sm text-gray-500 mb-3">{policy.description}</p>
                    <div className="flex items-center gap-4 flex-wrap">
                      <span
                        className={`text-xs font-medium px-2 py-1 rounded-full border ${getEnforcementColor(
                          policy.enforcement
                        )}`}
                      >
                        {policy.enforcement}
                      </span>
                      <span
                        className={`text-xs font-medium px-2 py-1 rounded-full ${getSeverityColor(
                          policy.severity
                        )}`}
                      >
                        {policy.severity}
                      </span>
                      <span className="text-xs text-gray-500">
                        Scope: <span className="font-medium">{getScopeLabel(policy.scope)}</span>
                      </span>
                      <span className="text-xs text-gray-500">
                        Last evaluated: <span className="font-medium">{policy.lastEvaluated}</span>
                      </span>
                    </div>
                  </div>
                </div>
                <div className="flex items-center gap-4">
                  <div className="text-right">
                    <div className="flex items-center gap-2">
                      {policy.violations.open > 0 && (
                        <span className="flex items-center gap-1 text-xs text-red-600">
                          <XCircle className="w-3 h-3" />
                          {policy.violations.open} open
                        </span>
                      )}
                      <span className="flex items-center gap-1 text-xs text-green-600">
                        <CheckCircle className="w-3 h-3" />
                        {policy.violations.resolved} resolved
                      </span>
                    </div>
                  </div>
                  <div className="flex items-center gap-1">
                    <button
                      onClick={() => setSelectedPolicy(policy)}
                      className="p-2 text-gray-400 hover:text-gray-300 hover:bg-surface-overlay rounded-lg"
                      title="View Policy"
                    >
                      <Eye className="w-4 h-4" />
                    </button>
                    <button className="p-2 text-gray-400 hover:text-gray-300 hover:bg-surface-overlay rounded-lg">
                      <MoreVertical className="w-4 h-4" />
                    </button>
                  </div>
                </div>
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Violations Tab */}
      {activeTab === 'violations' && (
        <div className="glass-panel overflow-hidden">
          <table className="w-full">
            <thead className="bg-surface-overlay/50">
              <tr className="text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                <th className="px-6 py-3">Status</th>
                <th className="px-6 py-3">Policy</th>
                <th className="px-6 py-3">Resource</th>
                <th className="px-6 py-3">Details</th>
                <th className="px-6 py-3">Severity</th>
                <th className="px-6 py-3">Created</th>
                <th className="px-6 py-3">Actions</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-surface-border">
              {violations.map((violation) => (
                <tr key={violation.id} className="hover:bg-surface-overlay/50">
                  <td className="px-6 py-4">
                    {violation.status === 'open' ? (
                      <span className="flex items-center gap-1 text-red-600">
                        <XCircle className="w-4 h-4" />
                        <span className="text-xs font-medium">Open</span>
                      </span>
                    ) : (
                      <span className="flex items-center gap-1 text-green-600">
                        <CheckCircle className="w-4 h-4" />
                        <span className="text-xs font-medium">Resolved</span>
                      </span>
                    )}
                  </td>
                  <td className="px-6 py-4">
                    <span className="text-sm font-medium text-white">{violation.policyName}</span>
                  </td>
                  <td className="px-6 py-4">
                    <div>
                      <span className="text-xs text-gray-500">{violation.resourceType}</span>
                      <p className="text-sm font-mono text-gray-300">{violation.resourceId}</p>
                    </div>
                  </td>
                  <td className="px-6 py-4">
                    <span className="text-sm text-gray-400">{violation.details}</span>
                  </td>
                  <td className="px-6 py-4">
                    <span
                      className={`text-xs font-medium px-2 py-1 rounded-full ${getSeverityColor(
                        violation.severity
                      )}`}
                    >
                      {violation.severity}
                    </span>
                  </td>
                  <td className="px-6 py-4 text-sm text-gray-500">
                    {formatDate(violation.created)}
                  </td>
                  <td className="px-6 py-4">
                    {violation.status === 'open' && (
                      <button className="text-xs font-medium text-primary-600 hover:text-primary-700">
                        Resolve
                      </button>
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {/* Policy Detail Modal */}
      {selectedPolicy && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="glass-panel w-full max-w-3xl max-h-[80vh] overflow-hidden">
            <div className="flex items-center justify-between p-6 border-b border-surface-border">
              <div>
                <h2 className="text-lg font-semibold text-white">{selectedPolicy.name}</h2>
                <p className="text-sm text-gray-500">{selectedPolicy.description}</p>
              </div>
              <button
                onClick={() => setSelectedPolicy(null)}
                className="p-1 text-gray-400 hover:text-gray-300"
              >
                <X className="w-5 h-5" />
              </button>
            </div>
            <div className="p-6 overflow-y-auto max-h-[calc(80vh-180px)]">
              <div className="flex items-center gap-4 mb-6">
                <span
                  className={`text-xs font-medium px-2 py-1 rounded-full border ${getEnforcementColor(
                    selectedPolicy.enforcement
                  )}`}
                >
                  {selectedPolicy.enforcement}
                </span>
                <span
                  className={`text-xs font-medium px-2 py-1 rounded-full ${getSeverityColor(
                    selectedPolicy.severity
                  )}`}
                >
                  {selectedPolicy.severity}
                </span>
                <span className="text-xs text-gray-500">
                  Scope: {getScopeLabel(selectedPolicy.scope)}
                </span>
              </div>
              <div>
                <div className="flex items-center gap-2 mb-2">
                  <Code className="w-4 h-4 text-gray-500" />
                  <h3 className="text-sm font-medium text-white">Rego Policy</h3>
                </div>
                <pre className="p-4 bg-gray-900 text-gray-100 rounded-lg text-sm overflow-x-auto">
                  <code>{selectedPolicy.rego}</code>
                </pre>
              </div>
            </div>
            <div className="flex justify-end gap-3 p-6 border-t border-surface-border">
              <button
                onClick={() => setSelectedPolicy(null)}
                className="px-4 py-2 text-sm font-medium text-gray-300 hover:text-white"
              >
                Close
              </button>
              <button className="px-4 py-2 bg-primary-600 text-white rounded-lg text-sm font-medium hover:bg-primary-700">
                Edit Policy
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Create Policy Modal */}
      {showCreateModal && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-surface-raised border border-surface-border rounded-xl p-6 w-full max-w-2xl">
            <div className="flex items-center justify-between mb-4">
              <h2 className="text-lg font-semibold">Create Policy</h2>
              <button
                onClick={() => setShowCreateModal(false)}
                className="p-1 text-gray-400 hover:text-gray-300"
              >
                <X className="w-5 h-5" />
              </button>
            </div>
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-gray-300 mb-1">Name</label>
                <input
                  type="text"
                  className="w-full px-3 py-2 glass-input text-sm"
                  placeholder="Enter policy name"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-300 mb-1">Description</label>
                <input
                  type="text"
                  className="w-full px-3 py-2 glass-input text-sm"
                  placeholder="Brief description of the policy"
                />
              </div>
              <div className="grid grid-cols-3 gap-4">
                <div>
                  <label className="block text-sm font-medium text-gray-300 mb-1">Scope</label>
                  <select className="w-full px-3 py-2 glass-input text-sm">
                    <option value="integration">Integration</option>
                    <option value="connection">Connection</option>
                    <option value="agent">Agent</option>
                    <option value="data_entity">Data Entity</option>
                    <option value="all">All Resources</option>
                  </select>
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-300 mb-1">Enforcement</label>
                  <select className="w-full px-3 py-2 glass-input text-sm">
                    <option value="enforce">Enforce (Block)</option>
                    <option value="warn">Warn</option>
                    <option value="audit">Audit Only</option>
                  </select>
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-300 mb-1">Severity</label>
                  <select className="w-full px-3 py-2 glass-input text-sm">
                    <option value="critical">Critical</option>
                    <option value="high">High</option>
                    <option value="medium">Medium</option>
                    <option value="low">Low</option>
                  </select>
                </div>
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-300 mb-1">Rego Policy</label>
                <textarea
                  className="w-full px-3 py-2 bg-surface-base/50 border border-surface-border rounded-lg text-sm font-mono text-gray-200 placeholder-gray-500 focus:border-primary-500/50 focus:ring-1 focus:ring-primary-500/20 outline-none"
                  rows={8}
                  placeholder={`package policy

deny[msg] {
  # Your policy rules here
  msg := "Violation message"
}`}
                />
              </div>
              <div className="flex justify-end gap-3 pt-4">
                <button
                  onClick={() => setShowCreateModal(false)}
                  className="px-4 py-2 text-sm font-medium text-gray-300 hover:text-white"
                >
                  Cancel
                </button>
                <button className="px-4 py-2 bg-primary-600 text-white rounded-lg text-sm font-medium hover:bg-primary-700">
                  Create Policy
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
