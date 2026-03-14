import { format } from 'date-fns';
import { X, ExternalLink, Sparkles } from 'lucide-react';
import { resolveEventType } from './eventTypes';
import type { TimelineEventData } from './TimelineEvent';

interface TimelineDetailPanelProps {
  event: TimelineEventData | null;
  onClose: () => void;
}

function DiffView({ before, after }: { before: unknown; after: unknown }) {
  const beforeStr = before ? JSON.stringify(before, null, 2).split('\n') : [];
  const afterStr = after ? JSON.stringify(after, null, 2).split('\n') : [];

  return (
    <div className="space-y-2">
      {before !== null && before !== undefined && (
        <div>
          <p className="text-xs font-medium text-red-400 mb-1">- Before</p>
          <pre className="text-xs bg-red-400/5 border border-red-400/10 p-3 rounded-lg overflow-x-auto text-red-300 font-mono">
            {beforeStr.join('\n')}
          </pre>
        </div>
      )}
      {after !== null && after !== undefined && (
        <div>
          <p className="text-xs font-medium text-green-400 mb-1">+ After</p>
          <pre className="text-xs bg-green-400/5 border border-green-400/10 p-3 rounded-lg overflow-x-auto text-green-300 font-mono">
            {afterStr.join('\n')}
          </pre>
        </div>
      )}
    </div>
  );
}

export function TimelineDetailPanel({ event, onClose }: TimelineDetailPanelProps) {
  if (!event) {
    return (
      <div className="glass-panel p-6 sticky top-6 flex items-center justify-center min-h-[400px]">
        <div className="text-center">
          <div className="w-12 h-12 rounded-full bg-surface-border/30 flex items-center justify-center mx-auto mb-3">
            <ExternalLink className="w-5 h-5 text-gray-500" />
          </div>
          <p className="text-gray-500 text-sm">Select an event to view details</p>
        </div>
      </div>
    );
  }

  const config = resolveEventType(event.action);
  const Icon = config.icon;
  const formattedTime = format(new Date(event.timestamp), 'PPpp');

  return (
    <div className="glass-panel sticky top-6 overflow-hidden">
      {/* Header */}
      <div className={`px-5 py-4 border-b border-surface-border flex items-center justify-between ${config.bgColor}`}>
        <div className="flex items-center gap-3">
          <div className={`p-2 rounded-lg ${config.bgColor}`}>
            <Icon className={`w-5 h-5 ${config.color}`} />
          </div>
          <div>
            <h3 className="text-sm font-semibold text-gray-200">{config.label}</h3>
            <p className="text-xs text-gray-400">{event.resourceName}</p>
          </div>
        </div>
        <button
          onClick={onClose}
          className="p-1 rounded-md hover:bg-surface-border/30 text-gray-400 hover:text-gray-200 transition-colors"
        >
          <X className="w-4 h-4" />
        </button>
      </div>

      <div className="p-5 space-y-5 max-h-[calc(100vh-200px)] overflow-y-auto">
        {/* Event Metadata */}
        <section>
          <h4 className="text-xs font-medium text-gray-500 uppercase tracking-wider mb-3">Event Details</h4>
          <dl className="space-y-2">
            <div className="flex justify-between">
              <dt className="text-xs text-gray-500">Type</dt>
              <dd className={`text-xs font-medium ${config.color}`}>{config.label}</dd>
            </div>
            <div className="flex justify-between">
              <dt className="text-xs text-gray-500">Timestamp</dt>
              <dd className="text-xs text-gray-300">{formattedTime}</dd>
            </div>
            <div className="flex justify-between">
              <dt className="text-xs text-gray-500">Actor</dt>
              <dd className="text-xs text-gray-300">{event.actor.name}</dd>
            </div>
            <div className="flex justify-between">
              <dt className="text-xs text-gray-500">Actor Type</dt>
              <dd className="text-xs text-gray-300 capitalize">{event.actor.type}</dd>
            </div>
            {event.ipAddress && (
              <div className="flex justify-between">
                <dt className="text-xs text-gray-500">IP Address</dt>
                <dd className="text-xs text-gray-300 font-mono">{event.ipAddress}</dd>
              </div>
            )}
            <div className="flex justify-between">
              <dt className="text-xs text-gray-500">Resource</dt>
              <dd className="text-xs text-gray-300 font-mono">{event.resourceType}/{event.resourceId}</dd>
            </div>
          </dl>
        </section>

        {/* State Changes */}
        {event.changes && (event.changes.before || event.changes.after) && (
          <section>
            <h4 className="text-xs font-medium text-gray-500 uppercase tracking-wider mb-3">State Changes</h4>
            <DiffView before={event.changes.before} after={event.changes.after} />
          </section>
        )}

        {/* Metadata */}
        {event.metadata && Object.keys(event.metadata).length > 0 && (
          <section>
            <h4 className="text-xs font-medium text-gray-500 uppercase tracking-wider mb-3">Metadata</h4>
            <pre className="text-xs bg-surface-overlay/50 border border-surface-border p-3 rounded-lg overflow-x-auto text-gray-300 font-mono">
              {JSON.stringify(event.metadata, null, 2)}
            </pre>
          </section>
        )}

        {/* Related Entities */}
        <section>
          <h4 className="text-xs font-medium text-gray-500 uppercase tracking-wider mb-3">Related Entities</h4>
          <div className="space-y-1.5">
            <button className="w-full flex items-center justify-between px-3 py-2 rounded-lg bg-surface-overlay/30 hover:bg-surface-overlay/50 transition-colors text-xs text-gray-300">
              <span className="capitalize">{event.resourceType}: {event.resourceName}</span>
              <ExternalLink className="w-3 h-3 text-gray-500" />
            </button>
          </div>
        </section>

        {/* Hash Chain */}
        {event.hash && (
          <section>
            <h4 className="text-xs font-medium text-gray-500 uppercase tracking-wider mb-3">Hash Chain</h4>
            <div className="bg-surface-overlay/30 border border-surface-border rounded-lg p-3 space-y-2">
              <div>
                <p className="text-xs text-gray-500 mb-0.5">SHA-256 Hash</p>
                <p className="text-xs text-gray-300 font-mono break-all">{event.hash}</p>
              </div>
            </div>
          </section>
        )}

        {/* AI Explanation */}
        <section>
          <h4 className="text-xs font-medium text-gray-500 uppercase tracking-wider mb-3 flex items-center gap-1.5">
            <Sparkles className="w-3 h-3 text-purple-400" />
            AI Explanation
          </h4>
          <div className="bg-purple-400/5 border border-purple-400/10 rounded-lg p-3">
            <p className="text-xs text-gray-400 italic">
              {event.action.includes('policy')
                ? `This policy was updated on ${event.resourceName}. Review the state changes above for details on what was modified.`
                : event.action.includes('approv')
                ? `An approval decision was made on "${event.resourceName}" by ${event.actor.name}.`
                : event.action.includes('created')
                ? `A new ${event.resourceType} "${event.resourceName}" was created by ${event.actor.name}.`
                : event.action.includes('deleted')
                ? `The ${event.resourceType} "${event.resourceName}" was removed by ${event.actor.name}.`
                : `Action "${event.action}" was performed on ${event.resourceType} "${event.resourceName}" by ${event.actor.name}.`}
            </p>
          </div>
        </section>
      </div>
    </div>
  );
}
