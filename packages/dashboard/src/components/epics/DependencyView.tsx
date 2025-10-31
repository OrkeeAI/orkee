// Dependency graph visualization component
import React from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { DecompositionResult } from '@/services/epics';
import { ArrowRight, Circle } from 'lucide-react';

interface DependencyViewProps {
  decompositionResult: DecompositionResult | null;
}

export function DependencyView({ decompositionResult }: DependencyViewProps) {
  if (!decompositionResult || decompositionResult.dependencyGraph.nodes.length === 0) {
    return (
      <Card>
        <CardContent className="p-6">
          <p className="text-sm text-muted-foreground">
            No dependency graph available. Decompose the epic first to view dependencies.
          </p>
        </CardContent>
      </Card>
    );
  }

  const { nodes, edges } = decompositionResult.dependencyGraph;

  // Build adjacency list for rendering
  const adjacencyList = new Map<string, string[]>();
  const incomingEdges = new Map<string, string[]>();

  edges.forEach(edge => {
    if (!adjacencyList.has(edge.from)) {
      adjacencyList.set(edge.from, []);
    }
    adjacencyList.get(edge.from)!.push(edge.to);

    if (!incomingEdges.has(edge.to)) {
      incomingEdges.set(edge.to, []);
    }
    incomingEdges.get(edge.to)!.push(edge.from);
  });

  // Find root nodes (no incoming edges)
  const rootNodes = nodes.filter(node => !incomingEdges.has(node.id));

  // Find leaf nodes (no outgoing edges)
  const leafNodes = nodes.filter(node => !adjacencyList.has(node.id));

  const getTaskById = (taskId: string) => {
    return decompositionResult.tasks.find(t => t.id === taskId);
  };

  return (
    <div className="space-y-6">
      {/* Graph Statistics */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">Dependency Graph Statistics</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-4 gap-4">
            <div>
              <p className="text-sm font-medium text-muted-foreground">Total Tasks</p>
              <p className="text-2xl font-bold">{nodes.length}</p>
            </div>
            <div>
              <p className="text-sm font-medium text-muted-foreground">Dependencies</p>
              <p className="text-2xl font-bold">{edges.length}</p>
            </div>
            <div>
              <p className="text-sm font-medium text-muted-foreground">Starting Points</p>
              <p className="text-2xl font-bold">{rootNodes.length}</p>
            </div>
            <div>
              <p className="text-sm font-medium text-muted-foreground">End Points</p>
              <p className="text-2xl font-bold">{leafNodes.length}</p>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Task Dependencies */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">Task Dependencies</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            {nodes.map(node => {
              const task = getTaskById(node.id);
              const dependencies = incomingEdges.get(node.id) || [];
              const dependents = adjacencyList.get(node.id) || [];

              return (
                <div key={node.id} className="p-4 rounded-lg border">
                  <div className="flex items-start gap-3">
                    <Circle className="h-5 w-5 mt-0.5 text-primary" />
                    <div className="flex-1 space-y-2">
                      <div>
                        <p className="font-medium">{node.label}</p>
                        {task?.parallelGroup && (
                          <Badge variant="outline" className="mt-1">
                            {task.parallelGroup}
                          </Badge>
                        )}
                      </div>

                      {dependencies.length > 0 && (
                        <div className="pl-4 border-l-2 border-blue-200">
                          <p className="text-xs font-medium text-muted-foreground mb-1">
                            Depends on:
                          </p>
                          <div className="space-y-1">
                            {dependencies.map(depId => {
                              const depNode = nodes.find(n => n.id === depId);
                              return (
                                <div key={depId} className="flex items-center gap-2 text-sm">
                                  <ArrowRight className="h-3 w-3 text-blue-500" />
                                  <span className="text-muted-foreground">{depNode?.label}</span>
                                </div>
                              );
                            })}
                          </div>
                        </div>
                      )}

                      {dependents.length > 0 && (
                        <div className="pl-4 border-l-2 border-green-200">
                          <p className="text-xs font-medium text-muted-foreground mb-1">
                            Required by:
                          </p>
                          <div className="space-y-1">
                            {dependents.map(depId => {
                              const depNode = nodes.find(n => n.id === depId);
                              return (
                                <div key={depId} className="flex items-center gap-2 text-sm">
                                  <ArrowRight className="h-3 w-3 text-green-500" />
                                  <span className="text-muted-foreground">{depNode?.label}</span>
                                </div>
                              );
                            })}
                          </div>
                        </div>
                      )}

                      {dependencies.length === 0 && dependents.length === 0 && (
                        <p className="text-xs text-muted-foreground">No dependencies</p>
                      )}
                    </div>
                  </div>
                </div>
              );
            })}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
