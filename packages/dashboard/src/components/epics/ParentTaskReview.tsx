// ABOUTME: Parent Task Review UI for two-phase task generation
// ABOUTME: Allows users to review and edit parent tasks before expanding to subtasks

import { useState } from 'react';
import { DragDropContext, Droppable, Draggable, DropResult } from '@hello-pangea/dnd';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Input } from '@/components/ui/input';
import { Textarea } from '@/components/ui/textarea';
import { GripVertical, Edit2, Trash2, Plus, ChevronDown, ChevronRight } from 'lucide-react';
import type { ParentTask } from '@/services/epics';

interface ParentTaskReviewProps {
  parentTasks: ParentTask[];
  estimatedTotalTasks: number;
  complexity: number;
  onTasksChange: (tasks: ParentTask[]) => void;
  onGenerateDetailedTasks: () => void;
  isGenerating?: boolean;
}

export function ParentTaskReview({
  parentTasks,
  estimatedTotalTasks,
  complexity,
  onTasksChange,
  onGenerateDetailedTasks,
  isGenerating = false,
}: ParentTaskReviewProps) {
  const [tasks, setTasks] = useState<ParentTask[]>(parentTasks);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [expandedIds, setExpandedIds] = useState<Set<string>>(new Set());

  const handleDragEnd = (result: DropResult) => {
    if (!result.destination) return;

    const items = Array.from(tasks);
    const [reorderedItem] = items.splice(result.source.index, 1);
    items.splice(result.destination.index, 0, reorderedItem);

    // Update order numbers
    const updatedItems = items.map((item, index) => ({
      ...item,
      order: index + 1,
    }));

    setTasks(updatedItems);
    onTasksChange(updatedItems);
  };

  const handleEdit = (id: string) => {
    setEditingId(editingId === id ? null : id);
  };

  const handleDelete = (id: string) => {
    const updatedTasks = tasks
      .filter(t => t.id !== id)
      .map((task, index) => ({ ...task, order: index + 1 }));
    setTasks(updatedTasks);
    onTasksChange(updatedTasks);
  };

  const handleAddNew = () => {
    const newTask: ParentTask = {
      id: `temp_${Date.now()}`,
      title: 'New Parent Task',
      description: '',
      order: tasks.length + 1,
      estimatedSubtasks: 2,
    };
    const updatedTasks = [...tasks, newTask];
    setTasks(updatedTasks);
    onTasksChange(updatedTasks);
    setEditingId(newTask.id);
  };

  const handleUpdate = (id: string, field: keyof ParentTask, value: string | number) => {
    const updatedTasks = tasks.map(task =>
      task.id === id ? { ...task, [field]: value } : task
    );
    setTasks(updatedTasks);
    onTasksChange(updatedTasks);
  };

  const toggleExpand = (id: string) => {
    const newExpanded = new Set(expandedIds);
    if (newExpanded.has(id)) {
      newExpanded.delete(id);
    } else {
      newExpanded.add(id);
    }
    setExpandedIds(newExpanded);
  };

  const getComplexityColor = () => {
    if (complexity <= 3) return 'text-green-600';
    if (complexity <= 6) return 'text-yellow-600';
    if (complexity <= 8) return 'text-orange-600';
    return 'text-red-600';
  };

  return (
    <div className="space-y-6">
      {/* Summary Card */}
      <Card>
        <CardHeader>
          <CardTitle>Parent Task Review</CardTitle>
          <CardDescription>
            Review and edit high-level tasks before generating detailed subtasks
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-3 gap-4">
            <div>
              <p className="text-sm font-medium text-muted-foreground">Parent Tasks</p>
              <p className="text-2xl font-bold">{tasks.length}</p>
            </div>
            <div>
              <p className="text-sm font-medium text-muted-foreground">Estimated Total Tasks</p>
              <p className="text-2xl font-bold">{estimatedTotalTasks}</p>
            </div>
            <div>
              <p className="text-sm font-medium text-muted-foreground">Complexity Score</p>
              <p className={`text-2xl font-bold ${getComplexityColor()}`}>{complexity}/10</p>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Parent Tasks List */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <CardTitle className="text-lg">Parent Tasks</CardTitle>
            <Button onClick={handleAddNew} size="sm" variant="outline">
              <Plus className="h-4 w-4 mr-2" />
              Add Task
            </Button>
          </div>
        </CardHeader>
        <CardContent>
          <DragDropContext onDragEnd={handleDragEnd}>
            <Droppable droppableId="parent-tasks">
              {(provided) => (
                <div
                  {...provided.droppableProps}
                  ref={provided.innerRef}
                  className="space-y-3"
                >
                  {tasks.map((task, index) => (
                    <Draggable key={task.id} draggableId={task.id} index={index}>
                      {(provided) => (
                        <div
                          ref={provided.innerRef}
                          {...provided.draggableProps}
                          className="border rounded-lg p-4 bg-card"
                        >
                          <div className="flex items-start gap-3">
                            <div {...provided.dragHandleProps} className="mt-1">
                              <GripVertical className="h-5 w-5 text-muted-foreground cursor-grab" />
                            </div>
                            <div className="flex-1">
                              <div className="flex items-start justify-between mb-2">
                                <div className="flex items-center gap-2">
                                  <Button
                                    variant="ghost"
                                    size="sm"
                                    className="h-6 w-6 p-0"
                                    onClick={() => toggleExpand(task.id)}
                                  >
                                    {expandedIds.has(task.id) ? (
                                      <ChevronDown className="h-4 w-4" />
                                    ) : (
                                      <ChevronRight className="h-4 w-4" />
                                    )}
                                  </Button>
                                  <Badge variant="outline">{task.order}</Badge>
                                  {editingId === task.id ? (
                                    <Input
                                      value={task.title}
                                      onChange={(e) => handleUpdate(task.id, 'title', e.target.value)}
                                      className="h-8"
                                      autoFocus
                                    />
                                  ) : (
                                    <h4 className="font-semibold">{task.title}</h4>
                                  )}
                                </div>
                                <div className="flex items-center gap-2">
                                  <Badge variant="secondary">
                                    ~{task.estimatedSubtasks || 2} subtasks
                                  </Badge>
                                  <Button
                                    variant="ghost"
                                    size="sm"
                                    className="h-8 w-8 p-0"
                                    onClick={() => handleEdit(task.id)}
                                  >
                                    <Edit2 className="h-4 w-4" />
                                  </Button>
                                  <Button
                                    variant="ghost"
                                    size="sm"
                                    className="h-8 w-8 p-0 text-destructive"
                                    onClick={() => handleDelete(task.id)}
                                  >
                                    <Trash2 className="h-4 w-4" />
                                  </Button>
                                </div>
                              </div>
                              {expandedIds.has(task.id) && (
                                <div className="space-y-2 mt-3 ml-8">
                                  {editingId === task.id ? (
                                    <>
                                      <Textarea
                                        value={task.description}
                                        onChange={(e) => handleUpdate(task.id, 'description', e.target.value)}
                                        placeholder="Task description..."
                                        className="min-h-[80px]"
                                      />
                                      <div className="flex items-center gap-2">
                                        <label className="text-sm text-muted-foreground">
                                          Estimated subtasks:
                                        </label>
                                        <Input
                                          type="number"
                                          value={task.estimatedSubtasks || 2}
                                          onChange={(e) =>
                                            handleUpdate(task.id, 'estimatedSubtasks', parseInt(e.target.value))
                                          }
                                          className="w-20 h-8"
                                          min={1}
                                          max={10}
                                        />
                                      </div>
                                    </>
                                  ) : (
                                    <p className="text-sm text-muted-foreground">
                                      {task.description || 'No description'}
                                    </p>
                                  )}
                                </div>
                              )}
                            </div>
                          </div>
                        </div>
                      )}
                    </Draggable>
                  ))}
                  {provided.placeholder}
                </div>
              )}
            </Droppable>
          </DragDropContext>
        </CardContent>
      </Card>

      {/* Generate Button */}
      <div className="flex justify-end">
        <Button
          onClick={onGenerateDetailedTasks}
          disabled={isGenerating || tasks.length === 0}
          size="lg"
        >
          {isGenerating ? 'Generating...' : 'Generate Detailed Tasks'}
        </Button>
      </div>
    </div>
  );
}
