import { useEffect, useState } from 'react';
import {
  Plus,
  TestTube,
  Trash2,
  CheckCircle,
  XCircle,
  Circle,
  Loader2,
  Database,
  Cloud,
  Globe,
  X,
  AlertCircle,
} from 'lucide-react';
import {
  useConnections,
  useCreateConnection,
  useDeleteConnection,
  useTestConnection,
} from '../hooks/useConnections';
import {
  CONNECTOR_TYPES,
  type ConnectorType,
  type CreateConnectionRequest,
} from '../services/connections';
import {
  ENVIRONMENT_EVENT,
  getStoredEnvironment,
  PRODUCTION_CONFIRMATION_KEY,
  PRODUCTION_REASON_KEY,
} from '../components/EnvironmentSwitcher';

// ──────────────────────────────────────────────
// Helpers
// ──────────────────────────────────────────────

const connectorIcons: Record<ConnectorType, React.ElementType> = {
  postgresql: Database,
  mysql: Database,
  snowflake: Cloud,
  oracle: Database,
  salesforce: Cloud,
  rest_api: Globe,
};

function formatRelativeTime(dateString: string): string {
  const now = Date.now();
  const date = new Date(dateString).getTime();
  const diffMs = now - date;
  const diffMin = Math.floor(diffMs / 60_000);
  if (diffMin < 1) return 'Just now';
  if (diffMin < 60) return `${diffMin}m ago`;
  const diffHr = Math.floor(diffMin / 60);
  if (diffHr < 24) return `${diffHr}h ago`;
  const diffDay = Math.floor(diffHr / 24);
  return `${diffDay}d ago`;
}

function formatFieldLabel(field: string): string {
  return field
    .replace(/_/g, ' ')
    .replace(/\b\w/g, (c) => c.toUpperCase());
}

// ──────────────────────────────────────────────
// Status badge
// ──────────────────────────────────────────────

function StatusBadge({ status }: { status: string }) {
  if (status === 'active') {
    return (
      <span className="inline-flex items-center gap-1.5 text-xs font-medium px-2.5 py-1 rounded-full bg-green-50 text-green-700">
        <CheckCircle className="w-3 h-3" />
        Active
      </span>
    );
  }
  if (status === 'error') {
    return (
      <span className="inline-flex items-center gap-1.5 text-xs font-medium px-2.5 py-1 rounded-full bg-red-50 text-red-700">
        <XCircle className="w-3 h-3" />
        Error
      </span>
    );
  }
  return (
    <span className="inline-flex items-center gap-1.5 text-xs font-medium px-2.5 py-1 rounded-full bg-gray-100 text-gray-500">
      <Circle className="w-3 h-3" />
      Untested
    </span>
  );
}

// ──────────────────────────────────────────────
// Create Connection modal
// ──────────────────────────────────────────────

interface CreateModalProps {
  open: boolean;
  onClose: () => void;
  runWithProductionGuard: (actionLabel: string, operation: () => Promise<void>) => Promise<boolean>;
}

function CreateConnectionModal({ open, onClose, runWithProductionGuard }: CreateModalProps) {
  const createMutation = useCreateConnection();

  const [selectedType, setSelectedType] = useState<ConnectorType | null>(null);
  const [name, setName] = useState('');
  const [configValues, setConfigValues] = useState<Record<string, string>>({});
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');

  function resetForm() {
    setSelectedType(null);
    setName('');
    setConfigValues({});
    setUsername('');
    setPassword('');
  }

  function handleClose() {
    resetForm();
    onClose();
  }

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!selectedType) return;

    const meta = CONNECTOR_TYPES[selectedType];

    const credentials: Record<string, unknown> = {};
    if (meta.authType === 'credential') {
      credentials.username = username;
      credentials.password = password;
    } else if (meta.authType === 'api_key') {
      credentials.api_key = password;
    } else if (meta.authType === 'oauth') {
      credentials.access_token = password;
    }

    const request: CreateConnectionRequest = {
      name,
      connector_type: selectedType,
      auth_type: meta.authType,
      config: { ...configValues },
      credentials,
    };

    const executed = await runWithProductionGuard('create a connection', async () => {
      await createMutation.mutateAsync(request);
    });
    if (executed) {
      handleClose();
    }
  }

  if (!open) return null;

  const connectorEntries = (Object.entries(CONNECTOR_TYPES) as [ConnectorType, (typeof CONNECTOR_TYPES)[ConnectorType]][]);

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-white rounded-xl shadow-lg max-w-lg w-full mx-4 max-h-[90vh] overflow-y-auto">
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-gray-100">
          <h2 className="text-lg font-semibold text-gray-900">
            {selectedType ? 'Configure Connection' : 'New Connection'}
          </h2>
          <button
            onClick={handleClose}
            className="p-1.5 text-gray-400 hover:text-gray-600 hover:bg-gray-100 rounded-lg"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Step 1 — Pick connector type */}
        {!selectedType && (
          <div className="p-6">
            <p className="text-sm text-gray-500 mb-4">Choose a connector type</p>
            <div className="grid grid-cols-2 gap-3">
              {connectorEntries.map(([type, meta]) => {
                const Icon = connectorIcons[type];
                return (
                  <button
                    key={type}
                    onClick={() => setSelectedType(type)}
                    className="flex items-center gap-3 p-4 border border-gray-200 rounded-lg hover:border-primary-300 hover:bg-primary-50 text-left transition-colors"
                  >
                    <div className="flex-shrink-0 w-10 h-10 bg-gray-100 rounded-lg flex items-center justify-center">
                      <Icon className="w-5 h-5 text-gray-600" />
                    </div>
                    <div>
                      <p className="text-sm font-medium text-gray-900">{meta.label}</p>
                      <p className="text-xs text-gray-500">{meta.authType}</p>
                    </div>
                  </button>
                );
              })}
            </div>
          </div>
        )}

        {/* Step 2 — Configuration form */}
        {selectedType && (
          <form onSubmit={handleSubmit} className="p-6 space-y-4">
            <button
              type="button"
              onClick={() => setSelectedType(null)}
              className="text-sm text-primary-600 hover:text-primary-700 mb-2"
            >
              &larr; Back to connector types
            </button>

            {/* Name */}
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Connection Name
              </label>
              <input
                type="text"
                required
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder={`My ${CONNECTOR_TYPES[selectedType].label} Connection`}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
              />
            </div>

            {/* Dynamic config fields */}
            {CONNECTOR_TYPES[selectedType].configFields.map((field) => (
              <div key={field}>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  {formatFieldLabel(field)}
                </label>
                <input
                  type="text"
                  value={configValues[field] ?? ''}
                  onChange={(e) =>
                    setConfigValues((prev) => ({ ...prev, [field]: e.target.value }))
                  }
                  placeholder={formatFieldLabel(field)}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
                />
              </div>
            ))}

            {/* Credentials */}
            {CONNECTOR_TYPES[selectedType].authType === 'credential' && (
              <>
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    Username
                  </label>
                  <input
                    type="text"
                    value={username}
                    onChange={(e) => setUsername(e.target.value)}
                    placeholder="Username"
                    className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
                  />
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    Password
                  </label>
                  <input
                    type="password"
                    value={password}
                    onChange={(e) => setPassword(e.target.value)}
                    placeholder="Password"
                    className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
                  />
                </div>
              </>
            )}

            {CONNECTOR_TYPES[selectedType].authType === 'api_key' && (
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  API Key
                </label>
                <input
                  type="password"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  placeholder="API Key"
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
                />
              </div>
            )}

            {CONNECTOR_TYPES[selectedType].authType === 'oauth' && (
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  Access Token
                </label>
                <input
                  type="password"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  placeholder="Access Token"
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
                />
              </div>
            )}

            {/* Actions */}
            <div className="flex justify-end gap-3 pt-2">
              <button
                type="button"
                onClick={handleClose}
                className="px-4 py-2 text-sm font-medium text-gray-700 bg-gray-100 rounded-lg hover:bg-gray-200"
              >
                Cancel
              </button>
              <button
                type="submit"
                disabled={createMutation.isPending || !name.trim()}
                className="flex items-center gap-2 px-4 py-2 text-sm font-medium text-white bg-primary-600 rounded-lg hover:bg-primary-700 disabled:opacity-50"
              >
                {createMutation.isPending && (
                  <Loader2 className="w-4 h-4 animate-spin" />
                )}
                {createMutation.isPending ? 'Creating...' : 'Create Connection'}
              </button>
            </div>
          </form>
        )}
      </div>
    </div>
  );
}

// ──────────────────────────────────────────────
// Delete confirmation modal
// ──────────────────────────────────────────────

interface DeleteModalProps {
  connectionName: string;
  onConfirm: () => void;
  onCancel: () => void;
  isPending: boolean;
}

function DeleteConfirmationModal({ connectionName, onConfirm, onCancel, isPending }: DeleteModalProps) {
  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-white rounded-xl shadow-lg p-6 max-w-md w-full mx-4">
        <h3 className="text-lg font-semibold text-gray-900">Delete Connection</h3>
        <p className="mt-2 text-sm text-gray-500">
          Are you sure you want to delete <span className="font-medium text-gray-700">{connectionName}</span>? This action cannot be undone.
        </p>
        <div className="mt-4 flex justify-end gap-3">
          <button
            onClick={onCancel}
            className="px-4 py-2 text-sm font-medium text-gray-700 bg-gray-100 rounded-lg hover:bg-gray-200"
          >
            Cancel
          </button>
          <button
            onClick={onConfirm}
            disabled={isPending}
            className="flex items-center gap-2 px-4 py-2 text-sm font-medium text-white bg-red-600 rounded-lg hover:bg-red-700 disabled:opacity-50"
          >
            {isPending && <Loader2 className="w-4 h-4 animate-spin" />}
            {isPending ? 'Deleting...' : 'Delete'}
          </button>
        </div>
      </div>
    </div>
  );
}

// ──────────────────────────────────────────────
// Loading skeleton
// ──────────────────────────────────────────────

function TableSkeleton() {
  return (
    <div className="bg-white rounded-xl shadow-sm border border-gray-100">
      <div className="px-6 py-4 border-b border-gray-100">
        <div className="flex gap-6">
          {[120, 80, 70, 90, 60].map((w, i) => (
            <div key={i} className="h-3 bg-gray-200 rounded animate-pulse" style={{ width: w }} />
          ))}
        </div>
      </div>
      {[1, 2, 3].map((row) => (
        <div key={row} className="px-6 py-4 border-b border-gray-50 flex items-center gap-6">
          <div className="h-4 bg-gray-100 rounded animate-pulse w-40" />
          <div className="h-4 bg-gray-100 rounded animate-pulse w-24" />
          <div className="h-6 bg-gray-100 rounded-full animate-pulse w-20" />
          <div className="h-4 bg-gray-100 rounded animate-pulse w-20" />
          <div className="flex gap-2">
            <div className="h-8 w-8 bg-gray-100 rounded animate-pulse" />
            <div className="h-8 w-8 bg-gray-100 rounded animate-pulse" />
          </div>
        </div>
      ))}
    </div>
  );
}

// ──────────────────────────────────────────────
// Main page
// ──────────────────────────────────────────────

export function ConnectionsPage() {
  const { data: connections, isLoading, error } = useConnections();
  const deleteMutation = useDeleteConnection();
  const testMutation = useTestConnection();
  const [environment, setEnvironment] = useState(getStoredEnvironment());

  const [isCreateOpen, setIsCreateOpen] = useState(false);
  const [deleteTarget, setDeleteTarget] = useState<{ id: string; name: string } | null>(null);
  const [testingId, setTestingId] = useState<string | null>(null);

  useEffect(() => {
    const onEnvironmentChanged = () => setEnvironment(getStoredEnvironment());
    window.addEventListener(ENVIRONMENT_EVENT, onEnvironmentChanged);
    return () => window.removeEventListener(ENVIRONMENT_EVENT, onEnvironmentChanged);
  }, []);

  async function runWithProductionGuard(actionLabel: string, operation: () => Promise<void>): Promise<boolean> {
    if (environment !== 'prod') {
      await operation();
      return true;
    }

    const confirmed = window.confirm(`Confirm production action: ${actionLabel}.`);
    if (!confirmed) return false;

    const reason = window.prompt('Provide a reason for this production change:')?.trim() ?? '';
    if (!reason) {
      window.alert('A change reason is required for production actions.');
      return false;
    }

    sessionStorage.setItem(PRODUCTION_CONFIRMATION_KEY, 'true');
    sessionStorage.setItem(PRODUCTION_REASON_KEY, reason);
    try {
      await operation();
      return true;
    } finally {
      sessionStorage.removeItem(PRODUCTION_CONFIRMATION_KEY);
      sessionStorage.removeItem(PRODUCTION_REASON_KEY);
    }
  }

  async function handleTest(id: string) {
    setTestingId(id);
    try {
      await runWithProductionGuard('test a connection', async () => {
        await testMutation.mutateAsync(id);
      });
    } finally {
      setTestingId(null);
    }
  }

  async function handleDelete() {
    if (!deleteTarget) return;
    const executed = await runWithProductionGuard('delete a connection', async () => {
      await deleteMutation.mutateAsync(deleteTarget.id);
    });
    if (executed) {
      setDeleteTarget(null);
    }
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-semibold text-gray-900">Connections</h1>
          <p className="mt-1 text-sm text-gray-500">
            Manage credentials for your data sources and targets
          </p>
        </div>
        <div className="flex items-center gap-3">
          <span
            className={`rounded-full px-3 py-1 text-xs font-semibold uppercase ${
              environment === 'prod'
                ? 'bg-red-100 text-red-700'
                : environment === 'staging'
                  ? 'bg-amber-100 text-amber-700'
                  : 'bg-emerald-100 text-emerald-700'
            }`}
          >
            {environment}
          </span>
        <button
          onClick={() => setIsCreateOpen(true)}
          className="flex items-center gap-2 px-4 py-2 text-sm font-medium text-white bg-primary-600 rounded-lg hover:bg-primary-700"
        >
          <Plus className="w-4 h-4" />
          New Connection
        </button>
        </div>
      </div>

      {/* Loading */}
      {isLoading && <TableSkeleton />}

      {/* Error */}
      {error && (
        <div className="flex flex-col items-center justify-center h-64 gap-3">
          <AlertCircle className="w-12 h-12 text-red-500" />
          <p className="text-gray-600">Failed to load connections</p>
          <p className="text-sm text-gray-400">{error.message}</p>
        </div>
      )}

      {/* Empty state */}
      {!isLoading && !error && connections && connections.length === 0 && (
        <div className="bg-white rounded-xl shadow-sm border border-gray-100 p-12 text-center">
          <Database className="w-12 h-12 mx-auto text-gray-400" />
          <h3 className="mt-4 text-lg font-medium text-gray-900">No connections yet</h3>
          <p className="mt-2 text-sm text-gray-500">
            Add your first connection to start integrating with external data sources.
          </p>
          <button
            onClick={() => setIsCreateOpen(true)}
            className="mt-4 inline-flex items-center gap-2 px-4 py-2 text-sm font-medium text-primary-600 bg-primary-50 rounded-lg hover:bg-primary-100"
          >
            <Plus className="w-4 h-4" />
            Add your first connection
          </button>
        </div>
      )}

      {/* Connections table */}
      {!isLoading && !error && connections && connections.length > 0 && (
        <div className="bg-white rounded-xl shadow-sm border border-gray-100 overflow-hidden">
          <table className="min-w-full divide-y divide-gray-100">
            <thead className="bg-gray-50">
              <tr>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Name
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Type
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Status
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Last Tested
                </th>
                <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Actions
                </th>
              </tr>
            </thead>
            <tbody className="bg-white divide-y divide-gray-50">
              {connections.map((conn) => {
                const meta = CONNECTOR_TYPES[conn.connector_type];
                const Icon = connectorIcons[conn.connector_type];
                const isTesting = testingId === conn.id;

                return (
                  <tr key={conn.id} className="hover:bg-gray-50">
                    <td className="px-6 py-4 whitespace-nowrap">
                      <div className="flex items-center gap-3">
                        <div className="flex-shrink-0 w-9 h-9 bg-gray-100 rounded-lg flex items-center justify-center">
                          <Icon className="w-4 h-4 text-gray-600" />
                        </div>
                        <span className="text-sm font-medium text-gray-900">{conn.name}</span>
                      </div>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap">
                      <span className="text-sm text-gray-600">{meta?.label ?? conn.connector_type}</span>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap">
                      <StatusBadge status={conn.status} />
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap">
                      <span className="text-sm text-gray-500">
                        {conn.last_tested_at ? formatRelativeTime(conn.last_tested_at) : 'Never'}
                      </span>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-right">
                      <div className="flex items-center justify-end gap-1">
                        <button
                          onClick={() => handleTest(conn.id)}
                          disabled={isTesting}
                          className="p-2 text-gray-400 hover:text-primary-600 hover:bg-primary-50 rounded-lg disabled:opacity-50"
                          title="Test connection"
                        >
                          {isTesting ? (
                            <Loader2 className="w-4 h-4 animate-spin" />
                          ) : (
                            <TestTube className="w-4 h-4" />
                          )}
                        </button>
                        <button
                          onClick={() => setDeleteTarget({ id: conn.id, name: conn.name })}
                          className="p-2 text-gray-400 hover:text-red-600 hover:bg-red-50 rounded-lg"
                          title="Delete connection"
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

      {/* Create modal */}
      <CreateConnectionModal
        open={isCreateOpen}
        onClose={() => setIsCreateOpen(false)}
        runWithProductionGuard={runWithProductionGuard}
      />

      {/* Delete confirmation modal */}
      {deleteTarget && (
        <DeleteConfirmationModal
          connectionName={deleteTarget.name}
          onConfirm={handleDelete}
          onCancel={() => setDeleteTarget(null)}
          isPending={deleteMutation.isPending}
        />
      )}
    </div>
  );
}
