// ABOUTME: AI-powered conversational mode services using AI SDK
// ABOUTME: Handles streaming conversations, insight extraction, quality metrics, and PRD generation

import { streamText, generateObject } from 'ai';
import { getPreferredModel } from '@/lib/ai/providers';
import { z } from 'zod';
import { conversationalService, type ConversationMessage, type ConversationInsight } from './conversational';

/**
 * Discovery question prompts for guiding conversations
 */
const DISCOVERY_PROMPTS = {
  system: `You are an expert product manager helping to discover requirements for a new project through conversation.
Your goal is to ask thoughtful, probing questions that help the user articulate their vision clearly.

Guidelines:
- Ask one focused question at a time
- Build on previous answers to go deeper
- Help identify gaps in their thinking
- Be conversational and supportive
- Extract concrete requirements, constraints, and success criteria`,

  initial: `I'll help you explore and refine your project idea through conversation. Let's start with understanding the core problem you're trying to solve.

What specific problem are you trying to solve with this project?`,
};

/**
 * Stream a conversational AI response based on conversation history
 */
export async function streamConversationalResponse(
  sessionId: string,
  userMessage: string,
  conversationHistory: ConversationMessage[],
  onChunk: (text: string) => void,
  onComplete: (fullText: string) => void,
  onError: (error: Error) => void
): Promise<void> {
  try {
    const { model } = getPreferredModel();

    // Build conversation context
    const messages = conversationHistory.map((msg) => ({
      role: msg.role === 'user' ? 'user' as const : 'assistant' as const,
      content: msg.content,
    }));

    // Add current user message
    messages.push({
      role: 'user' as const,
      content: userMessage,
    });

    const { textStream } = await streamText({
      model,
      system: DISCOVERY_PROMPTS.system,
      messages,
      temperature: 0.7,
      maxTokens: 1000,
    });

    let fullText = '';

    for await (const chunk of textStream) {
      fullText += chunk;
      onChunk(chunk);
    }

    onComplete(fullText);
  } catch (error) {
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
 * Extract insights from conversation using AI
 */
export async function extractInsights(
  sessionId: string,
  conversationHistory: ConversationMessage[]
): Promise<ConversationInsight[]> {
  const { model } = getPreferredModel();

  const conversationText = conversationHistory
    .map((msg) => `${msg.role}: ${msg.content}`)
    .join('\n\n');

  const prompt = `Analyze this conversation and extract key insights:

${conversationText}

Identify:
1. Requirements - What the user wants to build
2. Constraints - Limitations, deadlines, resources
3. Risks - Potential problems or challenges mentioned
4. Assumptions - Unstated beliefs or prerequisites
5. Decisions - Choices made during the conversation

For each insight, provide the type, the insight text, and a confidence score (0-1).`;

  const result = await generateObject({
    model,
    schema: InsightSchema,
    prompt,
    temperature: 0.3,
  });

  // Convert to ConversationInsight format and save to backend
  const insights: ConversationInsight[] = [];

  for (const insight of result.object.insights) {
    const saved = await conversationalService.createInsight(sessionId, {
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
  conversationHistory: ConversationMessage[],
  insights: ConversationInsight[]
): Promise<{
  quality_score: number;
  coverage: Record<string, boolean>;
  missing_areas: string[];
  is_ready_for_prd: boolean;
}> {
  const { model } = getPreferredModel();

  const conversationText = conversationHistory
    .map((msg) => `${msg.role}: ${msg.content}`)
    .join('\n\n');

  const insightsText = insights
    .map((ins) => `${ins.insight_type}: ${ins.insight_text}`)
    .join('\n');

  const prompt = `Analyze this conversation and extracted insights to assess PRD readiness:

CONVERSATION:
${conversationText}

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

  const result = await generateObject({
    model,
    schema: QualityMetricsSchema,
    prompt,
    temperature: 0.2,
  });

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
 * Generate a PRD from conversation using AI
 */
export async function generatePRDFromConversation(
  sessionId: string,
  title: string,
  conversationHistory: ConversationMessage[],
  insights: ConversationInsight[]
): Promise<{ prd_markdown: string; prd_data: z.infer<typeof PRDSchema> }> {
  const { model } = getPreferredModel();

  const conversationText = conversationHistory
    .map((msg) => `${msg.role}: ${msg.content}`)
    .join('\n\n');

  const insightsText = insights
    .map((ins) => `${ins.insight_type}: ${ins.insight_text}`)
    .join('\n');

  const prompt = `Generate a comprehensive Product Requirements Document based on this conversation:

CONVERSATION:
${conversationText}

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

Be specific and actionable. Use insights from the conversation to fill in details.`;

  const result = await generateObject({
    model,
    schema: PRDSchema,
    prompt,
    temperature: 0.4,
    maxTokens: 8000,
  });

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
