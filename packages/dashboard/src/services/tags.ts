// ABOUTME: Frontend service for tag operations
// ABOUTME: Handles CRUD operations for tags with pagination support

import { apiRequest } from './api';
import type { PaginationParams, PaginatedResponse } from '@/types/pagination';

export interface Tag {
  id: string;
  name: string;
  color: string | null;
  description: string | null;
  archivedAt: string | null;
}

export interface TagCreateInput {
  name: string;
  color?: string;
  description?: string;
}

export interface TagUpdateInput {
  name?: string;
  color?: string;
  description?: string;
  archivedAt?: string;
}

export interface ListTagsOptions extends PaginationParams {
  includeArchived?: boolean;
}

interface ApiResponse<T> {
  success: boolean;
  data: T | null;
  error?: string;
}

export class TagsService {
  async listTags(options?: ListTagsOptions): Promise<PaginatedResponse<Tag>> {
    const queryParts: string[] = [];

    if (options) {
      const { includeArchived, ...pagination } = options;

      if (pagination.page !== undefined) {
        queryParts.push(`page=${pagination.page}`);
      }
      if (pagination.limit !== undefined) {
        queryParts.push(`limit=${pagination.limit}`);
      }
      if (includeArchived !== undefined) {
        queryParts.push(`include_archived=${includeArchived}`);
      }
    }

    const query = queryParts.length > 0 ? `?${queryParts.join('&')}` : '';
    const response = await apiRequest<ApiResponse<PaginatedResponse<Tag>>>(
      `/api/tags${query}`
    );

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to fetch tags');
    }

    if (!response.data.success) {
      throw new Error(response.data.error || 'Failed to fetch tags');
    }

    return response.data.data!;
  }

  async getTag(tagId: string): Promise<Tag> {
    const response = await apiRequest<ApiResponse<Tag>>(
      `/api/tags/${tagId}`
    );

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to fetch tag');
    }

    if (!response.data.success) {
      throw new Error(response.data.error || 'Failed to fetch tag');
    }

    return response.data.data!;
  }

  async createTag(input: TagCreateInput): Promise<Tag> {
    const response = await apiRequest<ApiResponse<Tag>>(
      '/api/tags',
      {
        method: 'POST',
        body: JSON.stringify(input),
      }
    );

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to create tag');
    }

    if (!response.data.success) {
      throw new Error(response.data.error || 'Failed to create tag');
    }

    return response.data.data!;
  }

  async updateTag(tagId: string, input: TagUpdateInput): Promise<Tag> {
    const response = await apiRequest<ApiResponse<Tag>>(
      `/api/tags/${tagId}`,
      {
        method: 'PUT',
        body: JSON.stringify(input),
      }
    );

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to update tag');
    }

    if (!response.data.success) {
      throw new Error(response.data.error || 'Failed to update tag');
    }

    return response.data.data!;
  }

  async deleteTag(tagId: string): Promise<void> {
    const response = await apiRequest<ApiResponse<string>>(
      `/api/tags/${tagId}`,
      {
        method: 'DELETE',
      }
    );

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to delete tag');
    }

    if (!response.data.success) {
      throw new Error(response.data.error || 'Failed to delete tag');
    }
  }

  async archiveTag(tagId: string): Promise<Tag> {
    const response = await apiRequest<ApiResponse<Tag>>(
      `/api/tags/${tagId}/archive`,
      {
        method: 'POST',
      }
    );

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to archive tag');
    }

    if (!response.data.success) {
      throw new Error(response.data.error || 'Failed to archive tag');
    }

    return response.data.data!;
  }

  async unarchiveTag(tagId: string): Promise<Tag> {
    const response = await apiRequest<ApiResponse<Tag>>(
      `/api/tags/${tagId}/unarchive`,
      {
        method: 'POST',
      }
    );

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to unarchive tag');
    }

    if (!response.data.success) {
      throw new Error(response.data.error || 'Failed to unarchive tag');
    }

    return response.data.data!;
  }
}

export const tagsService = new TagsService();
