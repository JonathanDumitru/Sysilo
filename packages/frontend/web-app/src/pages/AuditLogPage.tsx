import { useState, useRef, useCallback, useMemo } from 'react';
import { format } from 'date-fns';
import {
  Search,
  Download,
  Loader2,
  ChevronDown,
  Filter,
} from 'lucide-react';
import { useAuditTimeline } from '../hooks/useAuditTimeline';
import { TimelineEvent } from '../components/audit/TimelineEvent';
import { TimelineDetailPanel } from '../components/audit/TimelineDetailPanel';
import { TimelineScrubber } from '../components/audit/TimelineScrubber';
import { EVENT_TYPES } from '../components/audit/eventTypes';
import type { TimelineEventData } from '../components/audit/TimelineEvent';

// Mock data used when API returns empty (development/demo)
const mockEvents: TimelineEventData[] = [
  {
    id: '1',
    action: 'integration.created',
    actor: { id: 'user-1', name: 'John Doe', type: 'user' },
    resourceType: 'integration',
    resourceId: 'int-123',
    resourceName: 'Salesforce Sync',
    timestamp: new Date(Date.now() - 2 * 60 * 60 * 1000).toISOString(),
    ipAddress: '192.168.1.100',
    metadata: { source: 'web-ui' },
    changes: { before: null, after: { name: 'Salesforce Sync', enabled: true } },
    hash: 'a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef12345678',
  },
  {
    id: '2',
    action: 'policy.updated',
    actor: { id: 'user-2', name: 'Jane Smith', type: 'user' },
    resourceType: 'policy',
    resourceId: 'pol-456',
    resourceName: 'API Key Rotation Policy',
    timestamp: new Date(Date.now() - 4 * 60 * 60 * 1000).toISOString(),
    ipAddress: '192.168.1.101',
    metadata: { source: 'web-ui' },
    changes: { before: { enforcement_mode: 'warn' }, after: { enforcement_mode: 'enforce' } },
    hash: 'b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef123456789a',
  },
  {
    id: '3',
    action: 'alert.fired',
    actor: { id: 'system', name: 'System', type: 'system' },
    resourceType: 'alert',
    resourceId: 'alert-789',
    resourceName: 'High CPU Usage',
    timestamp: new Date(Date.now() - 6 * 60 * 60 * 1000).toISOString(),
    ipAddress: null,
    metadata: { triggered_value: 92.5, threshold: 80 },
    changes: null,
    hash: 'c3d4e5f6789012345678901234567890abcdef1234567890abcdef123456789ab0',
  },
  {
    id: '4',
    action: 'task.completed',
    actor: { id: 'agent-1', name: 'prod-agent-01', type: 'agent' },
    resourceType: 'task',
    resourceId: 'task-321',
    resourceName: 'Database Query Task',
    timestamp: new Date(Date.now() - 8 * 60 * 60 * 1000).toISOString(),
    ipAddress: '10.0.0.50',
    metadata: { duration_ms: 1250, rows_returned: 500 },
    changes: null,
    hash: 'd4e5f6789012345678901234567890abcdef1234567890abcdef123456789ab0c1',
  },
  {
    id: '5',
    action: 'connection.deleted',
    actor: { id: 'user-1', name: 'John Doe', type: 'user' },
    resourceType: 'connection',
    resourceId: 'conn-654',
    resourceName: 'Legacy Oracle DB',
    timestamp: new Date(Date.now() - 24 * 60 * 60 * 1000).toISOString(),
    ipAddress: '192.168.1.100',
    metadata: { reason: 'Deprecated' },
    changes: { before: { name: 'Legacy Oracle DB', type: 'oracle', enabled: false }, after: null },
    hash: 'e5f6789012345678901234567890abcdef1234567890abcdef123456789ab0c1d2',
  },
  {
    id: '6',
    action: 'approval.approved',
    actor: { id: 'user-3', name: 'Bob Wilson', type: 'user' },
    resourceType: 'approval_request',
    resourceId: 'apr-987',
    resourceName: 'New Integration Request',
    timestamp: new Date(Date.now() - 26 * 60 * 60 * 1000).toISOString(),
    ipAddress: '192.168.1.102',
    metadata: { stage: 2, comment: 'Looks good, approved.' },
    changes: { before: { status: 'pending', current_stage: 1 }, after: { status: 'approved', current_stage: 2 } },
    hash: 'f6789012345678901234567890abcdef1234567890abcdef123456789ab0c1d2e3',
  },
  {
    id: '7',
    action: 'incident.auto_created',
    actor: { id: 'system', name: 'System', type: 'system' },
    resourceType: 'incident',
    resourceId: 'inc-111',
    resourceName: 'Critical Alert Incident',
    timestamp: new Date(Date.now() - 48 * 60 * 60 * 1000).toISOString(),
    ipAddress: null,
    metadata: { triggered_by_alert: 'alert-789' },
    changes: null,
    hash: '789012345678901234567890abcdef1234567890abcdef123456789ab0c1d2e3f4',
  },
  {
    id: '8',
    action: 'standard.created',
    actor: { id: 'user-2', name: 'Jane Smith', type: 'user' },
    resourceType: 'standard',
    resourceId: 'std-222',
    resourceName: 'API Versioning Standard',
    timestamp: new Date(Date.now() - 72 * 60 * 60 * 1000).toISOString(),
    ipAddress: '192.168.1.101',
    metadata: { category: 'api', version: 1 },
    changes: { before: null, after: { name: 'API Versioning Standard', category: 'api' } },
    hash: '9012345678901234567890abcdef1234567890abcdef123456789ab0c1d2e3f4a5',
  },
];

const dateRangeOptions = [
  { value: '1h', label: 'Last 1 hour' },
  { value: '24h', label: 'Last 24 hours' },
  { value: '7d', label: 'Last 7 days' },
  { value: '30d', label: 'Last 30 days' },
  { value: '90d', label: 'Last 90 days' },
];

const eventTypeOptions = [
  { value: 'all', label: 'All Events' },
  ...Object.entries(EVENT_TYPES).map(([key, config]) => ({
    value: key,
    label: config.label,
  })),
];

const actorTypeOptions = [
  { value: 'all', label: 'All Actors' },
  { value: 'user', label: 'Users' },
  { value: 'system', label: 'System' },
  { value: 'agent', label: 'Agents' },
];

export function AuditLogPage() {
  const {
    events: apiEvents,
    groupedByDay: apiGroupedByDay,
    isLoading,
    hasMore,
    loadMore,
    filters,
    setFilters,
    startDate,
    endDate,
  } = useAuditTimeline();

  // Use mock data when API returns empty
  const useMock = apiEvents.length === 0 && !isLoading;

  const displayEvents = useMock ? mockEvents : apiEvents;

  // Group mock data by day if needed
  const mockGroupedByDay = useMemo(() => {
    const groups: Record<string, TimelineEventData[]> = {};
    mockEvents.forEach((e) => {
      const dayKey = format(new Date(e.timestamp), 'yyyy-MM-dd');
      if (!groups[dayKey]) groups[dayKey] = [];
      groups[dayKey].push(e);
    });
    return Object.entries(groups)
      .sort(([a], [b]) => b.localeCompare(a))
      .map(([dateStr, dayEvents]) => ({
        date: dateStr,
        label: format(new Date(dateStr), 'EEEE, MMMM d, yyyy'),
        events: dayEvents.sort(
          (a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime()
        ),
      }));
  }, []);

  const groupedByDay = useMock ? mockGroupedByDay : apiGroupedByDay;

  const [selectedEvent, setSelectedEvent] = useState<TimelineEventData | null>(null);
  const timelineRef = useRef<HTMLDivElement>(null);

  const handleScrubberRangeChange = useCallback(
    (_start: Date, _end: Date) => {
      // Range changes are driven by the scrubber handles
      // In a full implementation this would refetch with new date bounds
    },
    []
  );

  const handleScrubberPointClick = useCallback(
    (date: Date) => {
      // Find nearest event to clicked date and scroll to it
      if (!timelineRef.current || displayEvents.length === 0) return;
      const targetTime = date.getTime();
      let closest = displayEvents[0];
      let closestDiff = Math.abs(new Date(closest.timestamp).getTime() - targetTime);
      displayEvents.forEach((e) => {
        const diff = Math.abs(new Date(e.timestamp).getTime() - targetTime);
        if (diff < closestDiff) {
          closest = e;
          closestDiff = diff;
        }
      });
      const el = document.getElementById(`timeline-event-${closest.id}`);
      el?.scrollIntoView({ behavior: 'smooth', block: 'center' });
      setSelectedEvent(closest);
    },
    [displayEvents]
  );

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-100">Audit Timeline</h1>
          <p className="text-gray-500 text-sm mt-1">
            Interactive timeline of all platform activity
          </p>
        </div>
        <button className="flex items-center gap-2 px-4 py-2 glass-card text-sm font-medium text-gray-300 hover:text-gray-100 transition-colors">
          <Download className="w-4 h-4" />
          Export
        </button>
      </div>

      {/* Scrubber */}
      <TimelineScrubber
        events={displayEvents}
        startDate={startDate}
        endDate={endDate}
        onRangeChange={handleScrubberRangeChange}
        onPointClick={handleScrubberPointClick}
      />

      {/* Filters */}
      <div className="glass-card p-4">
        <div className="flex items-center gap-3 flex-wrap">
          <div className="flex items-center gap-2 text-gray-500">
            <Filter className="w-4 h-4" />
          </div>
          <div className="relative flex-1 min-w-[200px] max-w-sm">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-500" />
            <input
              type="text"
              placeholder="Search events..."
              value={filters.search}
              onChange={(e) => setFilters({ ...filters, search: e.target.value })}
              className="w-full pl-10 pr-4 py-2 bg-surface-overlay/50 border border-surface-border rounded-lg text-sm text-gray-200 placeholder-gray-500 focus:outline-none focus:ring-1 focus:ring-primary-500/50 focus:border-primary-500/30"
            />
          </div>
          <div className="relative">
            <select
              value={filters.dateRange}
              onChange={(e) => setFilters({ ...filters, dateRange: e.target.value })}
              className="appearance-none pl-3 pr-8 py-2 bg-surface-overlay/50 border border-surface-border rounded-lg text-sm text-gray-300 focus:outline-none focus:ring-1 focus:ring-primary-500/50 cursor-pointer"
            >
              {dateRangeOptions.map((opt) => (
                <option key={opt.value} value={opt.value}>
                  {opt.label}
                </option>
              ))}
            </select>
            <ChevronDown className="absolute right-2 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-500 pointer-events-none" />
          </div>
          <div className="relative">
            <select
              value={filters.eventType}
              onChange={(e) => setFilters({ ...filters, eventType: e.target.value })}
              className="appearance-none pl-3 pr-8 py-2 bg-surface-overlay/50 border border-surface-border rounded-lg text-sm text-gray-300 focus:outline-none focus:ring-1 focus:ring-primary-500/50 cursor-pointer"
            >
              {eventTypeOptions.map((opt) => (
                <option key={opt.value} value={opt.value}>
                  {opt.label}
                </option>
              ))}
            </select>
            <ChevronDown className="absolute right-2 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-500 pointer-events-none" />
          </div>
          <div className="relative">
            <select
              value={filters.actor}
              onChange={(e) => setFilters({ ...filters, actor: e.target.value })}
              className="appearance-none pl-3 pr-8 py-2 bg-surface-overlay/50 border border-surface-border rounded-lg text-sm text-gray-300 focus:outline-none focus:ring-1 focus:ring-primary-500/50 cursor-pointer"
            >
              {actorTypeOptions.map((opt) => (
                <option key={opt.value} value={opt.value}>
                  {opt.label}
                </option>
              ))}
            </select>
            <ChevronDown className="absolute right-2 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-500 pointer-events-none" />
          </div>
          <span className="text-xs text-gray-600 ml-auto">
            {displayEvents.length} event{displayEvents.length !== 1 ? 's' : ''}
          </span>
        </div>
      </div>

      {/* Main content: Timeline + Detail Panel */}
      <div className="flex gap-6">
        {/* Timeline (70%) */}
        <div className="w-[70%] min-w-0" ref={timelineRef}>
          {isLoading ? (
            <div className="flex items-center justify-center py-20">
              <Loader2 className="w-6 h-6 text-primary-400 animate-spin" />
              <span className="ml-3 text-gray-500 text-sm">Loading timeline...</span>
            </div>
          ) : groupedByDay.length === 0 ? (
            <div className="glass-panel p-12 text-center">
              <p className="text-gray-500">No events found for the selected filters.</p>
            </div>
          ) : (
            <div className="space-y-8">
              {groupedByDay.map((group) => (
                <div key={group.date}>
                  {/* Day header */}
                  <div className="flex items-center gap-4 mb-6">
                    <div className="h-px flex-1 bg-surface-border" />
                    <span className="text-xs text-gray-600 uppercase tracking-wider font-medium whitespace-nowrap">
                      {group.label}
                    </span>
                    <div className="h-px flex-1 bg-surface-border" />
                  </div>

                  {/* Timeline events for this day */}
                  <div className="relative">
                    {/* Vertical timeline line */}
                    <div className="absolute left-1/2 -translate-x-1/2 top-0 bottom-0 w-0.5 bg-surface-border" />

                    <div className="space-y-6">
                      {group.events.map((event, idx) => (
                        <div
                          key={event.id}
                          id={`timeline-event-${event.id}`}
                          className="relative flex"
                        >
                          <TimelineEvent
                            event={event}
                            side={idx % 2 === 0 ? 'left' : 'right'}
                            isSelected={selectedEvent?.id === event.id}
                            onClick={() =>
                              setSelectedEvent(
                                selectedEvent?.id === event.id ? null : event
                              )
                            }
                          />
                        </div>
                      ))}
                    </div>
                  </div>
                </div>
              ))}

              {/* Load more */}
              {hasMore && !useMock && (
                <div className="flex justify-center py-4">
                  <button
                    onClick={loadMore}
                    className="px-6 py-2 glass-card text-sm text-gray-400 hover:text-gray-200 transition-colors"
                  >
                    Load more events
                  </button>
                </div>
              )}
            </div>
          )}
        </div>

        {/* Detail Panel (30%) */}
        <div className="w-[30%] min-w-0">
          <TimelineDetailPanel
            event={selectedEvent}
            onClose={() => setSelectedEvent(null)}
          />
        </div>
      </div>
    </div>
  );
}
