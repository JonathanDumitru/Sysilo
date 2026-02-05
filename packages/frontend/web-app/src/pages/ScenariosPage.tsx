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
  rehost: 'bg-blue-100 text-blue-700',
  replatform: 'bg-purple-100 text-purple-700',
  refactor: 'bg-indigo-100 text-indigo-700',
  replace: 'bg-orange-100 text-orange-700',
  retire: 'bg-red-100 text-red-700',
  retain: 'bg-gray-100 text-gray-700',
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
          <h1 className="text-2xl font-bold text-gray-900">What-If Scenarios</h1>
          <p className="text-gray-500">Model and compare rationalization strategies</p>
        </div>
        <div className="flex items-center gap-3">
          {compareSelection.length >= 2 && (
            <button
              onClick={() => setShowCompareModal(true)}
              className="px-4 py-2 border border-primary-600 text-primary-600 rounded-lg text-sm font-medium hover:bg-primary-50"
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
            className="w-full pl-10 pr-4 py-2 border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500"
          />
        </div>
        <select className="px-3 py-2 border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500">
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
              className={`bg-white rounded-xl p-5 shadow-sm border cursor-pointer transition-all ${
                selectedScenario?.id === scenario.id
                  ? 'border-primary-500 ring-2 ring-primary-100'
                  : 'border-gray-100 hover:border-gray-200'
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
                    className="mt-1 rounded border-gray-300 text-primary-600 focus:ring-primary-500 disabled:opacity-50"
                  />
                  <div>
                    <div className="flex items-center gap-2 mb-1">
                      <h3 className="font-semibold text-gray-900">{scenario.name}</h3>
                      <span
                        className={`text-xs font-medium px-2 py-0.5 rounded-full capitalize ${
                          scenario.status === 'analyzed'
                            ? 'bg-green-100 text-green-700'
                            : scenario.status === 'draft'
                            ? 'bg-gray-100 text-gray-700'
                            : 'bg-blue-100 text-blue-700'
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

              <div className="flex items-center gap-4 mt-4 pt-3 border-t border-gray-100">
                <span className="text-xs text-gray-500">
                  {scenario.applications.length} applications
                </span>
                {scenario.analysis && (
                  <>
                    <span className="text-xs font-medium text-green-600">
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
        <div className="bg-white rounded-xl shadow-sm border border-gray-100 h-fit sticky top-6">
          {selectedScenario ? (
            <div>
              <div className="p-6 border-b border-gray-100">
                <div className="flex items-center justify-between mb-3">
                  <span
                    className={`text-xs font-medium px-2 py-0.5 rounded-full capitalize ${
                      selectedScenario.status === 'analyzed'
                        ? 'bg-green-100 text-green-700'
                        : 'bg-gray-100 text-gray-700'
                    }`}
                  >
                    {selectedScenario.status}
                  </span>
                  <div className="flex items-center gap-2">
                    <button className="p-1.5 text-gray-400 hover:text-gray-600 rounded">
                      <Copy className="w-4 h-4" />
                    </button>
                    <button className="p-1.5 text-gray-400 hover:text-red-600 rounded">
                      <Trash2 className="w-4 h-4" />
                    </button>
                  </div>
                </div>
                <h2 className="text-xl font-semibold text-gray-900 mb-2">
                  {selectedScenario.name}
                </h2>
                <p className="text-sm text-gray-600">{selectedScenario.description}</p>
              </div>

              {/* Applications in Scenario */}
              <div className="p-6 border-b border-gray-100">
                <h3 className="text-sm font-medium text-gray-900 mb-4">
                  Applications ({selectedScenario.applications.length})
                </h3>
                <div className="space-y-2">
                  {selectedScenario.applications.map((app) => (
                    <div
                      key={app.id}
                      className="flex items-center justify-between p-3 bg-gray-50 rounded-lg"
                    >
                      <div className="flex items-center gap-3">
                        <span className="text-sm font-medium text-gray-900">{app.name}</span>
                        <span
                          className={`text-xs font-medium px-2 py-0.5 rounded-full capitalize ${actionColors[app.action]}`}
                        >
                          {app.action}
                        </span>
                      </div>
                      <span className="text-sm text-gray-600">{formatCurrency(app.cost)}</span>
                    </div>
                  ))}
                </div>
              </div>

              {/* Analysis Results */}
              {selectedScenario.analysis ? (
                <div className="p-6 border-b border-gray-100">
                  <h3 className="text-sm font-medium text-gray-900 mb-4">Financial Analysis</h3>
                  <div className="grid grid-cols-2 gap-4">
                    <div className="p-3 bg-gray-50 rounded-lg">
                      <div className="flex items-center gap-2 mb-1">
                        <DollarSign className="w-4 h-4 text-gray-400" />
                        <span className="text-xs text-gray-500">Total Cost</span>
                      </div>
                      <p className="text-lg font-semibold text-gray-900">
                        {formatCurrency(selectedScenario.analysis.totalCost)}
                      </p>
                    </div>
                    <div className="p-3 bg-green-50 rounded-lg">
                      <div className="flex items-center gap-2 mb-1">
                        <TrendingUp className="w-4 h-4 text-green-600" />
                        <span className="text-xs text-green-700">Annual Savings</span>
                      </div>
                      <p className="text-lg font-semibold text-green-700">
                        {formatCurrency(selectedScenario.analysis.annualSavings)}
                      </p>
                    </div>
                    <div className="p-3 bg-gray-50 rounded-lg">
                      <div className="flex items-center gap-2 mb-1">
                        <Clock className="w-4 h-4 text-gray-400" />
                        <span className="text-xs text-gray-500">Payback Period</span>
                      </div>
                      <p className="text-lg font-semibold text-gray-900">
                        {selectedScenario.analysis.paybackMonths} months
                      </p>
                    </div>
                    <div className="p-3 bg-gray-50 rounded-lg">
                      <div className="flex items-center gap-2 mb-1">
                        <Calculator className="w-4 h-4 text-gray-400" />
                        <span className="text-xs text-gray-500">NPV</span>
                      </div>
                      <p className="text-lg font-semibold text-gray-900">
                        {formatCurrency(selectedScenario.analysis.npv)}
                      </p>
                    </div>
                  </div>
                  <div className="mt-4 p-3 bg-primary-50 rounded-lg">
                    <div className="flex items-center justify-between">
                      <span className="text-sm font-medium text-primary-700">Return on Investment</span>
                      <span className="text-2xl font-bold text-primary-700">
                        {Math.round(selectedScenario.analysis.roi * 100)}%
                      </span>
                    </div>
                  </div>
                </div>
              ) : (
                <div className="p-6 border-b border-gray-100">
                  <div className="text-center py-4">
                    <Calculator className="w-8 h-8 text-gray-300 mx-auto mb-2" />
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
                      <button className="flex-1 px-4 py-2 border border-gray-200 rounded-lg text-sm font-medium text-gray-700 hover:bg-gray-50">
                        Create Project
                      </button>
                    </>
                  ) : (
                    <>
                      <button className="flex-1 px-4 py-2 bg-primary-600 text-white rounded-lg text-sm font-medium hover:bg-primary-700">
                        Edit Scenario
                      </button>
                      <button className="flex-1 flex items-center justify-center gap-2 px-4 py-2 border border-gray-200 rounded-lg text-sm font-medium text-gray-700 hover:bg-gray-50">
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
          <div className="bg-white rounded-xl p-6 w-full max-w-lg">
            <div className="flex items-center justify-between mb-4">
              <h2 className="text-lg font-semibold">Create Scenario</h2>
              <button
                onClick={() => setShowCreateModal(false)}
                className="p-1 text-gray-400 hover:text-gray-600"
              >
                <X className="w-5 h-5" />
              </button>
            </div>
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">Name</label>
                <input
                  type="text"
                  className="w-full px-3 py-2 border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500"
                  placeholder="e.g., Cloud Migration Wave 2"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">Description</label>
                <textarea
                  className="w-full px-3 py-2 border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500"
                  rows={3}
                  placeholder="Describe the goal and scope of this scenario"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  Select Applications
                </label>
                <p className="text-xs text-gray-500 mb-2">
                  You can add applications after creating the scenario
                </p>
              </div>
              <div className="flex justify-end gap-3 pt-4">
                <button
                  onClick={() => setShowCreateModal(false)}
                  className="px-4 py-2 text-sm font-medium text-gray-700 hover:text-gray-900"
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
          <div className="bg-white rounded-xl p-6 w-full max-w-4xl max-h-[90vh] overflow-y-auto">
            <div className="flex items-center justify-between mb-6">
              <h2 className="text-lg font-semibold">Compare Scenarios</h2>
              <button
                onClick={() => setShowCompareModal(false)}
                className="p-1 text-gray-400 hover:text-gray-600"
              >
                <X className="w-5 h-5" />
              </button>
            </div>

            <div className="overflow-x-auto">
              <table className="w-full">
                <thead>
                  <tr className="border-b border-gray-200">
                    <th className="text-left text-sm font-medium text-gray-500 pb-3">Metric</th>
                    {compareSelection.map((id) => {
                      const scenario = analyzedScenarios.find((s) => s.id === id);
                      return (
                        <th key={id} className="text-left text-sm font-medium text-gray-900 pb-3 px-4">
                          {scenario?.name}
                        </th>
                      );
                    })}
                  </tr>
                </thead>
                <tbody className="divide-y divide-gray-100">
                  <tr>
                    <td className="py-3 text-sm text-gray-600">Applications</td>
                    {compareSelection.map((id) => {
                      const scenario = analyzedScenarios.find((s) => s.id === id);
                      return (
                        <td key={id} className="py-3 text-sm font-medium text-gray-900 px-4">
                          {scenario?.applications.length}
                        </td>
                      );
                    })}
                  </tr>
                  <tr>
                    <td className="py-3 text-sm text-gray-600">Total Cost</td>
                    {compareSelection.map((id) => {
                      const scenario = analyzedScenarios.find((s) => s.id === id);
                      return (
                        <td key={id} className="py-3 text-sm font-medium text-gray-900 px-4">
                          {scenario?.analysis && formatCurrency(scenario.analysis.totalCost)}
                        </td>
                      );
                    })}
                  </tr>
                  <tr>
                    <td className="py-3 text-sm text-gray-600">Annual Savings</td>
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
                            className={`text-sm font-medium ${isMax ? 'text-green-600' : 'text-gray-900'}`}
                          >
                            {scenario?.analysis && formatCurrency(scenario.analysis.annualSavings)}
                            {isMax && <Check className="w-4 h-4 inline ml-1" />}
                          </span>
                        </td>
                      );
                    })}
                  </tr>
                  <tr>
                    <td className="py-3 text-sm text-gray-600">ROI</td>
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
                            className={`text-sm font-medium ${isMax ? 'text-green-600' : 'text-gray-900'}`}
                          >
                            {scenario?.analysis && `${Math.round(scenario.analysis.roi * 100)}%`}
                            {isMax && <Check className="w-4 h-4 inline ml-1" />}
                          </span>
                        </td>
                      );
                    })}
                  </tr>
                  <tr>
                    <td className="py-3 text-sm text-gray-600">Payback Period</td>
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
                            className={`text-sm font-medium ${isMin ? 'text-green-600' : 'text-gray-900'}`}
                          >
                            {scenario?.analysis && `${scenario.analysis.paybackMonths} months`}
                            {isMin && <Check className="w-4 h-4 inline ml-1" />}
                          </span>
                        </td>
                      );
                    })}
                  </tr>
                  <tr>
                    <td className="py-3 text-sm text-gray-600">NPV</td>
                    {compareSelection.map((id) => {
                      const scenario = analyzedScenarios.find((s) => s.id === id);
                      return (
                        <td key={id} className="py-3 text-sm font-medium text-gray-900 px-4">
                          {scenario?.analysis && formatCurrency(scenario.analysis.npv)}
                        </td>
                      );
                    })}
                  </tr>
                  <tr>
                    <td className="py-3 text-sm text-gray-600">Risk Level</td>
                    {compareSelection.map((id) => {
                      const scenario = analyzedScenarios.find((s) => s.id === id);
                      return (
                        <td key={id} className="py-3 px-4">
                          <span
                            className={`text-xs font-medium px-2 py-1 rounded-full capitalize ${
                              scenario?.analysis?.riskLevel === 'low'
                                ? 'bg-green-100 text-green-700'
                                : scenario?.analysis?.riskLevel === 'medium'
                                ? 'bg-yellow-100 text-yellow-700'
                                : 'bg-red-100 text-red-700'
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

            <div className="flex justify-end gap-3 mt-6 pt-4 border-t border-gray-100">
              <button
                onClick={() => setShowCompareModal(false)}
                className="px-4 py-2 text-sm font-medium text-gray-700 hover:text-gray-900"
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
