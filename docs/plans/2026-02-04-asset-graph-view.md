# Asset Graph View Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add interactive graph visualization to the Asset Registry that shows assets as nodes with edges connecting related assets (same vendor, team, or shared tags).

**Architecture:** React Flow graph view component that transforms existing asset data into nodes/edges. Toggle between grid and graph views in AssetRegistryPage. Automatic edge generation from shared properties.

**Tech Stack:** React Flow (@xyflow/react v12 - already installed), React Query (existing), TypeScript, Tailwind CSS

---

### Task 1: Create AssetNode Component

**Files:**
- Create: `packages/frontend/web-app/src/components/graph/AssetNode.tsx`

**Step 1: Create the custom node component**

```tsx
import { memo } from 'react';
import { Handle, Position, type NodeProps } from '@xyflow/react';
import { Database, Server, Workflow, Globe } from 'lucide-react';

export interface AssetNodeData {
  id: string;
  name: string;
  asset_type: string;
  status: string;
  vendor?: string;
  tags: string[];
}

const typeIcons: Record<string, React.ElementType> = {
  database: Database,
  application: Server,
  service: Server,
  api: Workflow,
  integration: Workflow,
  default: Globe,
};

const statusColors: Record<string, string> = {
  active: 'border-green-400 bg-green-50',
  deprecated: 'border-yellow-400 bg-yellow-50',
  default: 'border-gray-300 bg-white',
};

function AssetNodeComponent({ data, selected }: NodeProps<AssetNodeData>) {
  const Icon = typeIcons[data.asset_type] ?? typeIcons.default;
  const statusStyle = statusColors[data.status] ?? statusColors.default;

  return (
    <div
      className={`px-4 py-3 rounded-lg border-2 shadow-sm min-w-[160px] transition-all ${statusStyle} ${
        selected ? 'ring-2 ring-primary-500 ring-offset-2' : ''
      }`}
    >
      <Handle type="target" position={Position.Left} className="!bg-gray-400" />

      <div className="flex items-center gap-2">
        <div className="p-1.5 bg-white rounded border border-gray-200">
          <Icon className="w-4 h-4 text-gray-600" />
        </div>
        <div className="flex-1 min-w-0">
          <div className="font-medium text-sm text-gray-900 truncate">{data.name}</div>
          <div className="text-xs text-gray-500">{data.asset_type}</div>
        </div>
      </div>

      {data.vendor && (
        <div className="mt-2 text-xs text-gray-500 truncate">{data.vendor}</div>
      )}

      <Handle type="source" position={Position.Right} className="!bg-gray-400" />
    </div>
  );
}

export const AssetNode = memo(AssetNodeComponent);
```

**Step 2: Create index export**

Create `packages/frontend/web-app/src/components/graph/index.ts`:

```ts
export { AssetNode, type AssetNodeData } from './AssetNode';
```

**Step 3: Verify TypeScript compiles**

Run: `cd /Users/dev/Downloads/Sysilo/packages/frontend/web-app && npx tsc --noEmit`
Expected: No errors

**Step 4: Commit**

```bash
git add packages/frontend/web-app/src/components/graph/
git commit -m "feat(frontend): add AssetNode component for graph visualization"
```

---

### Task 2: Create Graph Data Transformation Hook

**Files:**
- Create: `packages/frontend/web-app/src/hooks/useAssetGraph.ts`

**Step 1: Create the hook that transforms assets to nodes/edges**

```ts
import { useMemo } from 'react';
import type { Node, Edge } from '@xyflow/react';
import type { Asset } from '../services/assets';
import type { AssetNodeData } from '../components/graph';

interface AssetGraphData {
  nodes: Node<AssetNodeData>[];
  edges: Edge[];
}

/**
 * Transforms assets into React Flow nodes and edges.
 * Edges connect assets that share vendors, teams, or tags.
 */
export function useAssetGraph(assets: Asset[]): AssetGraphData {
  return useMemo(() => {
    if (!assets.length) {
      return { nodes: [], edges: [] };
    }

    // Create nodes - arrange in a grid initially (React Flow will auto-layout)
    const COLS = 4;
    const X_SPACING = 250;
    const Y_SPACING = 150;

    const nodes: Node<AssetNodeData>[] = assets.map((asset, index) => ({
      id: asset.id,
      type: 'asset',
      position: {
        x: (index % COLS) * X_SPACING + 50,
        y: Math.floor(index / COLS) * Y_SPACING + 50,
      },
      data: {
        id: asset.id,
        name: asset.name,
        asset_type: asset.asset_type,
        status: asset.status,
        vendor: asset.vendor,
        tags: asset.tags,
      },
    }));

    // Create edges for related assets
    const edges: Edge[] = [];
    const edgeSet = new Set<string>(); // Prevent duplicates

    for (let i = 0; i < assets.length; i++) {
      for (let j = i + 1; j < assets.length; j++) {
        const a = assets[i];
        const b = assets[j];
        const edgeId = `${a.id}-${b.id}`;

        if (edgeSet.has(edgeId)) continue;

        // Connect by shared vendor
        if (a.vendor && b.vendor && a.vendor === b.vendor) {
          edges.push({
            id: `vendor-${edgeId}`,
            source: a.id,
            target: b.id,
            type: 'default',
            style: { stroke: '#94a3b8', strokeWidth: 1 },
            label: a.vendor,
            labelStyle: { fontSize: 10, fill: '#64748b' },
          });
          edgeSet.add(edgeId);
          continue;
        }

        // Connect by shared team
        if (a.team && b.team && a.team === b.team) {
          edges.push({
            id: `team-${edgeId}`,
            source: a.id,
            target: b.id,
            type: 'default',
            style: { stroke: '#60a5fa', strokeWidth: 1 },
          });
          edgeSet.add(edgeId);
          continue;
        }

        // Connect by shared tags (at least one common tag)
        const sharedTags = a.tags.filter((tag) => b.tags.includes(tag));
        if (sharedTags.length > 0) {
          edges.push({
            id: `tags-${edgeId}`,
            source: a.id,
            target: b.id,
            type: 'default',
            style: { stroke: '#a78bfa', strokeWidth: 1, strokeDasharray: '5,5' },
          });
          edgeSet.add(edgeId);
        }
      }
    }

    return { nodes, edges };
  }, [assets]);
}
```

**Step 2: Verify TypeScript compiles**

Run: `cd /Users/dev/Downloads/Sysilo/packages/frontend/web-app && npx tsc --noEmit`
Expected: No errors

**Step 3: Commit**

```bash
git add packages/frontend/web-app/src/hooks/useAssetGraph.ts
git commit -m "feat(frontend): add useAssetGraph hook for graph data transformation"
```

---

### Task 3: Create AssetGraphView Component

**Files:**
- Create: `packages/frontend/web-app/src/components/AssetGraphView.tsx`

**Step 1: Create the graph view component**

```tsx
import { useCallback, useState } from 'react';
import {
  ReactFlow,
  Background,
  Controls,
  MiniMap,
  Panel,
  useNodesState,
  useEdgesState,
  type Node,
  type NodeTypes,
} from '@xyflow/react';
import '@xyflow/react/dist/style.css';

import type { Asset } from '../services/assets';
import { useAssetGraph } from '../hooks/useAssetGraph';
import { AssetNode, type AssetNodeData } from './graph';

const nodeTypes: NodeTypes = {
  asset: AssetNode,
};

interface AssetGraphViewProps {
  assets: Asset[];
  onAssetClick?: (asset: Asset) => void;
}

export function AssetGraphView({ assets, onAssetClick }: AssetGraphViewProps) {
  const { nodes: initialNodes, edges: initialEdges } = useAssetGraph(assets);
  const [nodes, setNodes, onNodesChange] = useNodesState(initialNodes);
  const [edges, , onEdgesChange] = useEdgesState(initialEdges);
  const [selectedAsset, setSelectedAsset] = useState<Asset | null>(null);

  const onNodeClick = useCallback(
    (_: React.MouseEvent, node: Node<AssetNodeData>) => {
      const asset = assets.find((a) => a.id === node.data.id);
      if (asset) {
        setSelectedAsset(asset);
        onAssetClick?.(asset);
      }
    },
    [assets, onAssetClick]
  );

  const onPaneClick = useCallback(() => {
    setSelectedAsset(null);
  }, []);

  // Update nodes when assets change
  const { nodes: newNodes, edges: newEdges } = useAssetGraph(assets);
  if (initialNodes.length !== newNodes.length) {
    setNodes(newNodes);
  }

  return (
    <div className="h-[600px] bg-gray-50 rounded-xl border border-gray-200">
      <ReactFlow
        nodes={nodes}
        edges={edges}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
        onNodeClick={onNodeClick}
        onPaneClick={onPaneClick}
        nodeTypes={nodeTypes}
        fitView
        minZoom={0.2}
        maxZoom={2}
      >
        <Background gap={20} size={1} color="#e5e7eb" />
        <Controls className="bg-white border border-gray-200 rounded-lg" />
        <MiniMap
          nodeStrokeWidth={3}
          zoomable
          pannable
          className="bg-white border border-gray-200 rounded-lg"
        />
        <Panel position="top-left" className="bg-white rounded-lg shadow-sm border border-gray-200 p-3">
          <div className="text-sm font-medium text-gray-900">Asset Relationships</div>
          <div className="mt-2 space-y-1 text-xs">
            <div className="flex items-center gap-2">
              <div className="w-4 h-0.5 bg-slate-400" />
              <span className="text-gray-600">Same vendor</span>
            </div>
            <div className="flex items-center gap-2">
              <div className="w-4 h-0.5 bg-blue-400" />
              <span className="text-gray-600">Same team</span>
            </div>
            <div className="flex items-center gap-2">
              <div className="w-4 h-0.5 bg-violet-400" style={{ backgroundImage: 'repeating-linear-gradient(90deg, #a78bfa 0, #a78bfa 3px, transparent 3px, transparent 6px)' }} />
              <span className="text-gray-600">Shared tags</span>
            </div>
          </div>
        </Panel>
        <Panel position="top-right" className="bg-white rounded-lg shadow-sm border border-gray-200 p-2">
          <div className="text-xs text-gray-500">
            {nodes.length} assets · {edges.length} relationships
          </div>
        </Panel>
      </ReactFlow>

      {/* Selected asset info */}
      {selectedAsset && (
        <div className="absolute bottom-4 left-4 right-4 bg-white rounded-lg shadow-lg border border-gray-200 p-4 max-w-md">
          <div className="flex items-start justify-between">
            <div>
              <h3 className="font-semibold text-gray-900">{selectedAsset.name}</h3>
              <p className="text-sm text-gray-500">{selectedAsset.description ?? 'No description'}</p>
            </div>
            <span className={`px-2 py-0.5 rounded text-xs font-medium ${
              selectedAsset.status === 'active' ? 'bg-green-100 text-green-700' : 'bg-gray-100 text-gray-600'
            }`}>
              {selectedAsset.status}
            </span>
          </div>
          {selectedAsset.tags.length > 0 && (
            <div className="mt-2 flex flex-wrap gap-1">
              {selectedAsset.tags.map((tag) => (
                <span key={tag} className="text-xs px-2 py-0.5 bg-primary-50 text-primary-700 rounded">
                  {tag}
                </span>
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  );
}
```

**Step 2: Verify TypeScript compiles**

Run: `cd /Users/dev/Downloads/Sysilo/packages/frontend/web-app && npx tsc --noEmit`
Expected: No errors

**Step 3: Commit**

```bash
git add packages/frontend/web-app/src/components/AssetGraphView.tsx
git commit -m "feat(frontend): add AssetGraphView component with React Flow"
```

---

### Task 4: Integrate Graph View into Asset Registry Page

**Files:**
- Modify: `packages/frontend/web-app/src/pages/AssetRegistryPage.tsx`

**Step 1: Add view mode state and toggle**

Add import at top:
```tsx
import { AssetGraphView } from '../components/AssetGraphView.js';
```

Add state after existing state declarations (around line 11):
```tsx
const [viewMode, setViewMode] = useState<'grid' | 'graph'>('grid');
```

**Step 2: Update Graph View button to toggle**

Replace the Graph View button (lines 89-92) with:
```tsx
<button
  onClick={() => setViewMode(viewMode === 'grid' ? 'graph' : 'grid')}
  className={`flex items-center gap-2 px-3 py-2 rounded-lg text-sm ${
    viewMode === 'graph'
      ? 'bg-primary-100 text-primary-700 border border-primary-200'
      : 'bg-white border border-gray-200 text-gray-600 hover:bg-gray-50'
  }`}
>
  <Network className="w-4 h-4" />
  {viewMode === 'graph' ? 'Grid View' : 'Graph View'}
</button>
```

**Step 3: Conditionally render grid or graph**

Replace the asset grid section (lines 134-181) with:
```tsx
{/* Asset view - Grid or Graph */}
{!isLoading && !error && assets.length > 0 && (
  viewMode === 'graph' ? (
    <AssetGraphView assets={assets} />
  ) : (
    <div className="grid grid-cols-3 gap-4">
      {assets.map((asset: Asset) => {
        const Icon = typeIcons[asset.asset_type.toLowerCase()] ?? Server;
        return (
          <div
            key={asset.id}
            className="bg-white rounded-xl p-5 shadow-sm border border-gray-100 hover:border-primary-200 cursor-pointer transition-colors"
          >
            {/* ... existing card content ... */}
          </div>
        );
      })}
    </div>
  )
)}
```

**Step 4: Verify TypeScript compiles**

Run: `cd /Users/dev/Downloads/Sysilo/packages/frontend/web-app && npx tsc --noEmit`
Expected: No errors

**Step 5: Manual test**

1. Start frontend: `cd packages/frontend/web-app && npm run dev`
2. Navigate to Asset Registry
3. Click "Discover Assets" and run mock discovery to get some assets
4. Click "Graph View" button
5. Verify: Assets appear as nodes, edges connect related assets
6. Click an asset node - verify info panel shows
7. Toggle back to "Grid View" - verify grid renders

**Step 6: Commit**

```bash
git add packages/frontend/web-app/src/pages/AssetRegistryPage.tsx
git commit -m "feat(frontend): integrate graph view toggle into Asset Registry"
```

---

### Task 5: Add Edge Types for Better Visualization

**Files:**
- Modify: `packages/frontend/web-app/src/components/AssetGraphView.tsx`

**Step 1: Add animated edges for active connections**

Add import:
```tsx
import { MarkerType } from '@xyflow/react';
```

Update the edges in `useAssetGraph.ts` to use animated edges:

In `packages/frontend/web-app/src/hooks/useAssetGraph.ts`, update the vendor edge creation:
```ts
edges.push({
  id: `vendor-${edgeId}`,
  source: a.id,
  target: b.id,
  type: 'default',
  animated: false,
  style: { stroke: '#94a3b8', strokeWidth: 2 },
  markerEnd: { type: MarkerType.ArrowClosed, color: '#94a3b8' },
  label: a.vendor,
  labelStyle: { fontSize: 10, fill: '#64748b' },
  labelBgStyle: { fill: '#f8fafc', fillOpacity: 0.9 },
  labelBgPadding: [4, 2] as [number, number],
  labelBgBorderRadius: 4,
});
```

**Step 2: Verify TypeScript compiles**

Run: `cd /Users/dev/Downloads/Sysilo/packages/frontend/web-app && npx tsc --noEmit`
Expected: No errors

**Step 3: Commit**

```bash
git add packages/frontend/web-app/src/hooks/useAssetGraph.ts
git commit -m "feat(frontend): improve edge styling with markers and labels"
```

---

## Summary

This implementation adds:
1. **AssetNode** - Custom React Flow node styled to match the design system
2. **useAssetGraph** - Hook that transforms assets into graph data with automatic edge generation
3. **AssetGraphView** - Full graph visualization with legend, minimap, and selected asset info
4. **View Toggle** - Seamless switching between grid and graph views in Asset Registry

The graph automatically connects assets that share:
- Same vendor (solid gray lines with label)
- Same team (solid blue lines)
- Shared tags (dashed purple lines)
