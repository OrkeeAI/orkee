// ABOUTME: Frontend service for model operations
// ABOUTME: Handles listing and managing AI models with pricing information

import { apiRequest } from './api';
import type { PaginationParams, PaginatedResponse } from '@/types/pagination';
import { buildPaginationQuery } from '@/types/pagination';

export interface Model {
  id: string;
  provider: string;
  model: string;
  display_name: string;
  description: string;
  cost_per_1k_input_tokens: number;
  cost_per_1k_output_tokens: number;
  max_context_tokens: number;
  is_available: boolean;
}

interface ApiResponse<T> {
  success: boolean;
  data: T | null;
  error?: string;
}

export class ModelsService {
  async listModels(pagination?: PaginationParams): Promise<PaginatedResponse<Model>> {
    const query = pagination ? buildPaginationQuery(pagination) : '';
    const response = await apiRequest<ApiResponse<PaginatedResponse<Model>>>(
      `/api/models${query}`
    );

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to fetch models');
    }

    return response.data;
  }

  async getModel(modelId: string): Promise<Model> {
    const response = await apiRequest<ApiResponse<Model>>(
      `/api/models/${modelId}`
    );

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to fetch model');
    }

    return response.data;
  }

  async listModelsByProvider(
    provider: string,
    pagination?: PaginationParams
  ): Promise<PaginatedResponse<Model>> {
    const query = pagination ? buildPaginationQuery(pagination) : '';
    const response = await apiRequest<ApiResponse<PaginatedResponse<Model>>>(
      `/api/models/provider/${provider}${query}`
    );

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to fetch models for provider');
    }

    return response.data;
  }
}

export const modelsService = new ModelsService();
