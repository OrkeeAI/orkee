// ABOUTME: Manual task provider implementation using REST API
// ABOUTME: Provides task CRUD operations backed by Orkee's SQLite database

import { BaseTaskProvider } from './base';
import { Task, TaskProviderType } from '../types';

export class ManualTaskProvider extends BaseTaskProvider {
  public readonly name = 'Manual Tasks';
  public readonly type = TaskProviderType.Manual;
  private apiBaseUrl: string;
  private projectId?: string;

  constructor(options?: { apiBaseUrl?: string }) {
    super();
    this.apiBaseUrl = options?.apiBaseUrl || 'http://localhost:4001';
  }

  protected async doInitialize(): Promise<void> {
    // Verify API connection
    const response = await fetch(`${this.apiBaseUrl}/api/health`);
    if (!response.ok) {
      throw new Error('Failed to connect to Orkee API');
    }
  }

  async getTasks(projectPath: string): Promise<Task[]> {
    const projectId = await this.getProjectIdByPath(projectPath);
    const response = await fetch(`${this.apiBaseUrl}/api/projects/${projectId}/tasks`);
    const data = await response.json();

    if (!data.success) {
      throw new Error(data.error || 'Failed to fetch tasks');
    }

    return this.transformTasks(data.data || []);
  }

  async createTask(projectPath: string, task: Partial<Task>): Promise<Task> {
    this.validateTask(task);

    const projectId = await this.getProjectIdByPath(projectPath);
    const response = await fetch(`${this.apiBaseUrl}/api/projects/${projectId}/tasks`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(this.serializeTask(task)),
    });

    const data = await response.json();
    if (!data.success) {
      throw new Error(data.error || 'Failed to create task');
    }

    const createdTask = this.transformTask(data.data);

    this.emitTaskEvent({
      type: 'created',
      task: createdTask,
      timestamp: new Date(),
    });

    return createdTask;
  }

  async updateTask(projectPath: string, taskId: string, updates: Partial<Task>): Promise<Task> {
    const projectId = await this.getProjectIdByPath(projectPath);
    const response = await fetch(`${this.apiBaseUrl}/api/projects/${projectId}/tasks/${taskId}`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(this.serializeTask(updates)),
    });

    const data = await response.json();
    if (!data.success) {
      throw new Error(data.error || 'Failed to update task');
    }

    const updatedTask = this.transformTask(data.data);

    this.emitTaskEvent({
      type: 'updated',
      task: updatedTask,
      previousStatus: updates.status,
      timestamp: new Date(),
    });

    return updatedTask;
  }

  async deleteTask(projectPath: string, taskId: string): Promise<void> {
    const projectId = await this.getProjectIdByPath(projectPath);
    const response = await fetch(`${this.apiBaseUrl}/api/projects/${projectId}/tasks/${taskId}`, {
      method: 'DELETE',
    });

    const data = await response.json();
    if (!data.success) {
      throw new Error(data.error || 'Failed to delete task');
    }

    this.emitTaskEvent({
      type: 'deleted',
      task: { id: taskId } as Task,
      timestamp: new Date(),
    });
  }

  watchTasks(projectPath: string, callback: (tasks: Task[]) => void): () => void {
    // Poll for now, WebSocket support can be added later
    const interval = setInterval(async () => {
      try {
        const tasks = await this.getTasks(projectPath);
        callback(tasks);
      } catch (error) {
        console.error('Error watching tasks:', error);
      }
    }, 5000);

    return () => clearInterval(interval);
  }

  private async getProjectIdByPath(projectPath: string): Promise<string> {
    if (this.projectId) {
      return this.projectId;
    }

    const response = await fetch(`${this.apiBaseUrl}/api/projects/by-path`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ projectRoot: projectPath }),
    });

    const data = await response.json();
    if (!data.success) {
      throw new Error('Project not found');
    }

    this.projectId = data.data.id;
    return this.projectId;
  }

  private transformTask(data: any): Task {
    return {
      ...data,
      projectId: data.project_id || data.projectId,
      createdByUserId: data.created_by_user_id || data.createdByUserId,
      assignedAgentId: data.assigned_agent_id || data.assignedAgentId,
      assignedAgent: data.assigned_agent || data.assignedAgent,
      reviewedByAgentId: data.reviewed_by_agent_id || data.reviewedByAgentId,
      parentId: data.parent_id || data.parentId,
      estimatedHours: data.estimated_hours || data.estimatedHours,
      actualHours: data.actual_hours || data.actualHours,
      complexityScore: data.complexity_score || data.complexityScore,
      testStrategy: data.test_strategy || data.testStrategy,
      acceptanceCriteria: data.acceptance_criteria || data.acceptanceCriteria,
      outputFormat: data.output_format || data.outputFormat,
      validationRules: data.validation_rules || data.validationRules,
      executionLog: data.execution_log || data.executionLog,
      errorLog: data.error_log || data.errorLog,
      retryCount: data.retry_count || data.retryCount || 0,
      dueDate: data.due_date || data.dueDate ? new Date(data.due_date || data.dueDate) : undefined,
      startedAt: data.started_at || data.startedAt ? new Date(data.started_at || data.startedAt) : undefined,
      completedAt: data.completed_at || data.completedAt ? new Date(data.completed_at || data.completedAt) : undefined,
      createdAt: new Date(data.created_at || data.createdAt),
      updatedAt: new Date(data.updated_at || data.updatedAt),
    };
  }

  private transformTasks(data: any[]): Task[] {
    return data.map(item => this.transformTask(item));
  }

  private serializeTask(task: Partial<Task>): any {
    return {
      title: task.title,
      description: task.description,
      status: task.status,
      priority: task.priority,
      assigned_agent_id: task.assignedAgentId,
      reviewed_by_agent_id: task.reviewedByAgentId,
      parent_id: task.parentId,
      position: task.position,
      dependencies: task.dependencies ? JSON.stringify(task.dependencies) : undefined,
      blockers: task.blockers ? JSON.stringify(task.blockers) : undefined,
      due_date: task.dueDate?.toISOString(),
      estimated_hours: task.estimatedHours,
      actual_hours: task.actualHours,
      complexity_score: task.complexityScore,
      details: task.details,
      test_strategy: task.testStrategy,
      acceptance_criteria: task.acceptanceCriteria,
      prompt: task.prompt,
      context: task.context,
      output_format: task.outputFormat,
      validation_rules: task.validationRules ? JSON.stringify(task.validationRules) : undefined,
      tags: task.tags ? JSON.stringify(task.tags) : undefined,
      category: task.category,
      metadata: task.metadata ? JSON.stringify(task.metadata) : undefined,
    };
  }
}
