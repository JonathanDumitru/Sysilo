import { useState, useEffect, useRef, useMemo, useCallback } from 'react';
import { useNavigate } from 'react-router-dom';
import {
  Search,
  LayoutDashboard,
  Workflow,
  Link2,
  Network,
  Database,
  Server,
  Activity,
  Shield,
  BarChart3,
  Settings,
  FileText,
  FileCheck,
  Bell,
  AlertOctagon,
  Plus,
  PlayCircle,
  CheckSquare,
  Sparkles,
  Eye,
  ArrowRight,
} from 'lucide-react';
import { useCommandPalette } from '../hooks/useCommandPalette';
import { useKeyboardShortcuts } from '../hooks/useKeyboardShortcuts';

// ─── Types ───────────────────────────────────────────────────────────────────

interface Command {
  id: string;
  label: string;
  description?: string;
  icon: React.ComponentType<{ className?: string }>;
  category: 'navigation' | 'action' | 'recent' | 'ai';
  shortcut?: string;
  action: () => void;
  keywords?: string[];
}

// ─── Fuzzy search ────────────────────────────────────────────────────────────

function fuzzyMatch(query: string, text: string): boolean {
  const lowerQuery = query.toLowerCase();
  const lowerText = text.toLowerCase();

  // Direct substring match
  if (lowerText.includes(lowerQuery)) return true;

  // Fuzzy: every character in query appears in order in text
  let qi = 0;
  for (let ti = 0; ti < lowerText.length && qi < lowerQuery.length; ti++) {
    if (lowerText[ti] === lowerQuery[qi]) qi++;
  }
  return qi === lowerQuery.length;
}

function fuzzyScore(query: string, text: string): number {
  const lowerQuery = query.toLowerCase();
  const lowerText = text.toLowerCase();

  // Exact match gets highest score
  if (lowerText === lowerQuery) return 100;
  // Starts with query
  if (lowerText.startsWith(lowerQuery)) return 90;
  // Contains query as substring
  if (lowerText.includes(lowerQuery)) return 80;
  // Word boundary match
  const words = lowerText.split(/\s+/);
  if (words.some((w) => w.startsWith(lowerQuery))) return 70;
  // Fuzzy match
  return 50;
}

// ─── Category labels ─────────────────────────────────────────────────────────

const CATEGORY_LABELS: Record<string, string> = {
  navigation: 'Navigation',
  action: 'Actions',
  recent: 'Recent',
  ai: 'AI',
};

const CATEGORY_ORDER = ['recent', 'navigation', 'action', 'ai'];

// ─── Component ───────────────────────────────────────────────────────────────

export function CommandPalette() {
  const { isOpen, close, toggle } = useCommandPalette();
  const navigate = useNavigate();
  const [query, setQuery] = useState('');
  const [selectedIndex, setSelectedIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const listRef = useRef<HTMLDivElement>(null);

  // Register Cmd+K / Ctrl+K shortcut
  useKeyboardShortcuts([
    {
      key: 'k',
      meta: true,
      handler: toggle,
      description: 'Toggle command palette',
    },
  ]);

  // Build commands list using navigate
  const commands = useMemo<Command[]>(() => {
    const nav = (path: string) => () => {
      navigate(path);
      close();
    };

    return [
      // ── Navigation ──
      {
        id: 'nav-dashboard',
        label: 'Go to Dashboard',
        description: 'Overview and metrics',
        icon: LayoutDashboard,
        category: 'navigation',
        shortcut: 'G D',
        action: nav('/dashboard'),
        keywords: ['home', 'overview', 'metrics'],
      },
      {
        id: 'nav-integrations',
        label: 'Go to Integrations',
        description: 'Manage integrations and workflows',
        icon: Workflow,
        category: 'navigation',
        shortcut: 'G I',
        action: nav('/integrations'),
        keywords: ['workflow', 'api', 'sync'],
      },
      {
        id: 'nav-connections',
        label: 'Go to Connections',
        description: 'Data source connections',
        icon: Link2,
        category: 'navigation',
        shortcut: 'G C',
        action: nav('/connections'),
        keywords: ['source', 'database', 'connector'],
      },
      {
        id: 'nav-assets',
        label: 'Go to Assets',
        description: 'Asset registry and inventory',
        icon: Network,
        category: 'navigation',
        shortcut: 'G A',
        action: nav('/assets'),
        keywords: ['registry', 'inventory', 'catalog'],
      },
      {
        id: 'nav-datahub',
        label: 'Go to Data Hub',
        description: 'Centralized data management',
        icon: Database,
        category: 'navigation',
        shortcut: 'G H',
        action: nav('/data-hub'),
        keywords: ['data', 'hub', 'storage'],
      },
      {
        id: 'nav-agents',
        label: 'Go to Agents',
        description: 'Agent management',
        icon: Server,
        category: 'navigation',
        shortcut: 'G N',
        action: nav('/agents'),
        keywords: ['bot', 'automation', 'worker'],
      },
      {
        id: 'nav-operations',
        label: 'Go to Operations',
        description: 'Operations center',
        icon: Activity,
        category: 'navigation',
        shortcut: 'G O',
        action: nav('/operations'),
        keywords: ['ops', 'monitoring', 'health'],
      },
      {
        id: 'nav-governance',
        label: 'Go to Governance',
        description: 'Governance center',
        icon: Shield,
        category: 'navigation',
        shortcut: 'G G',
        action: nav('/governance'),
        keywords: ['compliance', 'policy', 'audit'],
      },
      {
        id: 'nav-rationalization',
        label: 'Go to Rationalization',
        description: 'Rationalization engine',
        icon: BarChart3,
        category: 'navigation',
        shortcut: 'G R',
        action: nav('/rationalization'),
        keywords: ['portfolio', 'analysis', 'optimize'],
      },
      {
        id: 'nav-settings',
        label: 'Go to Settings',
        description: 'Application settings',
        icon: Settings,
        category: 'navigation',
        shortcut: 'G S',
        action: nav('/settings'),
        keywords: ['preferences', 'config', 'profile'],
      },
      {
        id: 'nav-audit',
        label: 'Go to Audit Log',
        description: 'View audit trail',
        icon: FileText,
        category: 'navigation',
        action: nav('/governance/audit'),
        keywords: ['log', 'trail', 'history'],
      },
      {
        id: 'nav-policies',
        label: 'Go to Policies',
        description: 'Manage governance policies',
        icon: FileCheck,
        category: 'navigation',
        action: nav('/governance/policies'),
        keywords: ['rules', 'compliance'],
      },
      {
        id: 'nav-alerts',
        label: 'Go to Alerts',
        description: 'View and manage alerts',
        icon: Bell,
        category: 'navigation',
        action: nav('/operations/alerts'),
        keywords: ['notification', 'warning'],
      },
      {
        id: 'nav-incidents',
        label: 'Go to Incidents',
        description: 'Track active incidents',
        icon: AlertOctagon,
        category: 'navigation',
        action: nav('/operations/incidents'),
        keywords: ['issue', 'outage', 'problem'],
      },

      // ── Actions ──
      {
        id: 'action-create-integration',
        label: 'Create Integration',
        description: 'Set up a new integration',
        icon: Plus,
        category: 'action',
        action: nav('/integrations/new'),
        keywords: ['new', 'add', 'workflow'],
      },
      {
        id: 'action-create-connection',
        label: 'Create Connection',
        description: 'Add a new data source connection',
        icon: Plus,
        category: 'action',
        action: nav('/connections'),
        keywords: ['new', 'add', 'source'],
      },
      {
        id: 'action-create-playbook',
        label: 'Create Playbook',
        description: 'Build a new automation playbook',
        icon: Plus,
        category: 'action',
        action: nav('/operations/playbooks/new'),
        keywords: ['new', 'automation', 'workflow'],
      },
      {
        id: 'action-run-playbook',
        label: 'Run Playbook',
        description: 'Execute an automation playbook',
        icon: PlayCircle,
        category: 'action',
        action: nav('/operations/playbooks'),
        keywords: ['execute', 'start', 'automation'],
      },
      {
        id: 'action-review-governance',
        label: 'Review Governance Alerts',
        description: 'Review pending governance alerts',
        icon: Shield,
        category: 'action',
        action: nav('/governance'),
        keywords: ['compliance', 'review', 'check'],
      },
      {
        id: 'action-approve-policy',
        label: 'Approve Policy Request',
        description: 'Review and approve pending requests',
        icon: CheckSquare,
        category: 'action',
        action: nav('/governance/approvals'),
        keywords: ['approve', 'request', 'pending'],
      },
      {
        id: 'action-create-alert-rule',
        label: 'Create Alert Rule',
        description: 'Define a new alerting rule',
        icon: Bell,
        category: 'action',
        action: nav('/operations/alerts'),
        keywords: ['new', 'notification', 'rule'],
      },
      {
        id: 'action-create-incident',
        label: 'Create Incident',
        description: 'Report a new incident',
        icon: AlertOctagon,
        category: 'action',
        action: nav('/operations/incidents'),
        keywords: ['new', 'report', 'issue'],
      },

      // ── AI ──
      {
        id: 'ai-assistant',
        label: 'Ask AI Assistant',
        description: 'Get help from the AI assistant',
        icon: Sparkles,
        category: 'ai',
        action: () => {
          close();
          // Click the AI assist button to open the chat panel
          const aiButton = document.querySelector(
            'button.fixed.bottom-6.right-6'
          ) as HTMLButtonElement | null;
          if (aiButton) aiButton.click();
        },
        keywords: ['chat', 'help', 'question', 'copilot'],
      },
      {
        id: 'ai-explain',
        label: 'Explain Current View',
        description: 'Ask AI to explain this page',
        icon: Eye,
        category: 'ai',
        action: () => {
          close();
          const aiButton = document.querySelector(
            'button.fixed.bottom-6.right-6'
          ) as HTMLButtonElement | null;
          if (aiButton) aiButton.click();
        },
        keywords: ['what', 'explain', 'help', 'understand'],
      },
    ];
  }, [navigate, close]);

  // Filter and sort commands
  const filteredCommands = useMemo(() => {
    if (!query.trim()) return commands;

    return commands
      .filter((cmd) => {
        const searchText = [
          cmd.label,
          cmd.description || '',
          ...(cmd.keywords || []),
        ].join(' ');
        return fuzzyMatch(query, searchText);
      })
      .sort((a, b) => {
        const scoreA = fuzzyScore(query, a.label);
        const scoreB = fuzzyScore(query, b.label);
        return scoreB - scoreA;
      });
  }, [commands, query]);

  // Group by category
  const groupedCommands = useMemo(() => {
    const groups: Record<string, Command[]> = {};
    for (const cmd of filteredCommands) {
      if (!groups[cmd.category]) groups[cmd.category] = [];
      groups[cmd.category].push(cmd);
    }
    return CATEGORY_ORDER
      .filter((cat) => groups[cat]?.length)
      .map((cat) => ({ category: cat, commands: groups[cat] }));
  }, [filteredCommands]);

  // Flat list for keyboard navigation
  const flatCommands = useMemo(
    () => groupedCommands.flatMap((g) => g.commands),
    [groupedCommands]
  );

  // Reset state when opening/closing
  useEffect(() => {
    if (isOpen) {
      setQuery('');
      setSelectedIndex(0);
      // Focus input on next tick
      requestAnimationFrame(() => inputRef.current?.focus());
    }
  }, [isOpen]);

  // Reset selection when query changes
  useEffect(() => {
    setSelectedIndex(0);
  }, [query]);

  // Scroll selected item into view
  useEffect(() => {
    if (!listRef.current) return;
    const selected = listRef.current.querySelector('[data-selected="true"]');
    if (selected) {
      selected.scrollIntoView({ block: 'nearest' });
    }
  }, [selectedIndex]);

  // Keyboard navigation within the palette
  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      switch (e.key) {
        case 'ArrowDown':
          e.preventDefault();
          setSelectedIndex((i) => Math.min(i + 1, flatCommands.length - 1));
          break;
        case 'ArrowUp':
          e.preventDefault();
          setSelectedIndex((i) => Math.max(i - 1, 0));
          break;
        case 'Enter':
          e.preventDefault();
          if (flatCommands[selectedIndex]) {
            flatCommands[selectedIndex].action();
          } else if (query.trim()) {
            // No results — route to AI
            close();
            const aiButton = document.querySelector(
              'button.fixed.bottom-6.right-6'
            ) as HTMLButtonElement | null;
            if (aiButton) aiButton.click();
          }
          break;
        case 'Escape':
          e.preventDefault();
          close();
          break;
      }
    },
    [flatCommands, selectedIndex, close, query]
  );

  // Close on overlay click
  const handleOverlayClick = useCallback(
    (e: React.MouseEvent) => {
      if (e.target === e.currentTarget) close();
    },
    [close]
  );

  if (!isOpen) return null;

  let itemIndex = -1;

  return (
    <div
      className="fixed inset-0 bg-black/60 backdrop-blur-sm z-50 flex justify-center"
      onClick={handleOverlayClick}
    >
      <div className="glass-panel-strong max-w-xl w-full mx-4 mt-[20vh] max-h-[60vh] flex flex-col overflow-hidden">
        {/* Search input */}
        <div className="flex items-center gap-3 px-4 py-3 border-b border-surface-border">
          <Search className="w-5 h-5 text-gray-500 flex-shrink-0" />
          <input
            ref={inputRef}
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Type a command or search..."
            className="flex-1 bg-transparent border-none outline-none text-lg text-gray-200 placeholder-gray-500"
          />
          <kbd className="text-xs text-gray-500 bg-surface-overlay border border-surface-border rounded px-1.5 py-0.5 font-mono flex-shrink-0">
            ESC
          </kbd>
        </div>

        {/* Results */}
        <div ref={listRef} className="flex-1 overflow-y-auto py-2">
          {groupedCommands.length === 0 ? (
            <div className="px-4 py-8 text-center">
              <p className="text-gray-500 text-sm">No results found.</p>
              <p className="text-gray-600 text-xs mt-1">
                Press{' '}
                <kbd className="bg-surface-overlay border border-surface-border rounded px-1 py-0.5 font-mono text-xs">
                  Enter
                </kbd>{' '}
                to ask AI...
              </p>
            </div>
          ) : (
            groupedCommands.map(({ category, commands: cmds }) => (
              <div key={category}>
                <div className="text-xs text-gray-600 uppercase tracking-wider px-4 py-2 select-none">
                  {CATEGORY_LABELS[category]}
                </div>
                {cmds.map((cmd) => {
                  itemIndex++;
                  const isSelected = itemIndex === selectedIndex;
                  const currentIndex = itemIndex;
                  return (
                    <button
                      key={cmd.id}
                      data-selected={isSelected}
                      onClick={() => cmd.action()}
                      onMouseEnter={() => setSelectedIndex(currentIndex)}
                      className={`w-full flex items-center gap-3 px-4 py-2.5 text-left transition-colors ${
                        isSelected
                          ? 'bg-primary-500/20'
                          : 'hover:bg-white/5'
                      }`}
                    >
                      <cmd.icon
                        className={`w-4 h-4 flex-shrink-0 ${
                          isSelected ? 'text-primary-400' : 'text-gray-500'
                        }`}
                      />
                      <div className="flex-1 min-w-0">
                        <div className="text-sm text-gray-200 truncate">
                          {cmd.label}
                        </div>
                        {cmd.description && (
                          <div className="text-xs text-gray-500 truncate">
                            {cmd.description}
                          </div>
                        )}
                      </div>
                      {cmd.shortcut && (
                        <div className="flex-shrink-0 flex items-center gap-1">
                          {cmd.shortcut.split(' ').map((k, i) => (
                            <kbd
                              key={i}
                              className="bg-surface-overlay text-gray-500 border border-surface-border rounded px-1.5 py-0.5 font-mono text-xs"
                            >
                              {k}
                            </kbd>
                          ))}
                        </div>
                      )}
                      {isSelected && !cmd.shortcut && (
                        <ArrowRight className="w-3.5 h-3.5 text-gray-600 flex-shrink-0" />
                      )}
                    </button>
                  );
                })}
              </div>
            ))
          )}
        </div>

        {/* Footer */}
        <div className="flex items-center gap-4 px-4 py-2 border-t border-surface-border text-xs text-gray-600">
          <span className="flex items-center gap-1">
            <kbd className="bg-surface-overlay border border-surface-border rounded px-1 py-0.5 font-mono">
              &uarr;&darr;
            </kbd>
            navigate
          </span>
          <span className="flex items-center gap-1">
            <kbd className="bg-surface-overlay border border-surface-border rounded px-1 py-0.5 font-mono">
              &crarr;
            </kbd>
            select
          </span>
          <span className="flex items-center gap-1">
            <kbd className="bg-surface-overlay border border-surface-border rounded px-1 py-0.5 font-mono">
              esc
            </kbd>
            close
          </span>
        </div>
      </div>
    </div>
  );
}
