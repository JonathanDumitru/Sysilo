import { useState } from 'react';
import {
  Plus,
  Search,
  Layers,
  Scale,
  MoreVertical,
  X,
  Eye,
  CheckCircle,
  XCircle,
} from 'lucide-react';

// Mock data
const rulesets = [
  {
    id: '1',
    name: 'Production Readiness',
    description: 'All policies required before deploying to production',
    scope: 'integration',
    enabled: true,
    policyCount: 3,
    policies: [
      { id: '1', name: 'Integration Naming Convention', enforcement: 'enforce', severity: 'low' },
      { id: '2', name: 'Connection Security Requirements', enforcement: 'enforce', severity: 'critical' },
      { id: '3', name: 'Agent Deployment Policy', enforcement: 'warn', severity: 'medium' },
    ],
    createdAt: '2024-01-10T08:00:00Z',
    updatedAt: '2024-01-15T10:30:00Z',
  },
  {
    id: '2',
    name: 'Data Compliance',
    description: 'Policies for data handling and export compliance',
    scope: 'data_entity',
    enabled: true,
    policyCount: 2,
    policies: [
      { id: '4', name: 'Data Export Restrictions', enforcement: 'enforce', severity: 'high' },
      { id: '5', name: 'PII Masking Requirements', enforcement: 'enforce', severity: 'critical' },
    ],
    createdAt: '2024-01-08T14:00:00Z',
    updatedAt: '2024-01-14T09:00:00Z',
  },
  {
    id: '3',
    name: 'Agent Security Baseline',
    description: 'Minimum security standards for all deployed agents',
    scope: 'agent',
    enabled: true,
    policyCount: 2,
    policies: [
      { id: '3', name: 'Agent Deployment Policy', enforcement: 'warn', severity: 'medium' },
      { id: '6', name: 'Agent TLS Requirements', enforcement: 'enforce', severity: 'high' },
    ],
    createdAt: '2024-01-05T10:00:00Z',
    updatedAt: '2024-01-12T16:00:00Z',
  },
  {
    id: '4',
    name: 'Development Standards',
    description: 'Lightweight policy set for non-production environments',
    scope: 'all',
    enabled: false,
    policyCount: 1,
    policies: [
      { id: '1', name: 'Integration Naming Convention', enforcement: 'enforce', severity: 'low' },
    ],
    createdAt: '2024-01-03T11:00:00Z',
    updatedAt: '2024-01-03T11:00:00Z',
  },
];

export function RulesetsPage() {
  const [searchQuery, setSearchQuery] = useState('');
  const [scopeFilter, setScopeFilter] = useState<string>('all');
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [selectedRuleset, setSelectedRuleset] = useState<typeof rulesets[0] | null>(null);

  const getSeverityColor = (severity: string) => {
    switch (severity) {
      case 'critical':
        return 'bg-red-100 text-red-700';
      case 'high':
        return 'bg-orange-100 text-orange-700';
      case 'medium':
        return 'bg-yellow-100 text-yellow-700';
      case 'low':
        return 'bg-blue-100 text-blue-700';
      default:
        return 'bg-gray-100 text-gray-700';
    }
  };

  const getEnforcementColor = (enforcement: string) => {
    switch (enforcement) {
      case 'enforce':
        return 'bg-red-50 text-red-700 border-red-200';
      case 'warn':
        return 'bg-yellow-50 text-yellow-700 border-yellow-200';
      case 'audit':
        return 'bg-gray-50 text-gray-700 border-gray-200';
      default:
        return 'bg-gray-50 text-gray-700 border-gray-200';
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

  const filteredRulesets = rulesets.filter((rs) => {
    const matchesSearch =
      searchQuery === '' ||
      rs.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      rs.description.toLowerCase().includes(searchQuery.toLowerCase());
    const matchesScope = scopeFilter === 'all' || rs.scope === scopeFilter;
    return matchesSearch && matchesScope;
  });

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">Rulesets</h1>
          <p className="text-gray-500">Group and manage related policies as reusable collections</p>
        </div>
        <button
          onClick={() => setShowCreateModal(true)}
          className="flex items-center gap-2 px-4 py-2 bg-primary-600 text-white rounded-lg text-sm font-medium hover:bg-primary-700"
        >
          <Plus className="w-4 h-4" />
          Create Ruleset
        </button>
      </div>

      {/* Filters */}
      <div className="flex items-center gap-4">
        <div className="relative flex-1 max-w-md">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400" />
          <input
            type="text"
            placeholder="Search rulesets..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-full pl-10 pr-4 py-2 border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500"
          />
        </div>
        <select
          value={scopeFilter}
          onChange={(e) => setScopeFilter(e.target.value)}
          className="px-3 py-2 border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500"
        >
          <option value="all">All Scopes</option>
          <option value="integration">Integration</option>
          <option value="connection">Connection</option>
          <option value="agent">Agent</option>
          <option value="data_entity">Data Entity</option>
        </select>
      </div>

      {/* Rulesets Grid */}
      <div className="grid gap-4">
        {filteredRulesets.map((ruleset) => (
          <div
            key={ruleset.id}
            className="bg-white rounded-xl p-6 shadow-sm border border-gray-100"
          >
            <div className="flex items-start justify-between">
              <div className="flex items-start gap-4">
                <div
                  className={`p-2 rounded-lg ${
                    ruleset.enabled ? 'bg-primary-50' : 'bg-gray-100'
                  }`}
                >
                  <Layers
                    className={`w-5 h-5 ${
                      ruleset.enabled ? 'text-primary-600' : 'text-gray-400'
                    }`}
                  />
                </div>
                <div className="flex-1">
                  <div className="flex items-center gap-3 mb-1">
                    <h3 className="text-lg font-semibold text-gray-900">{ruleset.name}</h3>
                    {!ruleset.enabled && (
                      <span className="text-xs font-medium px-2 py-0.5 rounded-full bg-gray-100 text-gray-500">
                        Disabled
                      </span>
                    )}
                  </div>
                  <p className="text-sm text-gray-500 mb-3">{ruleset.description}</p>
                  <div className="flex items-center gap-4 flex-wrap">
                    <span className="text-xs font-medium px-2 py-1 rounded-full bg-gray-100 text-gray-600">
                      {getScopeLabel(ruleset.scope)}
                    </span>
                    <span className="text-xs text-gray-500">
                      {ruleset.policyCount} {ruleset.policyCount === 1 ? 'policy' : 'policies'}
                    </span>
                    <span className="text-xs text-gray-400">
                      Updated {new Date(ruleset.updatedAt).toLocaleDateString()}
                    </span>
                  </div>
                </div>
              </div>
              <div className="flex items-center gap-2">
                <button
                  onClick={() => setSelectedRuleset(ruleset)}
                  className="p-1.5 rounded-lg text-gray-400 hover:text-primary-600 hover:bg-primary-50"
                  title="View policies"
                >
                  <Eye className="w-4 h-4" />
                </button>
                <button className="p-1.5 rounded-lg text-gray-400 hover:text-gray-600 hover:bg-gray-100">
                  <MoreVertical className="w-4 h-4" />
                </button>
              </div>
            </div>

            {/* Inline policy list */}
            <div className="mt-4 border-t border-gray-100 pt-4">
              <h4 className="text-xs font-semibold text-gray-500 uppercase tracking-wider mb-2">
                Included Policies
              </h4>
              <div className="space-y-2">
                {ruleset.policies.map((policy) => (
                  <div
                    key={policy.id}
                    className="flex items-center justify-between py-1.5 px-3 rounded-lg bg-gray-50"
                  >
                    <div className="flex items-center gap-2">
                      <Scale className="w-3.5 h-3.5 text-gray-400" />
                      <span className="text-sm text-gray-700">{policy.name}</span>
                    </div>
                    <div className="flex items-center gap-2">
                      <span
                        className={`text-xs font-medium px-2 py-0.5 rounded-full border ${getEnforcementColor(
                          policy.enforcement
                        )}`}
                      >
                        {policy.enforcement}
                      </span>
                      <span
                        className={`text-xs font-medium px-2 py-0.5 rounded-full ${getSeverityColor(
                          policy.severity
                        )}`}
                      >
                        {policy.severity}
                      </span>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          </div>
        ))}
      </div>

      {filteredRulesets.length === 0 && (
        <div className="text-center py-12">
          <Layers className="w-12 h-12 text-gray-300 mx-auto mb-4" />
          <h3 className="text-lg font-medium text-gray-900 mb-2">No rulesets found</h3>
          <p className="text-gray-500">
            {searchQuery || scopeFilter !== 'all'
              ? 'Try adjusting your filters'
              : 'Create your first ruleset to group related policies'}
          </p>
        </div>
      )}

      {/* View Ruleset Detail Modal */}
      {selectedRuleset && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-white rounded-xl shadow-2xl w-full max-w-2xl max-h-[80vh] overflow-y-auto">
            <div className="flex items-center justify-between p-6 border-b border-gray-200">
              <div>
                <h2 className="text-xl font-bold text-gray-900">{selectedRuleset.name}</h2>
                <p className="text-sm text-gray-500 mt-1">{selectedRuleset.description}</p>
              </div>
              <button
                onClick={() => setSelectedRuleset(null)}
                className="p-2 rounded-lg text-gray-400 hover:text-gray-600 hover:bg-gray-100"
              >
                <X className="w-5 h-5" />
              </button>
            </div>
            <div className="p-6">
              <div className="flex items-center gap-4 mb-6">
                <span className="text-sm font-medium text-gray-700">
                  Scope: <span className="text-gray-500">{getScopeLabel(selectedRuleset.scope)}</span>
                </span>
                <span className="text-sm font-medium text-gray-700">
                  Status:{' '}
                  {selectedRuleset.enabled ? (
                    <span className="text-green-600 inline-flex items-center gap-1">
                      <CheckCircle className="w-3.5 h-3.5" /> Enabled
                    </span>
                  ) : (
                    <span className="text-gray-500 inline-flex items-center gap-1">
                      <XCircle className="w-3.5 h-3.5" /> Disabled
                    </span>
                  )}
                </span>
              </div>
              <h3 className="text-sm font-semibold text-gray-900 mb-3">
                Policies ({selectedRuleset.policies.length})
              </h3>
              <div className="space-y-3">
                {selectedRuleset.policies.map((policy) => (
                  <div
                    key={policy.id}
                    className="flex items-center justify-between p-3 rounded-lg border border-gray-200"
                  >
                    <div className="flex items-center gap-3">
                      <Scale className="w-4 h-4 text-primary-500" />
                      <span className="text-sm font-medium text-gray-900">{policy.name}</span>
                    </div>
                    <div className="flex items-center gap-2">
                      <span
                        className={`text-xs font-medium px-2 py-0.5 rounded-full border ${getEnforcementColor(
                          policy.enforcement
                        )}`}
                      >
                        {policy.enforcement}
                      </span>
                      <span
                        className={`text-xs font-medium px-2 py-0.5 rounded-full ${getSeverityColor(
                          policy.severity
                        )}`}
                      >
                        {policy.severity}
                      </span>
                    </div>
                  </div>
                ))}
              </div>
              <div className="mt-6 pt-4 border-t border-gray-200 flex items-center justify-between text-xs text-gray-400">
                <span>Created {new Date(selectedRuleset.createdAt).toLocaleDateString()}</span>
                <span>Updated {new Date(selectedRuleset.updatedAt).toLocaleDateString()}</span>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Create Ruleset Modal */}
      {showCreateModal && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-white rounded-xl shadow-2xl w-full max-w-lg">
            <div className="flex items-center justify-between p-6 border-b border-gray-200">
              <h2 className="text-xl font-bold text-gray-900">Create Ruleset</h2>
              <button
                onClick={() => setShowCreateModal(false)}
                className="p-2 rounded-lg text-gray-400 hover:text-gray-600 hover:bg-gray-100"
              >
                <X className="w-5 h-5" />
              </button>
            </div>
            <div className="p-6 space-y-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">Name</label>
                <input
                  type="text"
                  placeholder="e.g., Production Readiness"
                  className="w-full px-3 py-2 border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">Description</label>
                <textarea
                  rows={3}
                  placeholder="Describe what this ruleset is for..."
                  className="w-full px-3 py-2 border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">Scope</label>
                <select className="w-full px-3 py-2 border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500">
                  <option value="all">All Resources</option>
                  <option value="integration">Integration</option>
                  <option value="connection">Connection</option>
                  <option value="agent">Agent</option>
                  <option value="data_entity">Data Entity</option>
                </select>
              </div>
              <div className="flex justify-end gap-3 pt-4">
                <button
                  onClick={() => setShowCreateModal(false)}
                  className="px-4 py-2 text-sm font-medium text-gray-700 hover:bg-gray-100 rounded-lg"
                >
                  Cancel
                </button>
                <button
                  onClick={() => setShowCreateModal(false)}
                  className="px-4 py-2 text-sm font-medium text-white bg-primary-600 hover:bg-primary-700 rounded-lg"
                >
                  Create Ruleset
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
