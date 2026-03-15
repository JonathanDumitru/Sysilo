import { useState, useMemo } from 'react';
import { useNavigate, Link } from 'react-router-dom';
import {
  BarChart3,
  TrendingUp,
  Target,
  DollarSign,
  Layers,
  ArrowUpRight,
  ArrowDownRight,
  Minus,
  Loader2,
  AlertCircle,
  GitBranch,
  BookOpen,
  FolderKanban,
} from 'lucide-react';
import {
  useTimeSummary,
  useTimeAssessments,
  useRecommendations,
  useScenarios,
  usePortfolioAnalytics,
} from '../hooks/useRationalization';
import type { TimeQuadrant } from '../services/rationalization';

// Format currency from raw number
function formatCurrency(value: number | undefined): string {
  if (value == null) return '$0';
  if (value >= 1_000_000) {
    return `$${(value / 1_000_000).toFixed(1)}M`;
  }
  if (value >= 1_000) {
    return `$${(value / 1_000).toFixed(0)}K`;
  }
  return `$${value.toFixed(0)}`;
}

// Format score to one decimal place
function formatScore(value: number | undefined): string {
  if (value == null) return '0.0';
  return value.toFixed(1);
}

export function RationalizationDashboardPage() {
  const [selectedQuadrant, setSelectedQuadrant] = useState<TimeQuadrant | null>(null);
  const navigate = useNavigate();

  // Real data hooks
  const { data: timeSummary, isLoading: timeSummaryLoading, error: timeSummaryError } = useTimeSummary();
  const { data: timeAssessments, isLoading: assessmentsLoading } = useTimeAssessments();
  const { data: recommendations, isLoading: recommendationsLoading } = useRecommendations();
  const { data: scenarios, isLoading: scenariosLoading } = useScenarios();
  const { data: analytics, isLoading: analyticsLoading } = usePortfolioAnalytics();

  // Group assessments by quadrant for the detail view
  const assessmentsByQuadrant = useMemo(() => {
    if (!timeAssessments) return {} as Record<TimeQuadrant, typeof timeAssessments>;
    const grouped: Record<string, typeof timeAssessments> = {
      tolerate: [],
      invest: [],
      migrate: [],
      eliminate: [],
    };
    for (const assessment of timeAssessments) {
      if (grouped[assessment.quadrant]) {
        grouped[assessment.quadrant].push(assessment);
      }
    }
    return grouped as Record<TimeQuadrant, typeof timeAssessments>;
  }, [timeAssessments]);

  // Build portfolio metrics from analytics data
  const portfolioMetrics = useMemo(() => {
    return [
      {
        name: 'Total Applications',
        value: analytics?.total_applications?.toString() ?? '--',
        icon: Layers,
      },
      {
        name: 'Annual IT Spend',
        value: analytics ? formatCurrency(analytics.total_cost) : '--',
        icon: DollarSign,
      },
      {
        name: 'Avg Health Score',
        value: analytics ? formatScore(analytics.avg_health_score) : '--',
        icon: Target,
      },
      {
        name: 'Avg Value Score',
        value: analytics ? formatScore(analytics.avg_value_score) : '--',
        icon: BarChart3,
      },
    ];
  }, [analytics]);

  // Top 3 recommendations
  const topRecommendations = useMemo(() => {
    if (!recommendations) return [];
    return recommendations.slice(0, 3);
  }, [recommendations]);

  const isLoading = timeSummaryLoading || analyticsLoading;
  const hasError = timeSummaryError;

  const getQuadrantColor = (quadrant: string) => {
    switch (quadrant) {
      case 'tolerate':
        return 'bg-amber-900/40 border-amber-700/50 text-amber-400';
      case 'invest':
        return 'bg-green-900/40 border-green-700/50 text-green-400';
      case 'migrate':
        return 'bg-blue-900/40 border-blue-700/50 text-blue-400';
      case 'eliminate':
        return 'bg-red-900/40 border-red-700/50 text-red-400';
      default:
        return 'bg-gray-700/50 border-surface-border text-gray-300';
    }
  };

  const getTypeColor = (type: string) => {
    switch (type) {
      case 'retirement':
        return 'bg-red-900/40 text-red-400';
      case 'migration':
        return 'bg-blue-900/40 text-blue-400';
      case 'consolidation':
        return 'bg-purple-900/40 text-purple-400';
      case 'optimization':
        return 'bg-green-900/40 text-green-400';
      default:
        return 'bg-gray-700/50 text-gray-300';
    }
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <Loader2 className="w-8 h-8 text-primary-600 animate-spin" />
        <span className="ml-3 text-gray-500">Loading rationalization data...</span>
      </div>
    );
  }

  if (hasError) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-center">
          <AlertCircle className="w-10 h-10 text-red-500 mx-auto mb-3" />
          <p className="text-white font-medium">Failed to load dashboard data</p>
          <p className="text-sm text-gray-500 mt-1">
            {hasError instanceof Error ? hasError.message : 'An unexpected error occurred'}
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-white">Rationalization Dashboard</h1>
          <p className="text-gray-500">Application portfolio analysis and optimization</p>
        </div>
        <div className="flex items-center gap-3">
          <button className="px-4 py-2 border border-surface-border rounded-lg text-sm font-medium text-gray-300 hover:bg-white/5">
            Export Report
          </button>
          <button
            onClick={() => navigate('/rationalization/scenarios')}
            className="px-4 py-2 bg-primary-600 text-white rounded-lg text-sm font-medium hover:bg-primary-700"
          >
            New Scenario
          </button>
        </div>
      </div>

      {/* Key Metrics */}
      <div className="grid grid-cols-4 gap-6">
        {portfolioMetrics.map((metric) => (
          <div
            key={metric.name}
            className="bg-surface-raised/80 backdrop-blur-glass rounded-xl p-6 shadow-glass border border-surface-border"
          >
            <div className="flex items-center justify-between">
              <div className="p-2 bg-primary-900/30 rounded-lg">
                <metric.icon className="w-5 h-5 text-primary-600" />
              </div>
            </div>
            <div className="mt-4">
              {analyticsLoading ? (
                <Loader2 className="w-5 h-5 text-gray-400 animate-spin" />
              ) : (
                <p className="text-3xl font-bold text-white">{metric.value}</p>
              )}
              <p className="text-sm font-medium text-gray-500">{metric.name}</p>
            </div>
          </div>
        ))}
      </div>

      {/* TIME Quadrant & Recommendations */}
      <div className="grid grid-cols-3 gap-6">
        {/* TIME Quadrant Visualization */}
        <div className="col-span-2 bg-surface-raised/80 backdrop-blur-glass rounded-xl p-6 shadow-glass border border-surface-border">
          <div className="flex items-center justify-between mb-6">
            <div>
              <h2 className="text-lg font-semibold text-white">TIME Quadrant Analysis</h2>
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
                    ? 'bg-amber-900/30 border-amber-500 ring-2 ring-amber-700/50'
                    : 'bg-amber-900/20 border-amber-700/50 hover:border-amber-600/50'
                }`}
              >
                <div className="flex items-center justify-between mb-2">
                  <span className="text-sm font-semibold text-amber-400">TOLERATE</span>
                  <span className="text-2xl font-bold text-amber-400">
                    {timeSummary?.tolerate ?? 0}
                  </span>
                </div>
                <p className="text-xs text-amber-500">Low Value • Good Health</p>
                <p className="text-xs text-gray-500 mt-1">Maintain with minimal investment</p>
              </div>

              {/* Invest - High Value, High Health */}
              <div
                onClick={() => setSelectedQuadrant(selectedQuadrant === 'invest' ? null : 'invest')}
                className={`p-4 rounded-xl border-2 cursor-pointer transition-all ${
                  selectedQuadrant === 'invest'
                    ? 'bg-green-900/30 border-green-500 ring-2 ring-green-700/50'
                    : 'bg-green-900/20 border-green-700/50 hover:border-green-600/50'
                }`}
              >
                <div className="flex items-center justify-between mb-2">
                  <span className="text-sm font-semibold text-green-400">INVEST</span>
                  <span className="text-2xl font-bold text-green-400">
                    {timeSummary?.invest ?? 0}
                  </span>
                </div>
                <p className="text-xs text-green-500">High Value • Good Health</p>
                <p className="text-xs text-gray-500 mt-1">Strategic assets to grow</p>
              </div>

              {/* Eliminate - Low Value, Low Health */}
              <div
                onClick={() => setSelectedQuadrant(selectedQuadrant === 'eliminate' ? null : 'eliminate')}
                className={`p-4 rounded-xl border-2 cursor-pointer transition-all ${
                  selectedQuadrant === 'eliminate'
                    ? 'bg-red-900/30 border-red-500 ring-2 ring-red-700/50'
                    : 'bg-red-900/20 border-red-700/50 hover:border-red-600/50'
                }`}
              >
                <div className="flex items-center justify-between mb-2">
                  <span className="text-sm font-semibold text-red-400">ELIMINATE</span>
                  <span className="text-2xl font-bold text-red-400">
                    {timeSummary?.eliminate ?? 0}
                  </span>
                </div>
                <p className="text-xs text-red-500">Low Value • Poor Health</p>
                <p className="text-xs text-gray-500 mt-1">Candidates for retirement</p>
              </div>

              {/* Migrate - High Value, Low Health */}
              <div
                onClick={() => setSelectedQuadrant(selectedQuadrant === 'migrate' ? null : 'migrate')}
                className={`p-4 rounded-xl border-2 cursor-pointer transition-all ${
                  selectedQuadrant === 'migrate'
                    ? 'bg-blue-900/30 border-blue-500 ring-2 ring-blue-700/50'
                    : 'bg-blue-900/20 border-blue-700/50 hover:border-blue-600/50'
                }`}
              >
                <div className="flex items-center justify-between mb-2">
                  <span className="text-sm font-semibold text-blue-400">MIGRATE</span>
                  <span className="text-2xl font-bold text-blue-400">
                    {timeSummary?.migrate ?? 0}
                  </span>
                </div>
                <p className="text-xs text-blue-500">High Value • Poor Health</p>
                <p className="text-xs text-gray-500 mt-1">Modernize or replace</p>
              </div>
            </div>
          </div>

          {/* Selected Quadrant Details */}
          {selectedQuadrant && (
            <div className="mt-4 p-4 bg-surface-overlay/50 rounded-lg">
              <h4 className="text-sm font-medium text-white mb-2 capitalize">
                {selectedQuadrant} Applications
              </h4>
              {assessmentsLoading ? (
                <div className="flex items-center gap-2 text-sm text-gray-500">
                  <Loader2 className="w-4 h-4 animate-spin" />
                  Loading applications...
                </div>
              ) : assessmentsByQuadrant[selectedQuadrant]?.length ? (
                <div className="flex flex-wrap gap-2">
                  {assessmentsByQuadrant[selectedQuadrant].map((assessment) => (
                    <span
                      key={assessment.id}
                      className={`text-xs px-2 py-1 rounded-full border ${getQuadrantColor(selectedQuadrant)}`}
                    >
                      {assessment.application_id}
                    </span>
                  ))}
                </div>
              ) : (
                <p className="text-sm text-gray-500">No applications in this quadrant</p>
              )}
            </div>
          )}
        </div>

        {/* AI Recommendations */}
        <div className="bg-surface-raised/80 backdrop-blur-glass rounded-xl p-6 shadow-glass border border-surface-border">
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-lg font-semibold text-white">AI Recommendations</h2>
            {recommendations && recommendations.length > 0 && (
              <span className="flex items-center gap-1 text-xs text-primary-600">
                <TrendingUp className="w-3 h-3" />
                {recommendations.length} total
              </span>
            )}
          </div>
          {recommendationsLoading ? (
            <div className="flex items-center justify-center py-8">
              <Loader2 className="w-5 h-5 text-gray-400 animate-spin" />
            </div>
          ) : topRecommendations.length === 0 ? (
            <p className="text-sm text-gray-500 py-4 text-center">No recommendations yet</p>
          ) : (
            <div className="space-y-3">
              {topRecommendations.map((rec) => (
                <div
                  key={rec.id}
                  className="p-3 bg-surface-overlay/50 rounded-lg border border-surface-border hover:border-gray-600 cursor-pointer transition-colors"
                >
                  <div className="flex items-start justify-between mb-2">
                    <span
                      className={`text-xs font-medium px-2 py-0.5 rounded-full ${getTypeColor(rec.recommendation_type ?? '')}`}
                    >
                      {rec.recommendation_type ?? 'general'}
                    </span>
                    {rec.confidence_score != null && (
                      <span className="text-xs text-gray-500">
                        {Math.round(rec.confidence_score * 100)}% confidence
                      </span>
                    )}
                  </div>
                  <p className="text-sm font-medium text-white mb-2">{rec.title}</p>
                  <div className="flex items-center justify-between text-xs text-gray-500">
                    {rec.estimated_savings != null && (
                      <span className="text-green-400 font-medium">
                        {formatCurrency(rec.estimated_savings)}/yr savings
                      </span>
                    )}
                    {rec.estimated_effort && (
                      <span className="capitalize">{rec.estimated_effort} effort</span>
                    )}
                  </div>
                </div>
              ))}
            </div>
          )}
          <button className="w-full mt-4 py-2 text-sm font-medium text-primary-600 hover:text-primary-700">
            View all recommendations →
          </button>
        </div>
      </div>

      {/* Active Scenarios */}
      <div className="bg-surface-raised/80 backdrop-blur-glass rounded-xl p-6 shadow-glass border border-surface-border">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-lg font-semibold text-white">Active Scenarios</h2>
          <a href="/rationalization/scenarios" className="text-sm text-primary-600 hover:text-primary-700">
            View all scenarios
          </a>
        </div>
        {scenariosLoading ? (
          <div className="flex items-center justify-center py-8">
            <Loader2 className="w-5 h-5 text-gray-400 animate-spin" />
          </div>
        ) : !scenarios || scenarios.length === 0 ? (
          <p className="text-sm text-gray-500 py-4 text-center">No scenarios created yet</p>
        ) : (
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
              <tbody className="divide-y divide-surface-border">
                {scenarios.map((scenario) => (
                  <tr key={scenario.id} className="text-sm">
                    <td className="py-3 font-medium text-white">{scenario.name}</td>
                    <td className="py-3 text-gray-400">
                      {scenario.affected_applications?.length ?? 0} apps
                    </td>
                    <td className="py-3">
                      <span className="text-green-400 font-medium">
                        {scenario.roi_percent != null ? `+${scenario.roi_percent.toFixed(0)}%` : '--'}
                      </span>
                    </td>
                    <td className="py-3">
                      <span
                        className={`text-xs font-medium px-2 py-1 rounded-full capitalize ${
                          scenario.status === 'complete' || scenario.status === 'completed'
                            ? 'bg-green-900/40 text-green-400'
                            : scenario.status === 'analyzing'
                            ? 'bg-blue-900/40 text-blue-400'
                            : 'bg-gray-700/50 text-gray-300'
                        }`}
                      >
                        {scenario.status ?? 'draft'}
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
        )}
      </div>

      {/* Quick Navigation */}
      <div className="bg-surface-raised/80 backdrop-blur-glass rounded-xl p-6 shadow-glass border border-surface-border">
        <h2 className="text-lg font-semibold text-white mb-4">Quick Navigation</h2>
        <div className="grid grid-cols-2 sm:grid-cols-4 gap-3">
          <Link to="/rationalization/applications" className="flex items-center gap-3 p-3 rounded-lg hover:bg-white/5 transition-colors">
            <Layers className="w-5 h-5 text-gray-400" />
            <span className="text-sm text-gray-300">Applications</span>
          </Link>
          <Link to="/rationalization/scenarios" className="flex items-center gap-3 p-3 rounded-lg hover:bg-white/5 transition-colors">
            <GitBranch className="w-5 h-5 text-gray-400" />
            <span className="text-sm text-gray-300">Scenarios</span>
          </Link>
          <Link to="/rationalization/playbooks" className="flex items-center gap-3 p-3 rounded-lg hover:bg-white/5 transition-colors">
            <BookOpen className="w-5 h-5 text-gray-400" />
            <span className="text-sm text-gray-300">Playbooks</span>
          </Link>
          <Link to="/rationalization/projects" className="flex items-center gap-3 p-3 rounded-lg hover:bg-white/5 transition-colors">
            <FolderKanban className="w-5 h-5 text-gray-400" />
            <span className="text-sm text-gray-300">Projects</span>
          </Link>
        </div>
      </div>
    </div>
  );
}
