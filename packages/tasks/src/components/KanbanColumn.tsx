import React from 'react';
import { useDroppable } from '@dnd-kit/core';
import {
  SortableContext,
  verticalListSortingStrategy,
} from '@dnd-kit/sortable';
import { KanbanColumn as KanbanColumnType, Task } from '../types';
import { TaskCard } from './TaskCard';
import { Plus } from 'lucide-react';

interface KanbanColumnProps {
  column: KanbanColumnType;
  onTaskUpdate?: (taskId: string, updates: Partial<Task>) => void;
  onTaskCreate?: (task: Partial<Task>) => void;
  onTaskDelete?: (taskId: string) => void;
}

export const KanbanColumn: React.FC<KanbanColumnProps> = ({
  column,
  onTaskUpdate,
  onTaskCreate,
  onTaskDelete,
}) => {
  const { setNodeRef, isOver } = useDroppable({
    id: column.id,
  });

  const taskIds = column.tasks.map(task => task.id);

  const handleAddTask = () => {
    if (onTaskCreate) {
      onTaskCreate({
        title: 'New Task',
        status: column.status,
      });
    }
  };

  return (
    <div
      ref={setNodeRef}
      className={`
        flex flex-col h-full bg-gray-50 dark:bg-gray-900 rounded-lg
        ${isOver ? 'ring-2 ring-blue-500' : ''}
      `}
    >
      <div className="p-3 border-b border-gray-200 dark:border-gray-700">
        <div className="flex items-center justify-between">
          <h3 className="font-semibold text-gray-900 dark:text-gray-100">
            {column.title}
          </h3>
          <span className="px-2 py-1 text-xs bg-gray-200 dark:bg-gray-700 rounded-full">
            {column.tasks.length}
          </span>
        </div>
      </div>
      
      <div className="flex-1 overflow-y-auto p-3 space-y-2">
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
            />
          ))}
        </SortableContext>
      </div>
      
      {onTaskCreate && (
        <div className="p-3 border-t border-gray-200 dark:border-gray-700">
          <button
            onClick={handleAddTask}
            className="w-full flex items-center justify-center gap-2 p-2 text-sm text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-100 hover:bg-gray-100 dark:hover:bg-gray-800 rounded transition-colors"
          >
            <Plus className="w-4 h-4" />
            Add Task
          </button>
        </div>
      )}
    </div>
  );
};