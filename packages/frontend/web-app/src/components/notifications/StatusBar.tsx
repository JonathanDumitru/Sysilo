import { useState, useEffect } from 'react';
import {
  AlertTriangle,
  AlertCircle,
  Shield,
  ChevronUp,
  Activity,
} from 'lucide-react';
import { useAlertInstances } from '../../hooks/useOperations';
import { useApprovalRequests } from '../../hooks/useGovernance';
import { useStatusBar } from '../../hooks/useStatusBar';

export function StatusBar() {
  const { toggleDrawer, isDrawerOpen } = useStatusBar();
  const [lastChecked, setLastChecked] = useState<Date>(new Date());
  const [secondsAgo, setSecondsAgo] = useState(0);

  const { data: allAlerts } = useAlertInstances();
  const { data: pendingApprovals } = useApprovalRequests('pending');

  // Compute counts from live data
  const firingAlerts = (allAlerts ?? []).filter((a) => a.status === 'firing');
  const criticalCount = firingAlerts.filter((a) => a.severity === 'critical').length;
  const warningCount = firingAlerts.filter(
    (a) => a.severity === 'medium' || a.severity === 'high' || a.severity === 'low'
  ).length;
  const governanceCount = (pendingApprovals ?? []).length;

  const allClear = criticalCount === 0 && warningCount === 0 && governanceCount === 0;

  // Update "last checked" timer
  useEffect(() => {
    const interval = setInterval(() => {
      setSecondsAgo(Math.floor((Date.now() - lastChecked.getTime()) / 1000));
    }, 1000);
    return () => clearInterval(interval);
  }, [lastChecked]);

  // Reset timer when data refreshes
  useEffect(() => {
    setLastChecked(new Date());
    setSecondsAgo(0);
  }, [allAlerts, pendingApprovals]);

  const formatTimeAgo = (secs: number) => {
    if (secs < 60) return `${secs}s ago`;
    const mins = Math.floor(secs / 60);
    return `${mins}m ago`;
  };

  return (
    <button
      onClick={toggleDrawer}
      className="relative z-30 flex items-center justify-between w-full h-9 px-4 bg-surface-raised/90 backdrop-blur-sm border-t border-surface-border cursor-pointer hover:bg-surface-raised transition-colors select-none"
    >
      {/* Left: Status counts */}
      <div className="flex items-center gap-4 text-xs font-medium">
        {allClear ? (
          <div className="flex items-center gap-2 text-green-400">
            <span className="relative flex h-2 w-2">
              <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-green-400 opacity-75" />
              <span className="relative inline-flex rounded-full h-2 w-2 bg-green-400" />
            </span>
            All systems healthy
          </div>
        ) : (
          <>
            {criticalCount > 0 && (
              <div className="flex items-center gap-1.5 text-red-400">
                <AlertCircle className="w-3.5 h-3.5 animate-pulse" />
                <span>{criticalCount} critical</span>
              </div>
            )}
            {warningCount > 0 && (
              <div className="flex items-center gap-1.5 text-amber-400">
                <AlertTriangle className="w-3.5 h-3.5" />
                <span>{warningCount} warnings</span>
              </div>
            )}
            {governanceCount > 0 && (
              <div className="flex items-center gap-1.5 text-blue-400">
                <Shield className="w-3.5 h-3.5" />
                <span>{governanceCount} governance pending</span>
              </div>
            )}
          </>
        )}
      </div>

      {/* Right: Timestamp + expand icon */}
      <div className="flex items-center gap-3 text-xs text-gray-500">
        <div className="flex items-center gap-1.5">
          <Activity className="w-3 h-3" />
          <span>Last checked: {formatTimeAgo(secondsAgo)}</span>
        </div>
        <ChevronUp
          className={`w-3.5 h-3.5 text-gray-400 transition-transform duration-200 ${
            isDrawerOpen ? 'rotate-180' : ''
          }`}
        />
      </div>
    </button>
  );
}
