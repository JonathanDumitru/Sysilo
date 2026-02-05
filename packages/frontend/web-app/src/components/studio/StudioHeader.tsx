import { Save, Play, ArrowLeft, Settings } from 'lucide-react';
import { Link } from 'react-router-dom';

interface StudioHeaderProps {
  name: string;
  onNameChange: (name: string) => void;
  onSave: () => void;
  onRun: () => void;
}

export function StudioHeader({ name, onNameChange, onSave, onRun }: StudioHeaderProps) {
  return (
    <div className="h-14 bg-white border-b border-gray-200 flex items-center justify-between px-4">
      {/* Left side */}
      <div className="flex items-center gap-4">
        <Link
          to="/integrations"
          className="p-2 text-gray-400 hover:text-gray-600 hover:bg-gray-100 rounded-lg transition-colors"
        >
          <ArrowLeft className="w-5 h-5" />
        </Link>

        <input
          type="text"
          value={name}
          onChange={(e) => onNameChange(e.target.value)}
          className="text-lg font-semibold text-gray-900 bg-transparent border-none outline-none focus:ring-2 focus:ring-primary-100 rounded px-2 -ml-2"
        />
      </div>

      {/* Right side */}
      <div className="flex items-center gap-2">
        <button className="flex items-center gap-2 px-3 py-1.5 text-gray-600 hover:bg-gray-100 rounded-lg transition-colors">
          <Settings className="w-4 h-4" />
          Settings
        </button>

        <div className="w-px h-6 bg-gray-200" />

        <button
          onClick={onSave}
          className="flex items-center gap-2 px-3 py-1.5 text-gray-600 hover:bg-gray-100 rounded-lg transition-colors"
        >
          <Save className="w-4 h-4" />
          Save
        </button>

        <button
          onClick={onRun}
          className="flex items-center gap-2 px-4 py-1.5 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors"
        >
          <Play className="w-4 h-4" />
          Run
        </button>
      </div>
    </div>
  );
}
