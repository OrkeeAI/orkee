import { BaseTaskProvider } from './base';
import { Task, TaskStatus, TaskProviderType, TaskPriority } from '../types';

interface TaskmasterTaskData {
  id: number | string;
  title: string;
  description?: string;
  details?: string;
  testStrategy?: string;
  status: string;
  priority?: string;
  dependencies?: number[];
  subtasks?: any[];
}

interface TaskmasterData {
  tasks: TaskmasterTaskData[];
  metadata?: {
    version?: string;
    lastSync?: string;
  };
}

export class TaskmasterProvider extends BaseTaskProvider {
  public readonly name = 'Taskmaster';
  public readonly type = TaskProviderType.Taskmaster;
  
  private tasksCache = new Map<string, Task[]>();
  private fileWatchers = new Map<string, NodeJS.Timeout>();
  private apiBaseUrl: string;

  constructor(apiBaseUrl: string = 'http://localhost:4001') {
    super();
    this.apiBaseUrl = apiBaseUrl;
  }

  protected async doInitialize(): Promise<void> {
    // Initialization logic if needed
  }

  async getTasks(projectPath: string): Promise<Task[]> {
    try {
      // Try to read tasks via the API endpoint (for browser compatibility)
      const response = await fetch(`${this.apiBaseUrl}/api/taskmaster/tasks`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ projectPath })
      });
      
      if (response.ok) {
        const result = await response.json();
        if (result.success && result.data) {
          // Handle the nested structure from taskmaster
          // Tasks can be organized under different contexts (e.g., "master")
          let allTasks: Task[] = [];
          
          // Check if tasks are under a context key like "master"
          if (result.data.master?.tasks) {
            const tasks = this.convertTaskmasterTasks(result.data.master.tasks, 'master');
            allTasks = allTasks.concat(tasks);
          } else if (result.data.tasks) {
            // Direct tasks array without context
            const tasks = this.convertTaskmasterTasks(result.data.tasks);
            allTasks = allTasks.concat(tasks);
          } else {
            // Check for other context keys
            Object.keys(result.data).forEach(key => {
              if (result.data[key]?.tasks && Array.isArray(result.data[key].tasks)) {
                const tasks = this.convertTaskmasterTasks(result.data[key].tasks, key);
                allTasks = allTasks.concat(tasks);
              }
            });
          }
          
          this.tasksCache.set(projectPath, allTasks);
          return allTasks;
        }
      }
      
      // Fallback: return empty array if API fails
      return [];
    } catch (error) {
      console.warn('Failed to fetch Taskmaster tasks via API:', error);
      return [];
    }
  }

  async createTask(projectPath: string, taskData: Partial<Task>): Promise<Task> {
    this.validateTask(taskData);
    
    const tasks = await this.getTasks(projectPath);
    
    const newTask: Task = {
      id: this.generateId(),
      title: taskData.title!,
      description: taskData.description,
      status: taskData.status || TaskStatus.Pending,
      priority: taskData.priority,
      tags: taskData.tags,
      assignee: taskData.assignee,
      dueDate: taskData.dueDate,
      createdAt: new Date(),
      updatedAt: new Date(),
      parentId: taskData.parentId,
      metadata: taskData.metadata
    };
    
    tasks.push(newTask);
    await this.saveTasks(projectPath, tasks);
    
    this.emitTaskEvent({
      type: 'created',
      task: newTask,
      timestamp: new Date()
    });
    
    return newTask;
  }

  async updateTask(projectPath: string, taskId: string, updates: Partial<Task>): Promise<Task> {
    const tasks = await this.getTasks(projectPath);
    const taskIndex = tasks.findIndex(t => t.id === taskId);
    
    if (taskIndex === -1) {
      throw new Error(`Task with id ${taskId} not found`);
    }
    
    const previousTask = tasks[taskIndex];
    const updatedTask = {
      ...previousTask,
      ...updates,
      id: taskId,
      updatedAt: new Date()
    };
    
    tasks[taskIndex] = updatedTask;
    await this.saveTasks(projectPath, tasks);
    
    this.emitTaskEvent({
      type: 'updated',
      task: updatedTask,
      previousStatus: previousTask.status !== updatedTask.status ? previousTask.status : undefined,
      timestamp: new Date()
    });
    
    return updatedTask;
  }

  async deleteTask(projectPath: string, taskId: string): Promise<void> {
    const tasks = await this.getTasks(projectPath);
    const taskIndex = tasks.findIndex(t => t.id === taskId);
    
    if (taskIndex === -1) {
      throw new Error(`Task with id ${taskId} not found`);
    }
    
    const deletedTask = tasks[taskIndex];
    tasks.splice(taskIndex, 1);
    
    await this.saveTasks(projectPath, tasks);
    
    this.emitTaskEvent({
      type: 'deleted',
      task: deletedTask,
      timestamp: new Date()
    });
  }

  watchTasks(projectPath: string, callback: (tasks: Task[]) => void): () => void {
    
    // Poll for changes every 2 seconds
    const interval = setInterval(async () => {
      try {
        const tasks = await this.getTasks(projectPath);
        callback(tasks);
      } catch (error) {
        console.error('Error watching tasks:', error);
      }
    }, 2000);
    
    this.fileWatchers.set(projectPath, interval);
    
    // Return cleanup function
    return () => {
      const watcher = this.fileWatchers.get(projectPath);
      if (watcher) {
        clearInterval(watcher);
        this.fileWatchers.delete(projectPath);
      }
    };
  }

  private convertTaskmasterTasks(taskmasterTasks: any[], context?: string): Task[] {
    return taskmasterTasks.map(task => {
      // Start with task.tags if it exists, otherwise empty array
      const tags = task.tags ? [...task.tags] : [];
      
      // Add the context as a tag if provided
      if (context && !tags.includes(context)) {
        tags.push(context);
      }
      
      return {
        id: String(task.id),
        title: task.title,
        description: task.description || task.details,
        status: this.mapTaskStatus(task.status),
        priority: this.mapTaskPriority(task.priority),
        tags: tags,
        assignee: undefined,
        dueDate: undefined,
        createdAt: new Date(),
        updatedAt: new Date(),
        parentId: undefined,
        metadata: {
          details: task.details,
          testStrategy: task.testStrategy,
          dependencies: task.dependencies,
          subtasks: task.subtasks
        }
      };
    });
  }

  private mapTaskStatus(status: string): TaskStatus {
    const statusMap: Record<string, TaskStatus> = {
      'pending': TaskStatus.Pending,
      'in-progress': TaskStatus.InProgress,
      'in_progress': TaskStatus.InProgress,
      'review': TaskStatus.Review,
      'done': TaskStatus.Done,
      'completed': TaskStatus.Done,
      'cancelled': TaskStatus.Cancelled,
      'deferred': TaskStatus.Deferred,
      'blocked': TaskStatus.Blocked
    };
    
    return statusMap[status.toLowerCase()] || TaskStatus.Pending;
  }

  private mapTaskPriority(priority?: string): TaskPriority | undefined {
    if (!priority) return undefined;
    
    const priorityMap: Record<string, TaskPriority> = {
      'low': TaskPriority.Low,
      'medium': TaskPriority.Medium,
      'high': TaskPriority.High,
      'critical': TaskPriority.Critical
    };
    
    return priorityMap[priority.toLowerCase()];
  }

  private async saveTasks(projectPath: string, tasks: Task[]): Promise<void> {
    // Wrap tasks in the master structure to match the expected format
    const taskmasterData = {
      master: {
        tasks: tasks.map(task => ({
          id: parseInt(task.id) || task.id, // Try to keep numeric IDs if possible
          title: task.title,
          description: task.description,
          details: task.metadata?.details || task.description,
          testStrategy: task.metadata?.testStrategy,
          status: this.reverseMapTaskStatus(task.status),
          priority: task.priority ? this.reverseMapTaskPriority(task.priority) : undefined,
          dependencies: task.metadata?.dependencies || [],
          subtasks: task.metadata?.subtasks || []
        })),
        metadata: {
          created: new Date().toISOString(),
          updated: new Date().toISOString(),
          description: "Tasks for master context"
        }
      }
    };
    
    try {
      // Save tasks via API endpoint 
      const response = await fetch(`${this.apiBaseUrl}/api/taskmaster/tasks/save`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ projectPath, data: taskmasterData })
      });
      
      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }
    } catch (error) {
      console.warn('Failed to save Taskmaster tasks via API:', error);
      throw error;
    }
  }

  private reverseMapTaskStatus(status: TaskStatus): string {
    const statusMap: Record<TaskStatus, string> = {
      [TaskStatus.Pending]: 'pending',
      [TaskStatus.InProgress]: 'in-progress',
      [TaskStatus.Review]: 'review',
      [TaskStatus.Done]: 'done',
      [TaskStatus.Cancelled]: 'cancelled',
      [TaskStatus.Deferred]: 'deferred',
      [TaskStatus.Blocked]: 'blocked'
    };
    
    return statusMap[status];
  }

  private reverseMapTaskPriority(priority: TaskPriority): string {
    const priorityMap: Record<TaskPriority, string> = {
      [TaskPriority.Low]: 'low',
      [TaskPriority.Medium]: 'medium',
      [TaskPriority.High]: 'high',
      [TaskPriority.Critical]: 'critical'
    };
    
    return priorityMap[priority];
  }
}