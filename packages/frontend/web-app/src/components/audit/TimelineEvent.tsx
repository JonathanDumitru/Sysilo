import { formatDistanceToNow } from 'date-fns';
import { resolveEventType } from './eventTypes';

export interface TimelineEventData {
  id: string;
  action: string;
  actor: { id: string; name: string; type: string };
  resourceType: string;
  resourceId: string;
  resourceName: string;
  timestamp: string;
  ipAddress: string | null;
  metadata: Record<string, unknown> | null;
  changes: { before: unknown; after: unknown } | null;
  hash?: string;
  description?: string;
}

interface TimelineEventProps {
  event: TimelineEventData;
  side: 'left' | 'right';
  isSelected: boolean;
  onClick: () => void;
}

function formatActionTitle(action: string): string {
  return action
    .replace(/[._]/g, ' ')
    .replace(/\b\w/g, (c) => c.toUpperCase());
}

export function TimelineEvent({ event, side, isSelected, onClick }: TimelineEventProps) {
  const config = resolveEventType(event.action);
  const Icon = config.icon;
  const relativeTime = formatDistanceToNow(new Date(event.timestamp), { addSuffix: true });

  return (
    <div className={`relative flex ${side === 'left' ? 'justify-end pr-8 md:pr-12' : 'justify-start pl-8 md:pl-12'} w-1/2 ${side === 'left' ? 'self-start' : 'self-end ml-auto'}`}>
      {/* Connector dot on the timeline */}
      <div
        className={`absolute top-4 ${side === 'left' ? '-right-[7px]' : '-left-[7px]'} w-3.5 h-3.5 rounded-full border-2 border-surface-base ${config.bgColor} z-10`}
        style={{ boxShadow: `0 0 8px ${config.color.includes('blue') ? 'rgba(96,165,250,0.4)' : config.color.includes('amber') ? 'rgba(251,191,36,0.4)' : config.color.includes('red') ? 'rgba(248,113,113,0.4)' : config.color.includes('green') ? 'rgba(74,222,128,0.4)' : config.color.includes('purple') ? 'rgba(192,132,252,0.4)' : config.color.includes('cyan') ? 'rgba(34,211,238,0.4)' : 'rgba(156,163,175,0.4)'}` }}
      />

      {/* Event card */}
      <div
        onClick={onClick}
        className={`glass-card p-4 w-full max-w-sm cursor-pointer border-l-2 ${config.borderColor} transition-all duration-200 ${
          isSelected
            ? 'border-primary-500/30 shadow-glow ring-1 ring-primary-400/20'
            : 'hover:border-surface-border-strong'
        }`}
      >
        <div className="flex items-start gap-3">
          <div className={`p-2 rounded-lg ${config.bgColor} shrink-0`}>
            <Icon className={`w-4 h-4 ${config.color}`} />
          </div>
          <div className="min-w-0 flex-1">
            <p className="text-sm font-medium text-gray-200 truncate">
              {formatActionTitle(event.action)}
            </p>
            <p className="text-xs text-gray-400 mt-0.5 truncate">
              {event.resourceName}
            </p>
            <div className="flex items-center gap-2 mt-2 text-xs text-gray-500">
              <span className="truncate">{event.actor.name}</span>
              <span className="text-gray-600">|</span>
              <span className="whitespace-nowrap">{relativeTime}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
