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

interface KanbanBoardProps {
  tasks: Task[];
  onTaskUpdate: (taskId: string, updates: Partial<Task>) => void;
  onTaskCreate?: (task: Partial<Task>) => void;
  onTaskDelete?: (taskId: string) => void;
}

export const KanbanBoard: React.FC<KanbanBoardProps> = ({
  tasks,
  onTaskUpdate,
  onTaskCreate,
  onTaskDelete,
}) => {
  const [activeId, setActiveId] = useState<string | null>(null);
  
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
    const columnDefinitions: Array<{ id: string; title: string; status: TaskStatus }> = [
      { id: 'pending', title: 'To Do', status: TaskStatus.Pending },
      { id: 'in-progress', title: 'In Progress', status: TaskStatus.InProgress },
      { id: 'review', title: 'Review', status: TaskStatus.Review },
      { id: 'done', title: 'Done', status: TaskStatus.Done },
    ];

    return columnDefinitions.map(col => ({
      ...col,
      tasks: tasks.filter(task => task.status === col.status),
    }));
  }, [tasks]);

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
    
    // Check if we're dropping on a column
    const overColumn = columns.find(col => col.id === overId);
    if (overColumn) {
      onTaskUpdate(activeTaskId, { status: overColumn.status });
    } else {
      // We're dropping on another task
      const overTask = tasks.find(task => task.id === overId);
      if (overTask) {
        onTaskUpdate(activeTaskId, { status: overTask.status });
      }
    }
    
    setActiveId(null);
  };

  return (
    <div className="h-full overflow-hidden">
      {React.createElement(DndContext, {
        sensors,
        collisionDetection: closestCorners,
        onDragStart: handleDragStart,
        onDragEnd: handleDragEnd,
        children: React.createElement(React.Fragment, {}, [React.createElement('div', {
          key: 'columns',
          className: 'flex gap-4 h-full overflow-x-auto pb-4'
        }, columns.map((column) => React.createElement('div', {
          key: column.id,
          className: 'flex-shrink-0 w-80'
        }, React.createElement(KanbanColumnComponent, {
          column,
          onTaskUpdate,
          onTaskCreate,
          onTaskDelete
        })))), React.createElement(DragOverlay, {
          key: 'overlay',
          children: activeTask && React.createElement(TaskCard, {
            task: activeTask,
            isDragging: true
          })
        })])
      })}
    </div>
  );
};