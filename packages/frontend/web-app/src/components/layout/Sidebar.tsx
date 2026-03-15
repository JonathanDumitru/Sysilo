import { useState } from 'react';
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
  Lock,
  Store,
  LayoutTemplate,
  Bot,
  Package,
  ShieldCheck,
  HeartPulse,
  X,
} from 'lucide-react';
import clsx from 'clsx';
import { usePlan } from '../../hooks/usePlan';
import { PlanBadge } from '../billing/PlanBadge';
import { UpgradeModal } from '../billing/UpgradeModal';

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

const platformNavigation = [
  { name: 'Marketplace', href: '/marketplace', icon: Store },
  { name: 'Templates', href: '/templates', icon: LayoutTemplate },
  { name: 'Agent Hub', href: '/agent-hub', icon: Bot },
  { name: 'Data Products', href: '/data-products', icon: Package },
  { name: 'Compliance API', href: '/governance/compliance-api', icon: ShieldCheck },
  { name: 'Immune System', href: '/operations/immune-system', icon: HeartPulse },
];

const aiNavigation = [
  { name: 'AI Assistant', href: '/ai', icon: Sparkles },
];

interface NavItemProps {
  item: { name: string; href: string; icon: React.ComponentType<{ className?: string }> };
  locked?: boolean;
  onLockedClick?: () => void;
  onNavigate?: () => void;
}

function NavItem({ item, locked, onLockedClick, onNavigate }: NavItemProps) {
  if (locked) {
    return (
      <li>
        <button
          onClick={onLockedClick}
          className="w-full flex items-center gap-3 px-3 py-2 rounded-lg text-sm font-medium text-gray-500 hover:bg-white/5 hover:text-gray-400 transition-colors"
        >
          <item.icon className="w-5 h-5" />
          {item.name}
          <Lock className="w-3.5 h-3.5 ml-auto" />
        </button>
      </li>
    );
  }

  return (
    <li>
      <NavLink
        to={item.href}
        end={item.href === '/operations' || item.href === '/governance'}
        onClick={onNavigate}
        className={({ isActive }) =>
          clsx(
            'flex items-center gap-3 px-3 py-2 rounded-lg text-sm font-medium transition-colors',
            isActive
              ? 'bg-primary-500/20 text-primary-400 border-l-2 border-primary-400'
              : 'text-gray-300 hover:bg-white/5 hover:text-white'
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
  lockedItems?: Set<string>;
  onLockedClick?: (href: string) => void;
  onNavigate?: () => void;
}

function NavSection({ title, items, lockedItems, onLockedClick, onNavigate }: NavSectionProps) {
  return (
    <div className="mb-4">
      <h3 className="px-3 mb-2 text-xs font-semibold text-gray-600 uppercase tracking-wider">
        {title}
      </h3>
      <ul className="space-y-1">
        {items.map((item) => (
          <NavItem
            key={item.name}
            item={item}
            locked={lockedItems?.has(item.href)}
            onLockedClick={() => onLockedClick?.(item.href)}
            onNavigate={onNavigate}
          />
        ))}
      </ul>
    </div>
  );
}

// Map route prefixes to the feature key that gates them
const routeFeatureMap: Record<string, string> = {
  '/governance': 'governance_enabled',
  '/rationalization': 'rationalization_enabled',
  '/ai': 'ai_enabled',
};

// Map route prefixes to the minimum required plan
const routePlanMap: Record<string, string> = {
  '/governance': 'business',
  '/rationalization': 'enterprise',
  '/ai': 'business',
};

interface SidebarProps {
  onClose?: () => void;
}

export function Sidebar({ onClose }: SidebarProps) {
  const { hasFeature } = usePlan();
  const [upgradeModal, setUpgradeModal] = useState<{ feature: string; plan: string } | null>(null);

  // Build set of locked nav hrefs
  const lockedItems = new Set<string>();
  for (const section of [governanceNavigation, rationalizationNavigation, aiNavigation]) {
    for (const item of section) {
      const prefix = Object.keys(routeFeatureMap).find((p) => item.href.startsWith(p));
      if (prefix && !hasFeature(routeFeatureMap[prefix] as any)) {
        lockedItems.add(item.href);
      }
    }
  }

  const handleLockedClick = (href: string) => {
    const prefix = Object.keys(routeFeatureMap).find((p) => href.startsWith(p));
    if (prefix) {
      setUpgradeModal({
        feature: routeFeatureMap[prefix],
        plan: routePlanMap[prefix] || 'business',
      });
    }
  };

  return (
    <aside className="w-64 lg:w-60 h-full bg-surface-raised/95 backdrop-blur-glass border-r border-surface-border text-white flex flex-col">
      {/* Logo */}
      <div className="h-14 md:h-16 flex items-center justify-between px-4 md:px-6 border-b border-surface-border">
        <span className="text-xl font-bold text-primary-400 drop-shadow-[0_0_8px_rgba(56,189,248,0.3)]">Sysilo</span>
        <div className="flex items-center gap-2">
          <PlanBadge />
          {/* Mobile close button */}
          <button
            onClick={onClose}
            className="p-1.5 text-gray-400 hover:text-gray-200 lg:hidden"
          >
            <X className="w-5 h-5" />
          </button>
        </div>
      </div>

      {/* Navigation */}
      <nav className="flex-1 py-4 px-3 overflow-y-auto">
        <NavSection title="Core" items={mainNavigation} onNavigate={onClose} />
        <NavSection title="Operations" items={operationsNavigation} onNavigate={onClose} />
        <NavSection title="Governance" items={governanceNavigation} lockedItems={lockedItems} onLockedClick={handleLockedClick} onNavigate={onClose} />
        <NavSection title="Rationalization" items={rationalizationNavigation} lockedItems={lockedItems} onLockedClick={handleLockedClick} onNavigate={onClose} />
        <NavSection title="Platform" items={platformNavigation} onNavigate={onClose} />
        <NavSection title="AI" items={aiNavigation} lockedItems={lockedItems} onLockedClick={handleLockedClick} onNavigate={onClose} />
      </nav>

      {/* Settings */}
      <div className="p-3 border-t border-surface-border">
        <NavLink
          to="/settings"
          onClick={onClose}
          className={({ isActive }) =>
            clsx(
              'flex items-center gap-3 px-3 py-2 rounded-lg text-sm font-medium transition-colors',
              isActive
                ? 'bg-primary-500/20 text-primary-400 border-l-2 border-primary-400'
                : 'text-gray-300 hover:bg-white/5 hover:text-white'
            )
          }
        >
          <Settings className="w-5 h-5" />
          Settings
        </NavLink>
      </div>

      {/* Upgrade Modal */}
      {upgradeModal && (
        <UpgradeModal
          isOpen={true}
          onClose={() => setUpgradeModal(null)}
          feature={upgradeModal.feature}
          requiredPlan={upgradeModal.plan}
        />
      )}
    </aside>
  );
}
