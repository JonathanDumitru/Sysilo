import { useCallback, useState, useEffect, useRef } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
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
  useReactFlow,
  ReactFlowProvider,
} from '@xyflow/react';
import '@xyflow/react/dist/style.css';
import { ArrowLeft, Play, Save, AlertCircle } from 'lucide-react';

import {
  IntegrationStepNode,
  WebhookStepNode,
  WaitStepNode,
  ConditionStepNode,
  ApprovalStepNode,
  type StepNodeData,
} from '@/components/playbooks/nodes';
import { PlaybookToolbox } from '@/components/playbooks/PlaybookToolbox';
import {
  usePlaybook,
  useCreatePlaybook,
  useUpdatePlaybook,
  useRunPlaybook,
} from '@/hooks/usePlaybooks';
import type { Step, StepType, TriggerType, Variable } from '@/services/playbooks';

// Register custom node types
const nodeTypes: NodeTypes = {
  integration: IntegrationStepNode,
  webhook: WebhookStepNode,
  wait: WaitStepNode,
  condition: ConditionStepNode,
  approval: ApprovalStepNode,
};

// Convert Step[] to React Flow nodes
function stepsToNodes(steps: Step[]): Node[] {
  return steps.map((step, index) => ({
    id: step.id,
    type: step.step_type,
    position: (step.config?.position as { x: number; y: number }) || { x: 100, y: 100 + index * 150 },
    data: {
      id: step.id,
      name: step.name,
      config: step.config,
    } as StepNodeData,
  }));
}

// Convert Step[] to React Flow edges based on on_success and on_failure arrays
function stepsToEdges(steps: Step[]): Edge[] {
  const edges: Edge[] = [];

  steps.forEach((step) => {
    // Handle success branch connections
    step.on_success.forEach((targetId) => {
      edges.push({
        id: `${step.id}-${targetId}-success`,
        source: step.id,
        target: targetId,
        sourceHandle: step.step_type === 'condition' ? 'true' : undefined,
        style: { stroke: step.step_type === 'condition' ? '#22c55e' : undefined },
      });
    });

    // Handle failure branch connections (used by condition nodes for false branch)
    step.on_failure.forEach((targetId) => {
      edges.push({
        id: `${step.id}-${targetId}-failure`,
        source: step.id,
        target: targetId,
        sourceHandle: step.step_type === 'condition' ? 'false' : undefined,
        style: { stroke: '#ef4444' },
      });
    });
  });

  return edges;
}

// Convert React Flow nodes/edges back to Step[]
function nodesToSteps(nodes: Node[], edges: Edge[]): Step[] {
  return nodes.map((node) => {
    const nodeData = node.data as StepNodeData;

    // Get all edges from this node
    const successEdges = edges.filter(
      (e) => e.source === node.id && e.sourceHandle !== 'false'
    );
    const failureEdges = edges.filter(
      (e) => e.source === node.id && e.sourceHandle === 'false'
    );

    return {
      id: node.id,
      name: nodeData.name,
      step_type: node.type as StepType,
      config: {
        ...nodeData.config,
        position: node.position,
      },
      on_success: successEdges.map((e) => e.target),
      on_failure: failureEdges.map((e) => e.target),
    };
  });
}

function PlaybookEditorContent() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const reactFlowWrapper = useRef<HTMLDivElement>(null);
  const { screenToFlowPosition } = useReactFlow();

  const isNew = !id || id === 'new';

  const [name, setName] = useState('New Playbook');
  const [description, setDescription] = useState('');
  const [triggerType, setTriggerType] = useState<TriggerType>('manual');
  const [variables, setVariables] = useState<Variable[]>([]);
  const [nodes, setNodes, onNodesChange] = useNodesState([] as Node[]);
  const [edges, setEdges, onEdgesChange] = useEdgesState([] as Edge[]);
  const [saveError, setSaveError] = useState<string | null>(null);

  const { data: playbook, isLoading, error: loadError } = usePlaybook(isNew ? '' : id!);
  const createMutation = useCreatePlaybook();
  const updateMutation = useUpdatePlaybook();
  const runMutation = useRunPlaybook();

  // Load existing playbook
  useEffect(() => {
    if (playbook) {
      setName(playbook.name);
      setDescription(playbook.description || '');
      setTriggerType(playbook.trigger_type);
      setVariables(playbook.variables);
      setNodes(stepsToNodes(playbook.steps));
      setEdges(stepsToEdges(playbook.steps));
    }
  }, [playbook, setNodes, setEdges]);

  // Handle new edge connections
  const onConnect = useCallback(
    (params: Connection) => {
      setEdges((eds) => addEdge(params, eds));
    },
    [setEdges]
  );

  // Handle drag-and-drop from toolbox
  const onDragOver = useCallback((event: React.DragEvent) => {
    event.preventDefault();
    event.dataTransfer.dropEffect = 'move';
  }, []);

  const onDrop = useCallback(
    (event: React.DragEvent) => {
      event.preventDefault();

      const stepType = event.dataTransfer.getData('application/reactflow') as StepType;
      const nodeDataStr = event.dataTransfer.getData('application/nodedata');

      if (!stepType || !nodeDataStr) return;

      const parsedData = JSON.parse(nodeDataStr) as { name: string; config: Record<string, unknown> };

      // Get the position where the node was dropped
      const position = screenToFlowPosition({
        x: event.clientX,
        y: event.clientY,
      });

      const newNodeId = `${stepType}-${Date.now()}`;
      const newNode: Node = {
        id: newNodeId,
        type: stepType,
        position,
        data: {
          id: newNodeId,
          name: parsedData.name,
          config: parsedData.config,
        } as StepNodeData,
      };

      setNodes((nds) => [...nds, newNode]);
    },
    [setNodes, screenToFlowPosition]
  );

  // Save playbook
  const handleSave = async () => {
    setSaveError(null);

    try {
      const steps = nodesToSteps(nodes, edges);

      if (isNew) {
        const result = await createMutation.mutateAsync({
          name,
          description: description || undefined,
          trigger_type: triggerType,
          steps,
          variables,
        });
        navigate(`/operations/playbooks/${result.id}/edit`, { replace: true });
      } else {
        await updateMutation.mutateAsync({
          id: id!,
          request: {
            name,
            description: description || undefined,
            trigger_type: triggerType,
            steps,
            variables,
          },
        });
      }
    } catch (err) {
      setSaveError(err instanceof Error ? err.message : 'Failed to save playbook');
    }
  };

  // Run playbook
  const handleRun = async () => {
    if (!id || isNew) return;

    try {
      const run = await runMutation.mutateAsync({ id, request: { variables: {} } });
      navigate(`/operations/playbooks/${id}/runs/${run.id}`);
    } catch (err) {
      setSaveError(err instanceof Error ? err.message : 'Failed to run playbook');
    }
  };

  // Loading state
  if (!isNew && isLoading) {
    return (
      <div className="h-[calc(100vh-4rem)] flex items-center justify-center -m-6">
        <div className="flex flex-col items-center gap-3">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary-600" />
          <p className="text-sm text-gray-500">Loading playbook...</p>
        </div>
      </div>
    );
  }

  // Error state
  if (!isNew && loadError) {
    return (
      <div className="h-[calc(100vh-4rem)] flex items-center justify-center -m-6">
        <div className="flex flex-col items-center gap-3 text-center">
          <AlertCircle className="w-12 h-12 text-red-500" />
          <h2 className="text-lg font-semibold text-gray-900">Failed to load playbook</h2>
          <p className="text-sm text-gray-500">
            {loadError instanceof Error ? loadError.message : 'An unexpected error occurred'}
          </p>
          <button
            onClick={() => navigate('/operations/playbooks')}
            className="mt-2 px-4 py-2 text-sm font-medium text-primary-600 hover:text-primary-700"
          >
            Back to Playbooks
          </button>
        </div>
      </div>
    );
  }

  const isSaving = createMutation.isPending || updateMutation.isPending;
  const isRunning = runMutation.isPending;

  return (
    <div className="h-[calc(100vh-4rem)] flex flex-col -m-6">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-3 border-b border-gray-200 bg-white">
        <div className="flex items-center gap-4">
          <button
            onClick={() => navigate('/operations/playbooks')}
            className="flex items-center gap-1 text-gray-500 hover:text-gray-700 text-sm"
          >
            <ArrowLeft className="w-4 h-4" />
            Back
          </button>
          <div className="h-6 w-px bg-gray-200" />
          <input
            type="text"
            value={name}
            onChange={(e) => setName(e.target.value)}
            className="text-lg font-semibold border-none focus:ring-0 focus:outline-none bg-transparent"
            placeholder="Playbook name"
          />
        </div>
        <div className="flex items-center gap-2">
          {saveError && (
            <div className="flex items-center gap-1 text-sm text-red-600 mr-2">
              <AlertCircle className="w-4 h-4" />
              <span>{saveError}</span>
            </div>
          )}
          {!isNew && (
            <button
              onClick={handleRun}
              disabled={isRunning || isSaving}
              className="flex items-center gap-2 px-4 py-2 text-sm font-medium text-primary-600 bg-primary-50 rounded-lg hover:bg-primary-100 disabled:opacity-50 disabled:cursor-not-allowed"
            >
              <Play className="w-4 h-4" />
              {isRunning ? 'Running...' : 'Run'}
            </button>
          )}
          <button
            onClick={handleSave}
            disabled={isSaving}
            className="flex items-center gap-2 px-4 py-2 text-sm font-medium text-white bg-primary-600 rounded-lg hover:bg-primary-700 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <Save className="w-4 h-4" />
            {isSaving ? 'Saving...' : 'Save'}
          </button>
        </div>
      </div>

      {/* Main content */}
      <div className="flex-1 flex">
        {/* Toolbox sidebar */}
        <PlaybookToolbox />

        {/* React Flow canvas */}
        <div
          ref={reactFlowWrapper}
          className="flex-1 bg-gray-50"
          onDragOver={onDragOver}
          onDrop={onDrop}
        >
          <ReactFlow
            nodes={nodes}
            edges={edges}
            onNodesChange={onNodesChange}
            onEdgesChange={onEdgesChange}
            onConnect={onConnect}
            nodeTypes={nodeTypes}
            fitView
            snapToGrid
            snapGrid={[15, 15]}
            defaultEdgeOptions={{
              type: 'smoothstep',
            }}
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
                {nodes.length} steps &middot; {edges.length} connections
              </div>
            </Panel>
            {nodes.length === 0 && (
              <Panel position="top-center" className="mt-20">
                <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6 text-center max-w-md">
                  <h3 className="text-sm font-medium text-gray-900 mb-1">
                    Start building your playbook
                  </h3>
                  <p className="text-xs text-gray-500">
                    Drag steps from the toolbox on the left onto the canvas to create your automation workflow.
                  </p>
                </div>
              </Panel>
            )}
          </ReactFlow>
        </div>
      </div>
    </div>
  );
}

export function AutomationPlaybookEditorPage() {
  return (
    <ReactFlowProvider>
      <PlaybookEditorContent />
    </ReactFlowProvider>
  );
}
