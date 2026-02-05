import { useState } from 'react';
import { ChevronDown, ChevronRight, Plus, Trash2 } from 'lucide-react';
import type { Variable } from '@/services/playbooks';

const VARIABLE_TYPES = ['string', 'number', 'boolean'] as const;

const typeColors: Record<string, string> = {
  string: 'bg-emerald-100 text-emerald-700',
  number: 'bg-blue-100 text-blue-700',
  boolean: 'bg-purple-100 text-purple-700',
};

// Validation regex for variable names
const VALID_NAME_REGEX = /^[a-zA-Z_][a-zA-Z0-9_]*$/;

interface PlaybookVariablesPanelProps {
  variables: Variable[];
  onChange: (variables: Variable[]) => void;
}

export function PlaybookVariablesPanel({ variables, onChange }: PlaybookVariablesPanelProps) {
  const [isExpanded, setIsExpanded] = useState(true);

  const addVariable = () => {
    const newVar: Variable = {
      name: '',
      var_type: 'string',
      required: false,
      default_value: undefined,
    };
    onChange([...variables, newVar]);
  };

  const updateVariable = (index: number, updates: Partial<Variable>) => {
    const updated = variables.map((v, i) => (i === index ? { ...v, ...updates } : v));
    onChange(updated);
  };

  const deleteVariable = (index: number) => {
    onChange(variables.filter((_, i) => i !== index));
  };

  const isNameValid = (name: string, index: number) => {
    if (!name) return false;
    if (!VALID_NAME_REGEX.test(name)) return false;
    // Check uniqueness
    return !variables.some((v, i) => i !== index && v.name === name);
  };

  return (
    <div className="border-t border-gray-200">
      {/* Header */}
      <button
        onClick={() => setIsExpanded(!isExpanded)}
        className="w-full flex items-center justify-between px-4 py-3 text-left hover:bg-gray-50"
      >
        <div className="flex items-center gap-2">
          {isExpanded ? (
            <ChevronDown className="w-4 h-4 text-gray-400" />
          ) : (
            <ChevronRight className="w-4 h-4 text-gray-400" />
          )}
          <span className="text-xs font-semibold text-gray-500 uppercase tracking-wider">
            Variables
          </span>
          {variables.length > 0 && (
            <span className="text-xs text-gray-400">({variables.length})</span>
          )}
        </div>
        <button
          onClick={(e) => {
            e.stopPropagation();
            addVariable();
          }}
          className="p-1 text-gray-400 hover:text-primary-600 hover:bg-primary-50 rounded"
          title="Add variable"
        >
          <Plus className="w-4 h-4" />
        </button>
      </button>

      {/* Content */}
      {isExpanded && (
        <div className="px-3 pb-3 space-y-2">
          {variables.length === 0 ? (
            <p className="text-xs text-gray-400 text-center py-4">
              No variables defined
            </p>
          ) : (
            variables.map((variable, index) => {
              const nameValid = isNameValid(variable.name, index);

              return (
                <div
                  key={index}
                  className="bg-white border border-gray-200 rounded-lg p-3 space-y-2"
                >
                  {/* Name input */}
                  <div className="flex items-center justify-between gap-2">
                    <input
                      type="text"
                      value={variable.name}
                      onChange={(e) => updateVariable(index, { name: e.target.value })}
                      placeholder="variable_name"
                      className={`flex-1 text-sm font-medium bg-transparent border-b focus:outline-none focus:border-primary-500 ${
                        variable.name && !nameValid
                          ? 'border-red-500 text-red-600'
                          : 'border-transparent hover:border-gray-300'
                      }`}
                      aria-label="Variable name"
                    />
                    <button
                      onClick={() => deleteVariable(index)}
                      className="p-1 text-gray-400 hover:text-red-500 hover:bg-red-50 rounded"
                      title="Delete variable"
                    >
                      <Trash2 className="w-3.5 h-3.5" />
                    </button>
                  </div>

                  {/* Type and Required badges */}
                  <div className="flex items-center gap-2">
                    {/* Type dropdown */}
                    <select
                      value={variable.var_type}
                      onChange={(e) => updateVariable(index, { var_type: e.target.value })}
                      className={`text-xs font-medium px-2 py-0.5 rounded-full border-0 cursor-pointer ${typeColors[variable.var_type] || typeColors.string}`}
                    >
                      {VARIABLE_TYPES.map((type) => (
                        <option key={type} value={type}>
                          {type}
                        </option>
                      ))}
                    </select>

                    {/* Required toggle */}
                    <button
                      onClick={() => updateVariable(index, { required: !variable.required })}
                      className={`text-xs font-medium px-2 py-0.5 rounded-full ${
                        variable.required
                          ? 'bg-red-100 text-red-700'
                          : 'bg-gray-100 text-gray-500'
                      }`}
                    >
                      {variable.required ? 'required' : 'optional'}
                    </button>
                  </div>

                  {/* Default value */}
                  <div className="flex items-center gap-2">
                    <span className="text-xs text-gray-400">default:</span>
                    <input
                      type={variable.var_type === 'number' ? 'number' : 'text'}
                      value={variable.default_value || ''}
                      onChange={(e) => updateVariable(index, { default_value: e.target.value || undefined })}
                      placeholder={
                        variable.var_type === 'boolean'
                          ? 'true/false'
                          : variable.var_type === 'number'
                          ? '0'
                          : 'value'
                      }
                      className="flex-1 text-xs bg-gray-50 border border-gray-200 rounded px-2 py-1 focus:outline-none focus:border-primary-500"
                      aria-label="Default value"
                    />
                  </div>

                  {/* Validation error */}
                  {variable.name && !nameValid && (
                    <p className="text-xs text-red-500">
                      {!VALID_NAME_REGEX.test(variable.name)
                        ? 'Invalid name format'
                        : 'Name must be unique'}
                    </p>
                  )}
                </div>
              );
            })
          )}
        </div>
      )}
    </div>
  );
}
