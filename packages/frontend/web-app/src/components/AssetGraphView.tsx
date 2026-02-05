import { useCallback, useState, useEffect } from 'react';
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

import type { Asset } from '../services/assets.js';
import { useAssetGraph } from '../hooks/useAssetGraph.js';
import { AssetNode, type AssetNodeData } from './graph/index.js';

const nodeTypes: NodeTypes = {
  asset: AssetNode,
};

interface AssetGraphViewProps {
  assets: Asset[];
  onAssetClick?: (asset: Asset) => void;
}

export function AssetGraphView({ assets, onAssetClick }: AssetGraphViewProps) {
  const { nodes: graphNodes, edges: graphEdges } = useAssetGraph(assets);
  const [nodes, setNodes, onNodesChange] = useNodesState(graphNodes);
  const [edges, setEdges, onEdgesChange] = useEdgesState(graphEdges);
  const [selectedAsset, setSelectedAsset] = useState<Asset | null>(null);

  // Update nodes/edges when assets change
  useEffect(() => {
    setNodes(graphNodes);
    setEdges(graphEdges);
  }, [graphNodes, graphEdges, setNodes, setEdges]);

  const onNodeClick = useCallback(
    (_: React.MouseEvent, node: Node) => {
      const nodeData = node.data as AssetNodeData;
      const asset = assets.find((a) => a.id === nodeData.id);
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

  return (
    <div className="h-[600px] bg-gray-50 rounded-xl border border-gray-200 relative">
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
          <div className="mt-2 space-y-1.5 text-xs">
            <div className="flex items-center gap-2">
              <div className="w-5 h-0.5 bg-slate-400 rounded" />
              <span className="text-gray-600">Same vendor</span>
            </div>
            <div className="flex items-center gap-2">
              <div className="w-5 h-0.5 bg-blue-400 rounded" />
              <span className="text-gray-600">Same team</span>
            </div>
            <div className="flex items-center gap-2">
              <div className="w-5 border-t-2 border-dashed border-violet-400" />
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
        <div className="absolute bottom-4 left-4 right-4 bg-white rounded-lg shadow-lg border border-gray-200 p-4 max-w-md z-10">
          <div className="flex items-start justify-between">
            <div>
              <h3 className="font-semibold text-gray-900">{selectedAsset.name}</h3>
              <p className="text-sm text-gray-500 mt-0.5">{selectedAsset.description ?? 'No description'}</p>
            </div>
            <span className={`px-2 py-0.5 rounded text-xs font-medium ${
              selectedAsset.status === 'active' ? 'bg-green-100 text-green-700' : 'bg-gray-100 text-gray-600'
            }`}>
              {selectedAsset.status}
            </span>
          </div>
          <div className="mt-2 flex items-center gap-3 text-xs text-gray-500">
            <span className="font-medium text-gray-700">{selectedAsset.asset_type}</span>
            {selectedAsset.vendor && <span>• {selectedAsset.vendor}</span>}
            {selectedAsset.version && <span>v{selectedAsset.version}</span>}
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
