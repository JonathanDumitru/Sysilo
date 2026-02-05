import { useState, useCallback } from 'react';
import {
  X,
  Search,
  Database,
  Server,
  Globe,
  Loader2,
  CheckCircle,
  XCircle,
  FlaskConical,
  ArrowRight,
} from 'lucide-react';
import {
  useConnections,
  useRunDiscovery,
  useMockDiscovery,
  useDiscoveryRuns,
} from '../hooks/useDiscovery.js';
import type { DiscoveryRun } from '../services/discovery.js';

interface DiscoveryModalProps {
  isOpen: boolean;
  onClose: () => void;
}

const connectorIcons: Record<string, React.ElementType> = {
  postgresql: Database,
  mysql: Database,
  snowflake: Database,
  oracle: Database,
  salesforce: Globe,
  rest_api: Server,
  s3: Server,
  default: Server,
};

// Toggle for local development without Kafka
const USE_MOCK_DISCOVERY = false;

type Phase = 'select' | 'running' | 'complete';

export function DiscoveryModal({ isOpen, onClose }: DiscoveryModalProps) {
  const [selectedConnections, setSelectedConnections] = useState<string[]>([]);
  const [activeRunIds, setActiveRunIds] = useState<string[]>([]);
  const [phase, setPhase] = useState<Phase>('select');
  const [mockAssetsCreated, setMockAssetsCreated] = useState(0);

  const { data: connections, isLoading: loadingConnections } = useConnections();
  const { mutateAsync: runDiscovery } = useRunDiscovery();
  const { mutate: mockDiscovery, isPending: isMockPending, isSuccess: isMockSuccess } = useMockDiscovery();
  const { data: discoveryRuns } = useDiscoveryRuns(activeRunIds);

  if (!isOpen) return null;

  // Derive status from discovery runs
  const allTerminal = discoveryRuns?.every(
    (r) => r.status === 'completed' || r.status === 'failed'
  );
  const totalAssetsFound = discoveryRuns?.reduce((sum, r) => sum + r.assets_found, 0) ?? 0;
  const connectionsCompleted = discoveryRuns?.filter((r) => r.status === 'completed').length ?? 0;

  // Auto-advance to complete phase
  if (phase === 'running' && allTerminal && discoveryRuns && discoveryRuns.length > 0) {
    // Use setTimeout to avoid setState during render
    setTimeout(() => setPhase('complete'), 0);
  }

  const handleToggleConnection = (id: string) => {
    setSelectedConnections((prev) =>
      prev.includes(id) ? prev.filter((c) => c !== id) : [...prev, id]
    );
  };

  const handleStartDiscovery = async () => {
    if (USE_MOCK_DISCOVERY) {
      let totalCreated = 0;
      selectedConnections.forEach((connectionId) => {
        mockDiscovery(
          { connection_id: connectionId, asset_count: 5 },
          {
            onSuccess: (data) => {
              totalCreated += data.assets_created;
              setMockAssetsCreated(totalCreated);
            },
          }
        );
      });
      setPhase('complete');
    } else {
      // Dispatch all discovery runs and collect run_ids
      const runIds: string[] = [];
      for (const connectionId of selectedConnections) {
        try {
          const response = await runDiscovery({ connection_id: connectionId });
          runIds.push(response.run_id);
        } catch (err) {
          console.error('Failed to start discovery for', connectionId, err);
        }
      }
      setActiveRunIds(runIds);
      setPhase('running');
    }
  };

  const handleClose = () => {
    setSelectedConnections([]);
    setActiveRunIds([]);
    setPhase('select');
    setMockAssetsCreated(0);
    onClose();
  };

  const handleViewAssets = () => {
    handleClose();
    // Assets list will auto-refresh via query invalidation
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      <div className="absolute inset-0 bg-black/50" onClick={handleClose} />

      <div className="relative bg-white rounded-xl shadow-xl w-full max-w-lg mx-4">
        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b border-gray-100">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-primary-50 rounded-lg">
              <Search className="w-5 h-5 text-primary-600" />
            </div>
            <div>
              <h2 className="text-lg font-semibold text-gray-900">
                {phase === 'select' && 'Discover Assets'}
                {phase === 'running' && 'Discovery Running'}
                {phase === 'complete' && 'Discovery Complete'}
              </h2>
              <p className="text-sm text-gray-500">
                {phase === 'select' && 'Select connections to scan for assets'}
                {phase === 'running' && 'Scanning selected connections...'}
                {phase === 'complete' && `Found ${USE_MOCK_DISCOVERY ? mockAssetsCreated : totalAssetsFound} assets`}
              </p>
            </div>
          </div>
          <button
            onClick={handleClose}
            className="p-2 text-gray-400 hover:text-gray-600 rounded-lg hover:bg-gray-100"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Content */}
        <div className="p-4 max-h-96 overflow-y-auto">
          {/* Phase 1: Select connections */}
          {phase === 'select' && (
            <>
              {loadingConnections ? (
                <div className="flex items-center justify-center py-8">
                  <Loader2 className="w-6 h-6 text-primary-500 animate-spin" />
                </div>
              ) : (
                <div className="space-y-2">
                  {connections?.map((connection) => {
                    const Icon = connectorIcons[connection.connector_type] ?? connectorIcons.default;
                    const isSelected = selectedConnections.includes(connection.id);

                    return (
                      <button
                        key={connection.id}
                        onClick={() => handleToggleConnection(connection.id)}
                        className={`w-full flex items-center gap-3 p-3 rounded-lg border transition-colors ${
                          isSelected
                            ? 'border-primary-500 bg-primary-50'
                            : 'border-gray-200 hover:border-gray-300 hover:bg-gray-50'
                        }`}
                      >
                        <div className={`p-2 rounded-lg ${isSelected ? 'bg-primary-100' : 'bg-gray-100'}`}>
                          <Icon className={`w-5 h-5 ${isSelected ? 'text-primary-600' : 'text-gray-600'}`} />
                        </div>
                        <div className="flex-1 text-left">
                          <div className="font-medium text-gray-900">{connection.name}</div>
                          <div className="text-sm text-gray-500">{connection.connector_type}</div>
                        </div>
                        <div
                          className={`w-5 h-5 rounded-full border-2 flex items-center justify-center ${
                            isSelected ? 'border-primary-500 bg-primary-500' : 'border-gray-300'
                          }`}
                        >
                          {isSelected && <CheckCircle className="w-3 h-3 text-white" />}
                        </div>
                      </button>
                    );
                  })}
                </div>
              )}
            </>
          )}

          {/* Phase 2: Running — show per-connection status */}
          {phase === 'running' && (
            <div className="space-y-3">
              {discoveryRuns?.map((run) => (
                <DiscoveryRunStatusRow key={run.id} run={run} />
              ))}
              {(!discoveryRuns || discoveryRuns.length === 0) && (
                <div className="flex items-center justify-center py-8">
                  <Loader2 className="w-6 h-6 text-primary-500 animate-spin" />
                  <span className="ml-2 text-sm text-gray-500">Starting discovery...</span>
                </div>
              )}
            </div>
          )}

          {/* Phase 3: Complete — summary */}
          {phase === 'complete' && (
            <div className="flex flex-col items-center justify-center py-6 text-center">
              <CheckCircle className="w-12 h-12 text-green-500 mb-3" />
              {USE_MOCK_DISCOVERY ? (
                <>
                  <h3 className="text-lg font-medium text-gray-900">Mock Discovery Complete</h3>
                  <p className="text-sm text-gray-500 mt-1">
                    {mockAssetsCreated} mock assets created
                  </p>
                  <div className="mt-3 flex items-center gap-2 text-xs text-amber-600 bg-amber-50 px-3 py-1.5 rounded-full">
                    <FlaskConical className="w-3 h-3" />
                    Dev mode — using mock data
                  </div>
                </>
              ) : (
                <>
                  <h3 className="text-lg font-medium text-gray-900">
                    Discovered {totalAssetsFound} Assets
                  </h3>
                  <p className="text-sm text-gray-500 mt-1">
                    Across {connectionsCompleted} connection{connectionsCompleted !== 1 ? 's' : ''}
                  </p>
                  {discoveryRuns?.some((r) => r.status === 'failed') && (
                    <p className="text-sm text-red-500 mt-2">
                      {discoveryRuns.filter((r) => r.status === 'failed').length} connection(s) failed
                    </p>
                  )}
                </>
              )}
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="flex items-center justify-end gap-3 p-4 border-t border-gray-100">
          {phase === 'select' && (
            <>
              <button
                onClick={handleClose}
                className="px-4 py-2 text-sm font-medium text-gray-700 hover:bg-gray-100 rounded-lg"
              >
                Cancel
              </button>
              <button
                onClick={handleStartDiscovery}
                disabled={selectedConnections.length === 0}
                className="flex items-center gap-2 px-4 py-2 text-sm font-medium text-white bg-primary-600 hover:bg-primary-700 rounded-lg disabled:opacity-50 disabled:cursor-not-allowed"
              >
                <Search className="w-4 h-4" />
                Start Discovery ({selectedConnections.length})
              </button>
            </>
          )}
          {phase === 'running' && (
            <button
              onClick={handleClose}
              className="px-4 py-2 text-sm font-medium text-gray-700 hover:bg-gray-100 rounded-lg"
            >
              Close
            </button>
          )}
          {phase === 'complete' && (
            <button
              onClick={handleViewAssets}
              className="flex items-center gap-2 px-4 py-2 text-sm font-medium text-white bg-primary-600 hover:bg-primary-700 rounded-lg"
            >
              View Assets
              <ArrowRight className="w-4 h-4" />
            </button>
          )}
        </div>
      </div>
    </div>
  );
}

/** Status row for a single discovery run */
function DiscoveryRunStatusRow({ run }: { run: DiscoveryRun }) {
  const statusConfig = {
    pending: { icon: Loader2, color: 'text-gray-400', bg: 'bg-gray-50', label: 'Pending', spin: true },
    scanning: { icon: Loader2, color: 'text-primary-500', bg: 'bg-primary-50', label: 'Scanning', spin: true },
    completed: { icon: CheckCircle, color: 'text-green-500', bg: 'bg-green-50', label: 'Complete', spin: false },
    failed: { icon: XCircle, color: 'text-red-500', bg: 'bg-red-50', label: 'Failed', spin: false },
  };

  const config = statusConfig[run.status] ?? statusConfig.pending;
  const Icon = config.icon;

  return (
    <div className={`flex items-center gap-3 p-3 rounded-lg ${config.bg}`}>
      <Icon className={`w-5 h-5 ${config.color} ${config.spin ? 'animate-spin' : ''}`} />
      <div className="flex-1">
        <div className="font-medium text-gray-900 text-sm">{run.connection_name}</div>
        <div className="text-xs text-gray-500">
          {config.label}
          {run.assets_found > 0 && ` · ${run.assets_found} assets`}
          {run.error_message && ` · ${run.error_message}`}
        </div>
      </div>
    </div>
  );
}
