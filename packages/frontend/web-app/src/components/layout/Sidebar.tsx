import { NavLink } from 'react-router-dom';
import {
  LayoutDashboard,
  Server,
  Link2,
  Workflow,
  Database,
  Network,
  Settings,
  Activity,
  Bell,
  AlertOctagon,
  PlayCircle,
  Shield,
  FileCheck,
  BookOpen,
  CheckSquare,
  FileText,
  BarChart3,
  Layers,
  GitBranch,
  FolderKanban,
  Sparkles,
} from 'lucide-react';
import clsx from 'clsx';

const mainNavigation = [
  { name: 'Dashboard', href: '/dashboard', icon: LayoutDashboard },
  { name: 'Agents', href: '/agents', icon: Server },
  { name: 'Connections', href: '/connections', icon: Link2 },
  { name: 'Integrations', href: '/integrations', icon: Workflow },
  { name: 'Data Hub', href: '/data-hub', icon: Database },
  { name: 'Asset Registry', href: '/assets', icon: Network },
];

const operationsNavigation = [
  { name: 'Operations', href: '/operations', icon: Activity },
  { name: 'Alerts', href: '/operations/alerts', icon: Bell },
  { name: 'Incidents', href: '/operations/incidents', icon: AlertOctagon },
  { name: 'Playbooks', href: '/operations/playbooks', icon: PlayCircle },
];

const governanceNavigation = [
  { name: 'Governance', href: '/governance', icon: Shield },
  { name: 'Policies', href: '/governance/policies', icon: FileCheck },
  { name: 'Standards', href: '/governance/standards', icon: BookOpen },
  { name: 'Approvals', href: '/governance/approvals', icon: CheckSquare },
  { name: 'Audit Log', href: '/governance/audit', icon: FileText },
];

const rationalizationNavigation = [
  { name: 'Rationalization', href: '/rationalization', icon: BarChart3 },
  { name: 'Applications', href: '/rationalization/applications', icon: Layers },
  { name: 'Scenarios', href: '/rationalization/scenarios', icon: GitBranch },
  { name: 'Playbooks', href: '/rationalization/playbooks', icon: BookOpen },
  { name: 'Projects', href: '/rationalization/projects', icon: FolderKanban },
];

const aiNavigation = [
  { name: 'AI Assistant', href: '/ai', icon: Sparkles },
];

interface NavItemProps {
  item: { name: string; href: string; icon: React.ComponentType<{ className?: string }> };
}

function NavItem({ item }: NavItemProps) {
  return (
    <li>
      <NavLink
        to={item.href}
        end={item.href === '/operations' || item.href === '/governance'}
        className={({ isActive }) =>
          clsx(
            'flex items-center gap-3 px-3 py-2 rounded-lg text-sm font-medium transition-colors',
            isActive
              ? 'bg-primary-600 text-white'
              : 'text-gray-300 hover:bg-gray-800 hover:text-white'
          )
        }
      >
        <item.icon className="w-5 h-5" />
        {item.name}
      </NavLink>
    </li>
  );
}

interface NavSectionProps {
  title: string;
  items: typeof mainNavigation;
}

function NavSection({ title, items }: NavSectionProps) {
  return (
    <div className="mb-4">
      <h3 className="px-3 mb-2 text-xs font-semibold text-gray-500 uppercase tracking-wider">
        {title}
      </h3>
      <ul className="space-y-1">
        {items.map((item) => (
          <NavItem key={item.name} item={item} />
        ))}
      </ul>
    </div>
  );
}

export function Sidebar() {
  return (
    <aside className="w-60 bg-gray-900 text-white flex flex-col">
      {/* Logo */}
      <div className="h-16 flex items-center px-6 border-b border-gray-800">
        <span className="text-xl font-bold">Sysilo</span>
      </div>

      {/* Navigation */}
      <nav className="flex-1 py-4 px-3 overflow-y-auto">
        <NavSection title="Platform" items={mainNavigation} />
        <NavSection title="Operations" items={operationsNavigation} />
        <NavSection title="Governance" items={governanceNavigation} />
        <NavSection title="Rationalization" items={rationalizationNavigation} />
        <NavSection title="AI" items={aiNavigation} />
      </nav>

      {/* Settings */}
      <div className="p-3 border-t border-gray-800">
        <NavLink
          to="/settings"
          className={({ isActive }) =>
            clsx(
              'flex items-center gap-3 px-3 py-2 rounded-lg text-sm font-medium transition-colors',
              isActive
                ? 'bg-primary-600 text-white'
                : 'text-gray-300 hover:bg-gray-800 hover:text-white'
            )
          }
        >
          <Settings className="w-5 h-5" />
          Settings
        </NavLink>
      </div>
    </aside>
  );
}
