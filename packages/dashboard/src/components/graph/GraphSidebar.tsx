// ABOUTME: Graph node details sidebar component
// ABOUTME: Displays metadata, dependencies, and actions for selected graph nodes

import { useMemo } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Separator } from '@/components/ui/separator';
import { X, ExternalLink, FileText, Code } from 'lucide-react';
import type { CodeGraph } from '@/services/graph';

interface GraphSidebarProps {
  nodeId: string;
  graphData: CodeGraph | undefined;
  onClose: () => void;
}

export function GraphSidebar({
  nodeId,
  graphData,
  onClose,
}: GraphSidebarProps) {
  const nodeDetails = useMemo(() => {
    if (!graphData) return null;

    const node = graphData.nodes.find((n) => n.id === nodeId);
    if (!node) return null;

    // Find incoming and outgoing edges
    const incomingEdges = graphData.edges.filter((e) => e.target === nodeId);
    const outgoingEdges = graphData.edges.filter((e) => e.source === nodeId);

    // Get connected nodes
    const dependencies = incomingEdges.map((edge) => {
      const sourceNode = graphData.nodes.find((n) => n.id === edge.source);
      return {
        id: edge.source,
        label: sourceNode?.label || edge.source,
        type: edge.edge_type,
      };
    });

    const dependents = outgoingEdges.map((edge) => {
      const targetNode = graphData.nodes.find((n) => n.id === edge.target);
      return {
        id: edge.target,
        label: targetNode?.label || edge.target,
        type: edge.edge_type,
      };
    });

    return {
      ...node,
      dependencies,
      dependents,
    };
  }, [graphData, nodeId]);

  if (!nodeDetails) {
    return (
      <Card className="w-[400px] h-full">
        <CardHeader className="flex flex-row items-center justify-between">
          <CardTitle>Node Details</CardTitle>
          <Button variant="ghost" size="icon" onClick={onClose}>
            <X className="h-4 w-4" />
          </Button>
        </CardHeader>
        <CardContent>
          <p className="text-sm text-muted-foreground">Node not found</p>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card className="w-[400px] h-full overflow-hidden flex flex-col">
      <CardHeader className="flex flex-row items-center justify-between flex-shrink-0">
        <CardTitle className="text-lg">Node Details</CardTitle>
        <Button variant="ghost" size="icon" onClick={onClose}>
          <X className="h-4 w-4" />
        </Button>
      </CardHeader>

      <CardContent className="space-y-4 overflow-y-auto flex-1">
        {/* Node Header */}
        <div>
          <h3 className="font-semibold mb-2 break-words">{nodeDetails.label}</h3>
          <Badge variant="secondary" className="capitalize">
            {nodeDetails.node_type}
          </Badge>
        </div>

        <Separator />

        {/* Metadata */}
        {'path' in nodeDetails.metadata && nodeDetails.metadata.path && (
          <div>
            <p className="text-sm font-medium text-muted-foreground mb-1">Path</p>
            <p className="text-sm font-mono break-all">{nodeDetails.metadata.path}</p>
          </div>
        )}

        {'line_start' in nodeDetails.metadata && (nodeDetails.metadata.line_start || nodeDetails.metadata.line_end) && (
          <div>
            <p className="text-sm font-medium text-muted-foreground mb-1">Lines</p>
            <p className="text-sm">
              {nodeDetails.metadata.line_start}
              {nodeDetails.metadata.line_end &&
                ` - ${nodeDetails.metadata.line_end}`}
            </p>
          </div>
        )}

        {'token_count' in nodeDetails.metadata && nodeDetails.metadata.token_count != null && (
          <div>
            <p className="text-sm font-medium text-muted-foreground mb-1">
              Token Count
            </p>
            <p className="text-sm">{nodeDetails.metadata.token_count.toLocaleString()}</p>
          </div>
        )}

        {'complexity' in nodeDetails.metadata && nodeDetails.metadata.complexity != null && (
          <div>
            <p className="text-sm font-medium text-muted-foreground mb-1">
              Complexity
            </p>
            <p className="text-sm">{nodeDetails.metadata.complexity.toFixed(2)}</p>
          </div>
        )}

        <Separator />

        {/* Dependencies */}
        {nodeDetails.dependencies.length > 0 && (
          <div>
            <p className="text-sm font-medium text-muted-foreground mb-2">
              Dependencies ({nodeDetails.dependencies.length})
            </p>
            <div className="space-y-1 max-h-[200px] overflow-y-auto">
              {nodeDetails.dependencies.map((dep) => (
                <div
                  key={dep.id}
                  className="text-sm flex items-start gap-2 p-2 rounded hover:bg-muted/50"
                >
                  <span className="text-muted-foreground">•</span>
                  <div className="flex-1 min-w-0">
                    <p className="break-words">{dep.label}</p>
                    <Badge variant="outline" className="text-xs mt-1">
                      {dep.type}
                    </Badge>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}

        {/* Dependents */}
        {nodeDetails.dependents.length > 0 && (
          <div>
            <p className="text-sm font-medium text-muted-foreground mb-2">
              Dependents ({nodeDetails.dependents.length})
            </p>
            <div className="space-y-1 max-h-[200px] overflow-y-auto">
              {nodeDetails.dependents.map((dep) => (
                <div
                  key={dep.id}
                  className="text-sm flex items-start gap-2 p-2 rounded hover:bg-muted/50"
                >
                  <span className="text-muted-foreground">•</span>
                  <div className="flex-1 min-w-0">
                    <p className="break-words">{dep.label}</p>
                    <Badge variant="outline" className="text-xs mt-1">
                      {dep.type}
                    </Badge>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}

        {nodeDetails.dependencies.length === 0 &&
          nodeDetails.dependents.length === 0 && (
            <div>
              <p className="text-sm text-muted-foreground">
                No connections found for this node
              </p>
            </div>
          )}

        <Separator />

        {/* Actions */}
        <div className="space-y-2">
          <p className="text-sm font-medium text-muted-foreground mb-2">Actions</p>
          {'path' in nodeDetails.metadata && nodeDetails.metadata.path && (
            <Button variant="outline" size="sm" className="w-full justify-start">
              <FileText className="h-4 w-4 mr-2" />
              Open File
            </Button>
          )}
          <Button variant="outline" size="sm" className="w-full justify-start">
            <Code className="h-4 w-4 mr-2" />
            Add to Context
          </Button>
          <Button variant="outline" size="sm" className="w-full justify-start">
            <ExternalLink className="h-4 w-4 mr-2" />
            View in Editor
          </Button>
        </div>
      </CardContent>
    </Card>
  );
}
