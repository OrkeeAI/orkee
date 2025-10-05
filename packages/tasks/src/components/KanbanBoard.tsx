import React, { useState, useMemo } from 'react';
import {
  DndContext,
  DragEndEvent,
  DragOverlay,
  DragStartEvent,
  closestCorners,
  KeyboardSensor,
  PointerSensor,
  useSensor,
  useSensors,
} from '@dnd-kit/core';
import {
  sortableKeyboardCoordinates,
} from '@dnd-kit/sortable';
import { Task, TaskStatus, KanbanColumnData } from '../types';
import { KanbanColumn as KanbanColumnComponent } from './KanbanColumn';
import { TaskCard } from './TaskCard';
import { TaskDetailsSheet } from './TaskDetailsSheet';
import { RefreshCw, Users, Settings } from 'lucide-react';

interface KanbanBoardProps {
  tasks: Task[];
  onTaskUpdate: (taskId: string, updates: Partial<Task>) => void;
  onTaskCreate?: (task: Partial<Task>) => void;
  onTaskDelete?: (taskId: string) => void;
  onRefresh?: () => void;
}

interface ColumnConfig {
  id: string;
  title: string;
  status: TaskStatus;
  visible: boolean;
  color: string;
}

export const KanbanBoard: React.FC<KanbanBoardProps> = ({
  tasks,
  onTaskUpdate,
  onTaskCreate,
  onTaskDelete,
  onRefresh,
}) => {
  const [activeId, setActiveId] = useState<string | null>(null);
  const [selectedTag, setSelectedTag] = useState<string>('all');
  const [showColumnsDropdown, setShowColumnsDropdown] = useState<boolean>(false);
  const [selectedTask, setSelectedTask] = useState<Task | null>(null);
  const [isSheetOpen, setIsSheetOpen] = useState(false);

  const handleTaskClick = (task: Task) => {
    setSelectedTask(task);
    setIsSheetOpen(true);
  };
  
  const [columnConfigs, setColumnConfigs] = useState<ColumnConfig[]>([
    { id: 'pending', title: 'Pending', status: TaskStatus.Pending, visible: true, color: 'text-gray-600' },
    { id: 'in-progress', title: 'In Progress', status: TaskStatus.InProgress, visible: true, color: 'text-blue-600' },
    { id: 'review', title: 'Review', status: TaskStatus.Review, visible: true, color: 'text-purple-600' },
    { id: 'done', title: 'Done', status: TaskStatus.Done, visible: true, color: 'text-green-600' },
    { id: 'cancelled', title: 'Cancelled', status: TaskStatus.Cancelled, visible: true, color: 'text-red-600' },
    { id: 'deferred', title: 'Deferred', status: TaskStatus.Deferred, visible: true, color: 'text-orange-600' },
    { id: 'blocked', title: 'Blocked', status: TaskStatus.Blocked, visible: false, color: 'text-yellow-600' },
  ]);
  
  // Extract unique tags from all tasks
  const availableTags = useMemo(() => {
    const tagSet = new Set<string>();
    tasks.forEach(task => {
      task.tags?.forEach(tag => tagSet.add(tag));
    });
    return Array.from(tagSet).sort();
  }, [tasks]);

  // Filter tasks based on selected tag
  const filteredTasks = useMemo(() => {
    if (selectedTag === 'all') {
      return tasks;
    }
    return tasks.filter(task => task.tags?.includes(selectedTag));
  }, [tasks, selectedTag]);
  
  const sensors = useSensors(
    useSensor(PointerSensor, {
      activationConstraint: {
        distance: 8,
      },
    }),
    useSensor(KeyboardSensor, {
      coordinateGetter: sortableKeyboardCoordinates,
    })
  );

  const columns = useMemo<KanbanColumnData[]>(() => {
    return columnConfigs
      .filter(col => col.visible)
      .map(col => ({
        ...col,
        tasks: filteredTasks.filter(task => task.status === col.status) || [],
      }));
  }, [filteredTasks, columnConfigs]);

  const activeTask = useMemo(
    () => tasks.find(task => task.id === activeId),
    [activeId, tasks]
  );

  const handleDragStart = (event: DragStartEvent) => {
    setActiveId(event.active.id as string);
  };

  const handleDragEnd = (event: DragEndEvent) => {
    const { active, over } = event;
    
    if (!over) {
      setActiveId(null);
      return;
    }

    const activeTaskId = active.id as string;
    const overId = over.id as string;
    
    // Find the task being dragged
    const activeTask = tasks.find(task => task.id === activeTaskId);
    if (!activeTask) {
      setActiveId(null);
      return;
    }
    
    // Check if we're dropping on a column
    const targetColumn = columns.find(col => col.id === overId);
    if (targetColumn) {
      // Dropped directly on a column
      if (activeTask.status !== targetColumn.status) {
        onTaskUpdate(activeTaskId, { status: targetColumn.status });
      }
    } else {
      // Dropped on another task - find which column it belongs to
      const targetTask = tasks.find(task => task.id === overId);
      if (targetTask && activeTask.status !== targetTask.status) {
        onTaskUpdate(activeTaskId, { status: targetTask.status });
      }
    }
    
    setActiveId(null);
  };

  const toggleColumn = (columnId: string) => {
    setColumnConfigs(prev => 
      prev.map(col => 
        col.id === columnId ? { ...col, visible: !col.visible } : col
      )
    );
  };

  const handleRefresh = () => {
    if (onRefresh) {
      onRefresh();
    }
  };

  return (
    <div className="h-full flex flex-col w-full">
      {/* Header Bar */}
      <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3 px-3 sm:px-4 py-3 border-b border-border bg-background">
        <div className="flex items-center gap-2 sm:gap-4 w-full sm:w-auto">
          {/* Tag Filter Selector */}
          <select
            value={selectedTag}
            onChange={(e) => setSelectedTag(e.target.value)}
            className="px-2 sm:px-3 py-1 border border-input rounded-md text-xs sm:text-sm flex-1 sm:flex-initial bg-background text-foreground"
          >
            <option value="all">All Tags</option>
            {availableTags.length > 0 && (
              <optgroup label="Tags">
                {availableTags.map(tag => (
                  <option key={tag} value={tag}>{tag}</option>
                ))}
              </optgroup>
            )}
          </select>

          {/* Task Count */}
          <span className="text-xs sm:text-sm text-muted-foreground whitespace-nowrap">
            {filteredTasks.length} task{filteredTasks.length !== 1 ? 's' : ''}
            {selectedTag !== 'all' && ` (${selectedTag})`}
          </span>
        </div>

        <div className="flex items-center gap-2 w-full sm:w-auto justify-end">
          {/* Dependencies Button - Hidden on mobile */}
          <button
            className="hidden sm:flex px-2 sm:px-3 py-1 text-xs sm:text-sm border border-input rounded-md hover:bg-muted items-center gap-2"
            onClick={() => console.log('Dependencies clicked')}
          >
            <Users className="w-3 h-3 sm:w-4 sm:h-4" />
            <span className="hidden lg:inline">Dependencies</span>
          </button>

          {/* Columns Dropdown */}
          <div className="relative">
            <button
              onClick={() => setShowColumnsDropdown(!showColumnsDropdown)}
              className="px-2 sm:px-3 py-1 text-xs sm:text-sm border border-input rounded-md hover:bg-muted flex items-center gap-1 sm:gap-2"
            >
              <Settings className="w-3 h-3 sm:w-4 sm:h-4" />
              <span className="hidden sm:inline">Columns</span>
            </button>
            {showColumnsDropdown && (
              <div className="absolute right-0 mt-1 w-48 bg-card border border-border rounded-md shadow-lg z-50">
                <div className="p-2">
                  {columnConfigs.map(col => (
                    <label key={col.id} className="flex items-center gap-2 p-2 hover:bg-muted rounded cursor-pointer">
                      <input
                        type="checkbox"
                        checked={col.visible}
                        onChange={() => toggleColumn(col.id)}
                        className="rounded"
                      />
                      <span className={`text-sm ${col.color}`}>{col.title}</span>
                    </label>
                  ))}
                </div>
              </div>
            )}
          </div>

          {/* Refresh Button */}
          <button
            onClick={handleRefresh}
            className="px-2 sm:px-3 py-1 text-xs sm:text-sm border border-input rounded-md hover:bg-muted flex items-center gap-1 sm:gap-2"
          >
            <RefreshCw className="w-3 h-3 sm:w-4 sm:h-4" />
            <span className="hidden sm:inline">Refresh</span>
          </button>
        </div>
      </div>

      {/* Kanban Columns */}
      <div className="flex-1 overflow-hidden">
        <DndContext
          sensors={sensors}
          collisionDetection={closestCorners}
          onDragStart={handleDragStart}
          onDragEnd={handleDragEnd}
        >
          <div className="flex h-full gap-2 p-2">
            {columns.map((column) => (
              <KanbanColumnComponent
                key={column.id}
                column={column}
                onTaskUpdate={onTaskUpdate}
                onTaskCreate={onTaskCreate}
                onTaskDelete={onTaskDelete}
                onTaskClick={handleTaskClick}
              />
            ))}
          </div>
          <DragOverlay>
            {activeTask && (
              <TaskCard
                task={activeTask}
                isDragging={true}
              />
            )}
          </DragOverlay>
        </DndContext>
      </div>
      
      <TaskDetailsSheet
        task={selectedTask}
        isOpen={isSheetOpen}
        onOpenChange={setIsSheetOpen}
        onUpdate={onTaskUpdate}
        onDelete={onTaskDelete}
      />
    </div>
  );
};