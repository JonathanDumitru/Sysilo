import { useState } from 'react';
import {
  Search,
  ClipboardCheck,
  CheckCircle,
  XCircle,
  Clock,
  ChevronRight,
  X,
  Settings,
} from 'lucide-react';

// Mock data
const approvalRequests = [
  {
    id: 'REQ-001',
    type: 'Integration',
    resourceName: 'Salesforce Production Connection',
    requester: { name: 'John Doe', email: 'john@example.com' },
    workflow: 'Production Deployment',
    currentStage: 1,
    totalStages: 3,
    stageName: 'Security Review',
    status: 'pending',
    createdAt: '2024-01-15T08:00:00Z',
    decisions: [
      { stage: 0, approver: 'Manager', decision: 'approved', comment: 'Approved', decidedAt: '2024-01-15T09:00:00Z' },
    ],
  },
  {
    id: 'REQ-002',
    type: 'Agent',
    resourceName: 'New Production Agent',
    requester: { name: 'Jane Smith', email: 'jane@example.com' },
    workflow: 'Production Deployment',
    currentStage: 2,
    totalStages: 3,
    stageName: 'IT Approval',
    status: 'pending',
    createdAt: '2024-01-15T05:00:00Z',
    decisions: [
      { stage: 0, approver: 'Manager', decision: 'approved', comment: 'LGTM', decidedAt: '2024-01-15T06:00:00Z' },
      { stage: 1, approver: 'Security Team', decision: 'approved', comment: 'Security requirements met', decidedAt: '2024-01-15T07:00:00Z' },
    ],
  },
  {
    id: 'REQ-003',
    type: 'Data Export',
    resourceName: 'Customer Data Export',
    requester: { name: 'Bob Wilson', email: 'bob@example.com' },
    workflow: 'Data Export Approval',
    currentStage: 0,
    totalStages: 2,
    stageName: 'Compliance Review',
    status: 'pending',
    createdAt: '2024-01-14T15:00:00Z',
    decisions: [],
  },
  {
    id: 'REQ-004',
    type: 'Connection',
    resourceName: 'Third-party API Connection',
    requester: { name: 'Alice Brown', email: 'alice@example.com' },
    workflow: 'Standard Approval',
    currentStage: 1,
    totalStages: 1,
    stageName: 'Manager Approval',
    status: 'approved',
    createdAt: '2024-01-13T10:00:00Z',
    decisions: [
      { stage: 0, approver: 'Manager', decision: 'approved', comment: 'Approved for production use', decidedAt: '2024-01-13T12:00:00Z' },
    ],
  },
  {
    id: 'REQ-005',
    type: 'Integration',
    resourceName: 'Legacy System Migration',
    requester: { name: 'Charlie Davis', email: 'charlie@example.com' },
    workflow: 'Production Deployment',
    currentStage: 1,
    totalStages: 3,
    stageName: 'Security Review',
    status: 'rejected',
    createdAt: '2024-01-12T14:00:00Z',
    decisions: [
      { stage: 0, approver: 'Manager', decision: 'approved', comment: 'Approved', decidedAt: '2024-01-12T15:00:00Z' },
      { stage: 1, approver: 'Security Team', decision: 'rejected', comment: 'Security concerns not addressed', decidedAt: '2024-01-12T17:00:00Z' },
    ],
  },
];

const workflows = [
  {
    id: '1',
    name: 'Production Deployment',
    description: 'Approval workflow for production deployments',
    stages: [
      { name: 'Manager Approval', approvers: ['manager'], required: 1 },
      { name: 'Security Review', approvers: ['security-team'], required: 1 },
      { name: 'IT Approval', approvers: ['it-team'], required: 2 },
    ],
    enabled: true,
    triggers: ['integration.create', 'agent.deploy'],
  },
  {
    id: '2',
    name: 'Data Export Approval',
    description: 'Approval for data exports containing sensitive data',
    stages: [
      { name: 'Compliance Review', approvers: ['compliance-team'], required: 1 },
      { name: 'Data Owner Approval', approvers: ['data-owner'], required: 1 },
    ],
    enabled: true,
    triggers: ['data.export'],
  },
  {
    id: '3',
    name: 'Standard Approval',
    description: 'Simple one-step approval for standard requests',
    stages: [
      { name: 'Manager Approval', approvers: ['manager'], required: 1 },
    ],
    enabled: true,
    triggers: ['connection.create'],
  },
];

type TabType = 'requests' | 'workflows';

export function ApprovalsPage() {
  const [activeTab, setActiveTab] = useState<TabType>('requests');
  const [searchQuery, setSearchQuery] = useState('');
  const [statusFilter, setStatusFilter] = useState<string>('all');
  const [selectedRequest, setSelectedRequest] = useState<typeof approvalRequests[0] | null>(null);
  const [showDecisionModal, setShowDecisionModal] = useState(false);

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'pending':
        return 'bg-yellow-900/40 text-yellow-400';
      case 'approved':
        return 'bg-green-900/40 text-green-400';
      case 'rejected':
        return 'bg-red-900/40 text-red-400';
      case 'expired':
        return 'bg-surface-overlay text-gray-300';
      default:
        return 'bg-surface-overlay text-gray-300';
    }
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'pending':
        return <Clock className="w-4 h-4" />;
      case 'approved':
        return <CheckCircle className="w-4 h-4" />;
      case 'rejected':
        return <XCircle className="w-4 h-4" />;
      default:
        return null;
    }
  };

  const formatDate = (dateStr: string) => {
    const date = new Date(dateStr);
    return date.toLocaleString();
  };

  const getTimeAgo = (dateStr: string) => {
    const date = new Date(dateStr);
    const now = new Date();
    const diff = now.getTime() - date.getTime();
    const hours = Math.floor(diff / (1000 * 60 * 60));
    const days = Math.floor(hours / 24);
    if (days > 0) return `${days} day${days > 1 ? 's' : ''} ago`;
    if (hours > 0) return `${hours} hour${hours > 1 ? 's' : ''} ago`;
    return 'Just now';
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-white">Approvals</h1>
          <p className="text-gray-500">Manage approval workflows and pending requests</p>
        </div>
      </div>

      {/* Tabs */}
      <div className="border-b border-surface-border">
        <nav className="flex gap-6">
          <button
            onClick={() => setActiveTab('requests')}
            className={`py-3 text-sm font-medium border-b-2 transition-colors ${
              activeTab === 'requests'
                ? 'border-primary-600 text-primary-600'
                : 'border-transparent text-gray-500 hover:text-gray-200'
            }`}
          >
            Approval Requests
            <span className="ml-2 px-2 py-0.5 text-xs rounded-full bg-yellow-900/40 text-yellow-400">
              {approvalRequests.filter((r) => r.status === 'pending').length}
            </span>
          </button>
          <button
            onClick={() => setActiveTab('workflows')}
            className={`py-3 text-sm font-medium border-b-2 transition-colors ${
              activeTab === 'workflows'
                ? 'border-primary-600 text-primary-600'
                : 'border-transparent text-gray-500 hover:text-gray-200'
            }`}
          >
            Workflows
            <span className="ml-2 px-2 py-0.5 text-xs rounded-full bg-surface-overlay text-gray-300">
              {workflows.length}
            </span>
          </button>
        </nav>
      </div>

      {/* Filters */}
      <div className="flex items-center gap-4">
        <div className="relative flex-1 max-w-md">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400" />
          <input
            type="text"
            placeholder="Search requests..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-full pl-10 pr-4 py-2 glass-input text-sm"
          />
        </div>
        {activeTab === 'requests' && (
          <select
            value={statusFilter}
            onChange={(e) => setStatusFilter(e.target.value)}
            className="px-3 py-2 glass-input text-sm"
          >
            <option value="all">All Statuses</option>
            <option value="pending">Pending</option>
            <option value="approved">Approved</option>
            <option value="rejected">Rejected</option>
          </select>
        )}
      </div>

      {/* Requests Tab */}
      {activeTab === 'requests' && (
        <div className="grid grid-cols-3 gap-6">
          {/* Requests List */}
          <div className="col-span-2 space-y-4">
            {approvalRequests.map((request) => (
              <div
                key={request.id}
                onClick={() => setSelectedRequest(request)}
                className={`glass-panel p-5 cursor-pointer transition-all ${
                  selectedRequest?.id === request.id
                    ? 'border-primary-500 ring-2 ring-primary-500/20'
                    : 'border-surface-border hover:border-surface-border'
                }`}
              >
                <div className="flex items-start justify-between">
                  <div className="flex-1">
                    <div className="flex items-center gap-3 mb-2">
                      <span className="text-xs font-mono text-gray-500">{request.id}</span>
                      <span className="text-xs px-1.5 py-0.5 bg-blue-900/40 text-blue-400 rounded">
                        {request.type}
                      </span>
                      <span
                        className={`flex items-center gap-1 text-xs font-medium px-2 py-0.5 rounded-full ${getStatusColor(
                          request.status
                        )}`}
                      >
                        {getStatusIcon(request.status)}
                        {request.status}
                      </span>
                    </div>
                    <h3 className="text-base font-semibold text-white mb-1">
                      {request.resourceName}
                    </h3>
                    <p className="text-sm text-gray-500">
                      Workflow: {request.workflow}
                    </p>
                  </div>
                  <ChevronRight className="w-5 h-5 text-gray-400" />
                </div>

                {/* Progress */}
                <div className="mt-4 pt-3 border-t border-surface-border">
                  <div className="flex items-center justify-between mb-2">
                    <span className="text-xs text-gray-500">
                      Stage {request.currentStage + 1} of {request.totalStages}: {request.stageName}
                    </span>
                    <span className="text-xs text-gray-400">{getTimeAgo(request.createdAt)}</span>
                  </div>
                  <div className="flex gap-1">
                    {Array.from({ length: request.totalStages }).map((_, i) => (
                      <div
                        key={i}
                        className={`flex-1 h-1.5 rounded-full ${
                          i < request.currentStage
                            ? 'bg-green-500'
                            : i === request.currentStage && request.status === 'pending'
                            ? 'bg-yellow-500'
                            : i === request.currentStage && request.status === 'rejected'
                            ? 'bg-red-500'
                            : 'bg-gray-700'
                        }`}
                      />
                    ))}
                  </div>
                </div>
              </div>
            ))}
          </div>

          {/* Detail Panel */}
          <div className="glass-panel h-fit sticky top-6">
            {selectedRequest ? (
              <div>
                <div className="p-5 border-b border-surface-border">
                  <div className="flex items-center justify-between mb-3">
                    <span className="text-xs font-mono text-gray-500">{selectedRequest.id}</span>
                    <span
                      className={`flex items-center gap-1 text-xs font-medium px-2 py-0.5 rounded-full ${getStatusColor(
                        selectedRequest.status
                      )}`}
                    >
                      {getStatusIcon(selectedRequest.status)}
                      {selectedRequest.status}
                    </span>
                  </div>
                  <h3 className="text-lg font-semibold text-white mb-1">
                    {selectedRequest.resourceName}
                  </h3>
                  <span className="text-xs px-1.5 py-0.5 bg-blue-900/40 text-blue-400 rounded">
                    {selectedRequest.type}
                  </span>
                </div>

                <div className="p-5 border-b border-surface-border">
                  <h4 className="text-sm font-medium text-white mb-3">Request Details</h4>
                  <div className="space-y-2 text-sm">
                    <div className="flex justify-between">
                      <span className="text-gray-500">Requester</span>
                      <span className="text-white">{selectedRequest.requester.name}</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-gray-500">Workflow</span>
                      <span className="text-white">{selectedRequest.workflow}</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-gray-500">Created</span>
                      <span className="text-white">{formatDate(selectedRequest.createdAt)}</span>
                    </div>
                  </div>
                </div>

                <div className="p-5 border-b border-surface-border">
                  <h4 className="text-sm font-medium text-white mb-3">Approval History</h4>
                  <div className="space-y-3">
                    {selectedRequest.decisions.length === 0 ? (
                      <p className="text-sm text-gray-500">No decisions yet</p>
                    ) : (
                      selectedRequest.decisions.map((decision, i) => (
                        <div key={i} className="flex gap-3">
                          {decision.decision === 'approved' ? (
                            <CheckCircle className="w-5 h-5 text-green-500 flex-shrink-0" />
                          ) : (
                            <XCircle className="w-5 h-5 text-red-500 flex-shrink-0" />
                          )}
                          <div className="flex-1">
                            <p className="text-sm text-white">
                              <span className="font-medium">{decision.approver}</span>{' '}
                              <span className={decision.decision === 'approved' ? 'text-green-600' : 'text-red-600'}>
                                {decision.decision}
                              </span>
                            </p>
                            {decision.comment && (
                              <p className="text-xs text-gray-500 mt-0.5">{decision.comment}</p>
                            )}
                            <p className="text-xs text-gray-400 mt-0.5">
                              {formatDate(decision.decidedAt)}
                            </p>
                          </div>
                        </div>
                      ))
                    )}
                  </div>
                </div>

                {selectedRequest.status === 'pending' && (
                  <div className="p-5">
                    <div className="flex gap-2">
                      <button
                        onClick={() => setShowDecisionModal(true)}
                        className="flex-1 px-4 py-2 bg-green-600 text-white rounded-lg text-sm font-medium hover:bg-green-700"
                      >
                        Approve
                      </button>
                      <button
                        onClick={() => setShowDecisionModal(true)}
                        className="flex-1 px-4 py-2 bg-red-600 text-white rounded-lg text-sm font-medium hover:bg-red-700"
                      >
                        Reject
                      </button>
                    </div>
                  </div>
                )}
              </div>
            ) : (
              <div className="p-8 text-center text-gray-500">
                <ClipboardCheck className="w-12 h-12 mx-auto mb-3 text-gray-300" />
                <p>Select a request to view details</p>
              </div>
            )}
          </div>
        </div>
      )}

      {/* Workflows Tab */}
      {activeTab === 'workflows' && (
        <div className="grid gap-4">
          {workflows.map((workflow) => (
            <div
              key={workflow.id}
              className="glass-panel p-6"
            >
              <div className="flex items-start justify-between">
                <div className="flex items-start gap-4">
                  <div className="p-2 bg-primary-900/30 rounded-lg">
                    <Settings className="w-5 h-5 text-primary-600" />
                  </div>
                  <div>
                    <div className="flex items-center gap-3 mb-1">
                      <h3 className="text-lg font-semibold text-white">{workflow.name}</h3>
                      {!workflow.enabled && (
                        <span className="text-xs font-medium px-2 py-0.5 rounded-full bg-surface-overlay text-gray-500">
                          Disabled
                        </span>
                      )}
                    </div>
                    <p className="text-sm text-gray-500 mb-3">{workflow.description}</p>

                    {/* Stages */}
                    <div className="flex items-center gap-2 flex-wrap">
                      {workflow.stages.map((stage, i) => (
                        <div key={i} className="flex items-center gap-2">
                          <span className="text-xs px-2 py-1 bg-surface-overlay text-gray-300 rounded">
                            {i + 1}. {stage.name}
                          </span>
                          {i < workflow.stages.length - 1 && (
                            <ChevronRight className="w-4 h-4 text-gray-400" />
                          )}
                        </div>
                      ))}
                    </div>

                    {/* Triggers */}
                    <div className="mt-3">
                      <span className="text-xs text-gray-500">Triggers: </span>
                      {workflow.triggers.map((trigger, i) => (
                        <span
                          key={i}
                          className="text-xs px-1.5 py-0.5 bg-blue-900/30 text-blue-400 rounded mr-1"
                        >
                          {trigger}
                        </span>
                      ))}
                    </div>
                  </div>
                </div>
                <button className="text-sm text-primary-600 hover:text-primary-700">
                  Edit
                </button>
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Decision Modal */}
      {showDecisionModal && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-surface-raised border border-surface-border rounded-xl p-6 w-full max-w-md">
            <div className="flex items-center justify-between mb-4">
              <h2 className="text-lg font-semibold">Make Decision</h2>
              <button
                onClick={() => setShowDecisionModal(false)}
                className="p-1 text-gray-400 hover:text-gray-300"
              >
                <X className="w-5 h-5" />
              </button>
            </div>
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-gray-300 mb-1">Comment</label>
                <textarea
                  className="w-full px-3 py-2 glass-input text-sm"
                  rows={3}
                  placeholder="Add a comment for your decision"
                />
              </div>
              <div className="flex gap-3">
                <button
                  onClick={() => setShowDecisionModal(false)}
                  className="flex-1 px-4 py-2 bg-green-600 text-white rounded-lg text-sm font-medium hover:bg-green-700"
                >
                  Approve
                </button>
                <button
                  onClick={() => setShowDecisionModal(false)}
                  className="flex-1 px-4 py-2 bg-red-600 text-white rounded-lg text-sm font-medium hover:bg-red-700"
                >
                  Reject
                </button>
              </div>
              <button
                onClick={() => setShowDecisionModal(false)}
                className="w-full px-4 py-2 text-sm font-medium text-gray-300 hover:text-white"
              >
                Cancel
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
