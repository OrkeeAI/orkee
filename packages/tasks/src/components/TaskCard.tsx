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
  onClick?: (task: Task) => void;
}

export const TaskCard: React.FC<TaskCardProps> = ({
  task,
  isDragging = false,
  onDelete,
  onClick,
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

  const handleClick = (e: React.MouseEvent) => {
    if (onClick && !isSortableDragging && !isDragging) {
      e.stopPropagation();
      onClick(task);
    }
  };

  return (
    <div
      ref={setNodeRef}
      style={style}
      className={`
        bg-white dark:bg-gray-800 
        p-2 sm:p-3 
        rounded-lg shadow-sm
        hover:shadow-md transition-shadow cursor-move
        ${isSortableDragging || isDragging ? 'opacity-50' : ''}
        ${isSortableDragging ? 'z-50' : ''}
      `}
      onClick={handleClick}
      {...attributes}
      {...listeners}
    >
      <div className="flex items-start justify-between mb-1.5 sm:mb-2 gap-2">
        <h4 className="text-xs sm:text-sm font-medium text-gray-900 dark:text-gray-100 flex-1 line-clamp-2">
          {task.title}
        </h4>
        <div className="flex items-center gap-0.5 sm:gap-1 flex-shrink-0">
          {task.priority && React.createElement(PriorityIcon, {
            className: `w-3 h-3 sm:w-4 sm:h-4 ${priorityColors[task.priority]}`
          })}
          {onDelete && (
            <button
              onClick={(e) => {
                e.stopPropagation();
                onDelete(task.id);
              }}
              className="p-0.5 sm:p-1 hover:bg-gray-100 dark:hover:bg-gray-700 rounded touch-manipulation"
            >
              {React.createElement(Trash2, { className: "w-3 h-3 text-gray-400" })}
            </button>
          )}
        </div>
      </div>
      
      {task.description && (
        <p className="text-[10px] sm:text-xs text-gray-600 dark:text-gray-400 mb-1.5 sm:mb-2 line-clamp-2">
          {task.description}
        </p>
      )}
      
      <div className="flex flex-wrap items-center gap-2 sm:gap-3 text-[10px] sm:text-xs text-gray-500 dark:text-gray-400">
        {task.assignee && (
          <div className="flex items-center gap-0.5 sm:gap-1">
            {React.createElement(User, { className: "w-2.5 h-2.5 sm:w-3 sm:h-3" })}
            <span className="truncate max-w-[80px] sm:max-w-none">{task.assignee}</span>
          </div>
        )}
        
        {task.dueDate && (
          <div className="flex items-center gap-0.5 sm:gap-1">
            {React.createElement(Calendar, { className: "w-2.5 h-2.5 sm:w-3 sm:h-3" })}
            <span>
              {new Date(task.dueDate).toLocaleDateString('en-US', { 
                month: 'short', 
                day: 'numeric' 
              })}
            </span>
          </div>
        )}
      </div>
      
      {task.tags && task.tags.length > 0 && (
        <div className="flex flex-wrap gap-0.5 sm:gap-1 mt-1.5 sm:mt-2">
          {task.tags.slice(0, 3).map((tag) => (
            <span
              key={tag}
              className="px-1.5 sm:px-2 py-0.5 text-[10px] sm:text-xs bg-gray-100 dark:bg-gray-700 rounded-full"
            >
              {tag}
            </span>
          ))}
          {task.tags.length > 3 && (
            <span className="px-1.5 sm:px-2 py-0.5 text-[10px] sm:text-xs text-gray-500 dark:text-gray-400">
              +{task.tags.length - 3}
            </span>
          )}
        </div>
      )}
    </div>
  );
};