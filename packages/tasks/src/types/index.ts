export interface Agent {
  id: string;
  name: string;
  type: 'ai' | 'human' | 'system';
  provider?: string;
  model?: string;
  displayName: string;
  avatarUrl?: string;
  description?: string;
  capabilities?: string[];
  languages?: string[];
  frameworks?: string[];
  maxContextTokens?: number;
  supportsTools?: boolean;
  supportsVision?: boolean;
  supportsWebSearch?: boolean;
  apiEndpoint?: string;
  temperature?: number;
  maxTokens?: number;
  systemPrompt?: string;
  costPer1kInputTokens?: number;
  costPer1kOutputTokens?: number;
  isAvailable: boolean;
  requiresApiKey: boolean;
  metadata?: Record<string, any>;
  createdAt: Date;
  updatedAt: Date;
}

export interface User {
  id: string;
  email: string;
  name: string;
  avatarUrl?: string;
  defaultAgentId?: string;
  theme?: 'light' | 'dark' | 'system';
  openaiApiKey?: string;
  anthropicApiKey?: string;
  googleApiKey?: string;
  xaiApiKey?: string;
  preferences?: Record<string, any>;
  createdAt: Date;
  updatedAt: Date;
  lastLoginAt?: Date;
}

export interface UserAgent {
  id: string;
  userId: string;
  agentId: string;
  agent: Agent;
  isActive: boolean;
  isFavorite: boolean;
  customName?: string;
  customSystemPrompt?: string;
  customTemperature?: number;
  customMaxTokens?: number;
  tasksAssigned: number;
  tasksCompleted: number;
  totalTokensUsed: number;
  totalCostCents: number;
  lastUsedAt?: Date;
  preferences?: Record<string, any>;
  createdAt: Date;
  updatedAt: Date;
}

export interface Task {
  id: string;
  projectId: string;
  title: string;
  description?: string;
  status: TaskStatus;
  priority?: TaskPriority;

  // User and agent attribution
  createdByUserId: string;
  assignedAgentId?: string;
  assignedAgent?: Agent;
  reviewedByAgentId?: string;

  // Hierarchy
  parentId?: string;
  position: number;
  subtasks?: Task[];

  // Dependencies
  dependencies?: string[];
  blockers?: string[];

  // Planning
  dueDate?: Date;
  estimatedHours?: number;
  actualHours?: number;
  complexityScore?: number;

  // Rich content
  details?: string;
  testStrategy?: string;
  acceptanceCriteria?: string;

  // AI-specific fields
  prompt?: string;
  context?: string;
  outputFormat?: string;
  validationRules?: any;

  // Execution tracking
  startedAt?: Date;
  completedAt?: Date;
  executionLog?: ExecutionStep[];
  errorLog?: ErrorEntry[];
  retryCount: number;

  // Categorization
  tagId?: string;
  tags?: string[];
  category?: string;

  // Metadata
  metadata?: Record<string, any>;
  createdAt: Date;
  updatedAt: Date;
}

export interface ExecutionStep {
  timestamp: Date;
  action: string;
  details?: string;
  status: 'success' | 'error' | 'warning';
}

export interface ErrorEntry {
  timestamp: Date;
  message: string;
  stack?: string;
  code?: string;
}

export enum TaskStatus {
  Pending = 'pending',
  InProgress = 'in-progress',
  Review = 'review',
  Done = 'done',
  Cancelled = 'cancelled',
  Deferred = 'deferred',
  Blocked = 'blocked'
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

export interface KanbanColumnData {
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