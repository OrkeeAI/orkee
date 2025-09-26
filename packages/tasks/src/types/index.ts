export interface Task {
  id: string;
  title: string;
  description?: string;
  status: TaskStatus;
  priority?: TaskPriority;
  tags?: string[];
  assignee?: string;
  dueDate?: Date;
  createdAt: Date;
  updatedAt: Date;
  subtasks?: Task[];
  parentId?: string;
  metadata?: Record<string, any>;
}

export enum TaskStatus {
  Pending = 'pending',
  InProgress = 'in-progress',
  Review = 'review',
  Done = 'done',
  Cancelled = 'cancelled',
  Deferred = 'deferred'
}

export enum TaskPriority {
  Low = 'low',
  Medium = 'medium',
  High = 'high',
  Critical = 'critical'
}

export interface TaskProvider {
  name: string;
  type: TaskProviderType;
  
  initialize(): Promise<void>;
  getTasks(projectPath: string): Promise<Task[]>;
  createTask(projectPath: string, task: Partial<Task>): Promise<Task>;
  updateTask(projectPath: string, taskId: string, updates: Partial<Task>): Promise<Task>;
  deleteTask(projectPath: string, taskId: string): Promise<void>;
  watchTasks?(projectPath: string, callback: (tasks: Task[]) => void): () => void;
}

export enum TaskProviderType {
  Taskmaster = 'taskmaster',
  Manual = 'manual',
  Linear = 'linear',
  Jira = 'jira',
  GitHub = 'github'
}

export interface TaskProviderConfig {
  type: TaskProviderType;
  projectPath: string;
  options?: Record<string, any>;
}

export interface KanbanColumn {
  id: string;
  title: string;
  status: TaskStatus;
  tasks: Task[];
}

export interface TaskEvent {
  type: 'created' | 'updated' | 'deleted' | 'moved';
  task: Task;
  previousStatus?: TaskStatus;
  timestamp: Date;
}