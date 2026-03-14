import { useEffect, useCallback } from 'react';
import { X } from 'lucide-react';
import { useKeyboardShortcutsHelp } from '../hooks/useCommandPalette';

interface ShortcutEntry {
  keys: string[];
  description: string;
}

interface ShortcutGroup {
  title: string;
  shortcuts: ShortcutEntry[];
}

const SHORTCUT_GROUPS: ShortcutGroup[] = [
  {
    title: 'General',
    shortcuts: [
      { keys: ['\u2318', 'K'], description: 'Open command palette' },
      { keys: ['Shift', '?'], description: 'Show keyboard shortcuts' },
      { keys: ['Esc'], description: 'Close panel / modal' },
    ],
  },
  {
    title: 'Navigation',
    shortcuts: [
      { keys: ['G', 'D'], description: 'Go to Dashboard' },
      { keys: ['G', 'I'], description: 'Go to Integrations' },
      { keys: ['G', 'C'], description: 'Go to Connections' },
      { keys: ['G', 'A'], description: 'Go to Assets' },
      { keys: ['G', 'H'], description: 'Go to Data Hub' },
      { keys: ['G', 'N'], description: 'Go to Agents' },
      { keys: ['G', 'O'], description: 'Go to Operations' },
      { keys: ['G', 'G'], description: 'Go to Governance' },
      { keys: ['G', 'R'], description: 'Go to Rationalization' },
      { keys: ['G', 'S'], description: 'Go to Settings' },
    ],
  },
];

export function KeyboardShortcutsHelp() {
  const { isOpen, close } = useKeyboardShortcutsHelp();

  // Close on Escape
  useEffect(() => {
    if (!isOpen) return;
    function handleKeyDown(e: KeyboardEvent) {
      if (e.key === 'Escape') {
        e.preventDefault();
        close();
      }
    }
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [isOpen, close]);

  const handleOverlayClick = useCallback(
    (e: React.MouseEvent) => {
      if (e.target === e.currentTarget) close();
    },
    [close]
  );

  if (!isOpen) return null;

  return (
    <div
      className="fixed inset-0 bg-black/60 backdrop-blur-sm z-50 flex items-center justify-center"
      onClick={handleOverlayClick}
    >
      <div className="glass-panel-strong max-w-lg w-full mx-4 overflow-hidden">
        {/* Header */}
        <div className="flex items-center justify-between px-5 py-4 border-b border-surface-border">
          <h2 className="text-base font-semibold text-gray-200">
            Keyboard Shortcuts
          </h2>
          <button
            onClick={close}
            className="p-1 text-gray-500 hover:text-gray-300 transition-colors rounded"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* Shortcuts grid */}
        <div className="px-5 py-4 max-h-[60vh] overflow-y-auto space-y-6">
          {SHORTCUT_GROUPS.map((group) => (
            <div key={group.title}>
              <h3 className="text-xs text-gray-600 uppercase tracking-wider mb-3 select-none">
                {group.title}
              </h3>
              <div className="space-y-2">
                {group.shortcuts.map((shortcut, i) => (
                  <div
                    key={i}
                    className="flex items-center justify-between py-1"
                  >
                    <span className="text-sm text-gray-300">
                      {shortcut.description}
                    </span>
                    <div className="flex items-center gap-1">
                      {shortcut.keys.map((key, j) => (
                        <span key={j}>
                          <kbd className="bg-surface-overlay text-gray-500 border border-surface-border rounded px-1.5 py-0.5 font-mono text-xs min-w-[24px] text-center inline-block">
                            {key}
                          </kbd>
                          {j < shortcut.keys.length - 1 && (
                            <span className="text-gray-700 mx-0.5 text-xs">
                              then
                            </span>
                          )}
                        </span>
                      ))}
                    </div>
                  </div>
                ))}
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
