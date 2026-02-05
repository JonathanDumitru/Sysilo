import {
  Sparkles,
  TrendingUp,
  AlertTriangle,
  CheckCircle,
  ChevronRight,
  ThumbsUp,
  ThumbsDown,
  X,
} from 'lucide-react';

interface AIRecommendationCardProps {
  id: string;
  type: 'retirement' | 'migration' | 'consolidation' | 'optimization' | 'investment';
  title: string;
  summary: string;
  confidence: number;
  estimatedSavings?: number;
  estimatedEffort: 'low' | 'medium' | 'high';
  riskLevel: 'low' | 'medium' | 'high';
  onAccept?: (id: string) => void;
  onReject?: (id: string) => void;
  onViewDetails?: (id: string) => void;
  onDismiss?: (id: string) => void;
  compact?: boolean;
}

const typeConfig = {
  retirement: {
    color: 'red',
    icon: AlertTriangle,
    label: 'Retirement',
  },
  migration: {
    color: 'blue',
    icon: TrendingUp,
    label: 'Migration',
  },
  consolidation: {
    color: 'purple',
    icon: TrendingUp,
    label: 'Consolidation',
  },
  optimization: {
    color: 'green',
    icon: CheckCircle,
    label: 'Optimization',
  },
  investment: {
    color: 'amber',
    icon: Sparkles,
    label: 'Investment',
  },
};

const effortColors = {
  low: 'bg-green-100 text-green-700',
  medium: 'bg-yellow-100 text-yellow-700',
  high: 'bg-red-100 text-red-700',
};

const riskColors = {
  low: 'text-green-600',
  medium: 'text-yellow-600',
  high: 'text-red-600',
};

export function AIRecommendationCard({
  id,
  type,
  title,
  summary,
  confidence,
  estimatedSavings,
  estimatedEffort,
  riskLevel,
  onAccept,
  onReject,
  onViewDetails,
  onDismiss,
  compact = false,
}: AIRecommendationCardProps) {
  const config = typeConfig[type];

  const formatCurrency = (amount: number): string => {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: 'USD',
      minimumFractionDigits: 0,
      maximumFractionDigits: 0,
    }).format(amount);
  };

  if (compact) {
    return (
      <div className="p-3 bg-white rounded-lg border border-gray-100 hover:border-gray-200 transition-colors">
        <div className="flex items-start gap-3">
          <div className={`p-1.5 rounded-lg bg-${config.color}-50`}>
            <Sparkles className={`w-4 h-4 text-${config.color}-600`} />
          </div>
          <div className="flex-1 min-w-0">
            <div className="flex items-center gap-2 mb-1">
              <span className={`text-xs font-medium px-1.5 py-0.5 rounded bg-${config.color}-100 text-${config.color}-700`}>
                {config.label}
              </span>
              <span className="text-xs text-gray-400">{Math.round(confidence * 100)}%</span>
            </div>
            <p className="text-sm font-medium text-gray-900 truncate">{title}</p>
            {estimatedSavings && (
              <p className="text-xs text-green-600 font-medium mt-1">
                {formatCurrency(estimatedSavings)}/yr potential savings
              </p>
            )}
          </div>
          <button
            onClick={() => onViewDetails?.(id)}
            className="p-1 text-gray-400 hover:text-primary-600 rounded"
          >
            <ChevronRight className="w-4 h-4" />
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="bg-white rounded-xl shadow-sm border border-gray-100 overflow-hidden">
      {/* Header */}
      <div className={`px-4 py-3 bg-gradient-to-r from-${config.color}-50 to-white border-b border-gray-100`}>
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <div className={`p-1.5 rounded-lg bg-${config.color}-100`}>
              <Sparkles className={`w-4 h-4 text-${config.color}-600`} />
            </div>
            <span className={`text-xs font-medium px-2 py-0.5 rounded-full bg-${config.color}-100 text-${config.color}-700`}>
              AI {config.label} Recommendation
            </span>
          </div>
          <div className="flex items-center gap-2">
            <span className="text-xs text-gray-500">{Math.round(confidence * 100)}% confidence</span>
            {onDismiss && (
              <button
                onClick={() => onDismiss(id)}
                className="p-1 text-gray-400 hover:text-gray-600 rounded"
              >
                <X className="w-4 h-4" />
              </button>
            )}
          </div>
        </div>
      </div>

      {/* Content */}
      <div className="p-4">
        <h3 className="text-lg font-semibold text-gray-900 mb-2">{title}</h3>
        <p className="text-sm text-gray-600 mb-4">{summary}</p>

        {/* Metrics */}
        <div className="grid grid-cols-3 gap-3 mb-4">
          {estimatedSavings && (
            <div className="p-3 bg-green-50 rounded-lg">
              <p className="text-xs text-green-600 mb-1">Est. Savings</p>
              <p className="text-lg font-bold text-green-700">{formatCurrency(estimatedSavings)}</p>
              <p className="text-xs text-green-600">per year</p>
            </div>
          )}
          <div className="p-3 bg-gray-50 rounded-lg">
            <p className="text-xs text-gray-500 mb-1">Effort</p>
            <span className={`inline-block text-xs font-medium px-2 py-1 rounded-full capitalize ${effortColors[estimatedEffort]}`}>
              {estimatedEffort}
            </span>
          </div>
          <div className="p-3 bg-gray-50 rounded-lg">
            <p className="text-xs text-gray-500 mb-1">Risk</p>
            <span className={`text-sm font-medium capitalize ${riskColors[riskLevel]}`}>
              {riskLevel}
            </span>
          </div>
        </div>

        {/* Actions */}
        <div className="flex items-center gap-2 pt-3 border-t border-gray-100">
          {onAccept && (
            <button
              onClick={() => onAccept(id)}
              className="flex-1 flex items-center justify-center gap-2 px-4 py-2 bg-primary-600 text-white rounded-lg text-sm font-medium hover:bg-primary-700 transition-colors"
            >
              <ThumbsUp className="w-4 h-4" />
              Accept
            </button>
          )}
          {onReject && (
            <button
              onClick={() => onReject(id)}
              className="flex-1 flex items-center justify-center gap-2 px-4 py-2 border border-gray-200 rounded-lg text-sm font-medium text-gray-700 hover:bg-gray-50 transition-colors"
            >
              <ThumbsDown className="w-4 h-4" />
              Reject
            </button>
          )}
          {onViewDetails && (
            <button
              onClick={() => onViewDetails(id)}
              className="flex items-center gap-1 px-4 py-2 text-sm font-medium text-primary-600 hover:text-primary-700"
            >
              View Details
              <ChevronRight className="w-4 h-4" />
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
