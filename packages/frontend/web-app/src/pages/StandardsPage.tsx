import { useState } from 'react';
import {
  Plus,
  Search,
  BookOpen,
  ChevronRight,
  X,
  Edit,
  Copy,
} from 'lucide-react';

// Mock data
const standards = [
  {
    id: '1',
    name: 'Integration Naming Standards',
    category: 'naming',
    description: 'Defines naming conventions for integrations, connections, and data flows',
    version: 3,
    isActive: true,
    createdAt: '2024-01-01T00:00:00Z',
    updatedAt: '2024-01-10T00:00:00Z',
    rules: [
      { id: '1', title: 'Lowercase Only', description: 'All names must be lowercase' },
      { id: '2', title: 'Hyphen Separators', description: 'Use hyphens to separate words, not underscores' },
      { id: '3', title: 'Environment Prefix', description: 'Prefix with environment (prod-, staging-, dev-)' },
    ],
    examples: [
      { valid: true, value: 'prod-salesforce-sync', note: 'Correct: lowercase, hyphens, env prefix' },
      { valid: false, value: 'Salesforce_Sync', note: 'Incorrect: uppercase and underscores' },
    ],
  },
  {
    id: '2',
    name: 'Security Configuration Standards',
    category: 'security',
    description: 'Security requirements for all connections and integrations',
    version: 5,
    isActive: true,
    createdAt: '2024-01-01T00:00:00Z',
    updatedAt: '2024-01-12T00:00:00Z',
    rules: [
      { id: '1', title: 'HTTPS Required', description: 'All external connections must use HTTPS' },
      { id: '2', title: 'Authentication Required', description: 'All connections must have authentication configured' },
      { id: '3', title: 'Credential Rotation', description: 'Credentials must be rotated every 90 days' },
    ],
    examples: [],
  },
  {
    id: '3',
    name: 'Data Management Standards',
    category: 'data_management',
    description: 'Standards for data handling, retention, and classification',
    version: 2,
    isActive: true,
    createdAt: '2024-01-05T00:00:00Z',
    updatedAt: '2024-01-08T00:00:00Z',
    rules: [
      { id: '1', title: 'Data Classification', description: 'All data entities must be classified (public, internal, confidential, restricted)' },
      { id: '2', title: 'PII Handling', description: 'PII must be encrypted at rest and in transit' },
      { id: '3', title: 'Retention Policies', description: 'Data retention periods must be defined for all entities' },
    ],
    examples: [],
  },
  {
    id: '4',
    name: 'Documentation Standards',
    category: 'documentation',
    description: 'Requirements for documentation of integrations and data flows',
    version: 1,
    isActive: false,
    createdAt: '2024-01-10T00:00:00Z',
    updatedAt: '2024-01-10T00:00:00Z',
    rules: [
      { id: '1', title: 'Description Required', description: 'All integrations must have a description' },
      { id: '2', title: 'Owner Assigned', description: 'Every integration must have an assigned owner' },
    ],
    examples: [],
  },
];

const categories = [
  { value: 'naming', label: 'Naming', color: 'bg-blue-900/40 text-blue-400' },
  { value: 'security', label: 'Security', color: 'bg-red-900/40 text-red-400' },
  { value: 'architecture', label: 'Architecture', color: 'bg-purple-900/40 text-purple-400' },
  { value: 'data_management', label: 'Data Management', color: 'bg-green-900/40 text-green-400' },
  { value: 'integration', label: 'Integration', color: 'bg-orange-900/40 text-orange-400' },
  { value: 'operations', label: 'Operations', color: 'bg-yellow-900/40 text-yellow-400' },
  { value: 'documentation', label: 'Documentation', color: 'bg-surface-overlay text-gray-300' },
];

export function StandardsPage() {
  const [searchQuery, setSearchQuery] = useState('');
  const [categoryFilter, setCategoryFilter] = useState<string>('all');
  const [selectedStandard, setSelectedStandard] = useState<typeof standards[0] | null>(null);
  const [showCreateModal, setShowCreateModal] = useState(false);

  const getCategoryColor = (category: string) => {
    const cat = categories.find((c) => c.value === category);
    return cat?.color || 'bg-surface-overlay text-gray-300';
  };

  const getCategoryLabel = (category: string) => {
    const cat = categories.find((c) => c.value === category);
    return cat?.label || category;
  };

  const formatDate = (dateStr: string) => {
    const date = new Date(dateStr);
    return date.toLocaleDateString();
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-white">Standards Library</h1>
          <p className="text-gray-500">Organization-wide standards and best practices</p>
        </div>
        <button
          onClick={() => setShowCreateModal(true)}
          className="flex items-center gap-2 px-4 py-2 bg-primary-600 text-white rounded-lg text-sm font-medium hover:bg-primary-700"
        >
          <Plus className="w-4 h-4" />
          Create Standard
        </button>
      </div>

      {/* Filters */}
      <div className="flex items-center gap-4">
        <div className="relative flex-1 max-w-md">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400" />
          <input
            type="text"
            placeholder="Search standards..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-full pl-10 pr-4 py-2 glass-input text-sm"
          />
        </div>
        <select
          value={categoryFilter}
          onChange={(e) => setCategoryFilter(e.target.value)}
          className="px-3 py-2 glass-input text-sm"
        >
          <option value="all">All Categories</option>
          {categories.map((cat) => (
            <option key={cat.value} value={cat.value}>
              {cat.label}
            </option>
          ))}
        </select>
      </div>

      {/* Standards Grid */}
      <div className="grid grid-cols-2 gap-6">
        {/* Standards List */}
        <div className="space-y-4">
          {standards.map((standard) => (
            <div
              key={standard.id}
              onClick={() => setSelectedStandard(standard)}
              className={`glass-panel p-5 cursor-pointer transition-all ${
                selectedStandard?.id === standard.id
                  ? 'border-primary-500 ring-2 ring-primary-500/20'
                  : 'border-surface-border hover:border-surface-border'
              }`}
            >
              <div className="flex items-start justify-between">
                <div className="flex items-start gap-3">
                  <div
                    className={`p-2 rounded-lg ${
                      standard.isActive ? 'bg-primary-900/30' : 'bg-surface-overlay'
                    }`}
                  >
                    <BookOpen
                      className={`w-5 h-5 ${
                        standard.isActive ? 'text-primary-600' : 'text-gray-400'
                      }`}
                    />
                  </div>
                  <div>
                    <div className="flex items-center gap-2 mb-1">
                      <h3 className="font-semibold text-white">{standard.name}</h3>
                      {!standard.isActive && (
                        <span className="text-xs px-1.5 py-0.5 bg-surface-overlay text-gray-500 rounded">
                          Inactive
                        </span>
                      )}
                    </div>
                    <p className="text-sm text-gray-500 line-clamp-2">{standard.description}</p>
                  </div>
                </div>
                <ChevronRight className="w-5 h-5 text-gray-400 flex-shrink-0" />
              </div>
              <div className="flex items-center gap-4 mt-4 pt-3 border-t border-surface-border">
                <span
                  className={`text-xs font-medium px-2 py-0.5 rounded-full ${getCategoryColor(
                    standard.category
                  )}`}
                >
                  {getCategoryLabel(standard.category)}
                </span>
                <span className="text-xs text-gray-500">v{standard.version}</span>
                <span className="text-xs text-gray-500">{standard.rules.length} rules</span>
                <span className="text-xs text-gray-400 ml-auto">
                  Updated {formatDate(standard.updatedAt)}
                </span>
              </div>
            </div>
          ))}
        </div>

        {/* Detail Panel */}
        <div className="glass-panel h-fit sticky top-6">
          {selectedStandard ? (
            <div>
              <div className="p-6 border-b border-surface-border">
                <div className="flex items-center justify-between mb-3">
                  <span
                    className={`text-xs font-medium px-2 py-0.5 rounded-full ${getCategoryColor(
                      selectedStandard.category
                    )}`}
                  >
                    {getCategoryLabel(selectedStandard.category)}
                  </span>
                  <div className="flex items-center gap-2">
                    <span className="text-xs text-gray-500">Version {selectedStandard.version}</span>
                    {!selectedStandard.isActive && (
                      <span className="text-xs px-1.5 py-0.5 bg-surface-overlay text-gray-500 rounded">
                        Inactive
                      </span>
                    )}
                  </div>
                </div>
                <h2 className="text-xl font-semibold text-white mb-2">
                  {selectedStandard.name}
                </h2>
                <p className="text-sm text-gray-400">{selectedStandard.description}</p>
              </div>

              <div className="p-6 border-b border-surface-border">
                <h3 className="text-sm font-medium text-white mb-4">
                  Rules ({selectedStandard.rules.length})
                </h3>
                <div className="space-y-3">
                  {selectedStandard.rules.map((rule, index) => (
                    <div
                      key={rule.id}
                      className="flex gap-3 p-3 bg-surface-overlay/50 rounded-lg"
                    >
                      <span className="flex items-center justify-center w-6 h-6 bg-primary-900/40 text-primary-400 text-xs font-medium rounded-full flex-shrink-0">
                        {index + 1}
                      </span>
                      <div>
                        <p className="text-sm font-medium text-white">{rule.title}</p>
                        <p className="text-xs text-gray-500 mt-0.5">{rule.description}</p>
                      </div>
                    </div>
                  ))}
                </div>
              </div>

              {selectedStandard.examples.length > 0 && (
                <div className="p-6 border-b border-surface-border">
                  <h3 className="text-sm font-medium text-white mb-4">Examples</h3>
                  <div className="space-y-2">
                    {selectedStandard.examples.map((example, index) => (
                      <div
                        key={index}
                        className={`p-3 rounded-lg border ${
                          example.valid
                            ? 'bg-green-900/30 border-green-800'
                            : 'bg-red-900/30 border-red-800'
                        }`}
                      >
                        <div className="flex items-center gap-2 mb-1">
                          <code
                            className={`text-sm font-mono ${
                              example.valid ? 'text-green-400' : 'text-red-400'
                            }`}
                          >
                            {example.value}
                          </code>
                          <span
                            className={`text-xs font-medium ${
                              example.valid ? 'text-green-600' : 'text-red-600'
                            }`}
                          >
                            {example.valid ? '✓ Valid' : '✗ Invalid'}
                          </span>
                        </div>
                        <p className="text-xs text-gray-400">{example.note}</p>
                      </div>
                    ))}
                  </div>
                </div>
              )}

              <div className="p-6">
                <div className="flex items-center justify-between text-xs text-gray-500 mb-4">
                  <span>Created {formatDate(selectedStandard.createdAt)}</span>
                  <span>Updated {formatDate(selectedStandard.updatedAt)}</span>
                </div>
                <div className="flex gap-2">
                  <button className="flex-1 flex items-center justify-center gap-2 px-4 py-2 border border-surface-border rounded-lg text-sm font-medium text-gray-300 hover:bg-surface-overlay/50">
                    <Edit className="w-4 h-4" />
                    Edit
                  </button>
                  <button className="flex-1 flex items-center justify-center gap-2 px-4 py-2 border border-surface-border rounded-lg text-sm font-medium text-gray-300 hover:bg-surface-overlay/50">
                    <Copy className="w-4 h-4" />
                    Duplicate
                  </button>
                </div>
              </div>
            </div>
          ) : (
            <div className="p-8 text-center text-gray-500">
              <BookOpen className="w-12 h-12 mx-auto mb-3 text-gray-300" />
              <p>Select a standard to view details</p>
            </div>
          )}
        </div>
      </div>

      {/* Create Standard Modal */}
      {showCreateModal && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-surface-raised border border-surface-border rounded-xl p-6 w-full max-w-lg">
            <div className="flex items-center justify-between mb-4">
              <h2 className="text-lg font-semibold">Create Standard</h2>
              <button
                onClick={() => setShowCreateModal(false)}
                className="p-1 text-gray-400 hover:text-gray-300"
              >
                <X className="w-5 h-5" />
              </button>
            </div>
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-gray-300 mb-1">Name</label>
                <input
                  type="text"
                  className="w-full px-3 py-2 glass-input text-sm"
                  placeholder="Enter standard name"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-300 mb-1">Category</label>
                <select className="w-full px-3 py-2 glass-input text-sm">
                  {categories.map((cat) => (
                    <option key={cat.value} value={cat.value}>
                      {cat.label}
                    </option>
                  ))}
                </select>
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-300 mb-1">Description</label>
                <textarea
                  className="w-full px-3 py-2 glass-input text-sm"
                  rows={3}
                  placeholder="Describe the standard and its purpose"
                />
              </div>
              <div className="flex justify-end gap-3 pt-4">
                <button
                  onClick={() => setShowCreateModal(false)}
                  className="px-4 py-2 text-sm font-medium text-gray-300 hover:text-white"
                >
                  Cancel
                </button>
                <button className="px-4 py-2 bg-primary-600 text-white rounded-lg text-sm font-medium hover:bg-primary-700">
                  Create Standard
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
