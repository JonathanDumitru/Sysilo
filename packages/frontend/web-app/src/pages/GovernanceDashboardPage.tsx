import { useState } from 'react';
import {
  Shield,
  FileCheck,
  ClipboardCheck,
  AlertTriangle,
  CheckCircle,
  BookOpen,
  Scale,
  XCircle,
} from 'lucide-react';

// Mock data
const complianceOverview = {
  overallScore: 87.5,
  frameworks: [
    { name: 'SOC 2 Type II', score: 92, status: 'compliant', controls: { total: 64, compliant: 59, nonCompliant: 3, partial: 2 } },
    { name: 'GDPR', score: 85, status: 'partial', controls: { total: 42, compliant: 34, nonCompliant: 4, partial: 4 } },
    { name: 'ISO 27001', score: 78, status: 'partial', controls: { total: 114, compliant: 87, nonCompliant: 15, partial: 12 } },
    { name: 'HIPAA', score: 95, status: 'compliant', controls: { total: 45, compliant: 43, nonCompliant: 1, partial: 1 } },
  ],
};

const policyStats = {
  total: 24,
  active: 22,
  violations: {
    open: 8,
    resolved: 156,
    thisWeek: 3,
  },
};

const approvalStats = {
  pending: 5,
  approved: 42,
  rejected: 3,
  avgTime: '4.2 hours',
};

const pendingApprovals = [
  {
    id: 'REQ-001',
    type: 'Integration',
    name: 'Salesforce Production Connection',
    requester: 'John Doe',
    created: '2 hours ago',
    stage: 'Security Review',
  },
  {
    id: 'REQ-002',
    type: 'Agent',
    name: 'New Production Agent',
    requester: 'Jane Smith',
    created: '5 hours ago',
    stage: 'IT Approval',
  },
  {
    id: 'REQ-003',
    type: 'Data Export',
    name: 'Customer Data Export',
    requester: 'Bob Wilson',
    created: '1 day ago',
    stage: 'Compliance Review',
  },
];

const recentViolations = [
  {
    id: '1',
    policy: 'Naming Convention',
    resource: 'integration-test-123',
    severity: 'low',
    status: 'open',
    created: '1 hour ago',
  },
  {
    id: '2',
    policy: 'Security Policy',
    resource: 'api-connection-xyz',
    severity: 'high',
    status: 'open',
    created: '3 hours ago',
  },
  {
    id: '3',
    policy: 'Data Retention',
    resource: 'data-export-job',
    severity: 'medium',
    status: 'resolved',
    created: '1 day ago',
  },
];

const auditStats = {
  totalEntries: 15420,
  todayEntries: 234,
  uniqueActors: 45,
  topActions: ['create', 'update', 'delete', 'approve'],
};

export function GovernanceDashboardPage() {
  const [timeRange, setTimeRange] = useState('30d');

  const getComplianceColor = (score: number) => {
    if (score >= 90) return 'text-green-600 bg-green-50';
    if (score >= 70) return 'text-yellow-600 bg-yellow-50';
    return 'text-red-600 bg-red-50';
  };

  const getStatusBadge = (status: string) => {
    switch (status) {
      case 'compliant':
        return 'bg-green-100 text-green-700';
      case 'partial':
        return 'bg-yellow-100 text-yellow-700';
      case 'non_compliant':
        return 'bg-red-100 text-red-700';
      default:
        return 'bg-gray-100 text-gray-700';
    }
  };

  const getSeverityColor = (severity: string) => {
    switch (severity) {
      case 'critical':
        return 'bg-red-100 text-red-700';
      case 'high':
        return 'bg-orange-100 text-orange-700';
      case 'medium':
        return 'bg-yellow-100 text-yellow-700';
      case 'low':
        return 'bg-blue-100 text-blue-700';
      default:
        return 'bg-gray-100 text-gray-700';
    }
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">Governance Center</h1>
          <p className="text-gray-500">Policy enforcement, compliance, and approval management</p>
        </div>
        <div className="flex items-center gap-3">
          <select
            value={timeRange}
            onChange={(e) => setTimeRange(e.target.value)}
            className="px-3 py-2 border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500"
          >
            <option value="7d">Last 7 days</option>
            <option value="30d">Last 30 days</option>
            <option value="90d">Last 90 days</option>
          </select>
          <button className="px-4 py-2 bg-primary-600 text-white rounded-lg text-sm font-medium hover:bg-primary-700">
            Run Assessment
          </button>
        </div>
      </div>

      {/* Key Metrics */}
      <div className="grid grid-cols-4 gap-6">
        <div className="bg-white rounded-xl p-6 shadow-sm border border-gray-100">
          <div className="flex items-center justify-between">
            <div className="p-2 bg-green-50 rounded-lg">
              <Shield className="w-5 h-5 text-green-600" />
            </div>
            <span className="text-xs font-medium text-green-600">+2.3%</span>
          </div>
          <div className="mt-4">
            <p className="text-3xl font-bold text-gray-900">{complianceOverview.overallScore}%</p>
            <p className="text-sm font-medium text-gray-500">Overall Compliance</p>
          </div>
        </div>

        <div className="bg-white rounded-xl p-6 shadow-sm border border-gray-100">
          <div className="flex items-center justify-between">
            <div className="p-2 bg-blue-50 rounded-lg">
              <FileCheck className="w-5 h-5 text-blue-600" />
            </div>
            <span className="text-xs font-medium text-gray-500">{policyStats.active} active</span>
          </div>
          <div className="mt-4">
            <p className="text-3xl font-bold text-gray-900">{policyStats.total}</p>
            <p className="text-sm font-medium text-gray-500">Policies</p>
          </div>
        </div>

        <div className="bg-white rounded-xl p-6 shadow-sm border border-gray-100">
          <div className="flex items-center justify-between">
            <div className="p-2 bg-yellow-50 rounded-lg">
              <ClipboardCheck className="w-5 h-5 text-yellow-600" />
            </div>
            <span className="text-xs font-medium text-yellow-600">{approvalStats.pending} pending</span>
          </div>
          <div className="mt-4">
            <p className="text-3xl font-bold text-gray-900">{approvalStats.approved}</p>
            <p className="text-sm font-medium text-gray-500">Approvals (30d)</p>
          </div>
        </div>

        <div className="bg-white rounded-xl p-6 shadow-sm border border-gray-100">
          <div className="flex items-center justify-between">
            <div className="p-2 bg-red-50 rounded-lg">
              <AlertTriangle className="w-5 h-5 text-red-600" />
            </div>
            <span className="text-xs font-medium text-red-600">+{policyStats.violations.thisWeek} this week</span>
          </div>
          <div className="mt-4">
            <p className="text-3xl font-bold text-gray-900">{policyStats.violations.open}</p>
            <p className="text-sm font-medium text-gray-500">Open Violations</p>
          </div>
        </div>
      </div>

      {/* Compliance Frameworks */}
      <div className="bg-white rounded-xl p-6 shadow-sm border border-gray-100">
        <div className="flex items-center justify-between mb-6">
          <h2 className="text-lg font-semibold text-gray-900">Compliance Frameworks</h2>
          <a href="/governance/compliance" className="text-sm text-primary-600 hover:text-primary-700">
            View details
          </a>
        </div>
        <div className="grid grid-cols-4 gap-4">
          {complianceOverview.frameworks.map((framework) => (
            <div
              key={framework.name}
              className="p-4 border border-gray-100 rounded-lg hover:border-gray-200 transition-colors"
            >
              <div className="flex items-center justify-between mb-3">
                <h3 className="font-medium text-gray-900">{framework.name}</h3>
                <span
                  className={`text-xs font-medium px-2 py-0.5 rounded-full capitalize ${getStatusBadge(
                    framework.status
                  )}`}
                >
                  {framework.status}
                </span>
              </div>
              <div className="flex items-end justify-between">
                <div>
                  <p className={`text-2xl font-bold ${getComplianceColor(framework.score).split(' ')[0]}`}>
                    {framework.score}%
                  </p>
                  <p className="text-xs text-gray-500 mt-1">
                    {framework.controls.compliant}/{framework.controls.total} controls
                  </p>
                </div>
                <div className="flex gap-1">
                  <div className="w-2 h-8 bg-green-200 rounded" style={{ height: `${framework.controls.compliant / framework.controls.total * 32}px` }} />
                  <div className="w-2 h-8 bg-yellow-200 rounded" style={{ height: `${framework.controls.partial / framework.controls.total * 32}px` }} />
                  <div className="w-2 h-8 bg-red-200 rounded" style={{ height: `${framework.controls.nonCompliant / framework.controls.total * 32}px` }} />
                </div>
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* Pending Approvals & Recent Violations */}
      <div className="grid grid-cols-2 gap-6">
        {/* Pending Approvals */}
        <div className="bg-white rounded-xl p-6 shadow-sm border border-gray-100">
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-lg font-semibold text-gray-900">Pending Approvals</h2>
            <a href="/governance/approvals" className="text-sm text-primary-600 hover:text-primary-700">
              View all
            </a>
          </div>
          <div className="space-y-3">
            {pendingApprovals.map((approval) => (
              <div
                key={approval.id}
                className="p-3 bg-gray-50 rounded-lg border border-gray-100"
              >
                <div className="flex items-start justify-between">
                  <div>
                    <div className="flex items-center gap-2 mb-1">
                      <span className="text-xs font-mono text-gray-500">{approval.id}</span>
                      <span className="text-xs px-1.5 py-0.5 bg-blue-100 text-blue-700 rounded">
                        {approval.type}
                      </span>
                    </div>
                    <p className="text-sm font-medium text-gray-900">{approval.name}</p>
                    <p className="text-xs text-gray-500 mt-1">
                      {approval.requester} · {approval.created}
                    </p>
                  </div>
                  <span className="text-xs px-2 py-1 bg-yellow-100 text-yellow-700 rounded-full">
                    {approval.stage}
                  </span>
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* Recent Violations */}
        <div className="bg-white rounded-xl p-6 shadow-sm border border-gray-100">
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-lg font-semibold text-gray-900">Recent Violations</h2>
            <a href="/governance/policies" className="text-sm text-primary-600 hover:text-primary-700">
              View all
            </a>
          </div>
          <div className="space-y-3">
            {recentViolations.map((violation) => (
              <div
                key={violation.id}
                className="flex items-center justify-between p-3 bg-gray-50 rounded-lg border border-gray-100"
              >
                <div className="flex items-center gap-3">
                  {violation.status === 'open' ? (
                    <XCircle className="w-5 h-5 text-red-500" />
                  ) : (
                    <CheckCircle className="w-5 h-5 text-green-500" />
                  )}
                  <div>
                    <p className="text-sm font-medium text-gray-900">{violation.policy}</p>
                    <p className="text-xs text-gray-500">{violation.resource}</p>
                  </div>
                </div>
                <div className="flex items-center gap-2">
                  <span
                    className={`text-xs font-medium px-2 py-0.5 rounded-full ${getSeverityColor(
                      violation.severity
                    )}`}
                  >
                    {violation.severity}
                  </span>
                  <span className="text-xs text-gray-400">{violation.created}</span>
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>

      {/* Audit Activity & Standards */}
      <div className="grid grid-cols-3 gap-6">
        {/* Audit Activity */}
        <div className="col-span-2 bg-white rounded-xl p-6 shadow-sm border border-gray-100">
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-lg font-semibold text-gray-900">Audit Activity</h2>
            <a href="/governance/audit" className="text-sm text-primary-600 hover:text-primary-700">
              View audit log
            </a>
          </div>
          <div className="grid grid-cols-4 gap-4">
            <div className="p-4 bg-gray-50 rounded-lg">
              <p className="text-2xl font-bold text-gray-900">
                {auditStats.totalEntries.toLocaleString()}
              </p>
              <p className="text-xs text-gray-500">Total Entries</p>
            </div>
            <div className="p-4 bg-gray-50 rounded-lg">
              <p className="text-2xl font-bold text-gray-900">{auditStats.todayEntries}</p>
              <p className="text-xs text-gray-500">Today</p>
            </div>
            <div className="p-4 bg-gray-50 rounded-lg">
              <p className="text-2xl font-bold text-gray-900">{auditStats.uniqueActors}</p>
              <p className="text-xs text-gray-500">Unique Actors</p>
            </div>
            <div className="p-4 bg-gray-50 rounded-lg">
              <div className="flex flex-wrap gap-1">
                {auditStats.topActions.map((action) => (
                  <span
                    key={action}
                    className="text-xs px-1.5 py-0.5 bg-gray-200 text-gray-700 rounded"
                  >
                    {action}
                  </span>
                ))}
              </div>
              <p className="text-xs text-gray-500 mt-2">Top Actions</p>
            </div>
          </div>
        </div>

        {/* Quick Links */}
        <div className="bg-white rounded-xl p-6 shadow-sm border border-gray-100">
          <h2 className="text-lg font-semibold text-gray-900 mb-4">Quick Actions</h2>
          <div className="space-y-2">
            <a
              href="/governance/policies"
              className="flex items-center gap-3 p-3 rounded-lg hover:bg-gray-50 transition-colors"
            >
              <Scale className="w-5 h-5 text-gray-400" />
              <span className="text-sm text-gray-700">Manage Policies</span>
            </a>
            <a
              href="/governance/standards"
              className="flex items-center gap-3 p-3 rounded-lg hover:bg-gray-50 transition-colors"
            >
              <BookOpen className="w-5 h-5 text-gray-400" />
              <span className="text-sm text-gray-700">View Standards</span>
            </a>
            <a
              href="/governance/approvals"
              className="flex items-center gap-3 p-3 rounded-lg hover:bg-gray-50 transition-colors"
            >
              <ClipboardCheck className="w-5 h-5 text-gray-400" />
              <span className="text-sm text-gray-700">Approval Workflows</span>
            </a>
            <a
              href="/governance/audit"
              className="flex items-center gap-3 p-3 rounded-lg hover:bg-gray-50 transition-colors"
            >
              <FileCheck className="w-5 h-5 text-gray-400" />
              <span className="text-sm text-gray-700">Audit Log</span>
            </a>
          </div>
        </div>
      </div>
    </div>
  );
}
