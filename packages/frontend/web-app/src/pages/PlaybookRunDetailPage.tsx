import { useParams, useNavigate } from 'react-router-dom';
import {
  ArrowLeft,
  Clock,
  CheckCircle2,
  XCircle,
  Loader2,
  AlertCircle,
  Pause,
  SkipForward,
  ThumbsUp,
  ThumbsDown,
} from 'lucide-react';
import {
  usePlaybookRun,
  useApproveRun,
  useRejectRun,
} from '@/hooks/usePlaybooks';
import type { RunStatus, StepStatus } from '@/services/playbooks';

const runStatusConfig: Record<RunStatus, { label: string; icon: React.ElementType; color: string }> = {
  pending: { label: 'Pending', icon: Clock, color: 'text-gray-600 bg-gray-100' },
  running: { label: 'Running', icon: Loader2, color: 'text-blue-600 bg-blue-100' },
  waiting_approval: { label: 'Waiting Approval', icon: Pause, color: 'text-amber-600 bg-amber-100' },
  completed: { label: 'Completed', icon: CheckCircle2, color: 'text-green-600 bg-green-100' },
  failed: { label: 'Failed', icon: XCircle, color: 'text-red-600 bg-red-100' },
  cancelled: { label: 'Cancelled', icon: XCircle, color: 'text-gray-600 bg-gray-100' },
};

const stepStatusConfig: Record<StepStatus, { label: string; icon: React.ElementType; color: string }> = {
  pending: { label: 'Pending', icon: Clock, color: 'text-gray-400' },
  running: { label: 'Running', icon: Loader2, color: 'text-blue-500' },
  completed: { label: 'Completed', icon: CheckCircle2, color: 'text-green-500' },
  failed: { label: 'Failed', icon: XCircle, color: 'text-red-500' },
  skipped: { label: 'Skipped', icon: SkipForward, color: 'text-gray-400' },
};

export function PlaybookRunDetailPage() {
  const { id: playbookId, runId } = useParams<{ id: string; runId: string }>();
  const navigate = useNavigate();

  const { data: run, isLoading, error } = usePlaybookRun(runId!);
  const approveMutation = useApproveRun();
  const rejectMutation = useRejectRun();

  const handleApprove = async () => {
    if (!runId) return;
    await approveMutation.mutateAsync(runId);
  };

  const handleReject = async () => {
    if (!runId) return;
    await rejectMutation.mutateAsync(runId);
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary-600" />
      </div>
    );
  }

  if (error || !run) {
    return (
      <div className="flex flex-col items-center justify-center h-64 gap-3">
        <AlertCircle className="w-12 h-12 text-red-500" />
        <p className="text-gray-600">Failed to load run details</p>
        <button
          onClick={() => navigate('/operations/playbooks')}
          className="text-primary-600 hover:text-primary-700"
        >
          Back to playbooks
        </button>
      </div>
    );
  }

  const statusInfo = runStatusConfig[run.status];
  const StatusIcon = statusInfo.icon;
  const isWaitingApproval = run.status === 'waiting_approval';

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-4">
          <button
            onClick={() => navigate(`/operations/playbooks/${playbookId}/edit`)}
            className="flex items-center gap-1 text-gray-500 hover:text-gray-700 text-sm"
          >
            <ArrowLeft className="w-4 h-4" />
            Back to Playbook
          </button>
          <div className="h-6 w-px bg-gray-200" />
          <div>
            <h1 className="text-xl font-semibold text-gray-900">Run Details</h1>
            <p className="text-sm text-gray-500">Run ID: {run.id.slice(0, 8)}...</p>
          </div>
        </div>
        <span className={`inline-flex items-center gap-2 px-3 py-1.5 text-sm font-medium rounded-full ${statusInfo.color}`}>
          <StatusIcon className={`w-4 h-4 ${run.status === 'running' ? 'animate-spin' : ''}`} />
          {statusInfo.label}
        </span>
      </div>

      {/* Run Info Card */}
      <div className="bg-white rounded-lg border border-gray-200 p-6">
        <div className="grid grid-cols-3 gap-6">
          <div>
            <p className="text-sm text-gray-500">Started</p>
            <p className="mt-1 text-sm font-medium text-gray-900">
              {run.started_at
                ? new Date(run.started_at).toLocaleString()
                : 'Not started yet'}
            </p>
          </div>
          <div>
            <p className="text-sm text-gray-500">Completed</p>
            <p className="mt-1 text-sm font-medium text-gray-900">
              {run.completed_at
                ? new Date(run.completed_at).toLocaleString()
                : '-'}
            </p>
          </div>
          <div>
            <p className="text-sm text-gray-500">Duration</p>
            <p className="mt-1 text-sm font-medium text-gray-900">
              {run.started_at && run.completed_at
                ? `${Math.round((new Date(run.completed_at).getTime() - new Date(run.started_at).getTime()) / 1000)}s`
                : run.started_at
                ? 'In progress...'
                : '-'}
            </p>
          </div>
        </div>

        {/* Approval Actions */}
        {isWaitingApproval && (
          <div className="mt-6 p-4 bg-amber-50 rounded-lg border border-amber-100">
            <p className="text-sm font-medium text-amber-800 mb-3">This run is waiting for approval</p>
            <div className="flex items-center gap-3">
              <button
                onClick={handleApprove}
                disabled={approveMutation.isPending}
                className="flex items-center gap-1.5 px-4 py-2 text-sm font-medium text-green-700 bg-green-100 rounded-lg hover:bg-green-200 disabled:opacity-50"
              >
                <ThumbsUp className="w-4 h-4" />
                Approve
              </button>
              <button
                onClick={handleReject}
                disabled={rejectMutation.isPending}
                className="flex items-center gap-1.5 px-4 py-2 text-sm font-medium text-red-700 bg-red-100 rounded-lg hover:bg-red-200 disabled:opacity-50"
              >
                <ThumbsDown className="w-4 h-4" />
                Reject
              </button>
            </div>
          </div>
        )}

        {run.status === 'failed' && run.step_states && (
          <div className="mt-4 p-4 bg-red-50 rounded-lg border border-red-100">
            <p className="text-sm font-medium text-red-800">Error Details</p>
            <p className="mt-1 text-sm text-red-600">
              {run.step_states.find(s => s.error)?.error || 'Unknown error'}
            </p>
          </div>
        )}
      </div>

      {/* Steps Timeline */}
      <div className="bg-white rounded-lg border border-gray-200">
        <div className="px-6 py-4 border-b border-gray-200">
          <h2 className="text-lg font-medium text-gray-900">Steps</h2>
        </div>
        <div className="divide-y divide-gray-100">
          {run.step_states && run.step_states.length > 0 ? (
            run.step_states.map((stepState, index) => {
              const stepStatusInfo = stepStatusConfig[stepState.status];
              const StepIcon = stepStatusInfo.icon;

              return (
                <div key={stepState.step_id} className="px-6 py-4 flex items-center justify-between">
                  <div className="flex items-center gap-4">
                    <div className="flex items-center justify-center w-8 h-8 rounded-full bg-gray-100 text-gray-600 text-sm font-medium">
                      {index + 1}
                    </div>
                    <div className={`p-2 rounded-full bg-opacity-10 ${stepStatusInfo.color.replace('text-', 'bg-').split(' ')[0]}`}>
                      <StepIcon className={`w-5 h-5 ${stepStatusInfo.color} ${stepState.status === 'running' ? 'animate-spin' : ''}`} />
                    </div>
                    <div>
                      <p className="text-sm font-medium text-gray-900">Step {stepState.step_id}</p>
                      <p className="text-xs text-gray-500">{stepStatusInfo.label}</p>
                      {stepState.error && (
                        <p className="text-xs text-red-500 mt-1">{stepState.error}</p>
                      )}
                    </div>
                  </div>

                  <div className="flex items-center gap-4">
                    {stepState.started_at && (
                      <span className="text-xs text-gray-500">
                        Started: {new Date(stepState.started_at).toLocaleTimeString()}
                      </span>
                    )}
                    {stepState.completed_at && (
                      <span className="text-xs text-gray-500">
                        Completed: {new Date(stepState.completed_at).toLocaleTimeString()}
                      </span>
                    )}
                  </div>
                </div>
              );
            })
          ) : (
            <div className="px-6 py-8 text-center text-gray-500">
              <p>No step execution data available yet.</p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
