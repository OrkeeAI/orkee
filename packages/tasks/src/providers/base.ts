import { EventEmitter } from 'eventemitter3';
import { Task, TaskProvider, TaskProviderType, TaskEvent } from '../types';

export abstract class BaseTaskProvider extends EventEmitter implements TaskProvider {
  public abstract readonly name: string;
  public abstract readonly type: TaskProviderType;
  
  protected initialized = false;

  async initialize(): Promise<void> {
    if (this.initialized) return;
    await this.doInitialize();
    this.initialized = true;
  }

  protected abstract doInitialize(): Promise<void>;
  
  abstract getTasks(projectPath: string): Promise<Task[]>;
  abstract createTask(projectPath: string, task: Partial<Task>): Promise<Task>;
  abstract updateTask(projectPath: string, taskId: string, updates: Partial<Task>): Promise<Task>;
  abstract deleteTask(projectPath: string, taskId: string): Promise<void>;

  watchTasks?(projectPath: string, callback: (tasks: Task[]) => void): () => void;

  protected emitTaskEvent(event: TaskEvent) {
    this.emit('taskEvent', event);
  }

  protected generateId(): string {
    return `${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
  }

  protected validateTask(task: Partial<Task>): void {
    if (!task.title?.trim()) {
      throw new Error('Task title is required');
    }
  }
}