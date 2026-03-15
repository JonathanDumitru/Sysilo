import { Bell, Search, User } from 'lucide-react';
import { useCommandPalette } from '../../hooks/useCommandPalette';

export function Header() {
  const { open } = useCommandPalette();

  return (
    <header className="h-16 bg-surface-raised/80 backdrop-blur-glass border-b border-surface-border flex items-center justify-between px-6">
      {/* Search — opens command palette */}
      <div className="flex items-center gap-2 w-96">
        <button
          onClick={open}
          className="flex items-center gap-2 glass-input w-full text-left cursor-pointer hover:border-surface-border-strong transition-colors"
        >
          <Search className="w-4 h-4 text-gray-500" />
          <span className="text-sm flex-1 text-gray-500">
            Search integrations, agents, assets...
          </span>
          <kbd className="text-xs text-gray-500 bg-surface-overlay border border-surface-border px-1.5 py-0.5 rounded font-mono">⌘K</kbd>
        </button>
      </div>

      {/* Right side */}
      <div className="flex items-center gap-4">
        {/* Notifications */}
        <button className="relative p-2 text-gray-400 hover:text-gray-200 transition-colors">
          <Bell className="w-5 h-5" />
          <span className="absolute top-1 right-1 w-2 h-2 bg-status-critical rounded-full shadow-[0_0_6px_rgba(248,81,73,0.5)]" />
        </button>

        {/* User menu */}
        <button className="flex items-center gap-2 px-2 py-1 rounded-lg hover:bg-white/5 transition-colors">
          <div className="w-8 h-8 bg-primary-500/20 rounded-full flex items-center justify-center">
            <User className="w-4 h-4 text-primary-400" />
          </div>
          <span className="text-sm font-medium text-gray-300">Admin</span>
        </button>
      </div>
    </header>
  );
}
