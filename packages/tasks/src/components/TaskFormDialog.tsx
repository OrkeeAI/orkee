// ABOUTME: Task creation and editing form dialog with complexity slider
// ABOUTME: Handles both create and update modes with full validation
import React, { useState, useEffect } from 'react';
import { Task, TaskStatus, TaskPriority } from '../types';
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from './ui/dialog';

// API URL helper - matches the pattern from dashboard
const getApiBaseUrl = async (): Promise<string> => {
  // Check if we're in a browser environment
  if (typeof window === 'undefined') {
    return 'http://localhost:4001';
  }

  // Check if running in Tauri (desktop app)
  if ((window as any).__TAURI__) {
    try {
      const { invoke } = (window as any).__TAURI__.core;
      const port = await invoke('get_api_port');
      return `http://localhost:${port}`;
    } catch (error) {
      console.error('[TaskFormDialog] Failed to get API port from Tauri:', error);
      return 'http://localhost:4001';
    }
  }

  // Web mode - use default or environment variable
  return 'http://localhost:4001';
};

interface Tag {
  id: string;
  name: string;
  color?: string;
  description?: string;
  createdAt: string;
  archivedAt?: string;
}

interface TaskFormDialogProps {
  task?: Task | null;
  isOpen: boolean;
  onOpenChange: (open: boolean) => void;
  onTaskCreate?: (task: Partial<Task>) => void;
  onTaskUpdate?: (taskId: string, updates: Partial<Task>) => void;
  defaultStatus?: TaskStatus;
}

export const TaskFormDialog: React.FC<TaskFormDialogProps> = ({
  task,
  isOpen,
  onOpenChange,
  onTaskCreate,
  onTaskUpdate,
  defaultStatus,
}) => {
  const [title, setTitle] = useState('');
  const [description, setDescription] = useState('');
  const [status, setStatus] = useState<TaskStatus>(defaultStatus || TaskStatus.Pending);
  const [priority, setPriority] = useState<TaskPriority>(TaskPriority.Medium);
  const [tagId, setTagId] = useState('');
  const [complexityScore, setComplexityScore] = useState(5);
  const [assignedAgentId, setAssignedAgentId] = useState('');
  const [dueDate, setDueDate] = useState('');
  const [details, setDetails] = useState('');
  const [testStrategy, setTestStrategy] = useState('');
  const [acceptanceCriteria, setAcceptanceCriteria] = useState('');
  const [tags, setTags] = useState<Tag[]>([]);
  const [loadingTags, setLoadingTags] = useState(false);

  const isEditMode = !!task;

  // Load tags from API
  useEffect(() => {
    const loadTags = async () => {
      setLoadingTags(true);
      try {
        const apiBaseUrl = await getApiBaseUrl();
        const response = await fetch(`${apiBaseUrl}/api/tags`);
        const data = await response.json();
        if (data.success) {
          // Filter out archived tags
          const activeTags = data.data.filter((tag: Tag) => !tag.archivedAt);
          setTags(activeTags);
        }
      } catch (error) {
        console.error('Failed to load tags:', error);
      } finally {
        setLoadingTags(false);
      }
    };

    if (isOpen) {
      loadTags();
    }
  }, [isOpen]);

  // Initialize form with task data when editing
  useEffect(() => {
    if (task) {
      setTitle(task.title || '');
      setDescription(task.description || '');
      setStatus(task.status);
      setPriority(task.priority || TaskPriority.Medium);
      setTagId(task.tagId || '');
      setComplexityScore(task.complexityScore || 5);
      setAssignedAgentId(task.assignedAgentId || '');
      setDueDate(task.dueDate || '');
      setDetails(task.metadata?.details || '');
      setTestStrategy(task.metadata?.testStrategy || '');
      setAcceptanceCriteria(task.metadata?.acceptanceCriteria || '');
    } else {
      // Reset form for create mode
      setTitle('');
      setDescription('');
      setStatus(defaultStatus || TaskStatus.Pending);
      setPriority(TaskPriority.Medium);
      setTagId('');
      setComplexityScore(5);
      setAssignedAgentId('');
      setDueDate('');
      setDetails('');
      setTestStrategy('');
      setAcceptanceCriteria('');
    }
  }, [task, defaultStatus]);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();

    const taskData: Partial<Task> = {
      title,
      description,
      status,
      priority,
      tagId: tagId || undefined,
      complexityScore,
      assignedAgentId: assignedAgentId || undefined,
      dueDate: dueDate || undefined,
      metadata: {
        details: details || undefined,
        testStrategy: testStrategy || undefined,
        acceptanceCriteria: acceptanceCriteria || undefined,
      },
    };

    if (isEditMode && task && onTaskUpdate) {
      onTaskUpdate(task.id, taskData);
    } else if (onTaskCreate) {
      onTaskCreate(taskData);
    }

    onOpenChange(false);
  };

  const getComplexityLabel = (value: number): string => {
    if (value <= 2) return 'Trivial';
    if (value <= 4) return 'Simple';
    if (value <= 6) return 'Moderate';
    if (value <= 8) return 'Complex';
    return 'Very Complex';
  };

  return (
    <Dialog open={isOpen} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>{isEditMode ? 'Edit Task' : 'Create New Task'}</DialogTitle>
        </DialogHeader>

        <form onSubmit={handleSubmit} className="space-y-4">
          {/* Title */}
          <div className="space-y-2">
            <label htmlFor="title" className="text-sm font-medium">
              Title <span className="text-red-500">*</span>
            </label>
            <input
              id="title"
              type="text"
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              required
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-800"
              placeholder="Enter task title"
            />
          </div>

          {/* Description */}
          <div className="space-y-2">
            <label htmlFor="description" className="text-sm font-medium">
              Description
            </label>
            <textarea
              id="description"
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              rows={3}
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-800"
              placeholder="Enter task description"
            />
          </div>

          {/* Status and Priority Row */}
          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-2">
              <label htmlFor="status" className="text-sm font-medium">
                Status
              </label>
              <select
                id="status"
                value={status}
                onChange={(e) => setStatus(e.target.value as TaskStatus)}
                className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-800"
              >
                <option value={TaskStatus.Pending}>Pending</option>
                <option value={TaskStatus.InProgress}>In Progress</option>
                <option value={TaskStatus.Review}>Review</option>
                <option value={TaskStatus.Done}>Done</option>
                <option value={TaskStatus.Cancelled}>Cancelled</option>
                <option value={TaskStatus.Deferred}>Deferred</option>
                <option value={TaskStatus.Blocked}>Blocked</option>
              </select>
            </div>

            <div className="space-y-2">
              <label htmlFor="priority" className="text-sm font-medium">
                Priority
              </label>
              <select
                id="priority"
                value={priority}
                onChange={(e) => setPriority(e.target.value as TaskPriority)}
                className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-800"
              >
                <option value={TaskPriority.Low}>Low</option>
                <option value={TaskPriority.Medium}>Medium</option>
                <option value={TaskPriority.High}>High</option>
                <option value={TaskPriority.Critical}>Critical</option>
              </select>
            </div>
          </div>

          {/* Tag and Agent Row */}
          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-2">
              <label htmlFor="tag" className="text-sm font-medium">
                Tag
              </label>
              <select
                id="tag"
                value={tagId}
                onChange={(e) => setTagId(e.target.value)}
                disabled={loadingTags}
                className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-800 disabled:opacity-50"
              >
                <option value="">No tag</option>
                {tags.map((tag) => (
                  <option key={tag.id} value={tag.id}>
                    {tag.name}
                  </option>
                ))}
              </select>
            </div>

            <div className="space-y-2">
              <label htmlFor="assignedAgent" className="text-sm font-medium">
                Assigned Agent ID
              </label>
              <input
                id="assignedAgent"
                type="text"
                value={assignedAgentId}
                onChange={(e) => setAssignedAgentId(e.target.value)}
                className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-800"
                placeholder="Optional agent ID"
              />
            </div>
          </div>

          {/* Complexity Slider */}
          <div className="space-y-2">
            <label htmlFor="complexity" className="text-sm font-medium">
              Complexity: {complexityScore} ({getComplexityLabel(complexityScore)})
            </label>
            <input
              id="complexity"
              type="range"
              min="1"
              max="10"
              value={complexityScore}
              onChange={(e) => setComplexityScore(parseInt(e.target.value))}
              className="w-full h-2 bg-gray-200 dark:bg-gray-700 rounded-lg appearance-none cursor-pointer accent-blue-500"
            />
            <div className="flex justify-between text-xs text-gray-500">
              <span>1 (Trivial)</span>
              <span>5 (Moderate)</span>
              <span>10 (Very Complex)</span>
            </div>
          </div>

          {/* Due Date */}
          <div className="space-y-2">
            <label htmlFor="dueDate" className="text-sm font-medium">
              Due Date
            </label>
            <input
              id="dueDate"
              type="date"
              value={dueDate}
              onChange={(e) => setDueDate(e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-800"
            />
          </div>

          {/* Advanced Fields - Collapsible */}
          <details className="space-y-2">
            <summary className="text-sm font-medium cursor-pointer">
              Advanced Options
            </summary>

            <div className="mt-3 space-y-4 pl-4">
              {/* Details */}
              <div className="space-y-2">
                <label htmlFor="details" className="text-sm font-medium">
                  Details
                </label>
                <textarea
                  id="details"
                  value={details}
                  onChange={(e) => setDetails(e.target.value)}
                  rows={3}
                  className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-800"
                  placeholder="Additional implementation details"
                />
              </div>

              {/* Test Strategy */}
              <div className="space-y-2">
                <label htmlFor="testStrategy" className="text-sm font-medium">
                  Test Strategy
                </label>
                <textarea
                  id="testStrategy"
                  value={testStrategy}
                  onChange={(e) => setTestStrategy(e.target.value)}
                  rows={3}
                  className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-800"
                  placeholder="How to test this task"
                />
              </div>

              {/* Acceptance Criteria */}
              <div className="space-y-2">
                <label htmlFor="acceptanceCriteria" className="text-sm font-medium">
                  Acceptance Criteria
                </label>
                <textarea
                  id="acceptanceCriteria"
                  value={acceptanceCriteria}
                  onChange={(e) => setAcceptanceCriteria(e.target.value)}
                  rows={3}
                  className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-800"
                  placeholder="What defines this task as complete"
                />
              </div>
            </div>
          </details>

          <DialogFooter>
            <button
              type="button"
              onClick={() => onOpenChange(false)}
              className="px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-md hover:bg-gray-50 dark:hover:bg-gray-700 focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              Cancel
            </button>
            <button
              type="submit"
              className="px-4 py-2 text-sm font-medium text-white bg-blue-600 rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              {isEditMode ? 'Update Task' : 'Create Task'}
            </button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
};
