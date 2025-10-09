import { apiClient } from './api';

// TypeScript interfaces matching Rust types
export type ProjectStatus = 'pre-launch' | 'launched' | 'archived';
export type Priority = 'high' | 'medium' | 'low';
export type TaskSource = 'taskmaster' | 'manual';
export type TaskStatus = 'pending' | 'done' | 'in-progress' | 'review' | 'deferred' | 'cancelled';

export interface ManualSubtask {
  id: number;
  title: string;
  description: string;
  dependencies: number[];
  details?: string;
  status: TaskStatus;
  testStrategy?: string;
}

export interface ManualTask {
  id: number;
  title: string;
  description: string;
  details?: string;
  testStrategy?: string;
  priority: Priority;
  dependencies: number[];
  status: TaskStatus;
  subtasks: ManualSubtask[];
  createdAt: string;
  updatedAt: string;
}

export interface GitRepositoryInfo {
  owner: string;
  repo: string;
  url: string;
  branch?: string;
}

export interface Project {
  id: string;
  name: string;
  projectRoot: string;
  setupScript?: string;
  devScript?: string;
  cleanupScript?: string;
  createdAt: string;
  updatedAt: string;
  tags?: string[];
  description?: string;
  status: ProjectStatus;
  rank?: number;
  priority: Priority;
  taskSource?: TaskSource;
  manualTasks?: ManualTask[];
  mcpServers?: Record<string, boolean>;
  gitRepository?: GitRepositoryInfo;
}

export interface ProjectCreateInput {
  name: string;
  projectRoot: string;
  setupScript?: string;
  devScript?: string;
  cleanupScript?: string;
  tags?: string[];
  description?: string;
  status?: ProjectStatus;
  rank?: number;
  priority?: Priority;
  taskSource?: TaskSource;
  manualTasks?: ManualTask[];
  mcpServers?: Record<string, boolean>;
}

export interface ProjectUpdateInput {
  name?: string;
  projectRoot?: string;
  setupScript?: string;
  devScript?: string;
  cleanupScript?: string;
  tags?: string[];
  description?: string;
  status?: ProjectStatus;
  rank?: number;
  priority?: Priority;
  taskSource?: TaskSource;
  manualTasks?: ManualTask[];
  mcpServers?: Record<string, boolean>;
}

// API Response format from Rust server
interface ApiResponse<T> {
  success: boolean;
  data: T | null;
  error?: string;
}

export class ProjectsService {
  async getAllProjects(): Promise<Project[]> {
    const response = await apiClient.get<ApiResponse<Project[]>>('/api/projects');
    
    if (response.error || !response.data?.success) {
      throw new Error(response.data?.error || response.error || 'Failed to fetch projects');
    }
    
    return response.data.data || [];
  }

  async getProject(id: string): Promise<Project | null> {
    const response = await apiClient.get<ApiResponse<Project>>(`/api/projects/${id}`);
    
    if (response.error) {
      throw new Error(response.error);
    }
    
    if (!response.data?.success) {
      if (response.data?.error?.includes('not found')) {
        return null;
      }
      throw new Error(response.data?.error || 'Failed to fetch project');
    }
    
    return response.data.data;
  }

  async getProjectByName(name: string): Promise<Project | null> {
    const response = await apiClient.get<ApiResponse<Project>>(`/api/projects/by-name/${encodeURIComponent(name)}`);
    
    if (response.error) {
      throw new Error(response.error);
    }
    
    if (!response.data?.success) {
      if (response.data?.error?.includes('not found')) {
        return null;
      }
      throw new Error(response.data?.error || 'Failed to fetch project');
    }
    
    return response.data.data;
  }

  async getProjectByPath(projectRoot: string): Promise<Project | null> {
    const response = await apiClient.post<ApiResponse<Project>>('/api/projects/by-path', {
      projectRoot
    });
    
    if (response.error) {
      throw new Error(response.error);
    }
    
    if (!response.data?.success) {
      if (response.data?.error?.includes('not found')) {
        return null;
      }
      throw new Error(response.data?.error || 'Failed to fetch project');
    }
    
    return response.data.data;
  }

  async createProject(projectData: ProjectCreateInput): Promise<Project> {
    const response = await apiClient.post<ApiResponse<Project>>('/api/projects', projectData);
    
    if (response.error || !response.data?.success) {
      throw new Error(response.data?.error || response.error || 'Failed to create project');
    }
    
    if (!response.data.data) {
      throw new Error('No project data returned');
    }
    
    return response.data.data;
  }

  async updateProject(id: string, updates: ProjectUpdateInput): Promise<Project> {
    const { apiRequest } = await import('./api');
    const result = await apiRequest<ApiResponse<Project>>(`/api/projects/${id}`, {
      method: 'PUT',
      body: JSON.stringify(updates),
    });

    if (!result.success || !result.data) {
      throw new Error(result.error || 'Failed to update project');
    }

    if (!result.data.success) {
      throw new Error(result.data.error || 'Failed to update project');
    }

    if (!result.data.data) {
      throw new Error('No project data returned');
    }

    return result.data.data;
  }

  async deleteProject(id: string): Promise<boolean> {
    const { apiRequest } = await import('./api');
    const result = await apiRequest<ApiResponse<string>>(`/api/projects/${id}`, {
      method: 'DELETE',
    });

    if (!result.success || !result.data) {
      throw new Error(result.error || 'Failed to delete project');
    }

    if (!result.data.success) {
      throw new Error(result.data.error || 'Failed to delete project');
    }

    return true;
  }
}

export const projectsService = new ProjectsService();