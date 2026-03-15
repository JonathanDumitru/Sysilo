import { useCallback, useRef, useState, useMemo } from 'react';
import { format } from 'date-fns';
import type { TimelineEventData } from './TimelineEvent';

interface TimelineScrubberProps {
  events: TimelineEventData[];
  startDate: Date;
  endDate: Date;
  onRangeChange: (start: Date, end: Date) => void;
  onPointClick: (date: Date) => void;
}

export function TimelineScrubber({ events, startDate, endDate, onRangeChange, onPointClick }: TimelineScrubberProps) {
  const trackRef = useRef<HTMLDivElement>(null);
  const [dragging, setDragging] = useState<'start' | 'end' | null>(null);
  const [localStart, setLocalStart] = useState(0);
  const [localEnd, setLocalEnd] = useState(100);

  // Build density buckets for sparkline
  const { buckets, maxCount } = useMemo(() => {
    const numBuckets = 60;
    const range = endDate.getTime() - startDate.getTime();
    if (range <= 0) return { buckets: [], maxCount: 1 };

    const b = Array.from({ length: numBuckets }, () => 0);
    events.forEach((e) => {
      const t = new Date(e.timestamp).getTime();
      const idx = Math.min(
        numBuckets - 1,
        Math.max(0, Math.floor(((t - startDate.getTime()) / range) * numBuckets))
      );
      b[idx]++;
    });
    return { buckets: b, maxCount: Math.max(1, ...b) };
  }, [events, startDate, endDate]);

  const getPositionFromMouse = useCallback(
    (clientX: number) => {
      if (!trackRef.current) return 0;
      const rect = trackRef.current.getBoundingClientRect();
      return Math.min(100, Math.max(0, ((clientX - rect.left) / rect.width) * 100));
    },
    []
  );

  const handleMouseDown = useCallback(
    (handle: 'start' | 'end') => (e: React.MouseEvent) => {
      e.preventDefault();
      setDragging(handle);

      const handleMouseMove = (me: MouseEvent) => {
        const pos = getPositionFromMouse(me.clientX);
        if (handle === 'start') {
          setLocalStart(Math.min(pos, localEnd - 2));
        } else {
          setLocalEnd(Math.max(pos, localStart + 2));
        }
      };

      const handleMouseUp = (me: MouseEvent) => {
        document.removeEventListener('mousemove', handleMouseMove);
        document.removeEventListener('mouseup', handleMouseUp);
        setDragging(null);
        const pos = getPositionFromMouse(me.clientX);
        const range = endDate.getTime() - startDate.getTime();
        if (handle === 'start') {
          const newStart = Math.min(pos, localEnd - 2);
          setLocalStart(newStart);
          onRangeChange(
            new Date(startDate.getTime() + (newStart / 100) * range),
            new Date(startDate.getTime() + (localEnd / 100) * range)
          );
        } else {
          const newEnd = Math.max(pos, localStart + 2);
          setLocalEnd(newEnd);
          onRangeChange(
            new Date(startDate.getTime() + (localStart / 100) * range),
            new Date(startDate.getTime() + (newEnd / 100) * range)
          );
        }
      };

      document.addEventListener('mousemove', handleMouseMove);
      document.addEventListener('mouseup', handleMouseUp);
    },
    [getPositionFromMouse, localStart, localEnd, startDate, endDate, onRangeChange]
  );

  const handleTrackClick = useCallback(
    (e: React.MouseEvent) => {
      if (dragging) return;
      const pos = getPositionFromMouse(e.clientX);
      const range = endDate.getTime() - startDate.getTime();
      const clickDate = new Date(startDate.getTime() + (pos / 100) * range);
      onPointClick(clickDate);
    },
    [dragging, getPositionFromMouse, startDate, endDate, onPointClick]
  );

  return (
    <div className="glass-card p-4">
      <div className="flex items-center justify-between mb-2">
        <span className="text-xs text-gray-500">{format(startDate, 'MMM d, yyyy')}</span>
        <span className="text-xs text-gray-400 font-medium">Event Density</span>
        <span className="text-xs text-gray-500">{format(endDate, 'MMM d, yyyy')}</span>
      </div>

      {/* Sparkline density chart */}
      <div className="relative h-10 mb-1">
        <div className="flex items-end h-full gap-px">
          {buckets.map((count, i) => (
            <div
              key={i}
              className="flex-1 bg-primary-400/30 rounded-t-sm transition-all duration-150 hover:bg-primary-400/50"
              style={{ height: `${Math.max(2, (count / maxCount) * 100)}%` }}
            />
          ))}
        </div>

        {/* Selected range overlay */}
        <div
          className="absolute top-0 h-full bg-primary-400/10 border-x border-primary-400/30 pointer-events-none"
          style={{ left: `${localStart}%`, width: `${localEnd - localStart}%` }}
        />
      </div>

      {/* Scrubber track */}
      <div
        ref={trackRef}
        className="relative h-4 cursor-pointer"
        onClick={handleTrackClick}
      >
        <div className="absolute top-1/2 -translate-y-1/2 w-full h-0.5 bg-surface-border rounded" />

        {/* Start handle */}
        <div
          className="absolute top-1/2 -translate-y-1/2 -translate-x-1/2 w-3 h-3 rounded-full bg-primary-400 border-2 border-surface-base cursor-ew-resize hover:scale-125 transition-transform z-10"
          style={{ left: `${localStart}%` }}
          onMouseDown={handleMouseDown('start')}
        />

        {/* End handle */}
        <div
          className="absolute top-1/2 -translate-y-1/2 -translate-x-1/2 w-3 h-3 rounded-full bg-primary-400 border-2 border-surface-base cursor-ew-resize hover:scale-125 transition-transform z-10"
          style={{ left: `${localEnd}%` }}
          onMouseDown={handleMouseDown('end')}
        />
      </div>
    </div>
  );
}
