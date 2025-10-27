// ABOUTME: Interactive dependency graph visualization using React Flow
// ABOUTME: Shows feature dependencies with ability to create/delete connections

import { useCallback, useMemo } from 'react';
import {
  ReactFlow,
  Node,
  Edge,
  Background,
  Controls,
  MiniMap,
  useNodesState,
  useEdgesState,
  Connection,
  ConnectionMode,
} from '@xyflow/react';
import '@xyflow/react/dist/style.css';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Sparkles } from 'lucide-react';
import {
  useFeatureDependencies,
  useCreateFeatureDependency,
  useDeleteFeatureDependency,
  useAnalyzeDependencies,
  useCircularDependencies,
} from '@/hooks/useIdeate';
import type { FeatureDependency } from '@/services/ideate';
import { toast } from 'sonner';

interface DependencyMapperProps {
  sessionId: string;
  features: string[];
}

export function DependencyMapper({ sessionId, features }: DependencyMapperProps) {
  const { data: dependencies = [], isLoading } = useFeatureDependencies(sessionId);
  const { data: circularDeps = [] } = useCircularDependencies(sessionId);
  const createMutation = useCreateFeatureDependency(sessionId);
  const deleteMutation = useDeleteFeatureDependency(sessionId);
  const analyzeMutation = useAnalyzeDependencies(sessionId);

  // Create nodes from features
  const initialNodes: Node[] = useMemo(() => {
    return features.map((feature, index) => ({
      id: feature,
      type: 'default',
      position: {
        x: (index % 3) * 250,
        y: Math.floor(index / 3) * 150,
      },
      data: {
        label: (
          <div className="px-2 py-1">
            <div className="font-medium text-sm">{feature}</div>
          </div>
        ),
      },
    }));
  }, [features]);

  // Create edges from dependencies
  const initialEdges: Edge[] = useMemo(() => {
    return dependencies.map((dep: FeatureDependency) => {
      const isInCircle = circularDeps.some((circle) =>
        circle.cycle.includes(dep.from_feature_id) &&
        circle.cycle.includes(dep.to_feature_id)
      );

      return {
        id: dep.id,
        source: dep.from_feature_id,
        target: dep.to_feature_id,
        label: dep.dependency_type,
        animated: dep.strength === 'required',
        style: {
          stroke: isInCircle
            ? '#ef4444'
            : dep.strength === 'required'
            ? '#3b82f6'
            : dep.strength === 'recommended'
            ? '#8b5cf6'
            : '#94a3b8',
          strokeWidth: dep.strength === 'required' ? 2 : 1,
        },
        markerEnd: {
          type: 'arrowclosed' as const,
          color: isInCircle
            ? '#ef4444'
            : dep.strength === 'required'
            ? '#3b82f6'
            : dep.strength === 'recommended'
            ? '#8b5cf6'
            : '#94a3b8',
        },
      };
    });
  }, [dependencies, circularDeps]);

  const [nodes, , onNodesChange] = useNodesState(initialNodes);
  const [edges, , onEdgesChange] = useEdgesState(initialEdges);

  // Handle new connection (manual dependency creation)
  const onConnect = useCallback(
    async (connection: Connection) => {
      if (!connection.source || !connection.target) return;

      try {
        await createMutation.mutateAsync({
          fromFeatureId: connection.source,
          toFeatureId: connection.target,
          dependencyType: 'logical',
          strength: 'recommended',
        });
        toast.success('Dependency created');
      } catch {
        toast.error('Failed to create dependency');
      }
    },
    [createMutation]
  );

  // Handle edge deletion
  const onEdgeClick = useCallback(
    async (_event: React.MouseEvent, edge: Edge) => {
      if (window.confirm('Delete this dependency?')) {
        try {
          await deleteMutation.mutateAsync(edge.id);
          toast.success('Dependency deleted');
        } catch {
          toast.error('Failed to delete dependency');
        }
      }
    },
    [deleteMutation]
  );

  // Handle AI analysis
  const handleAnalyze = async () => {
    try {
      await analyzeMutation.mutateAsync();
      toast.success('Dependencies analyzed with AI');
    } catch (error: unknown) {
      toast.error((error as Error).message || 'Failed to analyze dependencies');
    }
  };

  if (isLoading) {
    return (
      <div className="h-[500px] flex items-center justify-center border rounded-lg">
        <div className="text-sm text-muted-foreground">Loading dependencies...</div>
      </div>
    );
  }

  if (features.length === 0) {
    return (
      <div className="h-[500px] flex items-center justify-center border rounded-lg">
        <div className="text-sm text-muted-foreground">
          Add features above to visualize dependencies
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <h3 className="font-medium">Dependency Graph</h3>
          {circularDeps.length > 0 && (
            <Badge variant="destructive">{circularDeps.length} circular</Badge>
          )}
        </div>
        <Button
          size="sm"
          variant="outline"
          onClick={handleAnalyze}
          disabled={analyzeMutation.isPending}
        >
          <Sparkles className="w-4 h-4 mr-2" />
          {analyzeMutation.isPending ? 'Analyzing...' : 'AI Analyze'}
        </Button>
      </div>

      <div className="h-[500px] border rounded-lg bg-muted/20">
        <ReactFlow
          nodes={nodes}
          edges={edges}
          onNodesChange={onNodesChange}
          onEdgesChange={onEdgesChange}
          onConnect={onConnect}
          onEdgeClick={onEdgeClick}
          connectionMode={ConnectionMode.Loose}
          fitView
        >
          <Background />
          <Controls />
          <MiniMap />
        </ReactFlow>
      </div>

      <div className="text-xs text-muted-foreground space-y-1">
        <div className="flex items-center gap-4">
          <div className="flex items-center gap-2">
            <div className="w-3 h-0.5 bg-blue-500" />
            <span>Required (animated)</span>
          </div>
          <div className="flex items-center gap-2">
            <div className="w-3 h-0.5 bg-purple-500" />
            <span>Recommended</span>
          </div>
          <div className="flex items-center gap-2">
            <div className="w-3 h-0.5 bg-slate-400" />
            <span>Optional</span>
          </div>
          <div className="flex items-center gap-2">
            <div className="w-3 h-0.5 bg-red-500" />
            <span>Circular (warning)</span>
          </div>
        </div>
        <div>Drag nodes to reposition. Click edges to delete. Connect nodes to create dependencies.</div>
      </div>
    </div>
  );
}
