import { useNavigate } from 'react-router-dom';
import {
  useKeyboardShortcuts,
  useKeySequences,
} from '../hooks/useKeyboardShortcuts';
import { useCommandPalette, useKeyboardShortcutsHelp } from '../hooks/useCommandPalette';

/**
 * Invisible component that registers all global keyboard shortcuts
 * and renders the pending-key indicator.
 */
export function GlobalShortcuts() {
  const navigate = useNavigate();
  const { close: closePalette } = useCommandPalette();
  const { close: closeHelp, toggle: toggleHelp } = useKeyboardShortcutsHelp();

  // Single-key shortcuts (besides Cmd+K which is in CommandPalette)
  useKeyboardShortcuts([
    {
      key: '?',
      shift: true,
      handler: toggleHelp,
      description: 'Show keyboard shortcuts help',
    },
    {
      key: 'Escape',
      handler: () => {
        closePalette();
        closeHelp();
      },
      description: 'Close any open panel',
    },
  ]);

  // Two-key navigation sequences
  const pendingKey = useKeySequences([
    {
      keys: ['g', 'd'],
      handler: () => navigate('/dashboard'),
      description: 'Go to Dashboard',
    },
    {
      keys: ['g', 'i'],
      handler: () => navigate('/integrations'),
      description: 'Go to Integrations',
    },
    {
      keys: ['g', 'c'],
      handler: () => navigate('/connections'),
      description: 'Go to Connections',
    },
    {
      keys: ['g', 'a'],
      handler: () => navigate('/assets'),
      description: 'Go to Assets',
    },
    {
      keys: ['g', 'h'],
      handler: () => navigate('/data-hub'),
      description: 'Go to Data Hub',
    },
    {
      keys: ['g', 'n'],
      handler: () => navigate('/agents'),
      description: 'Go to Agents',
    },
    {
      keys: ['g', 'o'],
      handler: () => navigate('/operations'),
      description: 'Go to Operations',
    },
    {
      keys: ['g', 'g'],
      handler: () => navigate('/governance'),
      description: 'Go to Governance',
    },
    {
      keys: ['g', 'r'],
      handler: () => navigate('/rationalization'),
      description: 'Go to Rationalization',
    },
    {
      keys: ['g', 's'],
      handler: () => navigate('/settings'),
      description: 'Go to Settings',
    },
  ]);

  // Pending key indicator
  if (!pendingKey) return null;

  return (
    <div className="fixed bottom-6 left-6 z-50 animate-fade-in">
      <div className="glass-panel px-3 py-2 flex items-center gap-2">
        <kbd className="bg-surface-overlay text-gray-400 border border-surface-border rounded px-2 py-1 font-mono text-sm">
          {pendingKey.toUpperCase()}
        </kbd>
        <span className="text-gray-500 text-sm">...</span>
      </div>
    </div>
  );
}
