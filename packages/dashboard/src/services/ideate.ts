// ABOUTME: Ideate session service layer for PRD ideation API integration
// ABOUTME: Handles session CRUD, mode selection, and section skip functionality

import { apiClient } from './api';

export type IdeateMode = 'quick' | 'guided' | 'comprehensive';
export type IdeateStatus = 'draft' | 'in_progress' | 'ready_for_prd' | 'completed';

export interface IdeateSession {
  id: string;
  project_id: string;
  initial_description: string;
  mode: IdeateMode;
  status: IdeateStatus;
  skipped_sections: string[] | null;
  created_at: string;
  updated_at: string;
}

export interface CreateIdeateInput {
  projectId: string;
  initialDescription: string;
  mode: IdeateMode;
}

export interface UpdateIdeateInput {
  initialDescription?: string;
  mode?: IdeateMode;
  status?: IdeateStatus;
  skippedSections?: string[];
}

export interface SkipSectionInput {
  section: string;
  ai_fill: boolean;
}

export interface SessionCompletionStatus {
  session_id: string;
  total_sections: number;
  completed_sections: number;
  skipped_sections: string[];
  is_ready_for_prd: boolean;
  missing_required_sections: string[];
}

class IdeateService {
  /**
   * Create a new ideate session
   */
  async createSession(input: CreateIdeateInput): Promise<IdeateSession> {
    const response = await apiClient.post<{ success: boolean; data: IdeateSession }>(
      '/api/ideate/start',
      input
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to create ideate session');
    }

    return response.data.data;
  }

  /**
   * Get a ideate session by ID
   */
  async getSession(sessionId: string): Promise<IdeateSession> {
    const response = await apiClient.get<{ success: boolean; data: IdeateSession }>(
      `/api/ideate/${sessionId}`
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to fetch ideate session');
    }

    return response.data.data;
  }

  /**
   * List all ideate sessions for a project
   */
  async listSessions(projectId: string): Promise<IdeateSession[]> {
    const response = await apiClient.get<{ success: boolean; data: IdeateSession[] }>(
      `/api/${projectId}/ideate/sessions`
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to fetch ideate sessions');
    }

    return response.data.data;
  }

  /**
   * Update a ideate session
   */
  async updateSession(sessionId: string, input: UpdateIdeateInput): Promise<void> {
    const response = await apiClient.put<{ success: boolean }>(
      `/api/ideate/${sessionId}`,
      input
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to update ideate session');
    }
  }

  /**
   * Delete a ideate session
   */
  async deleteSession(sessionId: string): Promise<void> {
    const response = await apiClient.delete<{ success: boolean }>(
      `/api/ideate/${sessionId}`
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to delete ideate session');
    }
  }

  /**
   * Skip a section with optional AI fill
   */
  async skipSection(sessionId: string, input: SkipSectionInput): Promise<void> {
    const response = await apiClient.post<{ success: boolean }>(
      `/api/ideate/${sessionId}/skip-section`,
      input
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to skip section');
    }
  }

  /**
   * Get session completion status
   */
  async getCompletionStatus(sessionId: string): Promise<SessionCompletionStatus> {
    const response = await apiClient.get<{ success: boolean; data: SessionCompletionStatus }>(
      `/api/ideate/${sessionId}/status`
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to fetch completion status');
    }

    return response.data.data;
  }
}

export const ideateService = new IdeateService();
