// ABOUTME: Conversational mode API service for chat-based PRD discovery
// ABOUTME: Handles conversation messages, discovery questions, and PRD generation from conversations

import { apiClient } from './api';

export type DiscoveryStatus = 'draft' | 'brainstorming' | 'refining' | 'validating' | 'finalized';
export type MessageRole = 'user' | 'assistant' | 'system';
export type MessageType = 'discovery' | 'refinement' | 'validation' | 'general';
export type InsightType = 'requirement' | 'constraint' | 'risk' | 'assumption' | 'decision';

export interface ConversationMessage {
  id: string;
  session_id: string;
  prd_id: string | null;
  message_order: number;
  role: MessageRole;
  content: string;
  message_type: MessageType | null;
  metadata: Record<string, unknown> | null;
  created_at: string;
}

export interface SendMessageInput {
  content: string;
  message_type?: MessageType;
}

export interface DiscoveryQuestion {
  id: string;
  category: 'problem' | 'users' | 'features' | 'technical' | 'risks' | 'constraints' | 'success';
  question_text: string;
  follow_up_prompts: string[] | null;
  context_keywords: string[] | null;
  priority: number;
  is_required: boolean;
  display_order: number;
  is_active: boolean;
  created_at: string;
}

export interface ConversationInsight {
  id: string;
  session_id: string;
  insight_type: InsightType;
  insight_text: string;
  confidence_score: number | null;
  source_message_ids: string[] | null;
  applied_to_prd: boolean;
  created_at: string;
}

export interface QualityMetrics {
  quality_score: number;
  missing_areas: string[];
  coverage: {
    problem: boolean;
    users: boolean;
    features: boolean;
    technical: boolean;
    risks: boolean;
    constraints: boolean;
    success: boolean;
  };
  is_ready_for_prd: boolean;
}

export interface GeneratePRDFromConversationInput {
  title: string;
}

export interface GeneratePRDFromConversationResult {
  prd_id: string;
  content_markdown: string;
  quality_score: number;
}

class ConversationalService {
  /**
   * Get conversation history for a session
   */
  async getHistory(sessionId: string): Promise<ConversationMessage[]> {
    const response = await apiClient.get<{ success: boolean; data: ConversationMessage[] }>(
      `/api/ideate/conversational/${sessionId}/history`
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to fetch conversation history');
    }

    return response.data.data;
  }

  /**
   * Send a message in the conversation
   */
  async sendMessage(sessionId: string, input: SendMessageInput): Promise<ConversationMessage> {
    const response = await apiClient.post<{ success: boolean; data: ConversationMessage }>(
      `/api/ideate/conversational/${sessionId}/message`,
      input
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to send message');
    }

    return response.data.data;
  }

  /**
   * Get streaming SSE URL for conversational responses
   */
  getStreamUrl(sessionId: string): string {
    return `/api/ideate/conversational/${sessionId}/stream`;
  }

  /**
   * Get discovery questions (optionally filtered by category)
   */
  async getDiscoveryQuestions(category?: string): Promise<DiscoveryQuestion[]> {
    const url = category
      ? `/api/ideate/conversational/questions?category=${category}`
      : '/api/ideate/conversational/questions';

    const response = await apiClient.get<{ success: boolean; data: DiscoveryQuestion[] }>(url);

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to fetch discovery questions');
    }

    return response.data.data;
  }

  /**
   * Get suggested questions based on conversation context
   */
  async getSuggestedQuestions(sessionId: string): Promise<DiscoveryQuestion[]> {
    const response = await apiClient.get<{ success: boolean; data: DiscoveryQuestion[] }>(
      `/api/ideate/conversational/${sessionId}/suggested-questions`
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to fetch suggested questions');
    }

    return response.data.data;
  }

  /**
   * Get extracted insights from the conversation
   */
  async getInsights(sessionId: string): Promise<ConversationInsight[]> {
    const response = await apiClient.get<{ success: boolean; data: ConversationInsight[] }>(
      `/api/ideate/conversational/${sessionId}/insights`
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to fetch insights');
    }

    return response.data.data;
  }

  /**
   * Get quality metrics for the conversation
   */
  async getQualityMetrics(sessionId: string): Promise<QualityMetrics> {
    const response = await apiClient.get<{ success: boolean; data: QualityMetrics }>(
      `/api/ideate/conversational/${sessionId}/quality`
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to fetch quality metrics');
    }

    return response.data.data;
  }

  /**
   * Update discovery status
   */
  async updateDiscoveryStatus(sessionId: string, status: DiscoveryStatus): Promise<void> {
    const response = await apiClient.put<{ success: boolean }>(
      `/api/ideate/conversational/${sessionId}/status`,
      { status }
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to update discovery status');
    }
  }

  /**
   * Generate PRD from conversation
   */
  async generatePRD(
    sessionId: string,
    input: GeneratePRDFromConversationInput
  ): Promise<GeneratePRDFromConversationResult> {
    const response = await apiClient.post<{ success: boolean; data: GeneratePRDFromConversationResult }>(
      `/api/ideate/conversational/${sessionId}/generate-prd`,
      input
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to generate PRD from conversation');
    }

    return response.data.data;
  }

  /**
   * Validate conversation readiness for PRD generation
   */
  async validateForPRD(sessionId: string): Promise<{
    is_valid: boolean;
    missing_required: string[];
    warnings: string[];
  }> {
    const response = await apiClient.get<{
      success: boolean;
      data: { is_valid: boolean; missing_required: string[]; warnings: string[] };
    }>(`/api/ideate/conversational/${sessionId}/validate`);

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to validate conversation');
    }

    return response.data.data;
  }
}

export const conversationalService = new ConversationalService();
