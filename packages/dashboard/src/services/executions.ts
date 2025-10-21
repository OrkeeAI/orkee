// ABOUTME: Frontend service for agent execution operations
// ABOUTME: Handles CRUD operations for executions and PR reviews with pagination

import { apiRequest } from './api';
import type { PaginationParams, PaginatedResponse } from '@/types/pagination';
import { buildPaginationQuery } from '@/types/pagination';

export type ExecutionStatus = 'pending' | 'running' | 'succeeded' | 'failed' | 'cancelled';
export type PrStatus = 'open' | 'closed' | 'merged' | 'draft';
export type ReviewStatus = 'pending' | 'approved' | 'changes_requested' | 'commented' | 'dismissed';
export type ReviewerType = 'human' | 'ai' | 'system';

export interface AgentExecution {
  id: string;
  taskId: string;
  agentId: string | null;
  model: string | null;
  status: ExecutionStatus;
  startedAt: string;
  completedAt: string | null;
  executionTimeSeconds: number | null;
  tokensInput: number | null;
  tokensOutput: number | null;
  totalCost: number | null;
  prompt: string | null;
  response: string | null;
  errorMessage: string | null;
  retryAttempt: number | null;
  filesChanged: number | null;
  linesAdded: number | null;
  linesRemoved: number | null;
  filesCreated: string[] | null;
  filesModified: string[] | null;
  filesDeleted: string[] | null;
  branchName: string | null;
  commitHash: string | null;
  commitMessage: string | null;
  prNumber: number | null;
  prUrl: string | null;
  prTitle: string | null;
  prStatus: PrStatus | null;
  prCreatedAt: string | null;
  prMergedAt: string | null;
  prMergeCommit: string | null;
  reviewStatus: ReviewStatus | null;
  reviewComments: number | null;
  testResults: unknown;
  performanceMetrics: unknown;
  metadata: unknown;
}

export interface PrReview {
  id: string;
  executionId: string;
  reviewerId: string | null;
  reviewerType: ReviewerType;
  reviewStatus: ReviewStatus;
  reviewBody: string | null;
  comments: unknown;
  suggestedChanges: unknown;
  submittedAt: string;
  approvalDate: string | null;
  dismissalReason: string | null;
}

export interface AgentExecutionCreateInput {
  taskId: string;
  agentId?: string;
  model?: string;
  prompt?: string;
  retryAttempt?: number;
}

export interface AgentExecutionUpdateInput {
  status?: ExecutionStatus;
  completedAt?: string;
  executionTimeSeconds?: number;
  tokensInput?: number;
  tokensOutput?: number;
  totalCost?: number;
  response?: string;
  errorMessage?: string;
  filesChanged?: number;
  linesAdded?: number;
  linesRemoved?: number;
  filesCreated?: string[];
  filesModified?: string[];
  filesDeleted?: string[];
  branchName?: string;
  commitHash?: string;
  commitMessage?: string;
  prNumber?: number;
  prUrl?: string;
  prTitle?: string;
  prStatus?: PrStatus;
  prCreatedAt?: string;
  prMergedAt?: string;
  prMergeCommit?: string;
  reviewStatus?: ReviewStatus;
  reviewComments?: number;
  testResults?: unknown;
  performanceMetrics?: unknown;
  metadata?: unknown;
}

export interface PrReviewCreateInput {
  executionId: string;
  reviewerId?: string;
  reviewerType: ReviewerType;
  reviewStatus: ReviewStatus;
  reviewBody?: string;
  comments?: unknown;
  suggestedChanges?: unknown;
}

export interface PrReviewUpdateInput {
  reviewStatus?: ReviewStatus;
  reviewBody?: string;
  comments?: unknown;
  suggestedChanges?: unknown;
  approvalDate?: string;
  dismissalReason?: string;
}

interface ApiResponse<T> {
  success: boolean;
  data: T | null;
  error?: string;
}

export class ExecutionsService {
  async listExecutions(
    taskId: string,
    pagination?: PaginationParams
  ): Promise<PaginatedResponse<AgentExecution>> {
    const query = pagination ? buildPaginationQuery(pagination) : '';
    const response = await apiRequest<ApiResponse<PaginatedResponse<AgentExecution>>>(
      `/api/tasks/${taskId}/executions${query}`
    );

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to fetch executions');
    }

    if (!response.data.success) {
      throw new Error(response.data.error || 'Failed to fetch executions');
    }

    return response.data.data!;
  }

  async getExecution(executionId: string): Promise<AgentExecution> {
    const response = await apiRequest<ApiResponse<AgentExecution>>(
      `/api/executions/${executionId}`
    );

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to fetch execution');
    }

    if (!response.data.success) {
      throw new Error(response.data.error || 'Failed to fetch execution');
    }

    return response.data.data!;
  }

  async createExecution(input: AgentExecutionCreateInput): Promise<AgentExecution> {
    const response = await apiRequest<ApiResponse<AgentExecution>>(
      '/api/executions',
      {
        method: 'POST',
        body: JSON.stringify(input),
      }
    );

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to create execution');
    }

    if (!response.data.success) {
      throw new Error(response.data.error || 'Failed to create execution');
    }

    return response.data.data!;
  }

  async updateExecution(
    executionId: string,
    input: AgentExecutionUpdateInput
  ): Promise<AgentExecution> {
    const response = await apiRequest<ApiResponse<AgentExecution>>(
      `/api/executions/${executionId}`,
      {
        method: 'PUT',
        body: JSON.stringify(input),
      }
    );

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to update execution');
    }

    if (!response.data.success) {
      throw new Error(response.data.error || 'Failed to update execution');
    }

    return response.data.data!;
  }

  async deleteExecution(executionId: string): Promise<void> {
    const response = await apiRequest<ApiResponse<string>>(
      `/api/executions/${executionId}`,
      {
        method: 'DELETE',
      }
    );

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to delete execution');
    }

    if (!response.data.success) {
      throw new Error(response.data.error || 'Failed to delete execution');
    }
  }

  async listReviews(executionId: string): Promise<PrReview[]> {
    const response = await apiRequest<ApiResponse<PrReview[]>>(
      `/api/executions/${executionId}/reviews`
    );

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to fetch reviews');
    }

    if (!response.data.success) {
      throw new Error(response.data.error || 'Failed to fetch reviews');
    }

    return response.data.data!;
  }

  async getReview(reviewId: string): Promise<PrReview> {
    const response = await apiRequest<ApiResponse<PrReview>>(
      `/api/reviews/${reviewId}`
    );

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to fetch review');
    }

    if (!response.data.success) {
      throw new Error(response.data.error || 'Failed to fetch review');
    }

    return response.data.data!;
  }

  async createReview(input: PrReviewCreateInput): Promise<PrReview> {
    const response = await apiRequest<ApiResponse<PrReview>>(
      '/api/reviews',
      {
        method: 'POST',
        body: JSON.stringify(input),
      }
    );

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to create review');
    }

    if (!response.data.success) {
      throw new Error(response.data.error || 'Failed to create review');
    }

    return response.data.data!;
  }

  async updateReview(reviewId: string, input: PrReviewUpdateInput): Promise<PrReview> {
    const response = await apiRequest<ApiResponse<PrReview>>(
      `/api/reviews/${reviewId}`,
      {
        method: 'PUT',
        body: JSON.stringify(input),
      }
    );

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to update review');
    }

    if (!response.data.success) {
      throw new Error(response.data.error || 'Failed to update review');
    }

    return response.data.data!;
  }

  async deleteReview(reviewId: string): Promise<void> {
    const response = await apiRequest<ApiResponse<string>>(
      `/api/reviews/${reviewId}`,
      {
        method: 'DELETE',
      }
    );

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to delete review');
    }

    if (!response.data.success) {
      throw new Error(response.data.error || 'Failed to delete review');
    }
  }
}

export const executionsService = new ExecutionsService();
