import { useCallback, useState } from 'react';
import { useParams } from 'react-router-dom';
import {
  ReactFlow,
  Background,
  Controls,
  MiniMap,
  Panel,
  useNodesState,
  useEdgesState,
  addEdge,
  type Connection,
  type Node,
  type Edge,
  type NodeTypes,
} from '@xyflow/react';
import '@xyflow/react/dist/style.css';

import { SourceNode } from '@/components/studio/nodes/SourceNode';
import { TransformNode } from '@/components/studio/nodes/TransformNode';
import { TargetNode } from '@/components/studio/nodes/TargetNode';
import { NodeToolbox } from '@/components/studio/NodeToolbox';
import { NodeConfigPanel } from '@/components/studio/panels/NodeConfigPanel';
import { StudioHeader } from '@/components/studio/StudioHeader';

const nodeTypes: NodeTypes = {
  source: SourceNode,
  transform: TransformNode,
  target: TargetNode,
};

const initialNodes: Node[] = [
  {
    id: 'source-1',
    type: 'source',
    position: { x: 100, y: 200 },
    data: {
      label: 'PostgreSQL Source',
      connector: 'postgresql',
      config: {},
    },
  },
  {
    id: 'transform-1',
    type: 'transform',
    position: { x: 400, y: 200 },
    data: {
      label: 'Transform',
      transformType: 'map',
      config: {},
    },
  },
  {
    id: 'target-1',
    type: 'target',
    position: { x: 700, y: 200 },
    data: {
      label: 'Snowflake Target',
      connector: 'snowflake',
      config: {},
    },
  },
];

const initialEdges: Edge[] = [
  { id: 'e1-2', source: 'source-1', target: 'transform-1' },
  { id: 'e2-3', source: 'transform-1', target: 'target-1' },
];

export function IntegrationStudioPage() {
  const { id } = useParams();
  const isNew = !id;

  const [nodes, setNodes, onNodesChange] = useNodesState(isNew ? [] : initialNodes);
  const [edges, setEdges, onEdgesChange] = useEdgesState(isNew ? [] : initialEdges);
  const [selectedNode, setSelectedNode] = useState<Node | null>(null);
  const [integrationName, setIntegrationName] = useState(
    isNew ? 'Untitled Integration' : 'Salesforce → Snowflake Sync'
  );

  const onConnect = useCallback(
    (params: Connection) => setEdges((eds) => addEdge(params, eds)),
    [setEdges]
  );

  const onNodeClick = useCallback((_: React.MouseEvent, node: Node) => {
    setSelectedNode(node);
  }, []);

  const onPaneClick = useCallback(() => {
    setSelectedNode(null);
  }, []);

  const onDragOver = useCallback((event: React.DragEvent) => {
    event.preventDefault();
    event.dataTransfer.dropEffect = 'move';
  }, []);

  const onDrop = useCallback(
    (event: React.DragEvent) => {
      event.preventDefault();

      const type = event.dataTransfer.getData('application/reactflow');
      const nodeData = JSON.parse(event.dataTransfer.getData('application/nodedata'));

      if (!type) return;

      const position = {
        x: event.clientX - 300,
        y: event.clientY - 100,
      };

      const newNode: Node = {
        id: `${type}-${Date.now()}`,
        type,
        position,
        data: nodeData,
      };

      setNodes((nds) => nds.concat(newNode));
    },
    [setNodes]
  );

  const updateNodeConfig = useCallback(
    (nodeId: string, config: Record<string, unknown>) => {
      setNodes((nds) =>
        nds.map((node) => {
          if (node.id === nodeId) {
            return {
              ...node,
              data: { ...node.data, config },
            };
          }
          return node;
        })
      );
    },
    [setNodes]
  );

  return (
    <div className="h-[calc(100vh-4rem)] flex flex-col -m-6">
      <StudioHeader
        name={integrationName}
        onNameChange={setIntegrationName}
        onSave={() => console.log('Save', { nodes, edges })}
        onRun={() => console.log('Run integration')}
      />

      <div className="flex-1 flex">
        {/* Toolbox */}
        <NodeToolbox />

        {/* Canvas */}
        <div className="flex-1 bg-gray-50">
          <ReactFlow
            nodes={nodes}
            edges={edges}
            onNodesChange={onNodesChange}
            onEdgesChange={onEdgesChange}
            onConnect={onConnect}
            onNodeClick={onNodeClick}
            onPaneClick={onPaneClick}
            onDragOver={onDragOver}
            onDrop={onDrop}
            nodeTypes={nodeTypes}
            fitView
            snapToGrid
            snapGrid={[15, 15]}
          >
            <Background gap={15} size={1} />
            <Controls />
            <MiniMap
              nodeStrokeWidth={3}
              zoomable
              pannable
              className="bg-white border border-gray-200 rounded-lg"
            />
            <Panel position="top-right" className="bg-white rounded-lg shadow-sm border border-gray-200 p-2">
              <div className="text-xs text-gray-500">
                {nodes.length} nodes · {edges.length} connections
              </div>
            </Panel>
          </ReactFlow>
        </div>

        {/* Config Panel */}
        {selectedNode && (
          <NodeConfigPanel
            node={selectedNode}
            onClose={() => setSelectedNode(null)}
            onUpdate={(config) => updateNodeConfig(selectedNode.id, config)}
          />
        )}
      </div>
    </div>
  );
}
