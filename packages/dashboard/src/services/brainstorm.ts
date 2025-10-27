// ABOUTME: Brainstorm session service layer for PRD ideation API integration
// ABOUTME: Handles session CRUD, mode selection, and section skip functionality

import { apiClient } from './api';

export type BrainstormMode = 'quick' | 'guided' | 'comprehensive';
export type BrainstormStatus = 'draft' | 'in_progress' | 'ready_for_prd' | 'completed';

export interface BrainstormSession {
  id: string;
  project_id: string;
  initial_description: string;
  mode: BrainstormMode;
  status: BrainstormStatus;
  skipped_sections: string[] | null;
  created_at: string;
  updated_at: string;
}

export interface CreateBrainstormInput {
  projectId: string;
  initialDescription: string;
  mode: BrainstormMode;
}

export interface UpdateBrainstormInput {
  initialDescription?: string;
  mode?: BrainstormMode;
  status?: BrainstormStatus;
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

class BrainstormService {
  /**
   * Create a new brainstorm session
   */
  async createSession(input: CreateBrainstormInput): Promise<BrainstormSession> {
    const response = await apiClient.post<{ success: boolean; data: BrainstormSession }>(
      '/api/brainstorm/start',
      input
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to create brainstorm session');
    }

    return response.data.data;
  }

  /**
   * Get a brainstorm session by ID
   */
  async getSession(sessionId: string): Promise<BrainstormSession> {
    const response = await apiClient.get<{ success: boolean; data: BrainstormSession }>(
      `/api/brainstorm/${sessionId}`
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to fetch brainstorm session');
    }

    return response.data.data;
  }

  /**
   * List all brainstorm sessions for a project
   */
  async listSessions(projectId: string): Promise<BrainstormSession[]> {
    const response = await apiClient.get<{ success: boolean; data: BrainstormSession[] }>(
      `/api/${projectId}/brainstorm/sessions`
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to fetch brainstorm sessions');
    }

    return response.data.data;
  }

  /**
   * Update a brainstorm session
   */
  async updateSession(sessionId: string, input: UpdateBrainstormInput): Promise<void> {
    const response = await apiClient.put<{ success: boolean }>(
      `/api/brainstorm/${sessionId}`,
      input
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to update brainstorm session');
    }
  }

  /**
   * Delete a brainstorm session
   */
  async deleteSession(sessionId: string): Promise<void> {
    const response = await apiClient.delete<{ success: boolean }>(
      `/api/brainstorm/${sessionId}`
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to delete brainstorm session');
    }
  }

  /**
   * Skip a section with optional AI fill
   */
  async skipSection(sessionId: string, input: SkipSectionInput): Promise<void> {
    const response = await apiClient.post<{ success: boolean }>(
      `/api/brainstorm/${sessionId}/skip-section`,
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
      `/api/brainstorm/${sessionId}/status`
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to fetch completion status');
    }

    return response.data.data;
  }
}

export const brainstormService = new BrainstormService();
