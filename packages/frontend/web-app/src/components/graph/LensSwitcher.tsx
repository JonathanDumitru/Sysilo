import { Activity, Shield, BarChart3, GitBranch } from 'lucide-react';
import type { GraphLens } from '../../hooks/useTopologyGraph.js';

const LENSES: { id: GraphLens; label: string; icon: React.ElementType }[] = [
  { id: 'health', label: 'Health', icon: Activity },
  { id: 'governance', label: 'Governance', icon: Shield },
  { id: 'time', label: 'TIME Score', icon: BarChart3 },
  { id: 'lineage', label: 'Lineage', icon: GitBranch },
];

interface LensSwitcherProps {
  activeLens: GraphLens;
  onLensChange: (lens: GraphLens) => void;
}

export function LensSwitcher({ activeLens, onLensChange }: LensSwitcherProps) {
  return (
    <div className="glass-panel p-1 flex gap-0.5">
      {LENSES.map(({ id, label, icon: Icon }) => {
        const isActive = activeLens === id;
        return (
          <button
            key={id}
            onClick={() => onLensChange(id)}
            className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium transition-all duration-200 ${
              isActive
                ? 'bg-primary-600/30 text-primary-300 shadow-[0_0_12px_rgba(14,165,233,0.2)] border border-primary-500/30'
                : 'text-gray-400 hover:text-gray-200 hover:bg-surface-overlay/50 border border-transparent'
            }`}
          >
            <Icon className="w-3.5 h-3.5" />
            <span>{label}</span>
          </button>
        );
      })}
    </div>
  );
}
