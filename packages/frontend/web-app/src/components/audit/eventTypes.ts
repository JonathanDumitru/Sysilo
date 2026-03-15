import {
  Shield,
  Key,
  AlertTriangle,
  CheckCircle,
  Server,
  User,
  CheckSquare,
  Database,
  Settings,
} from 'lucide-react';
import type { ComponentType } from 'react';

export interface EventTypeConfig {
  icon: ComponentType<{ className?: string }>;
  color: string;
  bgColor: string;
  borderColor: string;
  label: string;
}

export const EVENT_TYPES: Record<string, EventTypeConfig> = {
  policy_change: { icon: Shield, color: 'text-blue-400', bgColor: 'bg-blue-400/10', borderColor: 'border-blue-400/50', label: 'Policy Change' },
  credential_rotation: { icon: Key, color: 'text-amber-400', bgColor: 'bg-amber-400/10', borderColor: 'border-amber-400/50', label: 'Credential Rotation' },
  run_failure: { icon: AlertTriangle, color: 'text-red-400', bgColor: 'bg-red-400/10', borderColor: 'border-red-400/50', label: 'Run Failure' },
  run_success: { icon: CheckCircle, color: 'text-green-400', bgColor: 'bg-green-400/10', borderColor: 'border-green-400/50', label: 'Run Success' },
  agent_action: { icon: Server, color: 'text-purple-400', bgColor: 'bg-purple-400/10', borderColor: 'border-purple-400/50', label: 'Agent Action' },
  user_login: { icon: User, color: 'text-gray-400', bgColor: 'bg-gray-400/10', borderColor: 'border-gray-400/50', label: 'User Login' },
  approval: { icon: CheckSquare, color: 'text-blue-400', bgColor: 'bg-blue-400/10', borderColor: 'border-blue-400/50', label: 'Approval' },
  data_change: { icon: Database, color: 'text-cyan-400', bgColor: 'bg-cyan-400/10', borderColor: 'border-cyan-400/50', label: 'Data Change' },
  configuration: { icon: Settings, color: 'text-gray-400', bgColor: 'bg-gray-400/10', borderColor: 'border-gray-400/50', label: 'Configuration' },
};

export function resolveEventType(action: string): EventTypeConfig {
  if (action.includes('policy')) return EVENT_TYPES.policy_change;
  if (action.includes('credential') || action.includes('key') || action.includes('rotation')) return EVENT_TYPES.credential_rotation;
  if (action.includes('fail') || action.includes('error') || action.includes('fired')) return EVENT_TYPES.run_failure;
  if (action.includes('success') || action.includes('completed')) return EVENT_TYPES.run_success;
  if (action.includes('agent') || action.includes('task')) return EVENT_TYPES.agent_action;
  if (action.includes('login') || action.includes('auth')) return EVENT_TYPES.user_login;
  if (action.includes('approv') || action.includes('reject')) return EVENT_TYPES.approval;
  if (action.includes('data') || action.includes('connection')) return EVENT_TYPES.data_change;
  if (action.includes('config') || action.includes('setting')) return EVENT_TYPES.configuration;
  if (action.includes('created') || action.includes('updated') || action.includes('deleted')) return EVENT_TYPES.data_change;
  return EVENT_TYPES.configuration;
}
