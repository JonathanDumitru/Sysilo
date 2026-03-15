import { useState } from 'react';
import { Link } from 'react-router-dom';
import {
  Shield,
  FileCheck,
  ClipboardCheck,
  AlertTriangle,
  CheckCircle,
  BookOpen,
  Scale,
  XCircle,
  Loader2,
} from 'lucide-react';
import {
  usePolicies,
  useViolations,
  useApprovalRequests,
  useAuditStats,
  useComplianceSummary,
  useRunAssessment,
} from '../hooks/useGovernance';
import type { AssessmentResult, ApprovalRequestWithWorkflow, PolicyViolation } from '../services/governance';

function formatRelativeTime(timestamp: string): string {
  const now = Date.now();
  const then = new Date(timestamp).getTime();
  const diffMs = now - then;
  const diffSec = Math.floor(diffMs / 1000);
  const diffMin = Math.floor(diffSec / 60);
  const diffHour = Math.floor(diffMin / 60);
  const diffDay = Math.floor(diffHour / 24);

  if (diffDay > 0) return `${diffDay} day${diffDay > 1 ? 's' : ''} ago`;
  if (diffHour > 0) return `${diffHour} hour${diffHour > 1 ? 's' : ''} ago`;
  if (diffMin > 0) return `${diffMin} min${diffMin > 1 ? 's' : ''} ago`;
  return 'just now';
}

export function GovernanceDashboardPage() {
  const [timeRange, setTimeRange] = useState('30d');

  // Real data hooks
  const { data: complianceResults, isLoading: complianceLoading, error: complianceError } = useComplianceSummary();
  const { data: policies, isLoading: policiesLoading } = usePolicies();
  const { data: openViolations, isLoading: violationsLoading } = useViolations('open');
  const { data: recentViolations, isLoading: recentViolationsLoading } = useViolations(undefined, 5);
  const { data: allApprovals, isLoading: approvalsLoading } = useApprovalRequests();
  const { data: pendingApprovals, isLoading: pendingApprovalsLoading } = useApprovalRequests('pending');
  const { data: auditStats, isLoading: auditLoading } = useAuditStats();
  const runAssessment = useRunAssessment();

  // Derived stats
  const overallScore = complianceResults && complianceResults.length > 0
    ? complianceResults.reduce((sum: number, r: AssessmentResult) => sum + r.compliance_score, 0) / complianceResults.length
    : 0;

  const policyTotal = policies?.length ?? 0;
  const policyActive = policies?.filter((p) => p.enabled).length ?? 0;
  const openViolationCount = openViolations?.length ?? 0;

  const approvedCount = allApprovals?.filter((a: ApprovalRequestWithWorkflow) => a.request.status === 'approved').length ?? 0;
  const pendingCount = allApprovals?.filter((a: ApprovalRequestWithWorkflow) => a.request.status === 'pending').length ?? 0;

  const getComplianceColor = (score: number) => {
    if (score >= 90) return 'text-green-600 bg-green-900/30';
    if (score >= 70) return 'text-yellow-600 bg-yellow-900/30';
    return 'text-red-600 bg-red-900/30';
  };

  const getStatusBadge = (score: number) => {
    if (score >= 90) return 'bg-green-900/40 text-green-400';
    if (score >= 70) return 'bg-yellow-900/40 text-yellow-400';
    return 'bg-red-900/40 text-red-400';
  };

  const getStatusLabel = (score: number) => {
    if (score >= 90) return 'compliant';
    if (score >= 70) return 'partial';
    return 'non_compliant';
  };

  const getSeverityColor = (severity: string) => {
    switch (severity) {
      case 'critical':
        return 'bg-red-900/40 text-red-400';
      case 'high':
        return 'bg-orange-900/40 text-orange-400';
      case 'medium':
        return 'bg-yellow-900/40 text-yellow-400';
      case 'low':
        return 'bg-blue-900/40 text-blue-400';
      default:
        return 'bg-gray-700/50 text-gray-300';
    }
  };

  const isLoading = complianceLoading || policiesLoading || violationsLoading || approvalsLoading || auditLoading;

  if (complianceError) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-center">
          <AlertTriangle className="w-12 h-12 text-red-400 mx-auto mb-3" />
          <p className="text-white font-medium">Failed to load governance data</p>
          <p className="text-sm text-gray-500 mt-1">{(complianceError as Error).message}</p>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-white">Governance Center</h1>
          <p className="text-gray-500">Policy enforcement, compliance, and approval management</p>
        </div>
        <div className="flex items-center gap-3">
          <select
            value={timeRange}
            onChange={(e) => setTimeRange(e.target.value)}
            className="px-3 py-2 bg-surface-base/50 border border-surface-border rounded-lg text-sm text-gray-200 placeholder-gray-500 focus:border-primary-500/50 focus:ring-1 focus:ring-primary-500/20 outline-none"
          >
            <option value="7d">Last 7 days</option>
            <option value="30d">Last 30 days</option>
            <option value="90d">Last 90 days</option>
          </select>
          <button
            className="px-4 py-2 bg-primary-600 text-white rounded-lg text-sm font-medium hover:bg-primary-700 disabled:opacity-50 flex items-center gap-2"
            disabled={runAssessment.isPending || !complianceResults?.length}
            onClick={() => {
              if (complianceResults) {
                complianceResults.forEach((r: AssessmentResult) => runAssessment.mutate(r.framework_id));
              }
            }}
          >
            {runAssessment.isPending && <Loader2 className="w-4 h-4 animate-spin" />}
            Run Assessment
          </button>
        </div>
      </div>

      {/* Key Metrics */}
      <div className="grid grid-cols-4 gap-6">
        <div className="bg-surface-raised/80 backdrop-blur-glass rounded-xl p-6 shadow-glass border border-surface-border">
          <div className="flex items-center justify-between">
            <div className="p-2 bg-green-900/30 rounded-lg">
              <Shield className="w-5 h-5 text-green-600" />
            </div>
          </div>
          <div className="mt-4">
            {complianceLoading ? (
              <Loader2 className="w-6 h-6 animate-spin text-gray-400" />
            ) : (
              <p className="text-3xl font-bold text-white">{overallScore.toFixed(1)}%</p>
            )}
            <p className="text-sm font-medium text-gray-500">Overall Compliance</p>
          </div>
        </div>

        <div className="bg-surface-raised/80 backdrop-blur-glass rounded-xl p-6 shadow-glass border border-surface-border">
          <div className="flex items-center justify-between">
            <div className="p-2 bg-blue-900/30 rounded-lg">
              <FileCheck className="w-5 h-5 text-blue-600" />
            </div>
            <span className="text-xs font-medium text-gray-500">{policyActive} active</span>
          </div>
          <div className="mt-4">
            {policiesLoading ? (
              <Loader2 className="w-6 h-6 animate-spin text-gray-400" />
            ) : (
              <p className="text-3xl font-bold text-white">{policyTotal}</p>
            )}
            <p className="text-sm font-medium text-gray-500">Policies</p>
          </div>
        </div>

        <div className="bg-surface-raised/80 backdrop-blur-glass rounded-xl p-6 shadow-glass border border-surface-border">
          <div className="flex items-center justify-between">
            <div className="p-2 bg-yellow-900/30 rounded-lg">
              <ClipboardCheck className="w-5 h-5 text-yellow-600" />
            </div>
            <span className="text-xs font-medium text-yellow-600">{pendingCount} pending</span>
          </div>
          <div className="mt-4">
            {approvalsLoading ? (
              <Loader2 className="w-6 h-6 animate-spin text-gray-400" />
            ) : (
              <p className="text-3xl font-bold text-white">{approvedCount}</p>
            )}
            <p className="text-sm font-medium text-gray-500">Approvals (30d)</p>
          </div>
        </div>

        <div className="bg-surface-raised/80 backdrop-blur-glass rounded-xl p-6 shadow-glass border border-surface-border">
          <div className="flex items-center justify-between">
            <div className="p-2 bg-red-900/30 rounded-lg">
              <AlertTriangle className="w-5 h-5 text-red-600" />
            </div>
          </div>
          <div className="mt-4">
            {violationsLoading ? (
              <Loader2 className="w-6 h-6 animate-spin text-gray-400" />
            ) : (
              <p className="text-3xl font-bold text-white">{openViolationCount}</p>
            )}
            <p className="text-sm font-medium text-gray-500">Open Violations</p>
          </div>
        </div>
      </div>

      {/* Compliance Frameworks */}
      <div className="bg-surface-raised/80 backdrop-blur-glass rounded-xl p-6 shadow-glass border border-surface-border">
        <div className="flex items-center justify-between mb-6">
          <h2 className="text-lg font-semibold text-white">Compliance Frameworks</h2>
          <a href="/governance/compliance" className="text-sm text-primary-600 hover:text-primary-700">
            View details
          </a>
        </div>
        {complianceLoading ? (
          <div className="flex items-center justify-center py-8">
            <Loader2 className="w-6 h-6 animate-spin text-gray-400" />
          </div>
        ) : complianceResults && complianceResults.length > 0 ? (
          <div className="grid grid-cols-4 gap-4">
            {complianceResults.map((framework: AssessmentResult) => (
              <div
                key={framework.framework_id}
                className="p-4 border border-surface-border rounded-lg hover:border-surface-border transition-colors"
              >
                <div className="flex items-center justify-between mb-3">
                  <h3 className="font-medium text-white">{framework.framework_name}</h3>
                  <span
                    className={`text-xs font-medium px-2 py-0.5 rounded-full capitalize ${getStatusBadge(
                      framework.compliance_score
                    )}`}
                  >
                    {getStatusLabel(framework.compliance_score)}
                  </span>
                </div>
                <div className="flex items-end justify-between">
                  <div>
                    <p className={`text-2xl font-bold ${getComplianceColor(framework.compliance_score).split(' ')[0]}`}>
                      {framework.compliance_score.toFixed(0)}%
                    </p>
                    <p className="text-xs text-gray-500 mt-1">
                      {framework.compliant}/{framework.total_controls} controls
                    </p>
                  </div>
                  <div className="flex gap-1">
                    <div className="w-2 bg-green-800 rounded" style={{ height: `${framework.compliant / framework.total_controls * 32}px` }} />
                    <div className="w-2 bg-yellow-800 rounded" style={{ height: `${framework.partial / framework.total_controls * 32}px` }} />
                    <div className="w-2 bg-red-800 rounded" style={{ height: `${framework.non_compliant / framework.total_controls * 32}px` }} />
                  </div>
                </div>
              </div>
            ))}
          </div>
        ) : (
          <p className="text-sm text-gray-500 text-center py-8">No compliance frameworks configured.</p>
        )}
      </div>

      {/* Pending Approvals & Recent Violations */}
      <div className="grid grid-cols-2 gap-6">
        {/* Pending Approvals */}
        <div className="bg-surface-raised/80 backdrop-blur-glass rounded-xl p-6 shadow-glass border border-surface-border">
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-lg font-semibold text-white">Pending Approvals</h2>
            <a href="/governance/approvals" className="text-sm text-primary-600 hover:text-primary-700">
              View all
            </a>
          </div>
          {pendingApprovalsLoading ? (
            <div className="flex items-center justify-center py-8">
              <Loader2 className="w-6 h-6 animate-spin text-gray-400" />
            </div>
          ) : pendingApprovals && pendingApprovals.length > 0 ? (
            <div className="space-y-3">
              {pendingApprovals.map((approval: ApprovalRequestWithWorkflow) => (
                <div
                  key={approval.request.id}
                  className="p-3 bg-surface-overlay/50 rounded-lg border border-surface-border"
                >
                  <div className="flex items-start justify-between">
                    <div>
                      <div className="flex items-center gap-2 mb-1">
                        <span className="text-xs font-mono text-gray-500">{approval.request.id.slice(0, 8)}</span>
                        <span className="text-xs px-1.5 py-0.5 bg-blue-900/40 text-blue-400 rounded">
                          {approval.request.resource_type}
                        </span>
                      </div>
                      <p className="text-sm font-medium text-white">{approval.workflow_name}</p>
                      <p className="text-xs text-gray-500 mt-1">
                        {approval.request.requester_id} · {formatRelativeTime(approval.request.created_at)}
                      </p>
                    </div>
                    <span className="text-xs px-2 py-1 bg-yellow-900/40 text-yellow-400 rounded-full">
                      {approval.current_stage_name}
                    </span>
                  </div>
                </div>
              ))}
            </div>
          ) : (
            <p className="text-sm text-gray-500 text-center py-8">No pending approvals.</p>
          )}
        </div>

        {/* Recent Violations */}
        <div className="bg-surface-raised/80 backdrop-blur-glass rounded-xl p-6 shadow-glass border border-surface-border">
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-lg font-semibold text-white">Recent Violations</h2>
            <a href="/governance/policies" className="text-sm text-primary-600 hover:text-primary-700">
              View all
            </a>
          </div>
          {recentViolationsLoading ? (
            <div className="flex items-center justify-center py-8">
              <Loader2 className="w-6 h-6 animate-spin text-gray-400" />
            </div>
          ) : recentViolations && recentViolations.length > 0 ? (
            <div className="space-y-3">
              {recentViolations.map((violation: PolicyViolation) => (
                <div
                  key={violation.id}
                  className="flex items-center justify-between p-3 bg-surface-overlay/50 rounded-lg border border-surface-border"
                >
                  <div className="flex items-center gap-3">
                    {violation.status === 'open' ? (
                      <XCircle className="w-5 h-5 text-red-500" />
                    ) : (
                      <CheckCircle className="w-5 h-5 text-green-500" />
                    )}
                    <div>
                      <p className="text-sm font-medium text-white">{violation.policy_id}</p>
                      <p className="text-xs text-gray-500">{violation.resource_type}: {violation.resource_id}</p>
                    </div>
                  </div>
                  <div className="flex items-center gap-2">
                    <span
                      className={`text-xs font-medium px-2 py-0.5 rounded-full ${getSeverityColor(
                        (violation.details?.severity as string) ?? 'medium'
                      )}`}
                    >
                      {(violation.details?.severity as string) ?? 'medium'}
                    </span>
                    <span className="text-xs text-gray-400">{formatRelativeTime(violation.created_at)}</span>
                  </div>
                </div>
              ))}
            </div>
          ) : (
            <p className="text-sm text-gray-500 text-center py-8">No recent violations.</p>
          )}
        </div>
      </div>

      {/* Audit Activity & Standards */}
      <div className="grid grid-cols-3 gap-6">
        {/* Audit Activity */}
        <div className="col-span-2 bg-surface-raised/80 backdrop-blur-glass rounded-xl p-6 shadow-glass border border-surface-border">
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-lg font-semibold text-white">Audit Activity</h2>
            <a href="/governance/audit" className="text-sm text-primary-600 hover:text-primary-700">
              View audit log
            </a>
          </div>
          {auditLoading ? (
            <div className="flex items-center justify-center py-8">
              <Loader2 className="w-6 h-6 animate-spin text-gray-400" />
            </div>
          ) : auditStats ? (
            <div className="grid grid-cols-4 gap-4">
              <div className="p-4 bg-surface-overlay/50 rounded-lg">
                <p className="text-2xl font-bold text-white">
                  {auditStats.total_entries.toLocaleString()}
                </p>
                <p className="text-xs text-gray-500">Total Entries</p>
              </div>
              <div className="p-4 bg-surface-overlay/50 rounded-lg">
                <p className="text-2xl font-bold text-white">{auditStats.entries_today}</p>
                <p className="text-xs text-gray-500">Today</p>
              </div>
              <div className="p-4 bg-surface-overlay/50 rounded-lg">
                <p className="text-2xl font-bold text-white">{auditStats.unique_actors}</p>
                <p className="text-xs text-gray-500">Unique Actors</p>
              </div>
              <div className="p-4 bg-surface-overlay/50 rounded-lg">
                <div className="flex flex-wrap gap-1">
                  {auditStats.top_actions.map((item) => (
                    <span
                      key={item.action}
                      className="text-xs px-1.5 py-0.5 bg-gray-700 text-gray-300 rounded"
                    >
                      {item.action}
                    </span>
                  ))}
                </div>
                <p className="text-xs text-gray-500 mt-2">Top Actions</p>
              </div>
            </div>
          ) : (
            <p className="text-sm text-gray-500 text-center py-8">No audit data available.</p>
          )}
        </div>

        {/* Quick Links */}
        <div className="bg-surface-raised/80 backdrop-blur-glass rounded-xl p-6 shadow-glass border border-surface-border">
          <h2 className="text-lg font-semibold text-white mb-4">Quick Actions</h2>
          <div className="space-y-2">
            <Link
              to="/governance/policies"
              className="flex items-center gap-3 p-3 rounded-lg hover:bg-white/5 transition-colors"
            >
              <Scale className="w-5 h-5 text-gray-400" />
              <span className="text-sm text-gray-300">Manage Policies</span>
            </Link>
            <Link
              to="/governance/standards"
              className="flex items-center gap-3 p-3 rounded-lg hover:bg-white/5 transition-colors"
            >
              <BookOpen className="w-5 h-5 text-gray-400" />
              <span className="text-sm text-gray-300">View Standards</span>
            </Link>
            <Link
              to="/governance/approvals"
              className="flex items-center gap-3 p-3 rounded-lg hover:bg-white/5 transition-colors"
            >
              <ClipboardCheck className="w-5 h-5 text-gray-400" />
              <span className="text-sm text-gray-300">Approval Workflows</span>
            </Link>
            <Link
              to="/governance/audit"
              className="flex items-center gap-3 p-3 rounded-lg hover:bg-white/5 transition-colors"
            >
              <FileCheck className="w-5 h-5 text-gray-400" />
              <span className="text-sm text-gray-300">Audit Log</span>
            </Link>
          </div>
        </div>
      </div>
    </div>
  );
}
