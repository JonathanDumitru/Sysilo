import { Database, GitBranch, Shield, AlertTriangle } from 'lucide-react';

export function DataHubPage() {
  const entities = [
    { id: '1', name: 'customers', type: 'table', source: 'PostgreSQL', records: '1.2M', lastSync: '5 min ago', quality: 98 },
    { id: '2', name: 'orders', type: 'table', source: 'PostgreSQL', records: '5.8M', lastSync: '10 min ago', quality: 95 },
    { id: '3', name: 'products', type: 'table', source: 'Salesforce', records: '45K', lastSync: '1 hour ago', quality: 100 },
    { id: '4', name: 'invoices', type: 'view', source: 'ERP', records: '890K', lastSync: '2 hours ago', quality: 87 },
  ];

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-gray-900">Data Hub</h1>
        <p className="text-gray-500">Catalog, lineage, and quality for your data assets</p>
      </div>

      {/* Stats */}
      <div className="grid grid-cols-4 gap-4">
        <div className="bg-white rounded-xl p-5 shadow-sm border border-gray-100">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-primary-50 rounded-lg">
              <Database className="w-5 h-5 text-primary-600" />
            </div>
            <div>
              <p className="text-2xl font-bold text-gray-900">156</p>
              <p className="text-sm text-gray-500">Data Entities</p>
            </div>
          </div>
        </div>
        <div className="bg-white rounded-xl p-5 shadow-sm border border-gray-100">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-emerald-50 rounded-lg">
              <GitBranch className="w-5 h-5 text-emerald-600" />
            </div>
            <div>
              <p className="text-2xl font-bold text-gray-900">342</p>
              <p className="text-sm text-gray-500">Lineage Links</p>
            </div>
          </div>
        </div>
        <div className="bg-white rounded-xl p-5 shadow-sm border border-gray-100">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-purple-50 rounded-lg">
              <Shield className="w-5 h-5 text-purple-600" />
            </div>
            <div>
              <p className="text-2xl font-bold text-gray-900">23</p>
              <p className="text-sm text-gray-500">PII Fields</p>
            </div>
          </div>
        </div>
        <div className="bg-white rounded-xl p-5 shadow-sm border border-gray-100">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-amber-50 rounded-lg">
              <AlertTriangle className="w-5 h-5 text-amber-600" />
            </div>
            <div>
              <p className="text-2xl font-bold text-gray-900">5</p>
              <p className="text-sm text-gray-500">Quality Issues</p>
            </div>
          </div>
        </div>
      </div>

      {/* Data Catalog */}
      <div className="bg-white rounded-xl shadow-sm border border-gray-100">
        <div className="p-4 border-b border-gray-100">
          <h2 className="text-lg font-semibold text-gray-900">Data Catalog</h2>
        </div>
        <table className="w-full">
          <thead>
            <tr className="border-b border-gray-100">
              <th className="text-left text-xs font-medium text-gray-500 uppercase tracking-wider px-6 py-3">Entity</th>
              <th className="text-left text-xs font-medium text-gray-500 uppercase tracking-wider px-6 py-3">Type</th>
              <th className="text-left text-xs font-medium text-gray-500 uppercase tracking-wider px-6 py-3">Source</th>
              <th className="text-left text-xs font-medium text-gray-500 uppercase tracking-wider px-6 py-3">Records</th>
              <th className="text-left text-xs font-medium text-gray-500 uppercase tracking-wider px-6 py-3">Last Sync</th>
              <th className="text-left text-xs font-medium text-gray-500 uppercase tracking-wider px-6 py-3">Quality</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-gray-50">
            {entities.map((entity) => (
              <tr key={entity.id} className="hover:bg-gray-50 cursor-pointer">
                <td className="px-6 py-4">
                  <p className="text-sm font-medium text-gray-900">{entity.name}</p>
                </td>
                <td className="px-6 py-4">
                  <span className="text-xs font-medium px-2 py-1 bg-gray-100 text-gray-600 rounded">{entity.type}</span>
                </td>
                <td className="px-6 py-4">
                  <span className="text-sm text-gray-600">{entity.source}</span>
                </td>
                <td className="px-6 py-4">
                  <span className="text-sm text-gray-600">{entity.records}</span>
                </td>
                <td className="px-6 py-4">
                  <span className="text-sm text-gray-600">{entity.lastSync}</span>
                </td>
                <td className="px-6 py-4">
                  <div className="flex items-center gap-2">
                    <div className="w-16 h-2 bg-gray-100 rounded-full overflow-hidden">
                      <div
                        className={`h-full rounded-full ${entity.quality >= 95 ? 'bg-green-500' : entity.quality >= 85 ? 'bg-amber-500' : 'bg-red-500'}`}
                        style={{ width: `${entity.quality}%` }}
                      />
                    </div>
                    <span className="text-xs text-gray-500">{entity.quality}%</span>
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
