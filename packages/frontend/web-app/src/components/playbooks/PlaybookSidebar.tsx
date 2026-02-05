import { PlaybookToolbox } from './PlaybookToolbox';
import { PlaybookVariablesPanel } from './PlaybookVariablesPanel';
import type { Variable } from '@/services/playbooks';

interface PlaybookSidebarProps {
  variables: Variable[];
  onVariablesChange: (variables: Variable[]) => void;
}

export function PlaybookSidebar({ variables, onVariablesChange }: PlaybookSidebarProps) {
  return (
    <div className="w-64 bg-white border-r border-gray-200 flex flex-col overflow-hidden">
      <div className="flex-1 overflow-y-auto">
        <PlaybookToolbox />
        <PlaybookVariablesPanel variables={variables} onChange={onVariablesChange} />
      </div>
    </div>
  );
}
