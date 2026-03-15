import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import {
  Plus,
  Play,
  Pencil,
  Trash2,
  Workflow,
  AlertCircle,
  Clock,
  Zap,
  Calendar,
  Webhook,
} from 'lucide-react';
import {
  usePlaybooks,
  useDeletePlaybook,
  useRunPlaybook
} from '@/hooks/usePlaybooks';
import type { TriggerType } from '@/services/playbooks';

const triggerTypeLabels: Record<TriggerType, { label: string; icon: React.ElementType; color: string }> = {
  manual: { label: 'Manual', icon: Play, color: 'text-gray-400 bg-gray-700/50' },
  scheduled: { label: 'Scheduled', icon: Clock, color: 'text-blue-400 bg-blue-900/40' },
  event: { label: 'Event', icon: Zap, color: 'text-amber-400 bg-amber-900/40' },
  webhook: { label: 'Webhook', icon: Webhook, color: 'text-purple-400 bg-purple-900/40' },
};

export function AutomationPlaybooksListPage() {
  const navigate = useNavigate();
  const [deleteId, setDeleteId] = useState<string | null>(null);

  const { data, isLoading, error } = usePlaybooks();
  const deleteMutation = useDeletePlaybook();
  const runMutation = useRunPlaybook();

  const handleDelete = async (id: string) => {
    await deleteMutation.mutateAsync(id);
    setDeleteId(null);
  };

  const handleRun = async (id: string) => {
    const run = await runMutation.mutateAsync({ id, request: { variables: {} } });
    navigate(`/operations/playbooks/${id}/runs/${run.id}`);
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary-600" />
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex flex-col items-center justify-center h-64 gap-3">
        <AlertCircle className="w-12 h-12 text-red-500" />
        <p className="text-gray-400">Failed to load playbooks</p>
      </div>
    );
  }

  const playbooks = data?.playbooks || [];

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-semibold text-white">Automation Playbooks</h1>
          <p className="mt-1 text-sm text-gray-500">
            Create and manage automated operational workflows
          </p>
        </div>
        <button
          onClick={() => navigate('/operations/playbooks/new')}
          className="flex items-center gap-2 px-4 py-2 text-sm font-medium text-white bg-primary-600 rounded-lg hover:bg-primary-700"
        >
          <Plus className="w-4 h-4" />
          Create Playbook
        </button>
      </div>

      {/* Playbooks List */}
      {playbooks.length === 0 ? (
        <div className="bg-surface-raised/80 backdrop-blur-glass rounded-lg border border-surface-border p-12 text-center">
          <Workflow className="w-12 h-12 mx-auto text-gray-400" />
          <h3 className="mt-4 text-lg font-medium text-white">No playbooks yet</h3>
          <p className="mt-2 text-sm text-gray-500">
            Create your first automation playbook to streamline operational workflows.
          </p>
          <button
            onClick={() => navigate('/operations/playbooks/new')}
            className="mt-4 inline-flex items-center gap-2 px-4 py-2 text-sm font-medium text-primary-600 bg-primary-900/30 rounded-lg hover:bg-primary-900/50"
          >
            <Plus className="w-4 h-4" />
            Create your first playbook
          </button>
        </div>
      ) : (
        <div className="bg-surface-raised/80 backdrop-blur-glass rounded-lg border border-surface-border overflow-hidden">
          <table className="min-w-full divide-y divide-surface-border">
            <thead className="bg-surface-overlay/50">
              <tr>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Name
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Trigger
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Steps
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Last Updated
                </th>
                <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Actions
                </th>
              </tr>
            </thead>
            <tbody className="divide-y divide-surface-border">
              {playbooks.map((playbook) => {
                const triggerInfo = triggerTypeLabels[playbook.trigger_type];
                const TriggerIcon = triggerInfo.icon;

                return (
                  <tr
                    key={playbook.id}
                    className="hover:bg-white/5 cursor-pointer"
                    onClick={() => navigate(`/operations/playbooks/${playbook.id}/edit`)}
                  >
                    <td className="px-6 py-4 whitespace-nowrap">
                      <div className="flex items-center gap-3">
                        <div className="flex-shrink-0 w-10 h-10 bg-primary-900/40 rounded-lg flex items-center justify-center">
                          <Workflow className="w-5 h-5 text-primary-600" />
                        </div>
                        <div>
                          <div className="text-sm font-medium text-white">{playbook.name}</div>
                          {playbook.description && (
                            <div className="text-sm text-gray-500 truncate max-w-xs">
                              {playbook.description}
                            </div>
                          )}
                        </div>
                      </div>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap">
                      <span className={`inline-flex items-center gap-1.5 px-2.5 py-1 text-xs font-medium rounded-full ${triggerInfo.color}`}>
                        <TriggerIcon className="w-3.5 h-3.5" />
                        {triggerInfo.label}
                      </span>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                      {playbook.step_count} {playbook.step_count === 1 ? 'step' : 'steps'}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                      <div className="flex items-center gap-1.5">
                        <Calendar className="w-4 h-4" />
                        {new Date(playbook.updated_at).toLocaleDateString()}
                      </div>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-right">
                      <div className="flex items-center justify-end gap-2" onClick={(e) => e.stopPropagation()}>
                        <button
                          onClick={() => handleRun(playbook.id)}
                          disabled={runMutation.isPending}
                          className="p-2 text-gray-400 hover:text-green-600 hover:bg-green-900/30 rounded-lg"
                          title="Run playbook"
                        >
                          <Play className="w-4 h-4" />
                        </button>
                        <button
                          onClick={() => navigate(`/operations/playbooks/${playbook.id}/edit`)}
                          className="p-2 text-gray-400 hover:text-primary-600 hover:bg-primary-900/30 rounded-lg"
                          title="Edit playbook"
                        >
                          <Pencil className="w-4 h-4" />
                        </button>
                        <button
                          onClick={() => setDeleteId(playbook.id)}
                          className="p-2 text-gray-400 hover:text-red-600 hover:bg-red-900/30 rounded-lg"
                          title="Delete playbook"
                        >
                          <Trash2 className="w-4 h-4" />
                        </button>
                      </div>
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      )}

      {/* Delete Confirmation Modal */}
      {deleteId && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-surface-raised border border-surface-border rounded-lg p-6 max-w-md w-full mx-4">
            <h3 className="text-lg font-semibold text-white">Delete Playbook</h3>
            <p className="mt-2 text-sm text-gray-500">
              Are you sure you want to delete this playbook? This action cannot be undone.
            </p>
            <div className="mt-4 flex justify-end gap-3">
              <button
                onClick={() => setDeleteId(null)}
                className="px-4 py-2 text-sm font-medium text-gray-300 bg-surface-overlay rounded-lg hover:bg-gray-700"
              >
                Cancel
              </button>
              <button
                onClick={() => handleDelete(deleteId)}
                disabled={deleteMutation.isPending}
                className="px-4 py-2 text-sm font-medium text-white bg-red-600 rounded-lg hover:bg-red-700 disabled:opacity-50"
              >
                {deleteMutation.isPending ? 'Deleting...' : 'Delete'}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
