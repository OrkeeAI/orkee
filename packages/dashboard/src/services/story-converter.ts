// ABOUTME: AI-powered PRD-to-stories converter using the /ralph skill logic
// ABOUTME: Takes a PRD markdown string and produces a structured prd.json with right-sized user stories

import { generateObject } from 'ai';
import { getModelInstance, calculateCost } from '@/lib/ai/config';
import type { ModelConfig } from '@/types/models';
import { trackAIOperationWithCost } from '@/lib/ai/telemetry';
import { z } from 'zod';
import type { PrdJson, UserStory } from './agent-runs';

// ── Zod Schema ─────────────────────────────────────────────────────────────

const UserStorySchema = z.object({
  id: z.string().describe('Unique ID like US-001'),
  title: z.string().describe('Short imperative title'),
  description: z.string().describe('What the story delivers'),
  acceptanceCriteria: z.array(z.string()).describe('Testable criteria for completion'),
  epic: z.string().describe('Feature group this belongs to'),
  priority: z.number().int().describe('Execution order (1 = first)'),
  passes: z.literal(false).describe('Always false for new stories'),
  notes: z.string().describe('Empty string for new stories'),
});

const PrdJsonSchema = z.object({
  project: z.string().describe('Project name'),
  sourcePrd: z.string().describe('Source PRD filename or reference'),
  branchName: z.string().describe('Base branch name for the feature'),
  description: z.string().describe('One-line summary of the feature'),
  userStories: z.array(UserStorySchema).describe('Ordered user stories'),
});

// ── Prompt ──────────────────────────────────────────────────────────────────

const STORY_CONVERTER_PROMPT = `You are an expert at breaking down Product Requirements Documents (PRDs) into right-sized user stories for an autonomous coding agent (Ralph).

Your job is to convert a PRD markdown document into a structured prd.json that Ralph can execute story-by-story.

## Rules for Story Sizing

1. **Each story must be completable in a single coding session** (30-90 minutes of agent work)
2. **Each story must be independently testable** - acceptance criteria should be verifiable by running tests
3. **Stories must be ordered by dependency** - a story can only depend on stories with a lower priority number
4. **Acceptance criteria must be concrete and testable** - "user can see X", "API returns Y when Z", "test covers A"
5. **Each story should touch at most 3-5 files** - if it touches more, split it

## Story ID Format
Use US-001, US-002, etc. in sequential order.

## Priority
Priority 1 is executed first. Each story's priority should reflect its dependency order - foundation first, features second, polish last.

## Epic Grouping
Group related stories into epics (e.g., "Authentication", "Data Model", "API", "UI").

## Output
Generate a prd.json with all stories having passes: false and notes: "" (empty string).
The branchName should be a kebab-case slug derived from the feature name.`;

// ── Service ────────────────────────────────────────────────────────────────

const DEFAULT_MODEL_CONFIG: ModelConfig = {
  provider: 'anthropic' as const,
  model: 'claude-sonnet-4-5-20250929',
};

export async function convertPrdToStories(
  prdMarkdown: string,
  projectName: string,
  projectId: string,
  modelPreferences?: ModelConfig,
): Promise<PrdJson> {
  const { model, provider } = modelPreferences || DEFAULT_MODEL_CONFIG;

  const modelInstance = getModelInstance(provider, model);

  const result = await trackAIOperationWithCost(
    'convert_prd_to_stories',
    projectId,
    model,
    provider,
    (inputTokens: number, outputTokens: number) =>
      calculateCost(provider, model, inputTokens, outputTokens),
    () =>
      generateObject({
        model: modelInstance,
        schema: PrdJsonSchema,
        system: STORY_CONVERTER_PROMPT,
        prompt: `Convert this PRD to prd.json format for the project "${projectName}":\n\n${prdMarkdown}`,
        temperature: 0.4,
        maxTokens: 16000,
        experimental_telemetry: { isEnabled: true },
      }),
  );

  return result.object as PrdJson;
}

export type { PrdJson, UserStory };
