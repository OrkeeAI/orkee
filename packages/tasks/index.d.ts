export interface Task {
  id: string;
  title: string;
  description?: string;
  status: 'pending' | 'in-progress' | 'review' | 'done';
  priority?: 'low' | 'medium' | 'high' | 'critical';
  assignee?: string;
  dueDate?: string;
  tags?: string[];
  subtasks?: Task[];
  projectId?: string;
  parentId?: string | null;
  createdAt?: string;
  updatedAt?: string;
}

export interface KanbanColumnData {
  id: string;
  title: string;
  status: string;
  tasks: Task[];
}

export interface KanbanBoardProps {
  tasks: Task[];
  onTaskUpdate: (taskId: string, updates: Partial<Task>) => void;
  onTaskCreate?: (task: Partial<Task>) => void;
  onTaskDelete?: (taskId: string) => void;
}

export declare const KanbanBoard: React.FC<KanbanBoardProps>;
export declare const TaskDetailsSheet: React.FC<any>;

export interface UseTasksOptions {
  projectId: string;
  projectPath: string;
  providerType: string;
  enabled?: boolean;
}

export interface UseTasksReturn {
  tasks: Task[];
  isLoading: boolean;
  error: Error | null;
  createTask: (task: Partial<Task>) => Promise<void>;
  updateTask: (taskId: string, updates: Partial<Task>) => Promise<void>;
  deleteTask: (taskId: string) => Promise<void>;
}

export declare function useTasks(options: UseTasksOptions): UseTasksReturn;