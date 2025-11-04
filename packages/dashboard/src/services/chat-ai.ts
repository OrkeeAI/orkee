// ABOUTME: AI-powered chat mode services using AI SDK
// ABOUTME: Handles streaming chats, insight extraction, quality metrics, and PRD generation

import { streamText, generateObject } from 'ai';
import { getModel, getPreferredModel } from '@/lib/ai/providers';
import { getModelInstance, calculateCost } from '@/lib/ai/config';
import { z } from 'zod';
import { chatService, type ChatMessage, type ChatInsight } from './chat';
import { getModelForTask } from './model-preferences';
import { trackAIOperationWithCost } from '@/lib/ai/telemetry';

/**
 * Discovery question prompts for guiding chats
 */
const DISCOVERY_PROMPTS = {
  system: `You are an expert product manager helping to discover requirements for a new project through chat.
Your goal is to ask thoughtful, probing questions that help the user articulate their vision clearly.

Guidelines:
- Ask one focused question at a time
- Build on previous answers to go deeper
- Help identify gaps in their thinking
- Be conversational and supportive
- Extract concrete requirements, constraints, and success criteria`,

  initial: `I'll help you explore and refine your project idea through chat. Let's start with understanding the core problem you're trying to solve.

What specific problem are you trying to solve with this project?`,
};

/**
 * Stream a chat AI response based on chat history
 */
export async function streamChatResponse(
  sessionId: string,
  userMessage: string,
  chatHistory: ChatMessage[],
  onChunk: (text: string) => void,
  onComplete: (fullText: string) => void,
  onError: (error: Error) => void,
  abortSignal?: AbortSignal,
  selectedProvider?: 'anthropic' | 'openai' | 'google' | 'xai',
  selectedModel?: string,
  preferences?: ReturnType<typeof getModelForTask>,
  projectId?: string | null
): Promise<void> {
  try {
    console.log('streamChatResponse called with:', { selectedProvider, selectedModel });

    // Determine model to use: explicit selection > preferences > default
    let modelToUse;
    let providerName: 'anthropic' | 'openai' | 'google' | 'xai';
    let modelName: string;

    if (selectedProvider && selectedModel) {
      console.log(`Using explicit selection: ${selectedProvider} with model: ${selectedModel}`);
      modelToUse = getModel(selectedProvider, selectedModel);
      providerName = selectedProvider;
      modelName = selectedModel;
    } else if (preferences) {
      console.log(`Using model preferences: ${preferences.provider} with model: ${preferences.model}`);
      modelToUse = getModelInstance(preferences.provider, preferences.model);
      providerName = preferences.provider;
      modelName = preferences.model;
    } else {
      console.log('No provider/model selected or preferences provided, using default model');
      const preferred = getPreferredModel();
      modelToUse = preferred.model;
      providerName = preferred.provider;
      modelName = preferred.modelName;
    }

    // Build chat context
    const messages = chatHistory.map((msg) => ({
      role: msg.role === 'user' ? 'user' as const : 'assistant' as const,
      content: msg.content,
    }));

    // Add current user message
    messages.push({
      role: 'user' as const,
      content: userMessage,
    });

    console.log('[chat-ai.streamText] About to call streamText with model:', modelToUse);
    console.log('[chat-ai.streamText] Model object type:', typeof modelToUse);
    console.log('[chat-ai.streamText] Model object:', modelToUse);

    // Wrap streamText call with telemetry tracking
    const result = await trackAIOperationWithCost(
      'chat_stream',
      projectId || null,
      modelName,
      providerName,
      (inputTokens, outputTokens) => calculateCost(providerName, modelName, inputTokens, outputTokens),
      () => streamText({
        model: modelToUse,
        system: DISCOVERY_PROMPTS.system,
        messages,
        temperature: 0.7,
        maxTokens: 1000,
        abortSignal,
      })
    );

    console.log('[chat-ai.streamText] streamText call completed, streaming response');

    const { textStream, finishReason, usage, onFinish } = result;
    let fullText = '';

    // Consume the stream
    for await (const chunk of textStream) {
      fullText += chunk;
      onChunk(chunk);
    }

    console.log('[chat-ai.streamText] Stream consumption complete, awaiting finalization');

    // CRITICAL: Wait for all async properties and manually trigger onFinish
    // The telemetry wrapper replaces onFinish with a function that sends telemetry
    const finalReason = await finishReason;
    const finalUsage = await usage;

    console.log('[chat-ai.streamText] Stream finalized:', { finalReason, finalUsage });

    // Manually call onFinish to trigger telemetry tracking
    if (onFinish) {
      await onFinish({
        finishReason: finalReason,
        usage: finalUsage,
      });
    }

    onComplete(fullText);
  } catch (error) {
    console.error('Streaming error details:', error);
    const err = error instanceof Error ? error : new Error('Streaming failed');
    onError(err);
  }
}

/**
 * Schema for insight extraction
 */
const InsightSchema = z.object({
  insights: z.array(
    z.object({
      type: z.enum(['requirement', 'constraint', 'risk', 'assumption', 'decision']),
      text: z.string(),
      confidence: z.number().min(0).max(1),
      source_messages: z.array(z.string()),
    })
  ),
});

/**
 * Extract insights from chat using AI
 */
export async function extractInsights(
  sessionId: string,
  chatHistory: ChatMessage[],
  preferences?: ReturnType<typeof getModelForTask>,
  projectId?: string | null
): Promise<ChatInsight[]> {
  // Use preferences or fall back to default
  const modelConfig = preferences || { provider: 'anthropic' as const, model: 'claude-sonnet-4-5-20250929' };
  const model = getModelInstance(modelConfig.provider, modelConfig.model);

  const chatText = chatHistory
    .map((msg) => `${msg.role}: ${msg.content}`)
    .join('\n\n');

  const prompt = `Analyze this chat and extract key insights:

${chatText}

Identify:
1. Requirements - What the user wants to build
2. Constraints - Limitations, deadlines, resources
3. Risks - Potential problems or challenges mentioned
4. Assumptions - Unstated beliefs or prerequisites
5. Decisions - Choices made during the chat

For each insight, provide the type, the insight text, and a confidence score (0-1).`;

  const result = await trackAIOperationWithCost(
    'extract_insights',
    projectId || null,
    modelConfig.model,
    modelConfig.provider,
    (inputTokens, outputTokens) => calculateCost(modelConfig.provider, modelConfig.model, inputTokens, outputTokens),
    () => generateObject({
      model,
      schema: InsightSchema,
      prompt,
      temperature: 0.3,
    })
  );

  // Convert to ChatInsight format and save to backend
  const insights: ChatInsight[] = [];

  for (const insight of result.object.insights) {
    const saved = await chatService.createInsight(sessionId, {
      insight_type: insight.type,
      insight_text: insight.text,
      confidence_score: insight.confidence,
      source_message_ids: insight.source_messages,
    });
    insights.push(saved);
  }

  return insights;
}

/**
 * Schema for quality metrics
 */
const QualityMetricsSchema = z.object({
  score: z.number().min(0).max(100),
  coverage: z.object({
    problem: z.boolean(),
    users: z.boolean(),
    features: z.boolean(),
    technical: z.boolean(),
    risks: z.boolean(),
    constraints: z.boolean(),
    success: z.boolean(),
  }),
  missing_areas: z.array(z.string()),
  readiness_assessment: z.string(),
  is_ready: z.boolean(),
});

/**
 * Calculate quality metrics using AI
 */
export async function calculateQualityMetrics(
  sessionId: string,
  chatHistory: ChatMessage[],
  insights: ChatInsight[],
  preferences?: ReturnType<typeof getModelForTask>,
  projectId?: string | null
): Promise<{
  quality_score: number;
  coverage: Record<string, boolean>;
  missing_areas: string[];
  is_ready_for_prd: boolean;
}> {
  // Use preferences or fall back to default
  const modelConfig = preferences || { provider: 'anthropic' as const, model: 'claude-sonnet-4-5-20250929' };
  const model = getModelInstance(modelConfig.provider, modelConfig.model);

  const chatText = chatHistory
    .map((msg) => `${msg.role}: ${msg.content}`)
    .join('\n\n');

  const insightsText = insights
    .map((ins) => `${ins.insight_type}: ${ins.insight_text}`)
    .join('\n');

  const prompt = `Analyze this chat and extracted insights to assess PRD readiness:

CHAT:
${chatText}

EXTRACTED INSIGHTS:
${insightsText}

Evaluate coverage of these areas:
- Problem definition: Clear understanding of what problem is being solved
- Users: Who will use this and what are their needs
- Features: What functionality is required
- Technical: Architecture, technology choices, constraints
- Risks: Identified challenges and mitigation strategies
- Constraints: Timeline, budget, resource limitations
- Success criteria: How to measure if the solution works

Provide:
1. Overall quality score (0-100)
2. Coverage for each area (true/false)
3. List of missing or weak areas
4. Assessment of whether this is ready for PRD generation
5. Boolean indicating readiness`;

  const result = await trackAIOperationWithCost(
    'calculate_quality_metrics',
    projectId || null,
    modelConfig.model,
    modelConfig.provider,
    (inputTokens, outputTokens) => calculateCost(modelConfig.provider, modelConfig.model, inputTokens, outputTokens),
    () => generateObject({
      model,
      schema: QualityMetricsSchema,
      prompt,
      temperature: 0.2,
    })
  );

  return {
    quality_score: result.object.score,
    coverage: result.object.coverage,
    missing_areas: result.object.missing_areas,
    is_ready_for_prd: result.object.is_ready,
  };
}

/**
 * Schema for PRD generation
 */
const PRDSchema = z.object({
  title: z.string(),
  overview: z.string(),
  problem_statement: z.string(),
  target_users: z.string(),
  goals: z.array(z.string()),
  features: z.array(
    z.object({
      name: z.string(),
      description: z.string(),
      priority: z.enum(['must_have', 'should_have', 'nice_to_have']),
    })
  ),
  technical_requirements: z.string(),
  constraints: z.array(z.string()),
  risks: z.array(z.string()),
  success_criteria: z.array(z.string()),
  timeline: z.string().optional(),
});

/**
 * Generate a PRD from chat using AI
 */
export async function generatePRDFromChat(
  sessionId: string,
  title: string,
  chatHistory: ChatMessage[],
  insights: ChatInsight[],
  preferences?: ReturnType<typeof getModelForTask>,
  projectId?: string | null
): Promise<{ prd_markdown: string; prd_data: z.infer<typeof PRDSchema> }> {
  // Use preferences or fall back to default
  const modelConfig = preferences || { provider: 'anthropic' as const, model: 'claude-sonnet-4-5-20250929' };
  const model = getModelInstance(modelConfig.provider, modelConfig.model);

  const chatText = chatHistory
    .map((msg) => `${msg.role}: ${msg.content}`)
    .join('\n\n');

  const insightsText = insights
    .map((ins) => `${ins.insight_type}: ${ins.insight_text}`)
    .join('\n');

  const prompt = `Generate a comprehensive Product Requirements Document based on this chat:

CHAT:
${chatText}

EXTRACTED INSIGHTS:
${insightsText}

Create a structured PRD with:
- Clear problem statement
- Target user description
- Product goals
- Feature list with priorities
- Technical requirements
- Constraints and limitations
- Risk assessment
- Success criteria
- Timeline (if discussed)

Be specific and actionable. Use insights from the chat to fill in details.`;

  const result = await trackAIOperationWithCost(
    'generate_prd_from_chat',
    projectId || null,
    modelConfig.model,
    modelConfig.provider,
    (inputTokens, outputTokens) => calculateCost(modelConfig.provider, modelConfig.model, inputTokens, outputTokens),
    () => generateObject({
      model,
      schema: PRDSchema,
      prompt,
      temperature: 0.4,
      maxTokens: 8000,
    })
  );

  const prd = result.object;

  // Convert to markdown
  const markdown = `# ${prd.title}

## Overview
${prd.overview}

## Problem Statement
${prd.problem_statement}

## Target Users
${prd.target_users}

## Goals
${prd.goals.map((g, i) => `${i + 1}. ${g}`).join('\n')}

## Features

${prd.features
  .map(
    (f) => `### ${f.name} (${f.priority})

${f.description}`
  )
  .join('\n\n')}

## Technical Requirements
${prd.technical_requirements}

## Constraints
${prd.constraints.map((c, i) => `${i + 1}. ${c}`).join('\n')}

## Risks
${prd.risks.map((r, i) => `${i + 1}. ${r}`).join('\n')}

## Success Criteria
${prd.success_criteria.map((s, i) => `${i + 1}. ${s}`).join('\n')}

${prd.timeline ? `## Timeline\n${prd.timeline}` : ''}
`;

  return {
    prd_markdown: markdown,
    prd_data: prd,
  };
}

/**
 * Create insight from extracted data
 */
