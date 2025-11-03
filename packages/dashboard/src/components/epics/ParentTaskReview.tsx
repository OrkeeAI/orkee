// ABOUTME: Parent Task Review UI for two-phase task generation
// ABOUTME: Allows users to review and edit parent tasks before expanding to subtasks

import { useState } from 'react';
import {
  DndContext,
  closestCenter,
  KeyboardSensor,
  PointerSensor,
  useSensor,
  useSensors,
  DragEndEvent,
} from '@dnd-kit/core';
import {
  arrayMove,
  SortableContext,
  sortableKeyboardCoordinates,
  useSortable,
  verticalListSortingStrategy,
} from '@dnd-kit/sortable';
import { CSS } from '@dnd-kit/utilities';
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

interface SortableTaskItemProps {
  task: ParentTask;
  isEditing: boolean;
  isExpanded: boolean;
  onEdit: (id: string) => void;
  onDelete: (id: string) => void;
  onUpdate: (id: string, field: keyof ParentTask, value: string | number) => void;
  onToggleExpand: (id: string) => void;
}

function SortableTaskItem({
  task,
  isEditing,
  isExpanded,
  onEdit,
  onDelete,
  onUpdate,
  onToggleExpand,
}: SortableTaskItemProps) {
  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
  } = useSortable({ id: task.id });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
  };

  return (
    <div
      ref={setNodeRef}
      style={style}
      className="border rounded-lg p-4 bg-card"
    >
      <div className="flex items-start gap-3">
        <div {...attributes} {...listeners} className="mt-1 cursor-grab active:cursor-grabbing">
          <GripVertical className="h-5 w-5 text-muted-foreground" />
        </div>
        <div className="flex-1">
          <div className="flex items-start justify-between mb-2">
            <div className="flex items-center gap-2">
              <Button
                variant="ghost"
                size="sm"
                className="h-6 w-6 p-0"
                onClick={() => onToggleExpand(task.id)}
              >
                {isExpanded ? (
                  <ChevronDown className="h-4 w-4" />
                ) : (
                  <ChevronRight className="h-4 w-4" />
                )}
              </Button>
              <Badge variant="outline">{task.order}</Badge>
              {isEditing ? (
                <Input
                  value={task.title}
                  onChange={(e) => onUpdate(task.id, 'title', e.target.value)}
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
                onClick={() => onEdit(task.id)}
              >
                <Edit2 className="h-4 w-4" />
              </Button>
              <Button
                variant="ghost"
                size="sm"
                className="h-8 w-8 p-0 text-destructive"
                onClick={() => onDelete(task.id)}
              >
                <Trash2 className="h-4 w-4" />
              </Button>
            </div>
          </div>
          {isExpanded && (
            <div className="space-y-2 mt-3 ml-8">
              {isEditing ? (
                <>
                  <Textarea
                    value={task.description}
                    onChange={(e) => onUpdate(task.id, 'description', e.target.value)}
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
                        onUpdate(task.id, 'estimatedSubtasks', parseInt(e.target.value))
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
  );
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

  const sensors = useSensors(
    useSensor(PointerSensor),
    useSensor(KeyboardSensor, {
      coordinateGetter: sortableKeyboardCoordinates,
    })
  );

  const handleDragEnd = (event: DragEndEvent) => {
    const { active, over } = event;

    if (!over || active.id === over.id) return;

    const oldIndex = tasks.findIndex(task => task.id === active.id);
    const newIndex = tasks.findIndex(task => task.id === over.id);

    const reorderedTasks = arrayMove(tasks, oldIndex, newIndex);

    // Update order numbers
    const updatedItems = reorderedTasks.map((item, index) => ({
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
          <DndContext
            sensors={sensors}
            collisionDetection={closestCenter}
            onDragEnd={handleDragEnd}
          >
            <SortableContext
              items={tasks.map(task => task.id)}
              strategy={verticalListSortingStrategy}
            >
              <div className="space-y-3">
                {tasks.map((task) => (
                  <SortableTaskItem
                    key={task.id}
                    task={task}
                    isEditing={editingId === task.id}
                    isExpanded={expandedIds.has(task.id)}
                    onEdit={handleEdit}
                    onDelete={handleDelete}
                    onUpdate={handleUpdate}
                    onToggleExpand={toggleExpand}
                  />
                ))}
              </div>
            </SortableContext>
          </DndContext>
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
