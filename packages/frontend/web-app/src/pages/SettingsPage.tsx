import { usePlan } from '../hooks/usePlan';
import { useBillingPortal } from '../hooks/useBilling';
import { UsageMeter } from '../components/billing/UsageMeter';
import { PlanBadge } from '../components/billing/PlanBadge';
import { useNavigate } from 'react-router-dom';

export function SettingsPage() {
  const { planName, planStatus, isTrial, trialDaysLeft } = usePlan();
  const portal = useBillingPortal();
  const navigate = useNavigate();

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-white">Settings</h1>
        <p className="text-gray-400">Manage your workspace and preferences</p>
      </div>

      {/* Billing & Plan */}
      <div className="bg-surface-raised/80 backdrop-blur-glass rounded-xl shadow-glass border border-surface-border divide-y divide-surface-border">
        <div className="p-6">
          <h2 className="text-lg font-semibold text-white mb-4">Plan & Billing</h2>
          <div className="flex items-center justify-between mb-4">
            <div className="flex items-center gap-3">
              <span className="text-sm text-gray-400">Current plan:</span>
              <PlanBadge />
              {isTrial && trialDaysLeft !== null && (
                <span className="text-xs text-amber-600">{trialDaysLeft} days left</span>
              )}
            </div>
            <div className="flex items-center gap-2">
              <button
                onClick={() => navigate('/pricing')}
                className="px-4 py-2 text-sm font-medium text-white bg-primary-600 rounded-lg hover:bg-primary-700"
              >
                {planStatus === 'active' ? 'Change Plan' : 'Upgrade'}
              </button>
              {planStatus === 'active' && (
                <button
                  onClick={() => portal.mutate()}
                  disabled={portal.isPending}
                  className="px-4 py-2 text-sm font-medium text-gray-300 hover:bg-white/10 rounded-lg"
                >
                  Manage Billing
                </button>
              )}
            </div>
          </div>
          <UsageMeter />
        </div>
      </div>

      <div className="bg-surface-raised/80 backdrop-blur-glass rounded-xl shadow-glass border border-surface-border divide-y divide-surface-border">
        <div className="p-6">
          <h2 className="text-lg font-semibold text-white mb-4">General</h2>
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-300 mb-1">Workspace Name</label>
              <input
                type="text"
                defaultValue="Acme Corp"
                className="w-full max-w-md px-3 py-2 bg-surface-base/50 border border-surface-border rounded-lg text-sm text-gray-200 focus:border-primary-500/50 focus:ring-1 focus:ring-primary-500/20 outline-none"
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-300 mb-1">Timezone</label>
              <select className="w-full max-w-md px-3 py-2 bg-surface-base/50 border border-surface-border rounded-lg text-sm text-gray-200 focus:border-primary-500/50 focus:ring-1 focus:ring-primary-500/20 outline-none">
                <option>America/New_York (EST)</option>
                <option>America/Los_Angeles (PST)</option>
                <option>Europe/London (GMT)</option>
              </select>
            </div>
          </div>
        </div>

        <div className="p-6">
          <h2 className="text-lg font-semibold text-white mb-4">Notifications</h2>
          <div className="space-y-3">
            <label className="flex items-center gap-3">
              <input type="checkbox" defaultChecked className="rounded border-surface-border text-primary-600" />
              <span className="text-sm text-gray-300">Email alerts for failed integrations</span>
            </label>
            <label className="flex items-center gap-3">
              <input type="checkbox" defaultChecked className="rounded border-surface-border text-primary-600" />
              <span className="text-sm text-gray-300">Slack notifications for agent status changes</span>
            </label>
            <label className="flex items-center gap-3">
              <input type="checkbox" className="rounded border-surface-border text-primary-600" />
              <span className="text-sm text-gray-300">Weekly summary reports</span>
            </label>
          </div>
        </div>

        <div className="p-6">
          <h2 className="text-lg font-semibold text-white mb-4">API Access</h2>
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-300 mb-1">API Key</label>
              <div className="flex items-center gap-2">
                <input
                  type="password"
                  defaultValue="sk_live_xxxxxxxxxxxxx"
                  className="flex-1 max-w-md px-3 py-2 bg-surface-base/50 border border-surface-border rounded-lg text-sm font-mono text-gray-200 focus:border-primary-500/50 focus:ring-1 focus:ring-primary-500/20 outline-none"
                  readOnly
                />
                <button className="px-3 py-2 text-sm text-primary-600 hover:bg-primary-900/30 rounded-lg">
                  Reveal
                </button>
                <button className="px-3 py-2 text-sm text-primary-600 hover:bg-primary-900/30 rounded-lg">
                  Regenerate
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
