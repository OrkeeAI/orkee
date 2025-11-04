// ABOUTME: AI-powered roundtable services using AI SDK
// ABOUTME: Handles expert suggestions, discussion generation, and insight extraction

import { generateObject, streamText } from 'ai';
import { getModelInstance, calculateCost } from '@/lib/ai/config';
import { getModelForTask } from './model-preferences';
import { trackAIOperationWithCost } from '@/lib/ai/telemetry';
import { z } from 'zod';
import type {
  SuggestExpertsRequest,
  ExpertPersona,
  RoundtableMessage,
  ExpertSuggestion,
  RoundtableInsight,
  InsightPriority,
} from './ideate';

/**
 * Schema for expert suggestion
 */
const ExpertSuggestionSchema = z.object({
  name: z.string(),
  role: z.string(),
  expertise_area: z.string(),
  reason: z.string(),
  relevance_score: z.number().min(0).max(1),
});

/**
 * Schema for expert suggestions response
 */
const ExpertSuggestionsSchema = z.object({
  suggestions: z.array(ExpertSuggestionSchema),
});

/**
 * Schema for expert response
 */
const ExpertResponseSchema = z.object({
  response: z.string(),
});

/**
 * Schema for insight
 */
const InsightSchema = z.object({
  insight_text: z.string(),
  category: z.string(),
  priority: z.enum(['low', 'medium', 'high', 'critical']),
  source_experts: z.array(z.string()),
});

/**
 * Schema for insight extraction response
 */
const InsightExtractionSchema = z.object({
  insights: z.array(InsightSchema),
  summary: z.string(),
});

/**
 * System prompts for roundtable AI
 */
const EXPERT_SUGGESTION_SYSTEM_PROMPT = `You are an expert in assembling high-quality advisory panels for software projects.
Your goal is to suggest the most relevant experts who will provide diverse, valuable perspectives.
Consider technical, business, design, and operational viewpoints.
Ensure suggested experts complement each other and cover key project aspects.
Assign relevance scores between 0.0 and 1.0 based on how critical each expert is for this specific project.`;

const INSIGHT_EXTRACTION_SYSTEM_PROMPT = `You are an expert discussion analyst specializing in extracting actionable insights from expert conversations.
Your goal is to identify key themes, consensus points, disagreements, and actionable recommendations.
Prioritize insights that are: (1) actionable, (2) backed by expert reasoning, (3) address critical project concerns.
Assign priority levels based on impact: critical for must-have items, high for important, medium for nice-to-have, low for optional.
Attribute insights to the experts who mentioned them.`;

const EXPERT_RESPONSE_SYSTEM_PROMPT_PREFIX = `You are participating in an expert roundtable discussion. Stay true to your persona and provide insights
based on your specific expertise. Build on what other experts have said while adding your unique perspective.
Be concise and focused - aim for 150-250 words per response.`;

/**
 * Format conversation history for AI context
 */
export function formatConversationHistory(messages: RoundtableMessage[]): string {
  let formatted = '';

  for (const message of messages) {
    switch (message.role) {
      case 'expert':
        const name = message.expert_name || 'Expert';
        formatted += `${name}: ${message.content}\n\n`;
        break;
      case 'user':
        formatted += `User: ${message.content}\n\n`;
        break;
      case 'moderator':
        formatted += `Moderator: ${message.content}\n\n`;
        break;
      case 'system':
        // Skip system messages in context
        break;
    }
  }

  return formatted;
}

/**
 * Generate moderator opening statement
 */
export function buildModeratorOpening(topic: string, participants: ExpertPersona[]): string {
  const expertList = participants.map((e) => `${e.name} (${e.role})`).join(', ');

  return `Welcome everyone! Today we're discussing: "${topic}". Our panel includes: ${expertList}. Let's begin by having each expert share their initial thoughts on this topic.`;
}

/**
 * Build expert suggestion prompt
 */
export function buildExpertSuggestionPrompt(request: SuggestExpertsRequest): string {
  const numExperts = request.numExperts || 3;

  let prompt = `Project Description:\n${request.projectDescription}\n\n`;

  if (request.existingContent) {
    prompt += `Existing Content:\n${request.existingContent}\n\n`;
  }

  prompt += `Suggest ${numExperts} expert personas who would provide the most valuable insights for this project. Consider what perspectives would be most helpful.\n\nRespond with JSON in this format:\n{\n  "suggestions": [\n    {\n      "name": "Expert Name",\n      "role": "Job Title",\n      "expertise_area": "Primary expertise",\n      "reason": "Why this expert is relevant",\n      "relevance_score": 0.95\n    }\n  ]\n}`;

  return prompt;
}

/**
 * Build insight extraction prompt
 */
export function buildInsightExtractionPrompt(
  messages: RoundtableMessage[],
  categories?: string[]
): string {
  const conversation = formatConversationHistory(messages);

  let prompt = `Discussion:\n${conversation}\n\nExtract key insights from this expert roundtable discussion. Identify important points, consensus areas, disagreements, and actionable recommendations.\n\n`;

  if (categories && categories.length > 0) {
    prompt += `Organize insights into these categories: ${categories.join(', ')}\n\n`;
  }

  prompt += `Respond with JSON in this format:\n{\n  "insights": [\n    {\n      "insight_text": "The key insight",\n      "category": "Technical" or "UX" or "Business",\n      "priority": "low" or "medium" or "high" or "critical",\n      "source_experts": ["Expert Name 1", "Expert Name 2"]\n    }\n  ],\n  "summary": "Overall discussion summary"\n}`;

  return prompt;
}

/**
 * Suggest experts based on project context
 * Replaces Rust: suggest_experts()
 */
export async function suggestExperts(
  request: SuggestExpertsRequest,
  modelPreferences?: ReturnType<typeof getModelForTask>,
  projectId?: string | null
): Promise<Array<{ name: string; role: string; expertise_area: string; reason: string; relevance_score: number }>> {
  const modelConfig = modelPreferences || { provider: 'anthropic' as const, model: 'claude-sonnet-4-5-20250929' };
  const model = getModelInstance(modelConfig.provider, modelConfig.model);

  const prompt = buildExpertSuggestionPrompt(request);

  const result = await trackAIOperationWithCost(
    'suggest_experts',
    projectId || null,
    modelConfig.model,
    modelConfig.provider,
    (inputTokens, outputTokens) => calculateCost(modelConfig.provider, modelConfig.model, inputTokens, outputTokens),
    () =>
      generateObject({
        model,
        schema: ExpertSuggestionsSchema,
        prompt,
        system: EXPERT_SUGGESTION_SYSTEM_PROMPT,
        temperature: 0.4,
        maxTokens: 2000,
        experimental_telemetry: { isEnabled: true },
      })
  );

  return result.object.suggestions;
}

/**
 * Generate expert response during discussion
 * Replaces Rust: generate_expert_response()
 */
export async function generateExpertResponse(
  expert: ExpertPersona,
  topic: string,
  messages: RoundtableMessage[],
  allExperts: ExpertPersona[],
  modelPreferences?: ReturnType<typeof getModelForTask>,
  projectId?: string | null
): Promise<string> {
  const modelConfig = modelPreferences || { provider: 'anthropic' as const, model: 'claude-sonnet-4-5-20250929' };
  const model = getModelInstance(modelConfig.provider, modelConfig.model);

  const conversationContext = formatConversationHistory(messages);

  const prompt = `Topic: ${topic}\n\nPrevious discussion:\n${conversationContext}\n\nAs ${expert.name}, provide your perspective on this topic. Consider what other experts have said and add your unique insights. Keep your response focused and under 250 words.`;

  const systemPrompt = `${EXPERT_RESPONSE_SYSTEM_PROMPT_PREFIX}\n\n${expert.system_prompt}`;

  const result = await trackAIOperationWithCost(
    'generate_expert_response',
    projectId || null,
    modelConfig.model,
    modelConfig.provider,
    (inputTokens, outputTokens) => calculateCost(modelConfig.provider, modelConfig.model, inputTokens, outputTokens),
    () =>
      generateObject({
        model,
        schema: ExpertResponseSchema,
        prompt,
        system: systemPrompt,
        temperature: 0.7,
        maxTokens: 500,
        experimental_telemetry: { isEnabled: true },
      })
  );

  return result.object.response;
}

/**
 * Stream expert response for real-time updates
 * Enhanced version for streaming
 */
export async function streamExpertResponse(
  expert: ExpertPersona,
  topic: string,
  messages: RoundtableMessage[],
  allExperts: ExpertPersona[],
  onChunk: (text: string) => void,
  onComplete: (fullText: string) => void,
  onError: (error: Error) => void,
  abortSignal?: AbortSignal,
  modelPreferences?: ReturnType<typeof getModelForTask>,
  projectId?: string | null
): Promise<void> {
  try {
    const modelConfig = modelPreferences || { provider: 'anthropic' as const, model: 'claude-sonnet-4-5-20250929' };
    const model = getModelInstance(modelConfig.provider, modelConfig.model);

    const conversationContext = formatConversationHistory(messages);

    const prompt = `Topic: ${topic}\n\nPrevious discussion:\n${conversationContext}\n\nAs ${expert.name}, provide your perspective on this topic. Consider what other experts have said and add your unique insights. Keep your response focused and under 250 words.`;

    const systemPrompt = `${EXPERT_RESPONSE_SYSTEM_PROMPT_PREFIX}\n\n${expert.system_prompt}`;

    const result = await trackAIOperationWithCost(
      'stream_expert_response',
      projectId || null,
      modelConfig.model,
      modelConfig.provider,
      (inputTokens, outputTokens) => calculateCost(modelConfig.provider, modelConfig.model, inputTokens, outputTokens),
      () =>
        streamText({
          model,
          prompt,
          system: systemPrompt,
          temperature: 0.7,
          maxTokens: 500,
          experimental_telemetry: { isEnabled: true },
          abortSignal,
        })
    );

    let fullText = '';

    for await (const chunk of result.textStream) {
      fullText += chunk;
      onChunk(chunk);

      if (abortSignal?.aborted) {
        break;
      }
    }

    onComplete(fullText);
  } catch (error) {
    onError(error instanceof Error ? error : new Error(String(error)));
  }
}

/**
 * Extract insights from completed discussion
 * Replaces Rust: extract_insights()
 */
export async function extractInsights(
  messages: RoundtableMessage[],
  categories?: string[],
  modelPreferences?: ReturnType<typeof getModelForTask>,
  projectId?: string | null
): Promise<{
  insights: Array<{
    insight_text: string;
    category: string;
    priority: 'low' | 'medium' | 'high' | 'critical';
    source_experts: string[];
  }>;
  summary: string;
}> {
  if (messages.length === 0) {
    return {
      insights: [],
      summary: 'No messages to analyze.',
    };
  }

  const modelConfig = modelPreferences || { provider: 'anthropic' as const, model: 'claude-sonnet-4-5-20250929' };
  const model = getModelInstance(modelConfig.provider, modelConfig.model);

  const prompt = buildInsightExtractionPrompt(messages, categories);

  const result = await trackAIOperationWithCost(
    'extract_insights',
    projectId || null,
    modelConfig.model,
    modelConfig.provider,
    (inputTokens, outputTokens) => calculateCost(modelConfig.provider, modelConfig.model, inputTokens, outputTokens),
    () =>
      generateObject({
        model,
        schema: InsightExtractionSchema,
        prompt,
        system: INSIGHT_EXTRACTION_SYSTEM_PROMPT,
        temperature: 0.3,
        maxTokens: 3000,
        experimental_telemetry: { isEnabled: true },
      })
  );

  return result.object;
}

/**
 * Select which expert should speak next
 * Simple round-robin logic based on speak count
 */
export function selectNextExpert(
  participants: ExpertPersona[],
  messages: RoundtableMessage[]
): ExpertPersona | null {
  if (participants.length === 0) {
    return null;
  }

  // Count how many times each expert has spoken
  const speakCounts = new Map<string, number>();

  for (const message of messages) {
    if (message.role === 'expert' && message.expert_id) {
      speakCounts.set(message.expert_id, (speakCounts.get(message.expert_id) || 0) + 1);
    }
  }

  // Find expert who has spoken the least
  let nextExpert = participants[0];
  let minSpeaks = speakCounts.get(nextExpert.id) || 0;

  for (const expert of participants) {
    const speaks = speakCounts.get(expert.id) || 0;
    if (speaks < minSpeaks) {
      minSpeaks = speaks;
      nextExpert = expert;
    }
  }

  return nextExpert;
}

/**
 * Check if discussion should naturally end
 * Simple heuristic: end if we have enough expert messages
 */
export function shouldEndDiscussion(messages: RoundtableMessage[], minMessages: number = 10): boolean {
  const expertMessageCount = messages.filter((m) => m.role === 'expert').length;
  return expertMessageCount >= minMessages;
}
