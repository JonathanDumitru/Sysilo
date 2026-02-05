import { Link } from 'react-router-dom';
import { Plus, Play, Pause, MoreVertical, Search } from 'lucide-react';

export function IntegrationsPage() {
  const integrations = [
    {
      id: '1',
      name: 'Salesforce → Snowflake Customer Sync',
      description: 'Syncs customer data from Salesforce to Snowflake every hour',
      status: 'active',
      schedule: 'Every hour',
      lastRun: '10 minutes ago',
      lastRunStatus: 'completed',
    },
    {
      id: '2',
      name: 'HubSpot Contact Import',
      description: 'Imports new contacts from HubSpot to internal CRM',
      status: 'active',
      schedule: 'Every 15 minutes',
      lastRun: '5 minutes ago',
      lastRunStatus: 'running',
    },
    {
      id: '3',
      name: 'Daily Invoice Export',
      description: 'Exports invoices to accounting system daily at 6 AM',
      status: 'active',
      schedule: 'Daily at 6:00 AM',
      lastRun: '6 hours ago',
      lastRunStatus: 'completed',
    },
    {
      id: '4',
      name: 'Legacy ERP Sync',
      description: 'Syncs orders from legacy ERP system',
      status: 'paused',
      schedule: 'Every 30 minutes',
      lastRun: '2 days ago',
      lastRunStatus: 'failed',
    },
  ];

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">Integrations</h1>
          <p className="text-gray-500">Build and manage your data integrations</p>
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
        <div className="flex items-center gap-2 bg-white border border-gray-200 rounded-lg px-3 py-2 flex-1 max-w-md">
          <Search className="w-4 h-4 text-gray-400" />
          <input
            type="text"
            placeholder="Search integrations..."
            className="bg-transparent border-none outline-none text-sm flex-1"
          />
        </div>
        <select className="bg-white border border-gray-200 rounded-lg px-3 py-2 text-sm">
          <option>All Status</option>
          <option>Active</option>
          <option>Paused</option>
          <option>Draft</option>
        </select>
      </div>

      {/* Integration grid */}
      <div className="grid grid-cols-2 gap-4">
        {integrations.map((integration) => (
          <div
            key={integration.id}
            className="bg-white rounded-xl p-6 shadow-sm border border-gray-100 hover:border-primary-200 transition-colors"
          >
            <div className="flex items-start justify-between mb-4">
              <div>
                <Link
                  to={`/integrations/${integration.id}/edit`}
                  className="text-lg font-semibold text-gray-900 hover:text-primary-600"
                >
                  {integration.name}
                </Link>
                <p className="text-sm text-gray-500 mt-1">{integration.description}</p>
              </div>
              <div className="flex items-center gap-2">
                {integration.status === 'active' ? (
                  <button className="p-1.5 text-gray-400 hover:text-amber-600 hover:bg-amber-50 rounded">
                    <Pause className="w-4 h-4" />
                  </button>
                ) : (
                  <button className="p-1.5 text-gray-400 hover:text-green-600 hover:bg-green-50 rounded">
                    <Play className="w-4 h-4" />
                  </button>
                )}
                <button className="p-1.5 text-gray-400 hover:text-gray-600">
                  <MoreVertical className="w-4 h-4" />
                </button>
              </div>
            </div>

            <div className="flex items-center gap-4 text-sm">
              <span
                className={`inline-flex items-center gap-1.5 px-2 py-1 rounded-full text-xs font-medium ${
                  integration.status === 'active'
                    ? 'bg-green-50 text-green-700'
                    : integration.status === 'paused'
                    ? 'bg-amber-50 text-amber-700'
                    : 'bg-gray-100 text-gray-600'
                }`}
              >
                <span
                  className={`w-1.5 h-1.5 rounded-full ${
                    integration.status === 'active'
                      ? 'bg-green-500'
                      : integration.status === 'paused'
                      ? 'bg-amber-500'
                      : 'bg-gray-400'
                  }`}
                />
                {integration.status}
              </span>
              <span className="text-gray-400">|</span>
              <span className="text-gray-500">{integration.schedule}</span>
            </div>

            <div className="mt-4 pt-4 border-t border-gray-50 flex items-center justify-between">
              <span className="text-xs text-gray-400">Last run: {integration.lastRun}</span>
              <span
                className={`text-xs font-medium ${
                  integration.lastRunStatus === 'completed'
                    ? 'text-green-600'
                    : integration.lastRunStatus === 'running'
                    ? 'text-blue-600'
                    : 'text-red-600'
                }`}
              >
                {integration.lastRunStatus}
              </span>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
