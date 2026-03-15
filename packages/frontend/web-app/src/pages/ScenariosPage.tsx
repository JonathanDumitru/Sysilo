import { useState } from 'react';
import {
  Plus,
  Search,
  Play,
  Copy,
  Trash2,
  ChevronRight,
  Calculator,
  TrendingUp,
  Clock,
  DollarSign,
  X,
  Check,
} from 'lucide-react';

// Mock data
const scenarios = [
  {
    id: '1',
    name: 'Cloud Migration Wave 1',
    description: 'Migrate top 8 applications to cloud infrastructure',
    status: 'analyzed',
    createdAt: '2024-01-10T00:00:00Z',
    applications: [
      { id: '1', name: 'Core ERP', action: 'replatform', cost: 150000 },
      { id: '2', name: 'HR System', action: 'rehost', cost: 45000 },
      { id: '3', name: 'Analytics Platform', action: 'refactor', cost: 280000 },
    ],
    analysis: {
      totalCost: 475000,
      annualSavings: 180000,
      paybackMonths: 32,
      npv: 285000,
      roi: 0.38,
      riskLevel: 'medium',
    },
  },
  {
    id: '2',
    name: 'Legacy Retirement Plan',
    description: 'Retire 6 legacy applications and consolidate functionality',
    status: 'draft',
    createdAt: '2024-01-12T00:00:00Z',
    applications: [
      { id: '4', name: 'Legacy CRM', action: 'retire', cost: 25000 },
      { id: '5', name: 'Old Reporting', action: 'retire', cost: 15000 },
      { id: '6', name: 'Archive System', action: 'retire', cost: 10000 },
    ],
    analysis: null,
  },
  {
    id: '3',
    name: 'Vendor Consolidation',
    description: 'Replace multiple point solutions with integrated platform',
    status: 'analyzed',
    createdAt: '2024-01-08T00:00:00Z',
    applications: [
      { id: '7', name: 'Tool A', action: 'replace', cost: 120000 },
      { id: '8', name: 'Tool B', action: 'replace', cost: 95000 },
    ],
    analysis: {
      totalCost: 215000,
      annualSavings: 85000,
      paybackMonths: 30,
      npv: 145000,
      roi: 0.25,
      riskLevel: 'low',
    },
  },
];

const actionColors: Record<string, string> = {
  rehost: 'bg-blue-900/40 text-blue-400',
  replatform: 'bg-purple-900/40 text-purple-400',
  refactor: 'bg-indigo-900/40 text-indigo-400',
  replace: 'bg-orange-900/40 text-orange-400',
  retire: 'bg-red-900/40 text-red-400',
  retain: 'bg-gray-700/50 text-gray-300',
};

export function ScenariosPage() {
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedScenario, setSelectedScenario] = useState<typeof scenarios[0] | null>(null);
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [showCompareModal, setShowCompareModal] = useState(false);
  const [compareSelection, setCompareSelection] = useState<string[]>([]);

  const formatCurrency = (amount: number) => {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: 'USD',
      minimumFractionDigits: 0,
      maximumFractionDigits: 0,
    }).format(amount);
  };

  const formatDate = (dateStr: string) => {
    return new Date(dateStr).toLocaleDateString();
  };

  const toggleCompare = (id: string) => {
    if (compareSelection.includes(id)) {
      setCompareSelection(compareSelection.filter((s) => s !== id));
    } else if (compareSelection.length < 3) {
      setCompareSelection([...compareSelection, id]);
    }
  };

  const analyzedScenarios = scenarios.filter((s) => s.analysis !== null);

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-white">What-If Scenarios</h1>
          <p className="text-gray-500">Model and compare rationalization strategies</p>
        </div>
        <div className="flex items-center gap-3">
          {compareSelection.length >= 2 && (
            <button
              onClick={() => setShowCompareModal(true)}
              className="px-4 py-2 border border-primary-600 text-primary-600 rounded-lg text-sm font-medium hover:bg-primary-900/30"
            >
              Compare ({compareSelection.length})
            </button>
          )}
          <button
            onClick={() => setShowCreateModal(true)}
            className="flex items-center gap-2 px-4 py-2 bg-primary-600 text-white rounded-lg text-sm font-medium hover:bg-primary-700"
          >
            <Plus className="w-4 h-4" />
            Create Scenario
          </button>
        </div>
      </div>

      {/* Search & Filters */}
      <div className="flex items-center gap-4">
        <div className="relative flex-1 max-w-md">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400" />
          <input
            type="text"
            placeholder="Search scenarios..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-full pl-10 pr-4 py-2 bg-surface-base/50 border border-surface-border rounded-lg text-sm text-gray-200 placeholder-gray-500 focus:border-primary-500/50 focus:ring-1 focus:ring-primary-500/20 outline-none"
          />
        </div>
        <select className="px-3 py-2 bg-surface-base/50 border border-surface-border rounded-lg text-sm text-gray-200 placeholder-gray-500 focus:border-primary-500/50 focus:ring-1 focus:ring-primary-500/20 outline-none">
          <option value="all">All Status</option>
          <option value="draft">Draft</option>
          <option value="analyzed">Analyzed</option>
          <option value="approved">Approved</option>
        </select>
      </div>

      {/* Scenarios Grid */}
      <div className="grid grid-cols-2 gap-6">
        {/* Scenarios List */}
        <div className="space-y-4">
          {scenarios.map((scenario) => (
            <div
              key={scenario.id}
              onClick={() => setSelectedScenario(scenario)}
              className={`bg-surface-raised/80 backdrop-blur-glass rounded-xl p-5 shadow-glass border cursor-pointer transition-all ${
                selectedScenario?.id === scenario.id
                  ? 'border-primary-500 ring-2 ring-primary-900/50'
                  : 'border-surface-border hover:border-gray-600'
              }`}
            >
              <div className="flex items-start justify-between">
                <div className="flex items-start gap-3">
                  <input
                    type="checkbox"
                    checked={compareSelection.includes(scenario.id)}
                    onChange={(e) => {
                      e.stopPropagation();
                      toggleCompare(scenario.id);
                    }}
                    disabled={!scenario.analysis}
                    className="mt-1 rounded border-surface-border text-primary-600 focus:ring-primary-500 disabled:opacity-50"
                  />
                  <div>
                    <div className="flex items-center gap-2 mb-1">
                      <h3 className="font-semibold text-white">{scenario.name}</h3>
                      <span
                        className={`text-xs font-medium px-2 py-0.5 rounded-full capitalize ${
                          scenario.status === 'analyzed'
                            ? 'bg-green-900/40 text-green-400'
                            : scenario.status === 'draft'
                            ? 'bg-gray-700/50 text-gray-300'
                            : 'bg-blue-900/40 text-blue-400'
                        }`}
                      >
                        {scenario.status}
                      </span>
                    </div>
                    <p className="text-sm text-gray-500">{scenario.description}</p>
                  </div>
                </div>
                <ChevronRight className="w-5 h-5 text-gray-400 flex-shrink-0" />
              </div>

              <div className="flex items-center gap-4 mt-4 pt-3 border-t border-surface-border">
                <span className="text-xs text-gray-500">
                  {scenario.applications.length} applications
                </span>
                {scenario.analysis && (
                  <>
                    <span className="text-xs font-medium text-green-400">
                      {formatCurrency(scenario.analysis.annualSavings)}/yr savings
                    </span>
                    <span className="text-xs text-gray-500">
                      ROI: {Math.round(scenario.analysis.roi * 100)}%
                    </span>
                  </>
                )}
                <span className="text-xs text-gray-400 ml-auto">
                  Created {formatDate(scenario.createdAt)}
                </span>
              </div>
            </div>
          ))}
        </div>

        {/* Detail Panel */}
        <div className="bg-surface-raised/80 backdrop-blur-glass rounded-xl shadow-glass border border-surface-border h-fit sticky top-6">
          {selectedScenario ? (
            <div>
              <div className="p-6 border-b border-surface-border">
                <div className="flex items-center justify-between mb-3">
                  <span
                    className={`text-xs font-medium px-2 py-0.5 rounded-full capitalize ${
                      selectedScenario.status === 'analyzed'
                        ? 'bg-green-900/40 text-green-400'
                        : 'bg-gray-700/50 text-gray-300'
                    }`}
                  >
                    {selectedScenario.status}
                  </span>
                  <div className="flex items-center gap-2">
                    <button className="p-1.5 text-gray-400 hover:text-gray-300 rounded">
                      <Copy className="w-4 h-4" />
                    </button>
                    <button className="p-1.5 text-gray-400 hover:text-red-600 rounded">
                      <Trash2 className="w-4 h-4" />
                    </button>
                  </div>
                </div>
                <h2 className="text-xl font-semibold text-white mb-2">
                  {selectedScenario.name}
                </h2>
                <p className="text-sm text-gray-400">{selectedScenario.description}</p>
              </div>

              {/* Applications in Scenario */}
              <div className="p-6 border-b border-surface-border">
                <h3 className="text-sm font-medium text-white mb-4">
                  Applications ({selectedScenario.applications.length})
                </h3>
                <div className="space-y-2">
                  {selectedScenario.applications.map((app) => (
                    <div
                      key={app.id}
                      className="flex items-center justify-between p-3 bg-surface-overlay/50 rounded-lg"
                    >
                      <div className="flex items-center gap-3">
                        <span className="text-sm font-medium text-white">{app.name}</span>
                        <span
                          className={`text-xs font-medium px-2 py-0.5 rounded-full capitalize ${actionColors[app.action]}`}
                        >
                          {app.action}
                        </span>
                      </div>
                      <span className="text-sm text-gray-400">{formatCurrency(app.cost)}</span>
                    </div>
                  ))}
                </div>
              </div>

              {/* Analysis Results */}
              {selectedScenario.analysis ? (
                <div className="p-6 border-b border-surface-border">
                  <h3 className="text-sm font-medium text-white mb-4">Financial Analysis</h3>
                  <div className="grid grid-cols-2 gap-4">
                    <div className="p-3 bg-surface-overlay/50 rounded-lg">
                      <div className="flex items-center gap-2 mb-1">
                        <DollarSign className="w-4 h-4 text-gray-400" />
                        <span className="text-xs text-gray-500">Total Cost</span>
                      </div>
                      <p className="text-lg font-semibold text-white">
                        {formatCurrency(selectedScenario.analysis.totalCost)}
                      </p>
                    </div>
                    <div className="p-3 bg-green-900/30 rounded-lg">
                      <div className="flex items-center gap-2 mb-1">
                        <TrendingUp className="w-4 h-4 text-green-400" />
                        <span className="text-xs text-green-400">Annual Savings</span>
                      </div>
                      <p className="text-lg font-semibold text-green-400">
                        {formatCurrency(selectedScenario.analysis.annualSavings)}
                      </p>
                    </div>
                    <div className="p-3 bg-surface-overlay/50 rounded-lg">
                      <div className="flex items-center gap-2 mb-1">
                        <Clock className="w-4 h-4 text-gray-400" />
                        <span className="text-xs text-gray-500">Payback Period</span>
                      </div>
                      <p className="text-lg font-semibold text-white">
                        {selectedScenario.analysis.paybackMonths} months
                      </p>
                    </div>
                    <div className="p-3 bg-surface-overlay/50 rounded-lg">
                      <div className="flex items-center gap-2 mb-1">
                        <Calculator className="w-4 h-4 text-gray-400" />
                        <span className="text-xs text-gray-500">NPV</span>
                      </div>
                      <p className="text-lg font-semibold text-white">
                        {formatCurrency(selectedScenario.analysis.npv)}
                      </p>
                    </div>
                  </div>
                  <div className="mt-4 p-3 bg-primary-900/30 rounded-lg">
                    <div className="flex items-center justify-between">
                      <span className="text-sm font-medium text-primary-400">Return on Investment</span>
                      <span className="text-2xl font-bold text-primary-400">
                        {Math.round(selectedScenario.analysis.roi * 100)}%
                      </span>
                    </div>
                  </div>
                </div>
              ) : (
                <div className="p-6 border-b border-surface-border">
                  <div className="text-center py-4">
                    <Calculator className="w-8 h-8 text-gray-600 mx-auto mb-2" />
                    <p className="text-sm text-gray-500">Not yet analyzed</p>
                    <button className="mt-3 flex items-center gap-2 mx-auto px-4 py-2 bg-primary-600 text-white rounded-lg text-sm font-medium hover:bg-primary-700">
                      <Play className="w-4 h-4" />
                      Run Analysis
                    </button>
                  </div>
                </div>
              )}

              {/* Actions */}
              <div className="p-6">
                <div className="flex gap-2">
                  {selectedScenario.analysis ? (
                    <>
                      <button className="flex-1 px-4 py-2 bg-primary-600 text-white rounded-lg text-sm font-medium hover:bg-primary-700">
                        Submit for Approval
                      </button>
                      <button className="flex-1 px-4 py-2 border border-surface-border rounded-lg text-sm font-medium text-gray-300 hover:bg-surface-overlay/50">
                        Create Project
                      </button>
                    </>
                  ) : (
                    <>
                      <button className="flex-1 px-4 py-2 bg-primary-600 text-white rounded-lg text-sm font-medium hover:bg-primary-700">
                        Edit Scenario
                      </button>
                      <button className="flex-1 flex items-center justify-center gap-2 px-4 py-2 border border-surface-border rounded-lg text-sm font-medium text-gray-300 hover:bg-surface-overlay/50">
                        <Play className="w-4 h-4" />
                        Analyze
                      </button>
                    </>
                  )}
                </div>
              </div>
            </div>
          ) : (
            <div className="p-8 text-center text-gray-500">
              <Calculator className="w-12 h-12 mx-auto mb-3 text-gray-300" />
              <p>Select a scenario to view details</p>
            </div>
          )}
        </div>
      </div>

      {/* Create Scenario Modal */}
      {showCreateModal && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-surface-raised border border-surface-border rounded-xl p-6 w-full max-w-lg">
            <div className="flex items-center justify-between mb-4">
              <h2 className="text-lg font-semibold">Create Scenario</h2>
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
                  className="w-full px-3 py-2 bg-surface-base/50 border border-surface-border rounded-lg text-sm text-gray-200 placeholder-gray-500 focus:border-primary-500/50 focus:ring-1 focus:ring-primary-500/20 outline-none"
                  placeholder="e.g., Cloud Migration Wave 2"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-300 mb-1">Description</label>
                <textarea
                  className="w-full px-3 py-2 bg-surface-base/50 border border-surface-border rounded-lg text-sm text-gray-200 placeholder-gray-500 focus:border-primary-500/50 focus:ring-1 focus:ring-primary-500/20 outline-none"
                  rows={3}
                  placeholder="Describe the goal and scope of this scenario"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-300 mb-1">
                  Select Applications
                </label>
                <p className="text-xs text-gray-500 mb-2">
                  You can add applications after creating the scenario
                </p>
              </div>
              <div className="flex justify-end gap-3 pt-4">
                <button
                  onClick={() => setShowCreateModal(false)}
                  className="px-4 py-2 text-sm font-medium text-gray-300 hover:text-white"
                >
                  Cancel
                </button>
                <button className="px-4 py-2 bg-primary-600 text-white rounded-lg text-sm font-medium hover:bg-primary-700">
                  Create Scenario
                </button>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Compare Modal */}
      {showCompareModal && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-surface-raised border border-surface-border rounded-xl p-6 w-full max-w-4xl max-h-[90vh] overflow-y-auto">
            <div className="flex items-center justify-between mb-6">
              <h2 className="text-lg font-semibold">Compare Scenarios</h2>
              <button
                onClick={() => setShowCompareModal(false)}
                className="p-1 text-gray-400 hover:text-gray-300"
              >
                <X className="w-5 h-5" />
              </button>
            </div>

            <div className="overflow-x-auto">
              <table className="w-full">
                <thead>
                  <tr className="border-b border-surface-border">
                    <th className="text-left text-sm font-medium text-gray-500 pb-3">Metric</th>
                    {compareSelection.map((id) => {
                      const scenario = analyzedScenarios.find((s) => s.id === id);
                      return (
                        <th key={id} className="text-left text-sm font-medium text-white pb-3 px-4">
                          {scenario?.name}
                        </th>
                      );
                    })}
                  </tr>
                </thead>
                <tbody className="divide-y divide-surface-border">
                  <tr>
                    <td className="py-3 text-sm text-gray-400">Applications</td>
                    {compareSelection.map((id) => {
                      const scenario = analyzedScenarios.find((s) => s.id === id);
                      return (
                        <td key={id} className="py-3 text-sm font-medium text-white px-4">
                          {scenario?.applications.length}
                        </td>
                      );
                    })}
                  </tr>
                  <tr>
                    <td className="py-3 text-sm text-gray-400">Total Cost</td>
                    {compareSelection.map((id) => {
                      const scenario = analyzedScenarios.find((s) => s.id === id);
                      return (
                        <td key={id} className="py-3 text-sm font-medium text-white px-4">
                          {scenario?.analysis && formatCurrency(scenario.analysis.totalCost)}
                        </td>
                      );
                    })}
                  </tr>
                  <tr>
                    <td className="py-3 text-sm text-gray-400">Annual Savings</td>
                    {compareSelection.map((id) => {
                      const scenario = analyzedScenarios.find((s) => s.id === id);
                      const maxSavings = Math.max(
                        ...compareSelection.map(
                          (sid) => analyzedScenarios.find((s) => s.id === sid)?.analysis?.annualSavings || 0
                        )
                      );
                      const isMax = scenario?.analysis?.annualSavings === maxSavings;
                      return (
                        <td key={id} className="py-3 px-4">
                          <span
                            className={`text-sm font-medium ${isMax ? 'text-green-600' : 'text-white'}`}
                          >
                            {scenario?.analysis && formatCurrency(scenario.analysis.annualSavings)}
                            {isMax && <Check className="w-4 h-4 inline ml-1" />}
                          </span>
                        </td>
                      );
                    })}
                  </tr>
                  <tr>
                    <td className="py-3 text-sm text-gray-400">ROI</td>
                    {compareSelection.map((id) => {
                      const scenario = analyzedScenarios.find((s) => s.id === id);
                      const maxRoi = Math.max(
                        ...compareSelection.map(
                          (sid) => analyzedScenarios.find((s) => s.id === sid)?.analysis?.roi || 0
                        )
                      );
                      const isMax = scenario?.analysis?.roi === maxRoi;
                      return (
                        <td key={id} className="py-3 px-4">
                          <span
                            className={`text-sm font-medium ${isMax ? 'text-green-600' : 'text-white'}`}
                          >
                            {scenario?.analysis && `${Math.round(scenario.analysis.roi * 100)}%`}
                            {isMax && <Check className="w-4 h-4 inline ml-1" />}
                          </span>
                        </td>
                      );
                    })}
                  </tr>
                  <tr>
                    <td className="py-3 text-sm text-gray-400">Payback Period</td>
                    {compareSelection.map((id) => {
                      const scenario = analyzedScenarios.find((s) => s.id === id);
                      const minPayback = Math.min(
                        ...compareSelection.map(
                          (sid) =>
                            analyzedScenarios.find((s) => s.id === sid)?.analysis?.paybackMonths || Infinity
                        )
                      );
                      const isMin = scenario?.analysis?.paybackMonths === minPayback;
                      return (
                        <td key={id} className="py-3 px-4">
                          <span
                            className={`text-sm font-medium ${isMin ? 'text-green-600' : 'text-white'}`}
                          >
                            {scenario?.analysis && `${scenario.analysis.paybackMonths} months`}
                            {isMin && <Check className="w-4 h-4 inline ml-1" />}
                          </span>
                        </td>
                      );
                    })}
                  </tr>
                  <tr>
                    <td className="py-3 text-sm text-gray-400">NPV</td>
                    {compareSelection.map((id) => {
                      const scenario = analyzedScenarios.find((s) => s.id === id);
                      return (
                        <td key={id} className="py-3 text-sm font-medium text-white px-4">
                          {scenario?.analysis && formatCurrency(scenario.analysis.npv)}
                        </td>
                      );
                    })}
                  </tr>
                  <tr>
                    <td className="py-3 text-sm text-gray-400">Risk Level</td>
                    {compareSelection.map((id) => {
                      const scenario = analyzedScenarios.find((s) => s.id === id);
                      return (
                        <td key={id} className="py-3 px-4">
                          <span
                            className={`text-xs font-medium px-2 py-1 rounded-full capitalize ${
                              scenario?.analysis?.riskLevel === 'low'
                                ? 'bg-green-900/40 text-green-400'
                                : scenario?.analysis?.riskLevel === 'medium'
                                ? 'bg-yellow-900/40 text-yellow-400'
                                : 'bg-red-900/40 text-red-400'
                            }`}
                          >
                            {scenario?.analysis?.riskLevel}
                          </span>
                        </td>
                      );
                    })}
                  </tr>
                </tbody>
              </table>
            </div>

            <div className="flex justify-end gap-3 mt-6 pt-4 border-t border-surface-border">
              <button
                onClick={() => setShowCompareModal(false)}
                className="px-4 py-2 text-sm font-medium text-gray-300 hover:text-white"
              >
                Close
              </button>
              <button className="px-4 py-2 bg-primary-600 text-white rounded-lg text-sm font-medium hover:bg-primary-700">
                Export Comparison
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
