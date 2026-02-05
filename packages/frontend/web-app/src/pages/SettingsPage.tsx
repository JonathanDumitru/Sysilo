export function SettingsPage() {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-gray-900">Settings</h1>
        <p className="text-gray-500">Manage your workspace and preferences</p>
      </div>

      <div className="bg-white rounded-xl shadow-sm border border-gray-100 divide-y divide-gray-100">
        <div className="p-6">
          <h2 className="text-lg font-semibold text-gray-900 mb-4">General</h2>
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">Workspace Name</label>
              <input
                type="text"
                defaultValue="Acme Corp"
                className="w-full max-w-md px-3 py-2 border border-gray-300 rounded-lg text-sm"
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">Timezone</label>
              <select className="w-full max-w-md px-3 py-2 border border-gray-300 rounded-lg text-sm">
                <option>America/New_York (EST)</option>
                <option>America/Los_Angeles (PST)</option>
                <option>Europe/London (GMT)</option>
              </select>
            </div>
          </div>
        </div>

        <div className="p-6">
          <h2 className="text-lg font-semibold text-gray-900 mb-4">Notifications</h2>
          <div className="space-y-3">
            <label className="flex items-center gap-3">
              <input type="checkbox" defaultChecked className="rounded border-gray-300 text-primary-600" />
              <span className="text-sm text-gray-700">Email alerts for failed integrations</span>
            </label>
            <label className="flex items-center gap-3">
              <input type="checkbox" defaultChecked className="rounded border-gray-300 text-primary-600" />
              <span className="text-sm text-gray-700">Slack notifications for agent status changes</span>
            </label>
            <label className="flex items-center gap-3">
              <input type="checkbox" className="rounded border-gray-300 text-primary-600" />
              <span className="text-sm text-gray-700">Weekly summary reports</span>
            </label>
          </div>
        </div>

        <div className="p-6">
          <h2 className="text-lg font-semibold text-gray-900 mb-4">API Access</h2>
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">API Key</label>
              <div className="flex items-center gap-2">
                <input
                  type="password"
                  defaultValue="sk_live_xxxxxxxxxxxxx"
                  className="flex-1 max-w-md px-3 py-2 border border-gray-300 rounded-lg text-sm font-mono"
                  readOnly
                />
                <button className="px-3 py-2 text-sm text-primary-600 hover:bg-primary-50 rounded-lg">
                  Reveal
                </button>
                <button className="px-3 py-2 text-sm text-primary-600 hover:bg-primary-50 rounded-lg">
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
