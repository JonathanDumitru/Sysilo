import { Plus, TestTube, MoreVertical, CheckCircle, XCircle } from 'lucide-react';

export function ConnectionsPage() {
  const connections = [
    { id: '1', name: 'Production PostgreSQL', type: 'postgresql', status: 'active', lastTested: '5 min ago', testStatus: 'success' },
    { id: '2', name: 'Snowflake Warehouse', type: 'snowflake', status: 'active', lastTested: '1 hour ago', testStatus: 'success' },
    { id: '3', name: 'Salesforce Production', type: 'salesforce', status: 'active', lastTested: '30 min ago', testStatus: 'success' },
    { id: '4', name: 'Legacy ERP', type: 'oracle', status: 'error', lastTested: '2 hours ago', testStatus: 'failure' },
  ];

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">Connections</h1>
          <p className="text-gray-500">Manage credentials for your data sources and targets</p>
        </div>
        <button className="flex items-center gap-2 px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors">
          <Plus className="w-4 h-4" />
          New Connection
        </button>
      </div>

      <div className="bg-white rounded-xl shadow-sm border border-gray-100">
        <table className="w-full">
          <thead>
            <tr className="border-b border-gray-100">
              <th className="text-left text-xs font-medium text-gray-500 uppercase tracking-wider px-6 py-4">Connection</th>
              <th className="text-left text-xs font-medium text-gray-500 uppercase tracking-wider px-6 py-4">Type</th>
              <th className="text-left text-xs font-medium text-gray-500 uppercase tracking-wider px-6 py-4">Status</th>
              <th className="text-left text-xs font-medium text-gray-500 uppercase tracking-wider px-6 py-4">Last Tested</th>
              <th className="text-left text-xs font-medium text-gray-500 uppercase tracking-wider px-6 py-4">Actions</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-gray-50">
            {connections.map((conn) => (
              <tr key={conn.id} className="hover:bg-gray-50">
                <td className="px-6 py-4">
                  <p className="text-sm font-medium text-gray-900">{conn.name}</p>
                </td>
                <td className="px-6 py-4">
                  <span className="text-sm text-gray-600">{conn.type}</span>
                </td>
                <td className="px-6 py-4">
                  <span className={`inline-flex items-center gap-1.5 text-xs font-medium px-2.5 py-1 rounded-full ${
                    conn.status === 'active' ? 'bg-green-50 text-green-700' : 'bg-red-50 text-red-700'
                  }`}>
                    {conn.status === 'active' ? <CheckCircle className="w-3 h-3" /> : <XCircle className="w-3 h-3" />}
                    {conn.status}
                  </span>
                </td>
                <td className="px-6 py-4">
                  <span className="text-sm text-gray-600">{conn.lastTested}</span>
                </td>
                <td className="px-6 py-4">
                  <div className="flex items-center gap-2">
                    <button className="p-1.5 text-gray-400 hover:text-primary-600 hover:bg-primary-50 rounded">
                      <TestTube className="w-4 h-4" />
                    </button>
                    <button className="p-1.5 text-gray-400 hover:text-gray-600">
                      <MoreVertical className="w-4 h-4" />
                    </button>
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
