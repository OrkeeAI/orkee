import React from 'react';
import { useSortable } from '@dnd-kit/sortable';
import { CSS } from '@dnd-kit/utilities';
import { Task, TaskPriority } from '../types';
import { 
  Calendar, 
  User, 
  Trash2,
  AlertCircle,
  CheckCircle
} from 'lucide-react';

interface TaskCardProps {
  task: Task;
  isDragging?: boolean;
  onUpdate?: (taskId: string, updates: Partial<Task>) => void;
  onDelete?: (taskId: string) => void;
}

export const TaskCard: React.FC<TaskCardProps> = ({
  task,
  isDragging = false,
  onDelete,
}) => {
  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging: isSortableDragging,
  } = useSortable({
    id: task.id,
  });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
  };

  const priorityColors = {
    [TaskPriority.Low]: 'text-gray-500',
    [TaskPriority.Medium]: 'text-yellow-500',
    [TaskPriority.High]: 'text-orange-500',
    [TaskPriority.Critical]: 'text-red-500',
  };

  const PriorityIcon = task.priority === TaskPriority.Critical ? AlertCircle : CheckCircle;

  return (
    <div
      ref={setNodeRef}
      style={style}
      className={`
        bg-white dark:bg-gray-800 p-3 rounded-lg shadow-sm
        hover:shadow-md transition-shadow cursor-move
        ${isSortableDragging || isDragging ? 'opacity-50' : ''}
        ${isSortableDragging ? 'z-50' : ''}
      `}
      {...attributes}
      {...listeners}
    >
      <div className="flex items-start justify-between mb-2">
        <h4 className="text-sm font-medium text-gray-900 dark:text-gray-100 flex-1">
          {task.title}
        </h4>
        <div className="flex items-center gap-1">
          {task.priority && React.createElement(PriorityIcon, {
            className: `w-4 h-4 ${priorityColors[task.priority]}`
          })}
          <button
            onClick={(e) => {
              e.stopPropagation();
              onDelete?.(task.id);
            }}
            className="p-1 hover:bg-gray-100 dark:hover:bg-gray-700 rounded"
          >
            {React.createElement(Trash2, { className: "w-3 h-3 text-gray-400" })}
          </button>
        </div>
      </div>
      
      {task.description && (
        <p className="text-xs text-gray-600 dark:text-gray-400 mb-2 line-clamp-2">
          {task.description}
        </p>
      )}
      
      <div className="flex items-center gap-3 text-xs text-gray-500 dark:text-gray-400">
        {task.assignee && (
          <div className="flex items-center gap-1">
            {React.createElement(User, { className: "w-3 h-3" })}
            <span>{task.assignee}</span>
          </div>
        )}
        
        {task.dueDate && (
          <div className="flex items-center gap-1">
            {React.createElement(Calendar, { className: "w-3 h-3" })}
            <span>
              {new Date(task.dueDate).toLocaleDateString()}
            </span>
          </div>
        )}
      </div>
      
      {task.tags && task.tags.length > 0 && (
        <div className="flex flex-wrap gap-1 mt-2">
          {task.tags.map((tag) => (
            <span
              key={tag}
              className="px-2 py-0.5 text-xs bg-gray-100 dark:bg-gray-700 rounded-full"
            >
              {tag}
            </span>
          ))}
        </div>
      )}
    </div>
  );
};