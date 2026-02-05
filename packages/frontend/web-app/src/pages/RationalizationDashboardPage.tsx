import { useState } from 'react';
import {
  BarChart3,
  TrendingUp,
  Target,
  DollarSign,
  Layers,
  ArrowUpRight,
  ArrowDownRight,
  Minus,
} from 'lucide-react';

// Mock data for TIME quadrant
const quadrantData = {
  tolerate: { count: 12, apps: ['Legacy CRM', 'Old Reporting Tool', 'Archive System'] },
  invest: { count: 8, apps: ['Core ERP', 'Customer Portal', 'Analytics Platform'] },
  migrate: { count: 15, apps: ['HR System', 'Inventory Mgmt', 'Email Server'] },
  eliminate: { count: 6, apps: ['Unused Tool A', 'Deprecated API', 'Test System'] },
};

const portfolioMetrics = [
  { name: 'Total Applications', value: '41', change: '+3', trend: 'up', icon: Layers },
  { name: 'Annual IT Spend', value: '$4.2M', change: '-8%', trend: 'down', icon: DollarSign },
  { name: 'Avg Health Score', value: '6.8', change: '+0.5', trend: 'up', icon: Target },
  { name: 'Migration Projects', value: '5', change: '0', trend: 'neutral', icon: BarChart3 },
];

const recentRecommendations = [
  {
    id: '1',
    title: 'Retire Legacy CRM',
    type: 'retirement',
    confidence: 0.92,
    savings: '$180,000',
    effort: 'medium',
  },
  {
    id: '2',
    title: 'Modernize HR System',
    type: 'migration',
    confidence: 0.85,
    savings: '$95,000',
    effort: 'high',
  },
  {
    id: '3',
    title: 'Consolidate Reporting Tools',
    type: 'consolidation',
    confidence: 0.78,
    savings: '$65,000',
    effort: 'low',
  },
];

const activeScenarios = [
  { id: '1', name: 'Cloud Migration Wave 1', applications: 8, roi: '+24%', status: 'analyzing' },
  { id: '2', name: 'Legacy Retirement Plan', applications: 6, roi: '+18%', status: 'complete' },
  { id: '3', name: 'Vendor Consolidation', applications: 4, roi: '+12%', status: 'draft' },
];

export function RationalizationDashboardPage() {
  const [selectedQuadrant, setSelectedQuadrant] = useState<string | null>(null);

  const getQuadrantColor = (quadrant: string) => {
    switch (quadrant) {
      case 'tolerate':
        return 'bg-amber-100 border-amber-300 text-amber-800';
      case 'invest':
        return 'bg-green-100 border-green-300 text-green-800';
      case 'migrate':
        return 'bg-blue-100 border-blue-300 text-blue-800';
      case 'eliminate':
        return 'bg-red-100 border-red-300 text-red-800';
      default:
        return 'bg-gray-100 border-gray-300 text-gray-800';
    }
  };

  const getTrendIcon = (trend: string) => {
    switch (trend) {
      case 'up':
        return <ArrowUpRight className="w-4 h-4 text-green-600" />;
      case 'down':
        return <ArrowDownRight className="w-4 h-4 text-red-600" />;
      default:
        return <Minus className="w-4 h-4 text-gray-400" />;
    }
  };

  const getTypeColor = (type: string) => {
    switch (type) {
      case 'retirement':
        return 'bg-red-100 text-red-700';
      case 'migration':
        return 'bg-blue-100 text-blue-700';
      case 'consolidation':
        return 'bg-purple-100 text-purple-700';
      case 'optimization':
        return 'bg-green-100 text-green-700';
      default:
        return 'bg-gray-100 text-gray-700';
    }
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">Rationalization Dashboard</h1>
          <p className="text-gray-500">Application portfolio analysis and optimization</p>
        </div>
        <div className="flex items-center gap-3">
          <button className="px-4 py-2 border border-gray-200 rounded-lg text-sm font-medium text-gray-700 hover:bg-gray-50">
            Export Report
          </button>
          <button className="px-4 py-2 bg-primary-600 text-white rounded-lg text-sm font-medium hover:bg-primary-700">
            New Scenario
          </button>
        </div>
      </div>

      {/* Key Metrics */}
      <div className="grid grid-cols-4 gap-6">
        {portfolioMetrics.map((metric) => (
          <div
            key={metric.name}
            className="bg-white rounded-xl p-6 shadow-sm border border-gray-100"
          >
            <div className="flex items-center justify-between">
              <div className="p-2 bg-primary-50 rounded-lg">
                <metric.icon className="w-5 h-5 text-primary-600" />
              </div>
              <div className="flex items-center gap-1">
                {getTrendIcon(metric.trend)}
                <span
                  className={`text-xs font-medium ${
                    metric.trend === 'up'
                      ? 'text-green-600'
                      : metric.trend === 'down'
                      ? 'text-red-600'
                      : 'text-gray-500'
                  }`}
                >
                  {metric.change}
                </span>
              </div>
            </div>
            <div className="mt-4">
              <p className="text-3xl font-bold text-gray-900">{metric.value}</p>
              <p className="text-sm font-medium text-gray-500">{metric.name}</p>
            </div>
          </div>
        ))}
      </div>

      {/* TIME Quadrant & Recommendations */}
      <div className="grid grid-cols-3 gap-6">
        {/* TIME Quadrant Visualization */}
        <div className="col-span-2 bg-white rounded-xl p-6 shadow-sm border border-gray-100">
          <div className="flex items-center justify-between mb-6">
            <div>
              <h2 className="text-lg font-semibold text-gray-900">TIME Quadrant Analysis</h2>
              <p className="text-sm text-gray-500">Portfolio distribution by business value and technical health</p>
            </div>
            <a href="/rationalization/applications" className="text-sm text-primary-600 hover:text-primary-700">
              View all applications →
            </a>
          </div>

          {/* Quadrant Grid */}
          <div className="relative">
            {/* Axis Labels */}
            <div className="absolute -left-2 top-1/2 -translate-y-1/2 -rotate-90 text-xs font-medium text-gray-500 whitespace-nowrap">
              Technical Health →
            </div>
            <div className="absolute bottom-0 left-1/2 -translate-x-1/2 translate-y-6 text-xs font-medium text-gray-500">
              Business Value →
            </div>

            <div className="grid grid-cols-2 gap-3 ml-4">
              {/* Tolerate - Low Value, High Health */}
              <div
                onClick={() => setSelectedQuadrant(selectedQuadrant === 'tolerate' ? null : 'tolerate')}
                className={`p-4 rounded-xl border-2 cursor-pointer transition-all ${
                  selectedQuadrant === 'tolerate'
                    ? 'bg-amber-50 border-amber-400 ring-2 ring-amber-200'
                    : 'bg-amber-50/50 border-amber-200 hover:border-amber-300'
                }`}
              >
                <div className="flex items-center justify-between mb-2">
                  <span className="text-sm font-semibold text-amber-800">TOLERATE</span>
                  <span className="text-2xl font-bold text-amber-700">{quadrantData.tolerate.count}</span>
                </div>
                <p className="text-xs text-amber-600">Low Value • Good Health</p>
                <p className="text-xs text-gray-500 mt-1">Maintain with minimal investment</p>
              </div>

              {/* Invest - High Value, High Health */}
              <div
                onClick={() => setSelectedQuadrant(selectedQuadrant === 'invest' ? null : 'invest')}
                className={`p-4 rounded-xl border-2 cursor-pointer transition-all ${
                  selectedQuadrant === 'invest'
                    ? 'bg-green-50 border-green-400 ring-2 ring-green-200'
                    : 'bg-green-50/50 border-green-200 hover:border-green-300'
                }`}
              >
                <div className="flex items-center justify-between mb-2">
                  <span className="text-sm font-semibold text-green-800">INVEST</span>
                  <span className="text-2xl font-bold text-green-700">{quadrantData.invest.count}</span>
                </div>
                <p className="text-xs text-green-600">High Value • Good Health</p>
                <p className="text-xs text-gray-500 mt-1">Strategic assets to grow</p>
              </div>

              {/* Eliminate - Low Value, Low Health */}
              <div
                onClick={() => setSelectedQuadrant(selectedQuadrant === 'eliminate' ? null : 'eliminate')}
                className={`p-4 rounded-xl border-2 cursor-pointer transition-all ${
                  selectedQuadrant === 'eliminate'
                    ? 'bg-red-50 border-red-400 ring-2 ring-red-200'
                    : 'bg-red-50/50 border-red-200 hover:border-red-300'
                }`}
              >
                <div className="flex items-center justify-between mb-2">
                  <span className="text-sm font-semibold text-red-800">ELIMINATE</span>
                  <span className="text-2xl font-bold text-red-700">{quadrantData.eliminate.count}</span>
                </div>
                <p className="text-xs text-red-600">Low Value • Poor Health</p>
                <p className="text-xs text-gray-500 mt-1">Candidates for retirement</p>
              </div>

              {/* Migrate - High Value, Low Health */}
              <div
                onClick={() => setSelectedQuadrant(selectedQuadrant === 'migrate' ? null : 'migrate')}
                className={`p-4 rounded-xl border-2 cursor-pointer transition-all ${
                  selectedQuadrant === 'migrate'
                    ? 'bg-blue-50 border-blue-400 ring-2 ring-blue-200'
                    : 'bg-blue-50/50 border-blue-200 hover:border-blue-300'
                }`}
              >
                <div className="flex items-center justify-between mb-2">
                  <span className="text-sm font-semibold text-blue-800">MIGRATE</span>
                  <span className="text-2xl font-bold text-blue-700">{quadrantData.migrate.count}</span>
                </div>
                <p className="text-xs text-blue-600">High Value • Poor Health</p>
                <p className="text-xs text-gray-500 mt-1">Modernize or replace</p>
              </div>
            </div>
          </div>

          {/* Selected Quadrant Details */}
          {selectedQuadrant && (
            <div className="mt-4 p-4 bg-gray-50 rounded-lg">
              <h4 className="text-sm font-medium text-gray-900 mb-2 capitalize">
                {selectedQuadrant} Applications
              </h4>
              <div className="flex flex-wrap gap-2">
                {quadrantData[selectedQuadrant as keyof typeof quadrantData].apps.map((app) => (
                  <span
                    key={app}
                    className={`text-xs px-2 py-1 rounded-full border ${getQuadrantColor(selectedQuadrant)}`}
                  >
                    {app}
                  </span>
                ))}
              </div>
            </div>
          )}
        </div>

        {/* AI Recommendations */}
        <div className="bg-white rounded-xl p-6 shadow-sm border border-gray-100">
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-lg font-semibold text-gray-900">AI Recommendations</h2>
            <span className="flex items-center gap-1 text-xs text-primary-600">
              <TrendingUp className="w-3 h-3" />
              3 new
            </span>
          </div>
          <div className="space-y-3">
            {recentRecommendations.map((rec) => (
              <div
                key={rec.id}
                className="p-3 bg-gray-50 rounded-lg border border-gray-100 hover:border-gray-200 cursor-pointer transition-colors"
              >
                <div className="flex items-start justify-between mb-2">
                  <span
                    className={`text-xs font-medium px-2 py-0.5 rounded-full ${getTypeColor(rec.type)}`}
                  >
                    {rec.type}
                  </span>
                  <span className="text-xs text-gray-500">{Math.round(rec.confidence * 100)}% confidence</span>
                </div>
                <p className="text-sm font-medium text-gray-900 mb-2">{rec.title}</p>
                <div className="flex items-center justify-between text-xs text-gray-500">
                  <span className="text-green-600 font-medium">{rec.savings}/yr savings</span>
                  <span className="capitalize">{rec.effort} effort</span>
                </div>
              </div>
            ))}
          </div>
          <button className="w-full mt-4 py-2 text-sm font-medium text-primary-600 hover:text-primary-700">
            View all recommendations →
          </button>
        </div>
      </div>

      {/* Active Scenarios */}
      <div className="bg-white rounded-xl p-6 shadow-sm border border-gray-100">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-lg font-semibold text-gray-900">Active Scenarios</h2>
          <a href="/rationalization/scenarios" className="text-sm text-primary-600 hover:text-primary-700">
            View all scenarios
          </a>
        </div>
        <div className="overflow-x-auto">
          <table className="w-full">
            <thead>
              <tr className="text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                <th className="pb-3">Scenario</th>
                <th className="pb-3">Applications</th>
                <th className="pb-3">Projected ROI</th>
                <th className="pb-3">Status</th>
                <th className="pb-3">Actions</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-gray-100">
              {activeScenarios.map((scenario) => (
                <tr key={scenario.id} className="text-sm">
                  <td className="py-3 font-medium text-gray-900">{scenario.name}</td>
                  <td className="py-3 text-gray-600">{scenario.applications} apps</td>
                  <td className="py-3">
                    <span className="text-green-600 font-medium">{scenario.roi}</span>
                  </td>
                  <td className="py-3">
                    <span
                      className={`text-xs font-medium px-2 py-1 rounded-full capitalize ${
                        scenario.status === 'complete'
                          ? 'bg-green-100 text-green-700'
                          : scenario.status === 'analyzing'
                          ? 'bg-blue-100 text-blue-700'
                          : 'bg-gray-100 text-gray-700'
                      }`}
                    >
                      {scenario.status}
                    </span>
                  </td>
                  <td className="py-3">
                    <button className="text-primary-600 hover:text-primary-700 text-xs font-medium">
                      View Details
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}
