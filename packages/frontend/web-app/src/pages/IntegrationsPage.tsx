import { useState } from 'react';
import { Link } from 'react-router-dom';
import { Plus, Play, MoreVertical, Search, AlertCircle, Loader2 } from 'lucide-react';
import { useIntegrations, useRunIntegration } from '../hooks/useIntegrations';
import type { IntegrationStatus, IntegrationSummary } from '../services/integrations';

const STATUS_STYLES: Record<IntegrationStatus, { badge: string; dot: string }> = {
  active: { badge: 'bg-green-900/30 text-green-400', dot: 'bg-green-500' },
  inactive: { badge: 'bg-amber-900/30 text-amber-400', dot: 'bg-amber-500' },
  draft: { badge: 'bg-gray-700/50 text-gray-400', dot: 'bg-gray-400' },
  error: { badge: 'bg-red-900/30 text-red-400', dot: 'bg-red-500' },
};

function IntegrationCardSkeleton() {
  return (
    <div className="bg-surface-raised/80 backdrop-blur-glass rounded-xl p-6 shadow-glass border border-surface-border animate-pulse">
      <div className="flex items-start justify-between mb-4">
        <div className="flex-1">
          <div className="h-5 bg-gray-700 rounded w-3/4 mb-2" />
          <div className="h-4 bg-surface-overlay rounded w-full" />
        </div>
      </div>
      <div className="flex items-center gap-4">
        <div className="h-6 bg-surface-overlay rounded-full w-16" />
      </div>
      <div className="mt-4 pt-4 border-t border-surface-border">
        <div className="h-3 bg-surface-overlay rounded w-1/3" />
      </div>
    </div>
  );
}

export function IntegrationsPage() {
  const [searchQuery, setSearchQuery] = useState('');
  const [statusFilter, setStatusFilter] = useState<string>('all');

  const { data, isLoading, isError, error } = useIntegrations();
  const runMutation = useRunIntegration();

  const integrations = data?.integrations ?? [];

  const filtered = integrations.filter((integration: IntegrationSummary) => {
    const matchesSearch =
      !searchQuery ||
      integration.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      integration.description.toLowerCase().includes(searchQuery.toLowerCase());

    const matchesStatus = statusFilter === 'all' || integration.status === statusFilter;

    return matchesSearch && matchesStatus;
  });

  const handleRun = (e: React.MouseEvent, integrationId: string) => {
    e.preventDefault();
    e.stopPropagation();
    runMutation.mutate(integrationId);
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-white">Integrations</h1>
          <p className="text-gray-400">Build and manage your data integrations</p>
        </div>
        <Link
          to="/integrations/new"
          className="flex items-center gap-2 px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors"
        >
          <Plus className="w-4 h-4" />
          New Integration
        </Link>
      </div>

      {/* Search and filters */}
      <div className="flex items-center gap-4">
        <div className="flex items-center gap-2 bg-surface-base/50 border border-surface-border rounded-lg px-3 py-2 flex-1 max-w-md">
          <Search className="w-4 h-4 text-gray-400" />
          <input
            type="text"
            placeholder="Search integrations..."
            className="bg-transparent border-none outline-none text-sm text-gray-200 flex-1"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
          />
        </div>
        <select
          className="bg-surface-base/50 border border-surface-border rounded-lg px-3 py-2 text-sm text-gray-200"
          value={statusFilter}
          onChange={(e) => setStatusFilter(e.target.value)}
        >
          <option value="all">All Status</option>
          <option value="active">Active</option>
          <option value="inactive">Inactive</option>
          <option value="draft">Draft</option>
          <option value="error">Error</option>
        </select>
      </div>

      {/* Loading state */}
      {isLoading && (
        <div className="grid grid-cols-2 gap-4">
          <IntegrationCardSkeleton />
          <IntegrationCardSkeleton />
          <IntegrationCardSkeleton />
          <IntegrationCardSkeleton />
        </div>
      )}

      {/* Error state */}
      {isError && (
        <div className="bg-red-900/30 border border-red-500/30 rounded-xl p-6 flex items-center gap-3">
          <AlertCircle className="w-5 h-5 text-red-500 flex-shrink-0" />
          <div>
            <p className="font-medium text-red-400">Failed to load integrations</p>
            <p className="text-sm text-red-400/80 mt-1">
              {error instanceof Error ? error.message : 'An unexpected error occurred.'}
            </p>
          </div>
        </div>
      )}

      {/* Empty state */}
      {!isLoading && !isError && filtered.length === 0 && (
        <div className="bg-surface-raised/80 backdrop-blur-glass rounded-xl p-12 shadow-glass border border-surface-border text-center">
          <p className="text-gray-400 mb-4">
            {integrations.length === 0
              ? 'No integrations yet. Create your first integration to get started.'
              : 'No integrations match your filters.'}
          </p>
          {integrations.length === 0 && (
            <Link
              to="/integrations/new"
              className="inline-flex items-center gap-2 px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors"
            >
              <Plus className="w-4 h-4" />
              New Integration
            </Link>
          )}
        </div>
      )}

      {/* Integration grid */}
      {!isLoading && !isError && filtered.length > 0 && (
        <div className="grid grid-cols-2 gap-4">
          {filtered.map((integration: IntegrationSummary) => {
            const styles = STATUS_STYLES[integration.status] ?? STATUS_STYLES.draft;

            return (
              <div
                key={integration.id}
                className="bg-surface-raised/80 backdrop-blur-glass rounded-xl p-6 shadow-glass border border-surface-border hover:border-primary-200 transition-colors"
              >
                <div className="flex items-start justify-between mb-4">
                  <div>
                    <Link
                      to={`/integrations/${integration.id}/edit`}
                      className="text-lg font-semibold text-white hover:text-primary-600"
                    >
                      {integration.name}
                    </Link>
                    <p className="text-sm text-gray-400 mt-1">{integration.description}</p>
                  </div>
                  <div className="flex items-center gap-2">
                    <button
                      className="p-1.5 text-gray-400 hover:text-green-400 hover:bg-green-900/30 rounded disabled:opacity-50"
                      onClick={(e) => handleRun(e, integration.id)}
                      disabled={runMutation.isPending}
                      title="Run integration"
                    >
                      {runMutation.isPending && runMutation.variables === integration.id ? (
                        <Loader2 className="w-4 h-4 animate-spin" />
                      ) : (
                        <Play className="w-4 h-4" />
                      )}
                    </button>
                    <button className="p-1.5 text-gray-400 hover:text-gray-300">
                      <MoreVertical className="w-4 h-4" />
                    </button>
                  </div>
                </div>

                <div className="flex items-center gap-4 text-sm">
                  <span
                    className={`inline-flex items-center gap-1.5 px-2 py-1 rounded-full text-xs font-medium ${styles.badge}`}
                  >
                    <span className={`w-1.5 h-1.5 rounded-full ${styles.dot}`} />
                    {integration.status}
                  </span>
                </div>

                <div className="mt-4 pt-4 border-t border-surface-border flex items-center justify-between">
                  <span className="text-xs text-gray-400">
                    Created: {new Date(integration.created_at).toLocaleDateString()}
                  </span>
                </div>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}
