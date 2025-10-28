// ABOUTME: PRD output template service for API integration
// ABOUTME: Handles CRUD operations for markdown templates used to format PRD content
import { apiClient } from './api';

export interface PRDTemplate {
  id: string;
  name: string;
  description?: string;
  content: string;
  is_default: boolean;
  created_at: string;
  updated_at: string;
}

export interface CreateTemplateInput {
  name: string;
  description?: string;
  content: string;
  is_default?: boolean;
}

export interface UpdateTemplateInput {
  name?: string;
  description?: string;
  content?: string;
  is_default?: boolean;
}

export const templatesService = {
  async getAll(): Promise<PRDTemplate[]> {
    const response = await apiClient.get<PRDTemplate[]>('/api/templates');
    if (response.error) {
      throw new Error(response.error);
    }
    return response.data;
  },

  async getById(id: string): Promise<PRDTemplate> {
    const response = await apiClient.get<PRDTemplate>(`/api/templates/${id}`);
    if (response.error) {
      throw new Error(response.error);
    }
    return response.data;
  },

  async create(input: CreateTemplateInput): Promise<PRDTemplate> {
    const response = await apiClient.post<PRDTemplate>('/api/templates', input);
    if (response.error) {
      throw new Error(response.error);
    }
    return response.data;
  },

  async update(id: string, input: UpdateTemplateInput): Promise<PRDTemplate> {
    const response = await apiClient.put<PRDTemplate>(`/api/templates/${id}`, input);
    if (response.error) {
      throw new Error(response.error);
    }
    return response.data;
  },

  async delete(id: string): Promise<void> {
    const response = await apiClient.delete<void>(`/api/templates/${id}`);
    if (response.error) {
      throw new Error(response.error);
    }
  },
};
