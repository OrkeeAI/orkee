// ABOUTME: Chat mode API service for chat-based PRD discovery
// ABOUTME: Handles chat messages, discovery questions, and PRD generation from chats

import { apiClient } from './api';

export type DiscoveryStatus = 'draft' | 'brainstorming' | 'refining' | 'validating' | 'finalized';
export type MessageRole = 'user' | 'assistant' | 'system';
export type MessageType = 'discovery' | 'refinement' | 'validation' | 'general';
export type InsightType = 'requirement' | 'constraint' | 'risk' | 'assumption' | 'decision';

export interface ChatMessage {
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
  role?: MessageRole;
  model?: string;
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

export interface ChatInsight {
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

export interface GeneratePRDFromChatInput {
  title: string;
}

export interface GeneratePRDFromChatResult {
  prd_id: string;
  content_markdown: string;
  quality_score: number;
}

class ChatService {
  /**
   * Get chat history for a session
   */
  async getHistory(sessionId: string): Promise<ChatMessage[]> {
    const response = await apiClient.get<{ success: boolean; data: ChatMessage[] }>(
      `/api/ideate/chat/${sessionId}/history`
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to fetch chat history');
    }

    return response.data.data;
  }

  /**
   * Send a message in the chat
   */
  async sendMessage(sessionId: string, input: SendMessageInput): Promise<ChatMessage> {
    const response = await apiClient.post<{ success: boolean; data: ChatMessage }>(
      `/api/ideate/chat/${sessionId}/message`,
      input
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to send message');
    }

    return response.data.data;
  }


  /**
   * Get discovery questions (optionally filtered by category)
   */
  async getDiscoveryQuestions(category?: string): Promise<DiscoveryQuestion[]> {
    const url = category
      ? `/api/ideate/chat/questions?category=${category}`
      : '/api/ideate/chat/questions';

    const response = await apiClient.get<{ success: boolean; data: DiscoveryQuestion[] }>(url);

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to fetch discovery questions');
    }

    return response.data.data;
  }

  /**
   * Get suggested questions based on chat context
   */
  async getSuggestedQuestions(sessionId: string): Promise<DiscoveryQuestion[]> {
    const response = await apiClient.get<{ success: boolean; data: DiscoveryQuestion[] }>(
      `/api/ideate/chat/${sessionId}/suggested-questions`
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to fetch suggested questions');
    }

    return response.data.data;
  }

  /**
   * Get extracted insights from the chat
   */
  async getInsights(sessionId: string): Promise<ChatInsight[]> {
    const response = await apiClient.get<{ success: boolean; data: ChatInsight[] }>(
      `/api/ideate/chat/${sessionId}/insights`
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to fetch insights');
    }

    return response.data.data;
  }

  /**
   * Get quality metrics for the chat
   */
  async getQualityMetrics(sessionId: string): Promise<QualityMetrics> {
    const response = await apiClient.get<{ success: boolean; data: QualityMetrics }>(
      `/api/ideate/chat/${sessionId}/quality`
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
      `/api/ideate/chat/${sessionId}/status`,
      { status }
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to update discovery status');
    }
  }

  /**
   * Generate PRD from chat
   */
  async generatePRD(
    sessionId: string,
    input: GeneratePRDFromChatInput
  ): Promise<GeneratePRDFromChatResult> {
    const response = await apiClient.post<{ success: boolean; data: GeneratePRDFromChatResult }>(
      `/api/ideate/chat/${sessionId}/generate-prd`,
      input
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to generate PRD from chat');
    }

    return response.data.data;
  }

  /**
   * Validate chat readiness for PRD generation
   */
  async validateForPRD(sessionId: string): Promise<{
    is_valid: boolean;
    missing_required: string[];
    warnings: string[];
  }> {
    const response = await apiClient.get<{
      success: boolean;
      data: { is_valid: boolean; missing_required: string[]; warnings: string[] };
    }>(`/api/ideate/chat/${sessionId}/validate`);

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to validate chat');
    }

    return response.data.data;
  }

  /**
   * Create a new insight
   */
  async createInsight(
    sessionId: string,
    input: {
      insight_type: InsightType;
      insight_text: string;
      confidence_score?: number;
      source_message_ids?: string[];
    }
  ): Promise<ChatInsight> {
    const response = await apiClient.post<{ success: boolean; data: ChatInsight }>(
      `/api/ideate/chat/${sessionId}/insights`,
      input
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to create insight');
    }

    return response.data.data;
  }

  /**
   * Re-analyze entire session history to extract insights
   */
  async reanalyzeInsights(sessionId: string): Promise<{
    extracted_count: number;
    error_count: number;
    total_messages_processed: number;
  }> {
    const response = await apiClient.post<{
      success: boolean;
      data: {
        extracted_count: number;
        error_count: number;
        total_messages_processed: number;
      };
    }>(`/api/ideate/chat/${sessionId}/insights/reanalyze`, {});

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to re-analyze insights');
    }

    return response.data.data;
  }
}

export const chatService = new ChatService();
