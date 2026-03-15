import { useState, useCallback, useMemo } from 'react';
import { useQuery } from '@tanstack/react-query';
import { format, subDays } from 'date-fns';
import { queryAuditLog } from '../services/governance';
import type { TimelineEventData } from '../components/audit/TimelineEvent';

export interface AuditTimelineFilters {
  dateRange: string;
  eventType: string;
  actor: string;
  search: string;
}

export interface DayGroup {
  date: string;
  label: string;
  events: TimelineEventData[];
}

const DATE_RANGE_DAYS: Record<string, number> = {
  '1h': 0,
  '24h': 1,
  '7d': 7,
  '30d': 30,
  '90d': 90,
};

function mapAuditEntry(entry: {
  id: string;
  actor_id: string;
  actor_type: string;
  actor_name: string;
  action: string;
  resource_type: string;
  resource_id: string;
  resource_name: string;
  before_state: unknown;
  after_state: unknown;
  metadata: Record<string, unknown>;
  ip_address: string;
  timestamp: string;
  hash: string;
}): TimelineEventData {
  return {
    id: entry.id,
    action: entry.action,
    actor: { id: entry.actor_id, name: entry.actor_name, type: entry.actor_type },
    resourceType: entry.resource_type,
    resourceId: entry.resource_id,
    resourceName: entry.resource_name,
    timestamp: entry.timestamp,
    ipAddress: entry.ip_address || null,
    metadata: entry.metadata || null,
    changes:
      entry.before_state || entry.after_state
        ? { before: entry.before_state, after: entry.after_state }
        : null,
    hash: entry.hash,
  };
}

export function useAuditTimeline() {
  const [filters, setFilters] = useState<AuditTimelineFilters>({
    dateRange: '7d',
    eventType: 'all',
    actor: 'all',
    search: '',
  });

  const [offset, setOffset] = useState(0);
  const limit = 50;

  const startDate = useMemo(() => {
    const days = DATE_RANGE_DAYS[filters.dateRange] ?? 7;
    return days === 0
      ? new Date(Date.now() - 60 * 60 * 1000)
      : subDays(new Date(), days);
  }, [filters.dateRange]);

  const { data: rawEntries, isLoading } = useQuery({
    queryKey: ['audit-timeline', filters, offset],
    queryFn: () =>
      queryAuditLog({
        start_time: startDate.toISOString(),
        end_time: new Date().toISOString(),
        action: filters.eventType !== 'all' ? filters.eventType : undefined,
        limit,
        offset,
      }),
    staleTime: 30_000,
  });

  const events: TimelineEventData[] = useMemo(() => {
    if (!rawEntries) return [];
    return rawEntries.map(mapAuditEntry).filter((e) => {
      if (filters.actor !== 'all' && e.actor.type !== filters.actor) return false;
      if (filters.search) {
        const q = filters.search.toLowerCase();
        return (
          e.action.toLowerCase().includes(q) ||
          e.actor.name.toLowerCase().includes(q) ||
          e.resourceName.toLowerCase().includes(q)
        );
      }
      return true;
    });
  }, [rawEntries, filters.actor, filters.search]);

  const groupedByDay: DayGroup[] = useMemo(() => {
    const groups: Record<string, TimelineEventData[]> = {};
    events.forEach((e) => {
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
  }, [events]);

  const hasMore = (rawEntries?.length ?? 0) >= limit;

  const loadMore = useCallback(() => {
    setOffset((prev) => prev + limit);
  }, []);

  return {
    events,
    groupedByDay,
    isLoading,
    hasMore,
    loadMore,
    filters,
    setFilters,
    startDate,
    endDate: new Date(),
  };
}
