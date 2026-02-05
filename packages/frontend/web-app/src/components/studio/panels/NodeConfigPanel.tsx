import { X } from 'lucide-react';
import { Node } from '@xyflow/react';

interface NodeConfigPanelProps {
  node: Node;
  onClose: () => void;
  onUpdate: (config: Record<string, unknown>) => void;
}

export function NodeConfigPanel({ node, onClose, onUpdate }: NodeConfigPanelProps) {
  const nodeType = node.type as string;

  return (
    <div className="w-80 bg-white border-l border-gray-200 flex flex-col">
      {/* Header */}
      <div className="flex items-center justify-between p-4 border-b border-gray-100">
        <div>
          <h3 className="text-sm font-semibold text-gray-900">Configure Node</h3>
          <p className="text-xs text-gray-500">{node.data.label as string}</p>
        </div>
        <button
          onClick={onClose}
          className="p-1 text-gray-400 hover:text-gray-600 hover:bg-gray-100 rounded"
        >
          <X className="w-4 h-4" />
        </button>
      </div>

      {/* Content */}
      <div className="flex-1 overflow-y-auto p-4">
        {nodeType === 'source' && <SourceConfig node={node} onUpdate={onUpdate} />}
        {nodeType === 'transform' && <TransformConfig node={node} onUpdate={onUpdate} />}
        {nodeType === 'target' && <TargetConfig node={node} onUpdate={onUpdate} />}
      </div>
    </div>
  );
}

function SourceConfig({
  node,
  onUpdate,
}: {
  node: Node;
  onUpdate: (config: Record<string, unknown>) => void;
}) {
  const config = (node.data.config || {}) as Record<string, unknown>;

  return (
    <div className="space-y-4">
      <div>
        <label className="block text-sm font-medium text-gray-700 mb-1">Connection</label>
        <select className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm focus:ring-2 focus:ring-primary-500 focus:border-primary-500">
          <option>Select a connection...</option>
          <option>Production PostgreSQL</option>
          <option>Analytics Database</option>
        </select>
      </div>

      <div>
        <label className="block text-sm font-medium text-gray-700 mb-1">Table / Query</label>
        <select className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm focus:ring-2 focus:ring-primary-500 focus:border-primary-500 mb-2">
          <option>Use table</option>
          <option>Custom query</option>
        </select>
        <input
          type="text"
          placeholder="Enter table name..."
          className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
          value={(config.table as string) || ''}
          onChange={(e) => onUpdate({ ...config, table: e.target.value })}
        />
      </div>

      <div>
        <label className="block text-sm font-medium text-gray-700 mb-1">Batch Size</label>
        <input
          type="number"
          placeholder="1000"
          className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
          value={(config.batchSize as number) || 1000}
          onChange={(e) => onUpdate({ ...config, batchSize: parseInt(e.target.value) })}
        />
      </div>

      <div>
        <label className="flex items-center gap-2 text-sm text-gray-700">
          <input
            type="checkbox"
            checked={(config.incrementalLoad as boolean) || false}
            onChange={(e) => onUpdate({ ...config, incrementalLoad: e.target.checked })}
            className="rounded border-gray-300 text-primary-600 focus:ring-primary-500"
          />
          Enable incremental load
        </label>
      </div>
    </div>
  );
}

function TransformConfig({
  node,
  onUpdate,
}: {
  node: Node;
  onUpdate: (config: Record<string, unknown>) => void;
}) {
  const config = (node.data.config || {}) as Record<string, unknown>;
  const transformType = node.data.transformType as string;

  return (
    <div className="space-y-4">
      {transformType === 'map' && (
        <>
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">Field Mappings</label>
            <div className="space-y-2 p-3 bg-gray-50 rounded-lg">
              <div className="flex items-center gap-2 text-sm">
                <input
                  type="text"
                  placeholder="Source field"
                  className="flex-1 px-2 py-1.5 border border-gray-300 rounded text-sm"
                />
                <span className="text-gray-400">→</span>
                <input
                  type="text"
                  placeholder="Target field"
                  className="flex-1 px-2 py-1.5 border border-gray-300 rounded text-sm"
                />
              </div>
              <button className="text-xs text-primary-600 hover:text-primary-700 font-medium">
                + Add mapping
              </button>
            </div>
          </div>
        </>
      )}

      {transformType === 'filter' && (
        <>
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">Filter Condition</label>
            <textarea
              placeholder="e.g., status = 'active' AND created_at > '2024-01-01'"
              className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
              rows={3}
              value={(config.condition as string) || ''}
              onChange={(e) => onUpdate({ ...config, condition: e.target.value })}
            />
          </div>
        </>
      )}

      {transformType === 'aggregate' && (
        <>
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">Group By</label>
            <input
              type="text"
              placeholder="field1, field2"
              className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
              value={(config.groupBy as string) || ''}
              onChange={(e) => onUpdate({ ...config, groupBy: e.target.value })}
            />
          </div>
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">Aggregations</label>
            <select className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm focus:ring-2 focus:ring-primary-500 focus:border-primary-500">
              <option>SUM(amount)</option>
              <option>COUNT(*)</option>
              <option>AVG(value)</option>
            </select>
          </div>
        </>
      )}
    </div>
  );
}

function TargetConfig({
  node,
  onUpdate,
}: {
  node: Node;
  onUpdate: (config: Record<string, unknown>) => void;
}) {
  const config = (node.data.config || {}) as Record<string, unknown>;

  return (
    <div className="space-y-4">
      <div>
        <label className="block text-sm font-medium text-gray-700 mb-1">Connection</label>
        <select className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm focus:ring-2 focus:ring-primary-500 focus:border-primary-500">
          <option>Select a connection...</option>
          <option>Snowflake Warehouse</option>
          <option>BigQuery Analytics</option>
        </select>
      </div>

      <div>
        <label className="block text-sm font-medium text-gray-700 mb-1">Target Table</label>
        <input
          type="text"
          placeholder="schema.table_name"
          className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
          value={(config.table as string) || ''}
          onChange={(e) => onUpdate({ ...config, table: e.target.value })}
        />
      </div>

      <div>
        <label className="block text-sm font-medium text-gray-700 mb-1">Write Mode</label>
        <select
          className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
          value={(config.writeMode as string) || 'append'}
          onChange={(e) => onUpdate({ ...config, writeMode: e.target.value })}
        >
          <option value="append">Append</option>
          <option value="overwrite">Overwrite</option>
          <option value="upsert">Upsert (merge)</option>
        </select>
      </div>

      {config.writeMode === 'upsert' && (
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">Primary Key</label>
          <input
            type="text"
            placeholder="id"
            className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
            value={(config.primaryKey as string) || ''}
            onChange={(e) => onUpdate({ ...config, primaryKey: e.target.value })}
          />
        </div>
      )}
    </div>
  );
}
