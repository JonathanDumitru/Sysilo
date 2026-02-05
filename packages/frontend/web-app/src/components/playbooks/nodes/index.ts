export * from './IntegrationStepNode';
export * from './WebhookStepNode';
export * from './WaitStepNode';
export * from './ConditionStepNode';
export * from './ApprovalStepNode';

import type { StepStatus } from '../../../services/playbooks';

export interface StepNodeData extends Record<string, unknown> {
  id: string;
  name: string;
  config: Record<string, unknown>;
  status?: StepStatus;
}

export const statusStyles: Record<string, string> = {
  pending: 'border-gray-300 bg-white',
  running: 'border-blue-400 bg-blue-50 animate-pulse',
  completed: 'border-green-400 bg-green-50',
  failed: 'border-red-400 bg-red-50',
  skipped: 'border-gray-300 bg-gray-100',
};
