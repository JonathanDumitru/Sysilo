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

import {
  TopologyNode,
  FloatingStatsPanel,
  LensSwitcher,
  NodeDetailDrawer,
} from '../components/graph/index.js';
import {
  useTopologyGraph,
  type TopologyNodeData,
  type GraphLens,
} from '../hooks/useTopologyGraph.js';

// ---------------------------------------------------------------------------
// Node type registry
// ---------------------------------------------------------------------------

const nodeTypes: NodeTypes = {
  topology: TopologyNode,
};

// ---------------------------------------------------------------------------
// Legend data per lens
// ---------------------------------------------------------------------------

interface LegendItem {
  color: string;
  label: string;
}

const LEGENDS: Record<GraphLens, LegendItem[]> = {
  health: [
    { color: '#3FB950', label: 'Healthy' },
    { color: '#D29922', label: 'Warning' },
    { color: '#F85149', label: 'Critical' },
    { color: '#6B7280', label: 'Inactive' },
  ],
  governance: [
    { color: '#79C0FF', label: 'Compliant' },
    { color: '#F85149', label: 'Violation' },
    { color: '#6B7280', label: 'Uncovered' },
  ],
  time: [
    { color: '#3FB950', label: 'Invest' },
    { color: '#58A6FF', label: 'Tolerate' },
    { color: '#D29922', label: 'Migrate' },
    { color: '#F85149', label: 'Eliminate' },
  ],
  lineage: [
    { color: '#58A6FF', label: 'Data flow' },
    { color: '#4B5563', label: 'Dependency' },
  ],
};

// ---------------------------------------------------------------------------
// Dashboard Page — Graph-First Home
// ---------------------------------------------------------------------------

export function DashboardPage() {
  const {
    nodes: graphNodes,
    edges: graphEdges,
    activeLens,
    setLens,
    stats,
    topologyData,
  } = useTopologyGraph();

  const [nodes, setNodes, onNodesChange] = useNodesState(graphNodes);
  const [edges, setEdges, onEdgesChange] = useEdgesState(graphEdges);
  const [selectedNode, setSelectedNode] = useState<TopologyNodeData | null>(null);

  // Sync nodes/edges when lens changes
  useEffect(() => {
    setNodes(graphNodes);
    setEdges(graphEdges);
  }, [graphNodes, graphEdges, setNodes, setEdges]);

  // Node click handler — opens the detail drawer
  const onNodeClick = useCallback(
    (_: React.MouseEvent, node: Node) => {
      const data = node.data as TopologyNodeData;
      const found = topologyData.find((d) => d.id === data.id);
      if (found) setSelectedNode(found);
    },
    [topologyData],
  );

  // Pane click clears selection
  const onPaneClick = useCallback(() => {
    setSelectedNode(null);
  }, []);

  const legendItems = LEGENDS[activeLens];

  return (
    <div className="h-[calc(100vh-64px)] w-full relative -m-6">
      {/* ----------------------------------------------------------------- */}
      {/* Full-screen React Flow canvas                                      */}
      {/* ----------------------------------------------------------------- */}
      <ReactFlow
        nodes={nodes}
        edges={edges}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
        onNodeClick={onNodeClick}
        onPaneClick={onPaneClick}
        nodeTypes={nodeTypes}
        fitView
        fitViewOptions={{ padding: 0.15 }}
        minZoom={0.15}
        maxZoom={2.5}
        proOptions={{ hideAttribution: true }}
        className="!bg-surface-base"
      >
        {/* Background dots */}
        <Background gap={24} size={1} color="rgba(255,255,255,0.03)" />

        {/* Controls */}
        <Controls
          className="!bg-surface-raised/80 !backdrop-blur-[16px] !border !border-surface-border !rounded-lg !shadow-glass [&>button]:!bg-transparent [&>button]:!border-surface-border [&>button]:!text-gray-400 [&>button:hover]:!text-gray-200 [&>button:hover]:!bg-surface-overlay/50"
          showInteractive={false}
        />

        {/* Minimap */}
        <MiniMap
          nodeStrokeWidth={3}
          zoomable
          pannable
          maskColor="rgba(0,0,0,0.7)"
          className="!bg-surface-raised/60 !backdrop-blur-[16px] !border !border-surface-border !rounded-lg"
          nodeColor={(node) => {
            const d = node.data as TopologyNodeData & { _borderColor?: string };
            return (d._borderColor as string) ?? 'rgba(255,255,255,0.15)';
          }}
        />

        {/* --------------------------------------------------------------- */}
        {/* Top-left: Lens switcher                                          */}
        {/* --------------------------------------------------------------- */}
        <Panel position="top-left">
          <LensSwitcher activeLens={activeLens} onLensChange={setLens} />
        </Panel>

        {/* --------------------------------------------------------------- */}
        {/* Top-right: Floating stats panel                                  */}
        {/* --------------------------------------------------------------- */}
        <Panel position="top-right">
          <FloatingStatsPanel
            totalAssets={stats.totalAssets}
            activeIntegrations={stats.activeIntegrations}
            runningAgents={stats.runningAgents}
            openAlerts={stats.openAlerts}
          />
        </Panel>

        {/* --------------------------------------------------------------- */}
        {/* Bottom-left: Legend for current lens                              */}
        {/* --------------------------------------------------------------- */}
        <Panel position="bottom-left" className="!mb-2 !ml-2">
          <div className="glass-panel px-3 py-2.5">
            <div className="text-[10px] text-gray-500 uppercase tracking-wider font-medium mb-1.5">
              {activeLens === 'health'
                ? 'Health Status'
                : activeLens === 'governance'
                ? 'Governance'
                : activeLens === 'time'
                ? 'TIME Quadrant'
                : 'Data Flow'}
            </div>
            <div className="space-y-1">
              {legendItems.map((item) => (
                <div key={item.label} className="flex items-center gap-2">
                  {activeLens === 'lineage' ? (
                    <div
                      className="w-4 h-0.5 rounded"
                      style={{ backgroundColor: item.color }}
                    />
                  ) : (
                    <div
                      className="w-2.5 h-2.5 rounded-full"
                      style={{
                        backgroundColor: item.color,
                        boxShadow: `0 0 6px ${item.color}60`,
                      }}
                    />
                  )}
                  <span className="text-[11px] text-gray-400">{item.label}</span>
                </div>
              ))}
            </div>
            {/* Node shape legend */}
            <div className="border-t border-surface-border mt-2 pt-2">
              <div className="text-[10px] text-gray-500 uppercase tracking-wider font-medium mb-1.5">
                Node Types
              </div>
              <div className="space-y-1">
                <div className="flex items-center gap-2">
                  <div className="w-3 h-3 rounded-full border border-gray-500" />
                  <span className="text-[11px] text-gray-400">Agent</span>
                </div>
                <div className="flex items-center gap-2">
                  <div className="w-3 h-2.5 rounded border border-gray-500" />
                  <span className="text-[11px] text-gray-400">Integration / Connection</span>
                </div>
                <div className="flex items-center gap-2">
                  <div
                    className="w-3 h-3 border border-gray-500"
                    style={{ clipPath: 'polygon(25% 0%, 75% 0%, 100% 50%, 75% 100%, 25% 100%, 0% 50%)' }}
                  />
                  <span className="text-[11px] text-gray-400">Asset</span>
                </div>
              </div>
            </div>
          </div>
        </Panel>

        {/* --------------------------------------------------------------- */}
        {/* Bottom-right: Node / edge count                                  */}
        {/* --------------------------------------------------------------- */}
        <Panel position="bottom-right" className="!mb-2 !mr-2">
          <div className="glass-panel px-3 py-1.5 text-[11px] text-gray-500">
            {nodes.length} nodes &middot; {edges.length} edges
          </div>
        </Panel>
      </ReactFlow>

      {/* ----------------------------------------------------------------- */}
      {/* Node detail drawer (slides in from the right)                      */}
      {/* ----------------------------------------------------------------- */}
      <NodeDetailDrawer node={selectedNode} onClose={() => setSelectedNode(null)} />
    </div>
  );
}
