import { useState } from 'react';
import {
  Search,
  Filter,
  Plus,
  MoreHorizontal,
  ArrowUpDown,
  Eye,
  Edit,
  Trash2,
  TrendingUp,
  TrendingDown,
} from 'lucide-react';

// Mock data
const applications = [
  {
    id: '1',
    name: 'Core ERP System',
    type: 'Enterprise Resource Planning',
    owner: 'Finance Team',
    criticality: 'mission_critical',
    lifecycle: 'production',
    quadrant: 'invest',
    scores: { value: 8.5, health: 7.8, complexity: 6.2, cost: 4.5, fit: 8.0 },
    totalCost: 450000,
    dependencies: 12,
    lastAssessed: '2024-01-15',
  },
  {
    id: '2',
    name: 'Legacy CRM',
    type: 'Customer Relationship Management',
    owner: 'Sales Team',
    criticality: 'business_critical',
    lifecycle: 'sunset',
    quadrant: 'eliminate',
    scores: { value: 3.2, health: 2.8, complexity: 7.5, cost: 6.8, fit: 2.5 },
    totalCost: 180000,
    dependencies: 4,
    lastAssessed: '2024-01-10',
  },
  {
    id: '3',
    name: 'HR Management System',
    type: 'Human Resources',
    owner: 'HR Department',
    criticality: 'business_critical',
    lifecycle: 'production',
    quadrant: 'migrate',
    scores: { value: 7.5, health: 3.5, complexity: 8.0, cost: 5.5, fit: 6.0 },
    totalCost: 95000,
    dependencies: 8,
    lastAssessed: '2024-01-12',
  },
  {
    id: '4',
    name: 'Internal Wiki',
    type: 'Knowledge Management',
    owner: 'IT Department',
    criticality: 'operational',
    lifecycle: 'production',
    quadrant: 'tolerate',
    scores: { value: 4.0, health: 7.2, complexity: 2.5, cost: 2.0, fit: 5.5 },
    totalCost: 12000,
    dependencies: 2,
    lastAssessed: '2024-01-08',
  },
  {
    id: '5',
    name: 'Analytics Platform',
    type: 'Business Intelligence',
    owner: 'Data Team',
    criticality: 'mission_critical',
    lifecycle: 'growth',
    quadrant: 'invest',
    scores: { value: 9.0, health: 8.5, complexity: 5.0, cost: 6.0, fit: 9.0 },
    totalCost: 320000,
    dependencies: 15,
    lastAssessed: '2024-01-14',
  },
  {
    id: '6',
    name: 'Email Server',
    type: 'Communication',
    owner: 'IT Department',
    criticality: 'mission_critical',
    lifecycle: 'production',
    quadrant: 'migrate',
    scores: { value: 8.0, health: 4.0, complexity: 6.5, cost: 7.0, fit: 5.0 },
    totalCost: 85000,
    dependencies: 20,
    lastAssessed: '2024-01-11',
  },
];

const quadrantColors: Record<string, string> = {
  tolerate: 'bg-amber-900/40 text-amber-400 border-amber-700/50',
  invest: 'bg-green-900/40 text-green-400 border-green-700/50',
  migrate: 'bg-blue-900/40 text-blue-400 border-blue-700/50',
  eliminate: 'bg-red-900/40 text-red-400 border-red-700/50',
};

const criticalityColors: Record<string, string> = {
  mission_critical: 'bg-red-900/40 text-red-400',
  business_critical: 'bg-orange-900/40 text-orange-400',
  operational: 'bg-yellow-900/40 text-yellow-400',
  administrative: 'bg-gray-700/50 text-gray-300',
};

const lifecycleColors: Record<string, string> = {
  planning: 'bg-purple-900/40 text-purple-400',
  development: 'bg-blue-900/40 text-blue-400',
  growth: 'bg-green-900/40 text-green-400',
  production: 'bg-emerald-900/40 text-emerald-400',
  sunset: 'bg-orange-900/40 text-orange-400',
  retired: 'bg-gray-700/50 text-gray-300',
};

export function ApplicationPortfolioPage() {
  const [searchQuery, setSearchQuery] = useState('');
  const [quadrantFilter, setQuadrantFilter] = useState<string>('all');
  const [selectedApp, setSelectedApp] = useState<typeof applications[0] | null>(null);
  const [showActionMenu, setShowActionMenu] = useState<string | null>(null);

  const filteredApps = applications.filter((app) => {
    const matchesSearch = app.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      app.type.toLowerCase().includes(searchQuery.toLowerCase());
    const matchesQuadrant = quadrantFilter === 'all' || app.quadrant === quadrantFilter;
    return matchesSearch && matchesQuadrant;
  });

  const formatCurrency = (amount: number) => {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: 'USD',
      minimumFractionDigits: 0,
      maximumFractionDigits: 0,
    }).format(amount);
  };

  const getOverallScore = (scores: typeof applications[0]['scores']) => {
    const { value, health, complexity, cost, fit } = scores;
    return ((value + health + (10 - complexity) + (10 - cost) + fit) / 5).toFixed(1);
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-white">Application Portfolio</h1>
          <p className="text-gray-500">Manage and assess your application inventory</p>
        </div>
        <button className="flex items-center gap-2 px-4 py-2 bg-primary-600 text-white rounded-lg text-sm font-medium hover:bg-primary-700">
          <Plus className="w-4 h-4" />
          Add Application
        </button>
      </div>

      {/* Filters */}
      <div className="flex items-center gap-4">
        <div className="relative flex-1 max-w-md">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400" />
          <input
            type="text"
            placeholder="Search applications..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-full pl-10 pr-4 py-2 bg-surface-base/50 border border-surface-border rounded-lg text-sm text-gray-200 placeholder-gray-500 focus:border-primary-500/50 focus:ring-1 focus:ring-primary-500/20 outline-none"
          />
        </div>
        <select
          value={quadrantFilter}
          onChange={(e) => setQuadrantFilter(e.target.value)}
          className="px-3 py-2 bg-surface-base/50 border border-surface-border rounded-lg text-sm text-gray-200 placeholder-gray-500 focus:border-primary-500/50 focus:ring-1 focus:ring-primary-500/20 outline-none"
        >
          <option value="all">All Quadrants</option>
          <option value="invest">Invest</option>
          <option value="tolerate">Tolerate</option>
          <option value="migrate">Migrate</option>
          <option value="eliminate">Eliminate</option>
        </select>
        <button className="flex items-center gap-2 px-3 py-2 border border-surface-border rounded-lg text-sm text-gray-400 hover:bg-white/5">
          <Filter className="w-4 h-4" />
          More Filters
        </button>
      </div>

      {/* Summary Cards */}
      <div className="grid grid-cols-4 gap-4">
        {(['invest', 'tolerate', 'migrate', 'eliminate'] as const).map((quadrant) => {
          const count = applications.filter((a) => a.quadrant === quadrant).length;
          const totalCost = applications
            .filter((a) => a.quadrant === quadrant)
            .reduce((sum, a) => sum + a.totalCost, 0);
          return (
            <div
              key={quadrant}
              onClick={() => setQuadrantFilter(quadrantFilter === quadrant ? 'all' : quadrant)}
              className={`p-4 rounded-xl border-2 cursor-pointer transition-all ${
                quadrantFilter === quadrant
                  ? quadrantColors[quadrant] + ' ring-2 ring-offset-2 ring-offset-surface-base'
                  : 'bg-surface-raised/80 backdrop-blur-glass border-surface-border hover:border-gray-600'
              }`}
            >
              <div className="flex items-center justify-between mb-2">
                <span className="text-sm font-semibold capitalize">{quadrant}</span>
                <span className="text-2xl font-bold">{count}</span>
              </div>
              <p className="text-xs text-gray-500">{formatCurrency(totalCost)}/yr</p>
            </div>
          );
        })}
      </div>

      {/* Applications Table */}
      <div className="bg-surface-raised/80 backdrop-blur-glass rounded-xl shadow-glass border border-surface-border">
        <div className="overflow-x-auto">
          <table className="w-full">
            <thead>
              <tr className="border-b border-surface-border">
                <th className="text-left text-xs font-medium text-gray-500 uppercase tracking-wider px-6 py-4">
                  <div className="flex items-center gap-1 cursor-pointer hover:text-gray-200">
                    Application
                    <ArrowUpDown className="w-3 h-3" />
                  </div>
                </th>
                <th className="text-left text-xs font-medium text-gray-500 uppercase tracking-wider px-6 py-4">
                  Quadrant
                </th>
                <th className="text-left text-xs font-medium text-gray-500 uppercase tracking-wider px-6 py-4">
                  Criticality
                </th>
                <th className="text-left text-xs font-medium text-gray-500 uppercase tracking-wider px-6 py-4">
                  Lifecycle
                </th>
                <th className="text-left text-xs font-medium text-gray-500 uppercase tracking-wider px-6 py-4">
                  <div className="flex items-center gap-1 cursor-pointer hover:text-gray-200">
                    Score
                    <ArrowUpDown className="w-3 h-3" />
                  </div>
                </th>
                <th className="text-left text-xs font-medium text-gray-500 uppercase tracking-wider px-6 py-4">
                  Annual Cost
                </th>
                <th className="text-left text-xs font-medium text-gray-500 uppercase tracking-wider px-6 py-4">
                  Dependencies
                </th>
                <th className="text-right text-xs font-medium text-gray-500 uppercase tracking-wider px-6 py-4">
                  Actions
                </th>
              </tr>
            </thead>
            <tbody className="divide-y divide-surface-border">
              {filteredApps.map((app) => (
                <tr
                  key={app.id}
                  onClick={() => setSelectedApp(app)}
                  className={`hover:bg-white/5 cursor-pointer transition-colors ${
                    selectedApp?.id === app.id ? 'bg-primary-900/30' : ''
                  }`}
                >
                  <td className="px-6 py-4">
                    <div>
                      <p className="text-sm font-medium text-white">{app.name}</p>
                      <p className="text-xs text-gray-500">{app.type}</p>
                    </div>
                  </td>
                  <td className="px-6 py-4">
                    <span
                      className={`text-xs font-medium px-2.5 py-1 rounded-full border capitalize ${quadrantColors[app.quadrant]}`}
                    >
                      {app.quadrant}
                    </span>
                  </td>
                  <td className="px-6 py-4">
                    <span
                      className={`text-xs font-medium px-2 py-1 rounded-full ${criticalityColors[app.criticality]}`}
                    >
                      {app.criticality.replace('_', ' ')}
                    </span>
                  </td>
                  <td className="px-6 py-4">
                    <span
                      className={`text-xs font-medium px-2 py-1 rounded-full capitalize ${lifecycleColors[app.lifecycle]}`}
                    >
                      {app.lifecycle}
                    </span>
                  </td>
                  <td className="px-6 py-4">
                    <div className="flex items-center gap-2">
                      <span className="text-sm font-semibold text-white">
                        {getOverallScore(app.scores)}
                      </span>
                      {parseFloat(getOverallScore(app.scores)) >= 6 ? (
                        <TrendingUp className="w-4 h-4 text-green-500" />
                      ) : (
                        <TrendingDown className="w-4 h-4 text-red-500" />
                      )}
                    </div>
                  </td>
                  <td className="px-6 py-4">
                    <span className="text-sm text-gray-400">{formatCurrency(app.totalCost)}</span>
                  </td>
                  <td className="px-6 py-4">
                    <span className="text-sm text-gray-400">{app.dependencies}</span>
                  </td>
                  <td className="px-6 py-4 text-right">
                    <div className="relative">
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          setShowActionMenu(showActionMenu === app.id ? null : app.id);
                        }}
                        className="p-1 text-gray-400 hover:text-gray-300 rounded"
                      >
                        <MoreHorizontal className="w-5 h-5" />
                      </button>
                      {showActionMenu === app.id && (
                        <div className="absolute right-0 mt-1 w-48 bg-surface-raised border border-surface-border rounded-lg shadow-lg py-1 z-10">
                          <button className="flex items-center gap-2 w-full px-4 py-2 text-sm text-gray-300 hover:bg-white/5">
                            <Eye className="w-4 h-4" />
                            View Details
                          </button>
                          <button className="flex items-center gap-2 w-full px-4 py-2 text-sm text-gray-300 hover:bg-white/5">
                            <Edit className="w-4 h-4" />
                            Edit Application
                          </button>
                          <button className="flex items-center gap-2 w-full px-4 py-2 text-sm text-gray-300 hover:bg-white/5">
                            <TrendingUp className="w-4 h-4" />
                            Run Assessment
                          </button>
                          <hr className="my-1 border-surface-border" />
                          <button className="flex items-center gap-2 w-full px-4 py-2 text-sm text-red-400 hover:bg-red-900/20">
                            <Trash2 className="w-4 h-4" />
                            Delete
                          </button>
                        </div>
                      )}
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>

      {/* Application Detail Drawer */}
      {selectedApp && (
        <div className="fixed inset-y-0 right-0 w-96 bg-surface-raised shadow-xl border-l border-surface-border z-50 overflow-y-auto">
          <div className="p-6">
            <div className="flex items-center justify-between mb-6">
              <h2 className="text-lg font-semibold text-white">Application Details</h2>
              <button
                onClick={() => setSelectedApp(null)}
                className="text-gray-400 hover:text-gray-300"
              >
                ×
              </button>
            </div>

            <div className="space-y-6">
              {/* Basic Info */}
              <div>
                <h3 className="text-xl font-bold text-white mb-1">{selectedApp.name}</h3>
                <p className="text-sm text-gray-500">{selectedApp.type}</p>
                <div className="flex items-center gap-2 mt-3">
                  <span
                    className={`text-xs font-medium px-2.5 py-1 rounded-full border capitalize ${quadrantColors[selectedApp.quadrant]}`}
                  >
                    {selectedApp.quadrant}
                  </span>
                  <span
                    className={`text-xs font-medium px-2 py-1 rounded-full ${criticalityColors[selectedApp.criticality]}`}
                  >
                    {selectedApp.criticality.replace('_', ' ')}
                  </span>
                </div>
              </div>

              {/* Scores */}
              <div>
                <h4 className="text-sm font-medium text-white mb-3">Assessment Scores</h4>
                <div className="space-y-3">
                  {Object.entries(selectedApp.scores).map(([key, value]) => (
                    <div key={key}>
                      <div className="flex items-center justify-between text-sm mb-1">
                        <span className="text-gray-400 capitalize">{key}</span>
                        <span className="font-medium text-white">{value}/10</span>
                      </div>
                      <div className="w-full h-2 bg-surface-overlay rounded-full overflow-hidden">
                        <div
                          className={`h-full rounded-full transition-all ${
                            value >= 7
                              ? 'bg-green-500'
                              : value >= 5
                              ? 'bg-amber-500'
                              : 'bg-red-500'
                          }`}
                          style={{ width: `${value * 10}%` }}
                        />
                      </div>
                    </div>
                  ))}
                </div>
              </div>

              {/* Key Metrics */}
              <div className="grid grid-cols-2 gap-4">
                <div className="p-3 bg-surface-overlay/50 rounded-lg">
                  <p className="text-xs text-gray-500">Annual Cost</p>
                  <p className="text-lg font-semibold text-white">
                    {formatCurrency(selectedApp.totalCost)}
                  </p>
                </div>
                <div className="p-3 bg-surface-overlay/50 rounded-lg">
                  <p className="text-xs text-gray-500">Dependencies</p>
                  <p className="text-lg font-semibold text-white">{selectedApp.dependencies}</p>
                </div>
              </div>

              {/* Owner */}
              <div className="p-3 bg-surface-overlay/50 rounded-lg">
                <p className="text-xs text-gray-500 mb-1">Owner</p>
                <p className="text-sm font-medium text-white">{selectedApp.owner}</p>
              </div>

              {/* Actions */}
              <div className="flex gap-2 pt-4 border-t border-surface-border">
                <button className="flex-1 px-4 py-2 bg-primary-600 text-white rounded-lg text-sm font-medium hover:bg-primary-700">
                  Run Assessment
                </button>
                <button className="flex-1 px-4 py-2 border border-surface-border rounded-lg text-sm font-medium text-gray-300 hover:bg-white/5">
                  Create Scenario
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
