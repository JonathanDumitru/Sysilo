import { Plus, MoreVertical, RefreshCw } from 'lucide-react';

export function AgentsPage() {
  const agents = [
    {
      id: '1',
      name: 'prod-agent-01',
      status: 'connected',
      version: '1.2.0',
      lastHeartbeat: '10 seconds ago',
      location: 'AWS us-east-1',
      runningTasks: 3,
      maxTasks: 10,
    },
    {
      id: '2',
      name: 'prod-agent-02',
      status: 'connected',
      version: '1.2.0',
      lastHeartbeat: '15 seconds ago',
      location: 'AWS us-west-2',
      runningTasks: 5,
      maxTasks: 10,
    },
    {
      id: '3',
      name: 'on-prem-agent',
      status: 'connected',
      version: '1.1.5',
      lastHeartbeat: '5 seconds ago',
      location: 'On-Premise Data Center',
      runningTasks: 2,
      maxTasks: 5,
    },
    {
      id: '4',
      name: 'dev-agent',
      status: 'disconnected',
      version: '1.2.0',
      lastHeartbeat: '2 hours ago',
      location: 'Local Development',
      runningTasks: 0,
      maxTasks: 5,
    },
  ];

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-white">Agents</h1>
          <p className="text-gray-400">Manage your on-premise and cloud agents</p>
        </div>
        <div className="flex items-center gap-3">
          <button className="flex items-center gap-2 px-3 py-2 text-gray-400 hover:bg-white/10 rounded-lg transition-colors">
            <RefreshCw className="w-4 h-4" />
            Refresh
          </button>
          <button className="flex items-center gap-2 px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors">
            <Plus className="w-4 h-4" />
            Register Agent
          </button>
        </div>
      </div>

      {/* Agent list */}
      <div className="bg-surface-raised/80 backdrop-blur-glass rounded-xl shadow-glass border border-surface-border">
        <table className="w-full">
          <thead>
            <tr className="border-b border-surface-border">
              <th className="text-left text-xs font-medium text-gray-500 uppercase tracking-wider px-6 py-4">
                Agent
              </th>
              <th className="text-left text-xs font-medium text-gray-500 uppercase tracking-wider px-6 py-4">
                Status
              </th>
              <th className="text-left text-xs font-medium text-gray-500 uppercase tracking-wider px-6 py-4">
                Version
              </th>
              <th className="text-left text-xs font-medium text-gray-500 uppercase tracking-wider px-6 py-4">
                Last Heartbeat
              </th>
              <th className="text-left text-xs font-medium text-gray-500 uppercase tracking-wider px-6 py-4">
                Tasks
              </th>
              <th className="text-left text-xs font-medium text-gray-500 uppercase tracking-wider px-6 py-4">
                Actions
              </th>
            </tr>
          </thead>
          <tbody className="divide-y divide-surface-border">
            {agents.map((agent) => (
              <tr key={agent.id} className="hover:bg-white/5">
                <td className="px-6 py-4">
                  <div>
                    <p className="text-sm font-medium text-white">{agent.name}</p>
                    <p className="text-xs text-gray-400">{agent.location}</p>
                  </div>
                </td>
                <td className="px-6 py-4">
                  <span
                    className={`inline-flex items-center gap-1.5 text-xs font-medium px-2.5 py-1 rounded-full ${
                      agent.status === 'connected'
                        ? 'bg-green-900/30 text-green-400'
                        : 'bg-gray-700/50 text-gray-400'
                    }`}
                  >
                    <span
                      className={`w-1.5 h-1.5 rounded-full ${
                        agent.status === 'connected' ? 'bg-green-500' : 'bg-gray-400'
                      }`}
                    />
                    {agent.status}
                  </span>
                </td>
                <td className="px-6 py-4">
                  <span className="text-sm text-gray-400">{agent.version}</span>
                </td>
                <td className="px-6 py-4">
                  <span className="text-sm text-gray-400">{agent.lastHeartbeat}</span>
                </td>
                <td className="px-6 py-4">
                  <div className="flex items-center gap-2">
                    <div className="w-24 h-2 bg-surface-overlay rounded-full overflow-hidden">
                      <div
                        className="h-full bg-primary-500 rounded-full"
                        style={{
                          width: `${(agent.runningTasks / agent.maxTasks) * 100}%`,
                        }}
                      />
                    </div>
                    <span className="text-xs text-gray-400">
                      {agent.runningTasks}/{agent.maxTasks}
                    </span>
                  </div>
                </td>
                <td className="px-6 py-4">
                  <button className="p-1 text-gray-400 hover:text-gray-300">
                    <MoreVertical className="w-4 h-4" />
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
