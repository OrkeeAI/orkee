// ABOUTME: AI service layer for spec operations (analysis, generation, refinement, validation)
// ABOUTME: TypeScript equivalents of Rust ai_handlers.rs functions with model preferences support

import { generateObject } from 'ai';
import { getModelInstance, calculateCost } from '@/lib/ai/config';
import type { ModelConfig } from '@/types/models';
import {
  PRDAnalysisSchema,
  type PRDAnalysis,
  SpecScenarioSchema,
} from '@/lib/ai/schemas';
import { z } from 'zod';

// Result type including cost and usage
export interface AIResult<T> {
  data: T;
  usage: {
    inputTokens: number;
    outputTokens: number;
    totalTokens: number;
  };
  cost: number;
  model: string;
  provider: string;
}

/**
 * Analyze a PRD and extract structured information (capabilities, tasks, dependencies)
 *
 * Replicates Rust handler: packages/api/src/ai_handlers.rs:162-458
 *
 * @param prdContent - Markdown content of the PRD
 * @param modelConfig - Optional model configuration (provider + model)
 * @returns AI analysis with capabilities, suggested tasks, and dependencies
 */
export async function analyzePRD(
  prdContent: string,
  modelConfig?: ModelConfig
): Promise<AIResult<PRDAnalysis>> {
  // Use provided model or default
  const config = modelConfig || {
    provider: 'anthropic' as const,
    model: 'claude-sonnet-4-5-20250929',
  };

  const model = getModelInstance(config.provider, config.model);

  // System prompt from Rust (lines 268-305)
  const systemPrompt = `You are an expert software architect creating change proposals from PRDs.

CRITICAL FORMAT REQUIREMENTS:
1. Every requirement MUST use: ### Requirement: [Name]
2. Every scenario MUST use: #### Scenario: [Name] (exactly 4 hashtags)
3. Scenarios MUST follow this bullet format:
   - **WHEN** [condition]
   - **THEN** [outcome]
   - **AND** [additional] (optional)
4. Requirements MUST use SHALL or MUST (never should/may)
5. Every requirement MUST have at least one scenario

Generate:
1. Executive summary for proposal
2. Capability specifications using:
   ## ADDED Requirements
   [requirements with proper format]
3. Implementation tasks (specific and actionable)
4. Technical considerations (if complex)

Example format:
## ADDED Requirements

### Requirement: User Authentication
Users SHALL be able to authenticate using email and password.

#### Scenario: Successful login
- **WHEN** user enters valid credentials
- **THEN** user is logged in and redirected to dashboard
- **AND** session token is created
- **AND** last login time is updated

Rules:
- Use kebab-case for capability IDs (e.g., "user-auth")
- Complexity scores: 1-10 (1=trivial, 10=very complex)
- Priority: low, medium, or high
- Be specific and testable

RESPOND WITH ONLY VALID JSON.`;

  // User prompt from Rust (lines 221-265)
  const userPrompt = `Analyze the following Product Requirements Document (PRD) and extract structured information.

PRD Content:
${prdContent}

Respond with ONLY valid JSON matching this exact structure (no markdown, no code blocks):
{
  "summary": "Executive summary of the PRD",
  "capabilities": [
    {
      "id": "kebab-case-id",
      "name": "Human Readable Name",
      "purpose": "Purpose and context",
      "requirements": [
        {
          "name": "Requirement Name",
          "content": "Detailed requirement description",
          "scenarios": [
            {
              "name": "Scenario name",
              "when": "WHEN condition",
              "then": "THEN outcome",
              "and": ["AND condition 1", "AND condition 2"]
            }
          ]
        }
      ]
    }
  ],
  "suggestedTasks": [
    {
      "title": "Task title",
      "description": "Detailed description",
      "capabilityId": "capability-id",
      "requirementName": "Requirement Name",
      "complexity": 5,
      "estimatedHours": 8,
      "priority": "medium"
    }
  ],
  "dependencies": ["External dependency 1"],
  "technicalConsiderations": ["Technical consideration 1"]
}`;

  // Generate structured output using AI SDK
  const startTime = Date.now();
  const { object, usage } = await generateObject({
    model,
    schema: PRDAnalysisSchema,
    system: systemPrompt,
    prompt: userPrompt,
  });
  const durationMs = Date.now() - startTime;

  // Calculate cost
  const cost = calculateCost(
    config.provider,
    config.model,
    usage.promptTokens,
    usage.completionTokens
  );

  console.log(`[ai-spec.analyzePRD] Analysis complete in ${durationMs}ms`);
  console.log(`[ai-spec.analyzePRD] Usage: ${usage.promptTokens} input + ${usage.completionTokens} output = ${usage.totalTokens} total tokens`);
  console.log(`[ai-spec.analyzePRD] Cost: $${cost.toFixed(4)}`);

  return {
    data: object,
    usage: {
      inputTokens: usage.promptTokens,
      outputTokens: usage.completionTokens,
      totalTokens: usage.totalTokens,
    },
    cost,
    model: config.model,
    provider: config.provider,
  };
}

// Schema for spec generation response
const SpecGenerationSchema = z.object({
  requirements: z.array(
    z.object({
      name: z.string(),
      description: z.string(),
      scenarios: z.array(SpecScenarioSchema),
    })
  ),
});

export type SpecGenerationData = z.infer<typeof SpecGenerationSchema>;

/**
 * Generate a detailed specification for a capability
 *
 * Replicates Rust handler: packages/api/src/ai_handlers.rs:473-655
 *
 * @param capabilityName - Name of the capability
 * @param purpose - Optional purpose description
 * @param requirements - List of requirements to address
 * @param modelConfig - Optional model configuration
 * @returns Generated spec with requirements and scenarios
 */
export async function generateSpec(
  capabilityName: string,
  purpose: string | undefined,
  requirements: string[],
  modelConfig?: ModelConfig
): Promise<AIResult<SpecGenerationData>> {
  const config = modelConfig || {
    provider: 'anthropic' as const,
    model: 'claude-sonnet-4-5-20250929',
  };

  const model = getModelInstance(config.provider, config.model);

  // System prompt from Rust (lines 532-550)
  const systemPrompt = `You are an expert software architect creating detailed specifications.

Your task is to:
1. Create specific, testable requirements for the capability
2. For each requirement, define WHEN/THEN/AND scenarios
3. Make scenarios concrete and actionable
4. Ensure all scenarios follow the Given-When-Then pattern
5. Include both happy path and error scenarios

Important guidelines:
- Each requirement must have at least 2 scenarios
- Scenarios must be specific and testable
- Use clear, precise language
- Focus on user-facing behavior
- Include edge cases and error handling

Respond with ONLY valid JSON. Do not include markdown formatting, code blocks, or any other text.`;

  // User prompt from Rust (lines 498-532)
  const requirementsList = requirements.map((r, i) => `${i + 1}. ${r}`).join('\n');
  const userPrompt = `Generate a detailed specification for the following capability.

Capability: ${capabilityName}
${purpose ? `Purpose: ${purpose}` : ''}

Requirements to address:
${requirementsList}

Respond with ONLY valid JSON matching this exact structure (no markdown, no code blocks):
{
  "requirements": [
    {
      "name": "Requirement Name",
      "description": "Detailed description of what this requirement addresses",
      "scenarios": [
        {
          "name": "Scenario name",
          "when": "WHEN condition",
          "then": "THEN expected outcome",
          "and": ["AND additional condition 1", "AND additional condition 2"]
        }
      ]
    }
  ]
}`;

  const startTime = Date.now();
  const { object, usage } = await generateObject({
    model,
    schema: SpecGenerationSchema,
    system: systemPrompt,
    prompt: userPrompt,
  });
  const durationMs = Date.now() - startTime;

  const cost = calculateCost(
    config.provider,
    config.model,
    usage.promptTokens,
    usage.completionTokens
  );

  console.log(`[ai-spec.generateSpec] Generation complete in ${durationMs}ms`);
  console.log(`[ai-spec.generateSpec] Generated ${object.requirements.length} requirements`);

  return {
    data: object,
    usage: {
      inputTokens: usage.promptTokens,
      outputTokens: usage.completionTokens,
      totalTokens: usage.totalTokens,
    },
    cost,
    model: config.model,
    provider: config.provider,
  };
}

// Schema for task suggestions response
const TaskSuggestionsSchema = z.object({
  tasks: z.array(
    z.object({
      title: z.string(),
      description: z.string(),
      priority: z.string(),
      complexity_score: z.number(),
      linked_requirements: z.array(z.string()),
    })
  ),
});

export type TaskSuggestionsData = z.infer<typeof TaskSuggestionsSchema>;

/**
 * Suggest actionable development tasks from a specification
 *
 * Replicates Rust handler: packages/api/src/ai_handlers.rs:681-838
 *
 * @param capabilityId - Capability identifier
 * @param specMarkdown - Specification markdown content
 * @param modelConfig - Optional model configuration
 * @returns Suggested tasks with priorities and complexity scores
 */
export async function suggestTasks(
  capabilityId: string,
  specMarkdown: string,
  modelConfig?: ModelConfig
): Promise<AIResult<TaskSuggestionsData>> {
  const config = modelConfig || {
    provider: 'anthropic' as const,
    model: 'claude-sonnet-4-5-20250929',
  };

  const model = getModelInstance(config.provider, config.model);

  // System prompt from Rust (lines 722-749)
  const systemPrompt = `You are an expert software development project manager breaking down specifications into actionable tasks.

Your task is to:
1. Read the specification carefully
2. Identify all requirements and scenarios
3. Create specific, actionable tasks for developers
4. Ensure tasks cover implementation, testing, and documentation
5. Assign realistic complexity scores based on:
   - 1-3: Simple changes, configuration
   - 4-6: Medium features, moderate complexity
   - 7-9: Complex features, significant work
   - 10: Very complex, architectural changes
6. Set priorities based on:
   - high: Core functionality, critical path
   - medium: Important but not blocking
   - low: Nice-to-have, can be deferred
7. Link tasks to specific requirements they address

Important guidelines:
- Tasks should be granular enough to be assigned to individual developers
- Each task should be completable in 1-3 days
- Include both happy path and error handling tasks
- Don't forget testing and documentation tasks
- Use requirement IDs that match the spec (e.g., "req-1", "req-2")

Respond with ONLY valid JSON. Do not include markdown formatting, code blocks, or any other text.`;

  // User prompt from Rust (lines 693-720)
  const userPrompt = `Analyze the following specification and generate actionable development tasks.

Specification:
${specMarkdown}

Your tasks should:
1. Cover all requirements and scenarios in the spec
2. Be specific and actionable for developers
3. Include both implementation tasks and testing tasks
4. Have realistic complexity scores (1-10, where 10 is most complex)
5. Reference specific requirement IDs from the spec
6. Use priorities: "high", "medium", or "low"

Respond with ONLY valid JSON matching this exact structure (no markdown, no code blocks):
{
  "tasks": [
    {
      "title": "Task title",
      "description": "Detailed description of what needs to be done",
      "priority": "high",
      "complexity_score": 7,
      "linked_requirements": ["req-1", "req-2"]
    }
  ]
}`;

  const startTime = Date.now();
  const { object, usage } = await generateObject({
    model,
    schema: TaskSuggestionsSchema,
    system: systemPrompt,
    prompt: userPrompt,
  });
  const durationMs = Date.now() - startTime;

  const cost = calculateCost(
    config.provider,
    config.model,
    usage.promptTokens,
    usage.completionTokens
  );

  console.log(`[ai-spec.suggestTasks] Generated ${object.tasks.length} tasks in ${durationMs}ms`);

  return {
    data: object,
    usage: {
      inputTokens: usage.promptTokens,
      outputTokens: usage.completionTokens,
      totalTokens: usage.totalTokens,
    },
    cost,
    model: config.model,
    provider: config.provider,
  };
}

// Schema for spec refinement response
const SpecRefinementSchema = z.object({
  refined_spec: z.string(),
  changes_made: z.array(z.string()),
});

export type SpecRefinementData = z.infer<typeof SpecRefinementSchema>;

/**
 * Refine a specification based on user feedback
 *
 * Replicates Rust handler: packages/api/src/ai_handlers.rs:871-1002
 *
 * @param capabilityId - Capability identifier
 * @param currentSpecMarkdown - Current specification markdown
 * @param feedback - User feedback for refinement
 * @param modelConfig - Optional model configuration
 * @returns Refined spec with tracked changes
 */
export async function refineSpec(
  capabilityId: string,
  currentSpecMarkdown: string,
  feedback: string,
  modelConfig?: ModelConfig
): Promise<AIResult<SpecRefinementData>> {
  const config = modelConfig || {
    provider: 'anthropic' as const,
    model: 'claude-sonnet-4-5-20250929',
  };

  const model = getModelInstance(config.provider, config.model);

  // System prompt from Rust (lines 907-926)
  const systemPrompt = `You are an expert technical writer refining software specifications based on feedback.

Your task is to:
1. Carefully read the current specification
2. Understand the user's feedback and concerns
3. Update the specification to address the feedback while maintaining quality
4. Preserve the spec's structure, clarity, and testability
5. Track all changes made for transparency

Important guidelines:
- Keep the WHEN/THEN/AND scenario format
- Maintain markdown formatting
- Don't remove content unless explicitly requested
- Add clarifications, not just rephrasing
- Ensure all changes directly address the feedback
- List specific changes made (e.g., "Added error handling scenario for X", "Clarified requirement Y")

Respond with ONLY valid JSON. Do not include markdown formatting, code blocks, or any other text.`;

  // User prompt from Rust (lines 883-905)
  const userPrompt = `Refine the following specification based on user feedback.

Current Specification:
${currentSpecMarkdown}

User Feedback:
${feedback}

Your task is to:
1. Analyze the current specification and the feedback
2. Update the specification to address all feedback points
3. Keep the same markdown structure and format
4. Preserve existing content that isn't affected by the feedback
5. Track what changes you made

Respond with ONLY valid JSON matching this exact structure (no markdown, no code blocks):
{
  "refined_spec": "The complete refined specification in markdown format",
  "changes_made": ["Description of change 1", "Description of change 2"]
}`;

  const startTime = Date.now();
  const { object, usage } = await generateObject({
    model,
    schema: SpecRefinementSchema,
    system: systemPrompt,
    prompt: userPrompt,
  });
  const durationMs = Date.now() - startTime;

  const cost = calculateCost(
    config.provider,
    config.model,
    usage.promptTokens,
    usage.completionTokens
  );

  console.log(`[ai-spec.refineSpec] Refinement complete in ${durationMs}ms`);
  console.log(`[ai-spec.refineSpec] Made ${object.changes_made.length} changes`);

  return {
    data: object,
    usage: {
      inputTokens: usage.promptTokens,
      outputTokens: usage.completionTokens,
      totalTokens: usage.totalTokens,
    },
    cost,
    model: config.model,
    provider: config.provider,
  };
}

// Schema for completion validation response
const CompletionValidationSchema = z.object({
  is_complete: z.boolean(),
  validation_results: z.array(
    z.object({
      scenario: z.string(),
      passed: z.boolean(),
      confidence: z.number(),
      notes: z.string().nullable(),
    })
  ),
  overall_confidence: z.number(),
  recommendations: z.array(z.string()),
});

export type CompletionValidationData = z.infer<typeof CompletionValidationSchema>;

/**
 * Validate task completion against specification scenarios
 *
 * Replicates Rust handler: packages/api/src/ai_handlers.rs:1027-1198
 *
 * @param taskId - Task identifier
 * @param implementationDetails - Description of implementation
 * @param linkedScenarios - Scenarios to validate against
 * @param modelConfig - Optional model configuration
 * @returns Validation results with confidence scores
 */
export async function validateCompletion(
  taskId: string,
  implementationDetails: string,
  linkedScenarios: string[],
  modelConfig?: ModelConfig
): Promise<AIResult<CompletionValidationData>> {
  const config = modelConfig || {
    provider: 'anthropic' as const,
    model: 'claude-sonnet-4-5-20250929',
  };

  const model = getModelInstance(config.provider, config.model);

  // System prompt from Rust (lines 1082-1107)
  const systemPrompt = `You are an expert QA engineer validating task completion against specifications.

Your task is to:
1. Carefully read the implementation details
2. Compare against each specified scenario
3. Determine if the scenario is fully implemented
4. Assign confidence scores (0.0 to 1.0):
   - 1.0: Perfectly matches, no doubt
   - 0.9-0.99: Excellent match, minor uncertainties
   - 0.8-0.89: Good match, some assumptions made
   - 0.7-0.79: Adequate match, several assumptions
   - Below 0.7: Insufficient evidence or concerns
5. Provide specific notes for failed or uncertain scenarios
6. Give actionable recommendations for improvement

Important guidelines:
- Be thorough but fair in assessment
- If implementation is incomplete, set passed: false
- Consider edge cases and error handling
- Recommendations should be specific and actionable
- Overall confidence should reflect the weakest link
- If no scenarios provided, evaluate based on implementation quality

Respond with ONLY valid JSON. Do not include markdown formatting, code blocks, or any other text.`;

  // User prompt from Rust (lines 1049-1080)
  const scenariosList = linkedScenarios.map((s, i) => `${i + 1}. ${s}`).join('\n');
  const userPrompt = `Validate whether the following implementation completes the specified scenarios.

Implementation Details:
${implementationDetails}

Scenarios to Validate:
${scenariosList}

Your task is to:
1. Analyze the implementation details
2. Check if each scenario is adequately addressed
3. Assess confidence in completion for each scenario
4. Provide an overall assessment
5. Suggest improvements if needed

Respond with ONLY valid JSON matching this exact structure (no markdown, no code blocks):
{
  "is_complete": true,
  "validation_results": [
    {
      "scenario": "Scenario description",
      "passed": true,
      "confidence": 0.95,
      "notes": "Optional notes about this validation"
    }
  ],
  "overall_confidence": 0.93,
  "recommendations": ["Recommendation 1", "Recommendation 2"]
}`;

  const startTime = Date.now();
  const { object, usage } = await generateObject({
    model,
    schema: CompletionValidationSchema,
    system: systemPrompt,
    prompt: userPrompt,
  });
  const durationMs = Date.now() - startTime;

  const cost = calculateCost(
    config.provider,
    config.model,
    usage.promptTokens,
    usage.completionTokens
  );

  console.log(`[ai-spec.validateCompletion] Validation complete in ${durationMs}ms`);
  console.log(
    `[ai-spec.validateCompletion] Result: ${object.is_complete ? 'COMPLETE' : 'INCOMPLETE'} (confidence: ${object.overall_confidence})`
  );

  return {
    data: object,
    usage: {
      inputTokens: usage.promptTokens,
      outputTokens: usage.completionTokens,
      totalTokens: usage.totalTokens,
    },
    cost,
    model: config.model,
    provider: config.provider,
  };
}
