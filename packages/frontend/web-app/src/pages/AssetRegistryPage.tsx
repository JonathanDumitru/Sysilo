import { useState } from 'react';
import { Search, Filter, Network, LayoutGrid, Server, Database, Workflow, Loader2, AlertCircle, Radar } from 'lucide-react';

import { useAssets, useAssetSearch, type Asset } from '../hooks/useAssets.js';
import type { AssetType } from '../services/assets.js';
import { DiscoveryModal } from '../components/DiscoveryModal.js';
import { AssetGraphView } from '../components/AssetGraphView.js';

export function AssetRegistryPage() {
  const [searchQuery, setSearchQuery] = useState('');
  const [typeFilter, setTypeFilter] = useState<AssetType | undefined>();
  const [isDiscoveryOpen, setIsDiscoveryOpen] = useState(false);
  const [viewMode, setViewMode] = useState<'grid' | 'graph'>('grid');

  const assetsQuery = useAssets({
    asset_type: typeFilter,
    limit: 50,
  });
  const searchResults = useAssetSearch(searchQuery);

  const isSearching = searchQuery.length >= 2;
  const assets = isSearching
    ? searchResults.data ?? []
    : assetsQuery.data?.assets ?? [];
  const isLoading = isSearching ? searchResults.isLoading : assetsQuery.isLoading;
  const error = isSearching ? searchResults.error : assetsQuery.error;

  const typeIcons: Record<string, React.ElementType> = {
    application: Server,
    service: Server,
    database: Database,
    api: Workflow,
    data_store: Database,
    integration: Workflow,
    infrastructure: Server,
    platform: Server,
    tool: Server,
  };

  function handleTypeFilterChange(event: React.ChangeEvent<HTMLSelectElement>): void {
    const value = event.target.value;
    setTypeFilter(value ? (value as AssetType) : undefined);
  }

  function getStatusStyles(status: string): string {
    switch (status) {
      case 'active':
        return 'bg-green-100 text-green-700';
      case 'deprecated':
        return 'bg-yellow-100 text-yellow-700';
      default:
        return 'bg-gray-100 text-gray-600';
    }
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-gray-900">Asset Registry</h1>
        <p className="text-gray-500">Inventory and relationships across your technology landscape</p>
      </div>

      {/* Search and filters */}
      <div className="flex items-center gap-4">
        <div className="flex items-center gap-2 bg-white border border-gray-200 rounded-lg px-3 py-2 flex-1 max-w-md">
          <Search className="w-4 h-4 text-gray-400" />
          <input
            type="text"
            placeholder="Search assets..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="bg-transparent border-none outline-none text-sm flex-1"
          />
        </div>
        <select
          value={typeFilter ?? ''}
          onChange={handleTypeFilterChange}
          className="px-3 py-2 bg-white border border-gray-200 rounded-lg text-sm text-gray-600"
        >
          <option value="">All Types</option>
          <option value="application">Application</option>
          <option value="service">Service</option>
          <option value="database">Database</option>
          <option value="api">API</option>
          <option value="integration">Integration</option>
        </select>
        <button className="flex items-center gap-2 px-3 py-2 bg-white border border-gray-200 rounded-lg text-sm text-gray-600 hover:bg-gray-50">
          <Filter className="w-4 h-4" />
          Filters
        </button>
        <button
          onClick={() => setViewMode(viewMode === 'grid' ? 'graph' : 'grid')}
          className={`flex items-center gap-2 px-3 py-2 rounded-lg text-sm transition-colors ${
            viewMode === 'graph'
              ? 'bg-primary-100 text-primary-700 border border-primary-200'
              : 'bg-white border border-gray-200 text-gray-600 hover:bg-gray-50'
          }`}
        >
          {viewMode === 'graph' ? (
            <>
              <LayoutGrid className="w-4 h-4" />
              Grid View
            </>
          ) : (
            <>
              <Network className="w-4 h-4" />
              Graph View
            </>
          )}
        </button>
        <button
          onClick={() => setIsDiscoveryOpen(true)}
          className="flex items-center gap-2 px-4 py-2 bg-primary-600 text-white rounded-lg text-sm font-medium hover:bg-primary-700"
        >
          <Radar className="w-4 h-4" />
          Discover Assets
        </button>
      </div>

      {/* Loading state */}
      {isLoading && (
        <div className="flex items-center justify-center py-12">
          <Loader2 className="w-8 h-8 text-primary-500 animate-spin" />
          <span className="ml-2 text-gray-500">Loading assets...</span>
        </div>
      )}

      {/* Error state */}
      {error && (
        <div className="flex items-center gap-3 p-4 bg-red-50 border border-red-200 rounded-lg">
          <AlertCircle className="w-5 h-5 text-red-500" />
          <div>
            <p className="font-medium text-red-800">Failed to load assets</p>
            <p className="text-sm text-red-600">{error.message}</p>
          </div>
        </div>
      )}

      {/* Empty state */}
      {!isLoading && !error && assets.length === 0 && (
        <div className="text-center py-12">
          <Database className="w-12 h-12 text-gray-300 mx-auto mb-4" />
          <h3 className="text-lg font-medium text-gray-900">No assets found</h3>
          <p className="text-gray-500 mt-1">
            {isSearching
              ? 'Try a different search term'
              : 'Assets will appear here once discovered by agents'}
          </p>
        </div>
      )}

      {/* Asset view - Grid or Graph */}
      {!isLoading && !error && assets.length > 0 && (
        viewMode === 'graph' ? (
          <AssetGraphView assets={assets} />
        ) : (
          <div className="grid grid-cols-3 gap-4">
            {assets.map((asset: Asset) => {
              const Icon = typeIcons[asset.asset_type.toLowerCase()] ?? Server;
              return (
                <div
                  key={asset.id}
                  className="bg-white rounded-xl p-5 shadow-sm border border-gray-100 hover:border-primary-200 cursor-pointer transition-colors"
                >
                  <div className="flex items-start gap-4">
                    <div className="p-3 bg-gray-100 rounded-lg">
                      <Icon className="w-6 h-6 text-gray-600" />
                    </div>
                    <div className="flex-1 min-w-0">
                      <h3 className="text-lg font-semibold text-gray-900 truncate">{asset.name}</h3>
                      <p className="text-sm text-gray-500 truncate">{asset.description ?? 'No description'}</p>
                      <span className="inline-block mt-2 text-xs font-medium px-2 py-0.5 bg-gray-100 text-gray-600 rounded">
                        {asset.asset_type}
                      </span>
                    </div>
                  </div>
                  <div className="mt-4 pt-4 border-t border-gray-50 flex items-center justify-between text-sm">
                    <span className="text-gray-500">
                      {asset.vendor && <span className="font-medium text-gray-700">{asset.vendor}</span>}
                      {asset.version && <span className="ml-1">v{asset.version}</span>}
                    </span>
                    <span className={`px-2 py-0.5 rounded text-xs font-medium ${getStatusStyles(asset.status)}`}>
                      {asset.status}
                    </span>
                  </div>
                  {asset.tags.length > 0 && (
                    <div className="mt-3 flex flex-wrap gap-1">
                      {asset.tags.slice(0, 3).map((tag) => (
                        <span key={tag} className="text-xs px-2 py-0.5 bg-primary-50 text-primary-700 rounded">
                          {tag}
                        </span>
                      ))}
                      {asset.tags.length > 3 && (
                        <span className="text-xs text-gray-400">+{asset.tags.length - 3}</span>
                      )}
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        )
      )}

      {/* Results count */}
      {!isLoading && !error && assets.length > 0 && (
        <div className="text-sm text-gray-500">
          Showing {assets.length} {isSearching ? 'results' : `of ${assetsQuery.data?.total ?? 0} assets`}
        </div>
      )}

      {/* Discovery Modal */}
      <DiscoveryModal isOpen={isDiscoveryOpen} onClose={() => setIsDiscoveryOpen(false)} />
    </div>
  );
}
