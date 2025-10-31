// Task breakdown component for epic task decomposition
import React, { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Separator } from '@/components/ui/separator';
import {
  DecompositionResult,
  TaskCategory,
  SizeEstimate,
} from '@/services/epics';
import { CheckCircle2, Circle, ArrowRight } from 'lucide-react';

interface TaskBreakdownProps {
  epicId: string;
  decompositionResult: DecompositionResult | null;
  onCategoryChange?: (categories: TaskCategory[]) => void;
}

const sizeColors: Record<SizeEstimate, string> = {
  XS: 'bg-green-100 text-green-800',
  S: 'bg-blue-100 text-blue-800',
  M: 'bg-yellow-100 text-yellow-800',
  L: 'bg-orange-100 text-orange-800',
  XL: 'bg-red-100 text-red-800',
};

export function TaskBreakdown({
  decompositionResult,
}: TaskBreakdownProps) {
  const [expandedGroups, setExpandedGroups] = useState<Set<string>>(new Set());

  if (!decompositionResult) {
    return (
      <Card>
        <CardContent className="p-6">
          <p className="text-sm text-muted-foreground">
            No task breakdown available. Decompose the epic to generate tasks.
          </p>
        </CardContent>
      </Card>
    );
  }

  const toggleGroup = (groupId: string) => {
    const newExpanded = new Set(expandedGroups);
    if (newExpanded.has(groupId)) {
      newExpanded.delete(groupId);
    } else {
      newExpanded.add(groupId);
    }
    setExpandedGroups(newExpanded);
  };

  // Group tasks by category
  const tasksByCategory = decompositionResult.tasks.reduce((acc, task) => {
    const category = task.category || 'Uncategorized';
    if (!acc[category]) {
      acc[category] = [];
    }
    acc[category].push(task);
    return acc;
  }, {} as Record<string, any[]>);

  // Group tasks by parallel group
  const tasksByParallelGroup = decompositionResult.tasks.reduce((acc, task) => {
    const group = task.parallelGroup || 'No Group';
    if (!acc[group]) {
      acc[group] = [];
    }
    acc[group].push(task);
    return acc;
  }, {} as Record<string, any[]>);

  return (
    <div className="space-y-6">
      {/* Summary Stats */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">Task Breakdown Summary</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-3 gap-4">
            <div>
              <p className="text-sm font-medium text-muted-foreground">Total Tasks</p>
              <p className="text-2xl font-bold">{decompositionResult.tasks.length}</p>
            </div>
            <div>
              <p className="text-sm font-medium text-muted-foreground">Parallel Groups</p>
              <p className="text-2xl font-bold">{decompositionResult.parallelGroups.length}</p>
            </div>
            <div>
              <p className="text-sm font-medium text-muted-foreground">Dependencies</p>
              <p className="text-2xl font-bold">{decompositionResult.dependencyGraph.edges.length}</p>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Tasks by Category */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">Tasks by Category</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            {Object.entries(tasksByCategory).map(([category, tasks]) => (
              <div key={category}>
                <Button
                  variant="ghost"
                  className="w-full justify-between p-2"
                  onClick={() => toggleGroup(`category-${category}`)}
                >
                  <span className="font-medium">{category} ({tasks.length})</span>
                  <ArrowRight
                    className={`h-4 w-4 transition-transform ${
                      expandedGroups.has(`category-${category}`) ? 'rotate-90' : ''
                    }`}
                  />
                </Button>
                {expandedGroups.has(`category-${category}`) && (
                  <div className="ml-4 space-y-2 mt-2">
                    {tasks.map((task) => (
                      <div key={task.id} className="flex items-start gap-2 p-2 rounded-lg border">
                        <Circle className="h-4 w-4 mt-0.5 text-muted-foreground" />
                        <div className="flex-1">
                          <p className="font-medium text-sm">{task.title}</p>
                          {task.description && (
                            <p className="text-xs text-muted-foreground mt-1">{task.description}</p>
                          )}
                          <div className="flex gap-2 mt-2">
                            {task.sizeEstimate && (
                              <Badge variant="secondary" className={sizeColors[task.sizeEstimate as SizeEstimate]}>
                                {task.sizeEstimate}
                              </Badge>
                            )}
                            {task.effortHours && (
                              <Badge variant="outline">{task.effortHours}h</Badge>
                            )}
                            {task.parallelGroup && (
                              <Badge variant="outline">Group: {task.parallelGroup}</Badge>
                            )}
                          </div>
                        </div>
                      </div>
                    ))}
                  </div>
                )}
              </div>
            ))}
          </div>
        </CardContent>
      </Card>

      {/* Parallel Groups */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">Parallel Execution Groups</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            {decompositionResult.parallelGroups.map((group, index) => {
              const groupTasks = decompositionResult.tasks.filter(t =>
                group.taskIds.includes(t.id)
              );
              return (
                <div key={group.id}>
                  <div className="flex items-center gap-2 mb-2">
                    <Badge variant="default">Group {index + 1}</Badge>
                    <span className="text-sm text-muted-foreground">
                      {groupTasks.length} tasks can run in parallel
                    </span>
                  </div>
                  <div className="grid grid-cols-1 md:grid-cols-2 gap-2 ml-4">
                    {groupTasks.map((task) => (
                      <div key={task.id} className="p-2 rounded border bg-muted/30">
                        <p className="text-sm font-medium">{task.title}</p>
                      </div>
                    ))}
                  </div>
                </div>
              );
            })}
          </div>
        </CardContent>
      </Card>

      {/* Conflicts */}
      {decompositionResult.conflicts.length > 0 && (
        <Card>
          <CardHeader>
            <CardTitle className="text-lg text-orange-600">Potential Conflicts</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-2">
              {decompositionResult.conflicts.map((conflict, index) => {
                const task1 = decompositionResult.tasks.find(t => t.id === conflict.task1);
                const task2 = decompositionResult.tasks.find(t => t.id === conflict.task2);
                return (
                  <div key={index} className="p-3 rounded-lg border border-orange-200 bg-orange-50">
                    <p className="text-sm">
                      <span className="font-medium">{task1?.title}</span>
                      {' ↔ '}
                      <span className="font-medium">{task2?.title}</span>
                    </p>
                    <p className="text-xs text-muted-foreground mt-1">{conflict.reason}</p>
                  </div>
                );
              })}
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  );
}
