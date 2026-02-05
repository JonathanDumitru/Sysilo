import { Search, Filter, Network, Server, Database, Workflow } from 'lucide-react';

export function AssetRegistryPage() {
  const assets = [
    { id: '1', name: 'Salesforce', type: 'system', category: 'CRM', integrations: 8, dependencies: 12 },
    { id: '2', name: 'Snowflake', type: 'database', category: 'Data Warehouse', integrations: 15, dependencies: 8 },
    { id: '3', name: 'PostgreSQL Prod', type: 'database', category: 'OLTP', integrations: 12, dependencies: 24 },
    { id: '4', name: 'HubSpot', type: 'system', category: 'Marketing', integrations: 4, dependencies: 6 },
    { id: '5', name: 'Customer API', type: 'api', category: 'Internal', integrations: 6, dependencies: 3 },
    { id: '6', name: 'Legacy ERP', type: 'system', category: 'Finance', integrations: 3, dependencies: 18 },
  ];

  const typeIcons: Record<string, React.ElementType> = {
    system: Server,
    database: Database,
    api: Workflow,
  };

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
            className="bg-transparent border-none outline-none text-sm flex-1"
          />
        </div>
        <button className="flex items-center gap-2 px-3 py-2 bg-white border border-gray-200 rounded-lg text-sm text-gray-600 hover:bg-gray-50">
          <Filter className="w-4 h-4" />
          Filters
        </button>
        <button className="flex items-center gap-2 px-3 py-2 bg-white border border-gray-200 rounded-lg text-sm text-gray-600 hover:bg-gray-50">
          <Network className="w-4 h-4" />
          Graph View
        </button>
      </div>

      {/* Asset grid */}
      <div className="grid grid-cols-3 gap-4">
        {assets.map((asset) => {
          const Icon = typeIcons[asset.type] || Server;
          return (
            <div
              key={asset.id}
              className="bg-white rounded-xl p-5 shadow-sm border border-gray-100 hover:border-primary-200 cursor-pointer transition-colors"
            >
              <div className="flex items-start gap-4">
                <div className="p-3 bg-gray-100 rounded-lg">
                  <Icon className="w-6 h-6 text-gray-600" />
                </div>
                <div className="flex-1">
                  <h3 className="text-lg font-semibold text-gray-900">{asset.name}</h3>
                  <p className="text-sm text-gray-500">{asset.category}</p>
                  <span className="inline-block mt-2 text-xs font-medium px-2 py-0.5 bg-gray-100 text-gray-600 rounded">
                    {asset.type}
                  </span>
                </div>
              </div>
              <div className="mt-4 pt-4 border-t border-gray-50 flex items-center justify-between text-sm">
                <span className="text-gray-500">
                  <span className="font-medium text-gray-700">{asset.integrations}</span> integrations
                </span>
                <span className="text-gray-500">
                  <span className="font-medium text-gray-700">{asset.dependencies}</span> dependencies
                </span>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
