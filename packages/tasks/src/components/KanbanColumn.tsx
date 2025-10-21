import React, { useState } from 'react';
import { useDroppable } from '@dnd-kit/core';
import {
  SortableContext,
  verticalListSortingStrategy,
} from '@dnd-kit/sortable';
import { KanbanColumnData as KanbanColumnType, Task, TaskStatus } from '../types';
import { TaskCard } from './TaskCard';
import { TaskFormDialog } from './TaskFormDialog';
import {
  Plus,
  Circle,
  Clock,
  Eye,
  CheckCircle,
  XCircle,
  PauseCircle,
  Ban
} from 'lucide-react';

interface KanbanColumnProps {
  column: KanbanColumnType;
  onTaskUpdate?: (taskId: string, updates: Partial<Task>) => void;
  onTaskCreate?: (task: Partial<Task>) => void;
  onTaskDelete?: (taskId: string) => void;
  onTaskClick?: (task: Task) => void;
}

export const KanbanColumn: React.FC<KanbanColumnProps> = ({
  column,
  onTaskUpdate,
  onTaskCreate,
  onTaskDelete,
  onTaskClick,
}) => {
  const [isFormOpen, setIsFormOpen] = useState(false);
  const { setNodeRef, isOver } = useDroppable({
    id: column.id,
  });

  const taskIds = column.tasks.map(task => task.id);

  const getStatusIcon = () => {
    switch (column.status) {
      case TaskStatus.Pending:
        return <Circle className="w-4 h-4" />;
      case TaskStatus.InProgress:
        return <Clock className="w-4 h-4" />;
      case TaskStatus.Review:
        return <Eye className="w-4 h-4" />;
      case TaskStatus.Done:
        return <CheckCircle className="w-4 h-4" />;
      case TaskStatus.Cancelled:
        return <XCircle className="w-4 h-4" />;
      case TaskStatus.Deferred:
        return <PauseCircle className="w-4 h-4" />;
      case TaskStatus.Blocked:
        return <Ban className="w-4 h-4" />;
      default:
        return <Circle className="w-4 h-4" />;
    }
  };

  const getStatusColor = () => {
    switch (column.status) {
      case TaskStatus.Pending:
        return 'text-gray-600';
      case TaskStatus.InProgress:
        return 'text-blue-600';
      case TaskStatus.Review:
        return 'text-purple-600';
      case TaskStatus.Done:
        return 'text-green-600';
      case TaskStatus.Cancelled:
        return 'text-red-600';
      case TaskStatus.Deferred:
        return 'text-orange-600';
      case TaskStatus.Blocked:
        return 'text-yellow-600';
      default:
        return 'text-gray-600';
    }
  };

  const handleAddTask = () => {
    setIsFormOpen(true);
  };

  return (
    <div
      ref={setNodeRef}
      className={`
        flex flex-col bg-muted/30 rounded-lg border border-border
        ${isOver ? 'ring-2 ring-primary bg-primary/10' : ''}
        transition-colors duration-200
        flex-1 h-full min-w-0
      `}
    >
      <div className="p-2 sm:p-3 border-b border-border">
        <div className="flex items-center justify-between">
          <div className={`flex items-center gap-1 sm:gap-2 font-semibold text-sm sm:text-base ${getStatusColor()}`}>
            <div className="w-3 h-3 sm:w-4 sm:h-4">
              {getStatusIcon()}
            </div>
            <span>{column.title}</span>
          </div>
          <span className="px-1.5 sm:px-2 py-0.5 text-xs text-muted-foreground bg-secondary rounded-full font-medium">
            {column.tasks.length}
          </span>
        </div>
      </div>
      
      <div className="flex-1 overflow-y-auto p-2 sm:p-3 space-y-2">
        <SortableContext
          items={taskIds}
          strategy={verticalListSortingStrategy}
        >
          {column.tasks.map((task) => (
            <TaskCard
              key={task.id}
              task={task}
              onUpdate={onTaskUpdate}
              onDelete={onTaskDelete}
              onClick={onTaskClick}
            />
          ))}
        </SortableContext>
      </div>
      
      {onTaskCreate && (
        <div className="p-2 sm:p-3 border-t border-border">
          <button
            onClick={handleAddTask}
            className="w-full flex items-center justify-center gap-1 sm:gap-2 p-1.5 sm:p-2 text-xs sm:text-sm text-muted-foreground hover:text-foreground hover:bg-muted rounded transition-colors"
          >
            <Plus className="w-3 h-3 sm:w-4 sm:h-4" />
            <span className="hidden sm:inline">Add Task</span>
            <span className="sm:hidden">Add</span>
          </button>
        </div>
      )}

      <TaskFormDialog
        isOpen={isFormOpen}
        onOpenChange={setIsFormOpen}
        onTaskCreate={onTaskCreate}
        defaultStatus={column.status}
      />
    </div>
  );
};