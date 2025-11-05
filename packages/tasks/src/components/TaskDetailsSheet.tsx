import React, { useState } from 'react';
import { Task, TaskStatus, TaskPriority } from '../types';
import {
  Sheet,
  SheetContent,
  SheetHeader,
  SheetTitle,
  SheetDescription,
} from './ui/sheet';
import { TaskFormDialog } from './TaskFormDialog';
import {
  Calendar,
  User,
  Tag,
  AlertCircle,
  CheckCircle,
  Clock,
  FileText,
  ChevronRight,
  ChevronDown,
  ChevronUp,
  Edit,
  Trash2,
} from 'lucide-react';

interface TaskDetailsSheetProps {
  task: Task | null;
  isOpen: boolean;
  onOpenChange: (open: boolean) => void;
  onUpdate?: (taskId: string, updates: Partial<Task>) => void;
  onDelete?: (taskId: string) => void;
  executionSection?: React.ReactNode; // Optional execution UI section
}

export const TaskDetailsSheet: React.FC<TaskDetailsSheetProps> = ({
  task,
  isOpen,
  onOpenChange,
  onUpdate,
  onDelete,
  executionSection,
}) => {
  const [showDetails, setShowDetails] = useState(false);
  const [showMetadata, setShowMetadata] = useState(false);
  const [isEditDialogOpen, setIsEditDialogOpen] = useState(false);

  if (!task) return null;

  const priorityConfig = {
    [TaskPriority.Low]: { icon: CheckCircle, color: 'text-gray-500', label: 'Low' },
    [TaskPriority.Medium]: { icon: CheckCircle, color: 'text-yellow-500', label: 'Medium' },
    [TaskPriority.High]: { icon: AlertCircle, color: 'text-orange-500', label: 'High' },
    [TaskPriority.Critical]: { icon: AlertCircle, color: 'text-red-500', label: 'Critical' },
  };

  const statusConfig = {
    [TaskStatus.Pending]: { color: 'bg-gray-100 text-gray-700', label: 'Pending' },
    [TaskStatus.InProgress]: { color: 'bg-blue-100 text-blue-700', label: 'In Progress' },
    [TaskStatus.Review]: { color: 'bg-purple-100 text-purple-700', label: 'Review' },
    [TaskStatus.Done]: { color: 'bg-green-100 text-green-700', label: 'Done' },
    [TaskStatus.Cancelled]: { color: 'bg-red-100 text-red-700', label: 'Cancelled' },
    [TaskStatus.Deferred]: { color: 'bg-orange-100 text-orange-700', label: 'Deferred' },
    [TaskStatus.Blocked]: { color: 'bg-yellow-100 text-yellow-700', label: 'Blocked' },
  };

  const priority = task.priority && priorityConfig[task.priority];
  const status = statusConfig[task.status];

  return (
    <Sheet open={isOpen} onOpenChange={onOpenChange}>
      <SheetContent className="w-[95%] sm:w-[70%] md:w-[70%] lg:w-[70%] max-w-none overflow-y-auto">
        <SheetHeader className="space-y-2">
          <SheetDescription className="sr-only">
            {task.description || 'Task details and information'}
          </SheetDescription>
          <div className="flex items-start justify-between">
            <SheetTitle className="text-lg pr-2">{task.title}</SheetTitle>
            <div className="flex gap-1 flex-shrink-0">
              {onUpdate && (
                <button
                  className="p-1.5 hover:bg-gray-100 dark:hover:bg-gray-800 rounded-lg transition-colors"
                  onClick={() => {
                    setIsEditDialogOpen(true);
                  }}
                >
                  <Edit className="w-4 h-4 text-gray-600 dark:text-gray-400" />
                </button>
              )}
              {onDelete && (
                <button
                  className="p-1.5 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-lg transition-colors"
                  onClick={() => {
                    if (confirm('Are you sure you want to delete this task?')) {
                      onDelete(task.id);
                      onOpenChange(false);
                    }
                  }}
                >
                  <Trash2 className="w-4 h-4 text-red-600 dark:text-red-400" />
                </button>
              )}
            </div>
          </div>

          <div className="flex flex-wrap gap-1.5">
            <span className={`px-2 py-0.5 text-xs font-medium rounded-full ${status.color}`}>
              {status.label}
            </span>
            {priority && (
              <span className={`flex items-center gap-1 px-2 py-0.5 text-xs font-medium rounded-full bg-gray-100 dark:bg-gray-800 ${priority.color}`}>
                {React.createElement(priority.icon, { className: 'w-3 h-3' })}
                {priority.label}
              </span>
            )}
          </div>
        </SheetHeader>

        <div className="mt-4 space-y-3">
          {task.description && (
            <div className="space-y-1">
              <div className="flex items-center gap-2 text-sm font-medium text-gray-700 dark:text-gray-300">
                <FileText className="w-3.5 h-3.5" />
                Description
              </div>
              <p className="text-sm text-gray-600 dark:text-gray-400 pl-5 line-clamp-2">
                {task.description}
              </p>
            </div>
          )}

          <div className="grid grid-cols-2 gap-3">
            {task.assignee && (
              <div className="flex items-center gap-2">
                <User className="w-3.5 h-3.5 text-gray-500" />
                <div>
                  <p className="text-xs text-gray-500 dark:text-gray-400">Assignee</p>
                  <p className="text-sm font-medium">{task.assignee}</p>
                </div>
              </div>
            )}

            {task.dueDate && (
              <div className="flex items-center gap-2">
                <Calendar className="w-3.5 h-3.5 text-gray-500" />
                <div>
                  <p className="text-xs text-gray-500 dark:text-gray-400">Due Date</p>
                  <p className="text-sm font-medium">
                    {new Date(task.dueDate).toLocaleDateString('en-US', {
                      month: 'short',
                      day: 'numeric',
                      year: 'numeric'
                    })}
                  </p>
                </div>
              </div>
            )}

            {task.createdAt && (
              <div className="flex items-center gap-2">
                <Clock className="w-3.5 h-3.5 text-gray-500" />
                <div>
                  <p className="text-xs text-gray-500 dark:text-gray-400">Created</p>
                  <p className="text-sm font-medium">
                    {new Date(task.createdAt).toLocaleDateString('en-US', {
                      month: 'short',
                      day: 'numeric',
                      year: 'numeric'
                    })}
                  </p>
                </div>
              </div>
            )}
          </div>

          {task.tags && task.tags.length > 0 && (
            <div className="space-y-1">
              <div className="flex items-center gap-2 text-sm font-medium text-gray-700 dark:text-gray-300">
                <Tag className="w-3.5 h-3.5" />
                Tags
              </div>
              <div className="flex flex-wrap gap-1 pl-5">
                {task.tags.map((tag) => (
                  <span
                    key={tag}
                    className="px-2 py-0.5 text-xs bg-gray-100 dark:bg-gray-800 text-gray-700 dark:text-gray-300 rounded-full"
                  >
                    {tag}
                  </span>
                ))}
              </div>
            </div>
          )}

          {task.subtasks && task.subtasks.length > 0 && (
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2 text-sm font-medium text-gray-700 dark:text-gray-300">
                  <CheckCircle className="w-4 h-4" />
                  Subtasks
                </div>
                <span className="text-xs text-gray-500">
                  {task.subtasks.filter(st => st.completed).length}/{task.subtasks.length} completed
                </span>
              </div>
              <div className="space-y-1 pl-6">
                {task.subtasks.map((subtask) => (
                  <div
                    key={subtask.id}
                    className="flex items-start gap-2 p-2 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-800/50 transition-colors"
                  >
                    <input
                      type="checkbox"
                      checked={subtask.completed}
                      onChange={() => {
                        if (onUpdate) {
                          const updatedSubtasks = task.subtasks?.map(st =>
                            st.id === subtask.id ? { ...st, completed: !st.completed } : st
                          );
                          onUpdate(task.id, { subtasks: updatedSubtasks });
                        }
                      }}
                      className="mt-0.5 rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                    />
                    <div className="flex-1">
                      <p className={`text-sm ${subtask.completed ? 'line-through text-gray-400' : 'text-gray-700 dark:text-gray-300'}`}>
                        {subtask.title}
                      </p>
                      {subtask.description && (
                        <p className="text-xs text-gray-500 dark:text-gray-400 mt-0.5">
                          {subtask.description}
                        </p>
                      )}
                    </div>
                    <ChevronRight className="w-4 h-4 text-gray-400 flex-shrink-0" />
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Show More/Less Section */}
          {(task.metadata?.details || task.metadata?.testStrategy) && (
            <div className="pt-3 border-t border-gray-200 dark:border-gray-700">
              <button
                onClick={() => setShowDetails(!showDetails)}
                className="flex items-center gap-1 text-sm font-medium text-gray-700 dark:text-gray-300 hover:text-gray-900 dark:hover:text-gray-100 transition-colors"
              >
                {showDetails ? <ChevronUp className="w-4 h-4" /> : <ChevronDown className="w-4 h-4" />}
                {showDetails ? 'Show Less' : 'Show More Details'}
              </button>
              
              {showDetails && (
                <div className="mt-3 space-y-3">
                  {task.metadata?.details && (
                    <div className="space-y-1">
                      <p className="text-xs font-medium text-gray-600 dark:text-gray-400">Details</p>
                      <p className="text-sm text-gray-700 dark:text-gray-300 whitespace-pre-wrap">
                        {task.metadata.details}
                      </p>
                    </div>
                  )}
                  
                  {task.metadata?.testStrategy && (
                    <div className="space-y-1">
                      <p className="text-xs font-medium text-gray-600 dark:text-gray-400">Test Strategy</p>
                      <p className="text-sm text-gray-700 dark:text-gray-300 whitespace-pre-wrap">
                        {task.metadata.testStrategy}
                      </p>
                    </div>
                  )}
                </div>
              )}
            </div>
          )}
          
          {/* Metadata Section */}
          {task.metadata && Object.keys(task.metadata).length > 0 && (
            <div className="pt-3 border-t border-gray-200 dark:border-gray-700">
              <button
                onClick={() => setShowMetadata(!showMetadata)}
                className="flex items-center gap-1 text-sm font-medium text-gray-700 dark:text-gray-300 hover:text-gray-900 dark:hover:text-gray-100 transition-colors"
              >
                {showMetadata ? <ChevronUp className="w-4 h-4" /> : <ChevronDown className="w-4 h-4" />}
                Metadata ({Object.keys(task.metadata).length} fields)
              </button>

              {showMetadata && (
                <div className="mt-2 space-y-1 text-xs">
                  {Object.entries(task.metadata)
                    .filter(([key]) => key !== 'details' && key !== 'testStrategy')
                    .map(([key, value]) => (
                      <div key={key} className="flex justify-between">
                        <span className="text-gray-500 dark:text-gray-400 capitalize">
                          {key.replace(/_/g, ' ')}:
                        </span>
                        <span className="text-gray-700 dark:text-gray-300 font-medium text-right max-w-[60%]">
                          {typeof value === 'object' ? JSON.stringify(value) : String(value)}
                        </span>
                      </div>
                    ))}
                </div>
              )}
            </div>
          )}

          {/* Optional Execution Section (provided by dashboard) */}
          {executionSection && (
            <div className="pt-3 border-t border-gray-200 dark:border-gray-700">
              {executionSection}
            </div>
          )}
        </div>
      </SheetContent>

      <TaskFormDialog
        task={task}
        isOpen={isEditDialogOpen}
        onOpenChange={setIsEditDialogOpen}
        onTaskUpdate={onUpdate}
      />
    </Sheet>
  );
};