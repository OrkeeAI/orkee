// ABOUTME: AI service layer for OpenSpec spec generation and analysis
// ABOUTME: Handles PRD analysis, task generation, spec suggestions, and validation

import { generateObject, generateText } from 'ai';
import { getPreferredModel } from './providers';
import { AI_CONFIG, calculateCost } from './config';
import {
  PRDAnalysisSchema,
  type PRDAnalysis,
  SpecCapabilitySchema,
  type SpecCapability,
  TaskSuggestionSchema,
  type TaskSuggestion,
  OrphanTaskAnalysisSchema,
  type OrphanTaskAnalysis,
  TaskValidationSchema,
  type TaskValidation,
  SpecRefinementSchema,
  type SpecRefinement,
  type CostEstimate,
} from './schemas';

/**
 * Result type including cost information
 */
export interface AIResult<T> {
  data: T;
  usage: {
    inputTokens: number;
    outputTokens: number;
    totalTokens: number;
  };
  cost: CostEstimate;
  model: string;
  provider: 'openai' | 'anthropic';
}

/**
 * AI service for OpenSpec operations
 */
export class AISpecService {
  /**
   * Analyze a PRD and extract capabilities, requirements, and scenarios
   */
  async analyzePRD(prdContent: string): Promise<AIResult<PRDAnalysis>> {
    const { provider, model, modelName } = getPreferredModel();

    const prompt = `You are an expert software architect analyzing a Product Requirements Document (PRD).

Your task is to:
1. Extract high-level capabilities (functional areas) from the PRD
2. For each capability, define specific requirements
3. For each requirement, create WHEN/THEN/AND scenarios
4. Suggest 5-10 actionable tasks to implement the capabilities
5. Identify dependencies and technical considerations

PRD Content:
${prdContent}

Important guidelines:
- Capability IDs must be kebab-case (e.g., "user-auth", "data-sync")
- Each requirement must have at least one scenario
- Scenarios must follow WHEN/THEN/AND structure
- Tasks should be specific, actionable, and include complexity scores (1-10)
- Tasks should reference the capability and requirement they implement
- Be specific and actionable
- Focus on testable behaviors`;

    const result = await generateObject({
      model,
      schema: PRDAnalysisSchema,
      prompt,
      temperature: AI_CONFIG.defaults.temperature,
      maxTokens: AI_CONFIG.defaults.maxTokens,
    });

    const usage = {
      inputTokens: result.usage?.promptTokens || 0,
      outputTokens: result.usage?.completionTokens || 0,
      totalTokens: result.usage?.totalTokens || 0,
    };

    const cost: CostEstimate = {
      inputTokens: usage.inputTokens,
      outputTokens: usage.outputTokens,
      totalTokens: usage.totalTokens,
      estimatedCost: calculateCost(provider, modelName, usage.inputTokens, usage.outputTokens),
      model: modelName,
      provider,
    };

    return {
      data: result.object,
      usage,
      cost,
      model: modelName,
      provider,
    };
  }

  /**
   * Generate a spec capability from a description
   */
  async generateSpec(
    capabilityName: string,
    purpose: string,
    requirements?: string[]
  ): Promise<AIResult<SpecCapability>> {
    const { provider, model, modelName } = getPreferredModel();

    let prompt = `You are an expert software architect creating an OpenSpec specification.

Create a detailed spec for the "${capabilityName}" capability.

Purpose: ${purpose}`;

    if (requirements && requirements.length > 0) {
      prompt += `\n\nRequirements to address:\n${requirements.map((r, i) => `${i + 1}. ${r}`).join('\n')}`;
    }

    prompt += `\n\nCreate a comprehensive spec with:
1. A kebab-case capability ID
2. Clear, testable requirements
3. WHEN/THEN/AND scenarios for each requirement
4. Specific, actionable specifications`;

    const result = await generateObject({
      model,
      schema: SpecCapabilitySchema,
      prompt,
      temperature: AI_CONFIG.defaults.temperature,
      maxTokens: AI_CONFIG.defaults.maxTokens,
    });

    const usage = {
      inputTokens: result.usage?.promptTokens || 0,
      outputTokens: result.usage?.completionTokens || 0,
      totalTokens: result.usage?.totalTokens || 0,
    };

    const cost: CostEstimate = {
      inputTokens: usage.inputTokens,
      outputTokens: usage.outputTokens,
      totalTokens: usage.totalTokens,
      estimatedCost: calculateCost(provider, modelName, usage.inputTokens, usage.outputTokens),
      model: modelName,
      provider,
    };

    return {
      data: result.object,
      usage,
      cost,
      model: modelName,
      provider,
    };
  }

  /**
   * Generate task suggestions from a spec capability
   */
  async suggestTasks(
    capability: SpecCapability,
    existingTasks?: string[]
  ): Promise<AIResult<TaskSuggestion[]>> {
    const { provider, model, modelName } = getPreferredModel();

    let prompt = `You are an expert project manager breaking down specifications into actionable tasks.

Capability: ${capability.name}
Purpose: ${capability.purpose}

Requirements:
${capability.requirements.map((r, i) => `${i + 1}. ${r.name}: ${r.content}`).join('\n')}`;

    if (existingTasks && existingTasks.length > 0) {
      prompt += `\n\nExisting tasks (avoid duplicating these):\n${existingTasks.join('\n')}`;
    }

    prompt += `\n\nGenerate 5-10 specific, actionable tasks to implement this capability. Each task should:
1. Have a clear, concise title
2. Include implementation details
3. Reference the related requirement
4. Have a realistic complexity score (1-10)
5. Include priority level`;

    const result = await generateObject({
      model,
      schema: TaskSuggestionSchema.array(),
      prompt,
      temperature: AI_CONFIG.defaults.temperature,
      maxTokens: AI_CONFIG.defaults.maxTokens,
    });

    const usage = {
      inputTokens: result.usage?.promptTokens || 0,
      outputTokens: result.usage?.completionTokens || 0,
      totalTokens: result.usage?.totalTokens || 0,
    };

    const cost: CostEstimate = {
      inputTokens: usage.inputTokens,
      outputTokens: usage.outputTokens,
      totalTokens: usage.totalTokens,
      estimatedCost: calculateCost(provider, modelName, usage.inputTokens, usage.outputTokens),
      model: modelName,
      provider,
    };

    return {
      data: result.object,
      usage,
      cost,
      model: modelName,
      provider,
    };
  }

  /**
   * Analyze an orphan task and suggest where it fits in the spec
   */
  async analyzeOrphanTask(
    task: { title: string; description: string },
    existingCapabilities: Array<{ id: string; name: string; purpose: string }>
  ): Promise<AIResult<OrphanTaskAnalysis>> {
    const { provider, model, modelName } = getPreferredModel();

    const prompt = `You are an expert software architect organizing tasks into specifications.

Task to analyze:
Title: ${task.title}
Description: ${task.description}

Existing capabilities:
${existingCapabilities.map((c, i) => `${i + 1}. ${c.name} (${c.id}): ${c.purpose}`).join('\n')}

Determine:
1. Whether this task fits into an existing capability or needs a new one
2. What requirement this task addresses
3. What scenarios this task should satisfy

Provide a high-confidence suggestion with clear rationale.`;

    const result = await generateObject({
      model,
      schema: OrphanTaskAnalysisSchema,
      prompt,
      temperature: AI_CONFIG.defaults.temperature,
      maxTokens: AI_CONFIG.defaults.maxTokens,
    });

    const usage = {
      inputTokens: result.usage?.promptTokens || 0,
      outputTokens: result.usage?.completionTokens || 0,
      totalTokens: result.usage?.totalTokens || 0,
    };

    const cost: CostEstimate = {
      inputTokens: usage.inputTokens,
      outputTokens: usage.outputTokens,
      totalTokens: usage.totalTokens,
      estimatedCost: calculateCost(provider, modelName, usage.inputTokens, usage.outputTokens),
      model: modelName,
      provider,
    };

    return {
      data: result.object,
      usage,
      cost,
      model: modelName,
      provider,
    };
  }

  /**
   * Validate a task implementation against spec scenarios
   */
  async validateTaskCompletion(
    task: { title: string; description: string; implementation?: string },
    scenarios: Array<{ name: string; when: string; then: string; and?: string[] }>
  ): Promise<AIResult<TaskValidation>> {
    const { provider, model, modelName } = getPreferredModel();

    const prompt = `You are an expert QA engineer validating task implementation against specifications.

Task:
Title: ${task.title}
Description: ${task.description}
${task.implementation ? `Implementation details: ${task.implementation}` : ''}

Scenarios to validate:
${scenarios
  .map(
    (s, i) =>
      `${i + 1}. ${s.name}
   WHEN ${s.when}
   THEN ${s.then}
   ${s.and ? s.and.map((a) => `AND ${a}`).join('\n   ') : ''}`
  )
  .join('\n\n')}

For each scenario, determine:
1. Whether it passed (true/false)
2. Confidence level (0-1)
3. Any notes or concerns

Provide an overall assessment and recommendations.`;

    const result = await generateObject({
      model,
      schema: TaskValidationSchema,
      prompt,
      temperature: AI_CONFIG.defaults.temperature,
      maxTokens: AI_CONFIG.defaults.maxTokens,
    });

    const usage = {
      inputTokens: result.usage?.promptTokens || 0,
      outputTokens: result.usage?.completionTokens || 0,
      totalTokens: result.usage?.totalTokens || 0,
    };

    const cost: CostEstimate = {
      inputTokens: usage.inputTokens,
      outputTokens: usage.outputTokens,
      totalTokens: usage.totalTokens,
      estimatedCost: calculateCost(provider, modelName, usage.inputTokens, usage.outputTokens),
      model: modelName,
      provider,
    };

    return {
      data: result.object,
      usage,
      cost,
      model: modelName,
      provider,
    };
  }

  /**
   * Refine a spec based on feedback
   */
  async refineSpec(
    capability: SpecCapability,
    feedback: string
  ): Promise<AIResult<SpecRefinement>> {
    const { provider, model, modelName } = getPreferredModel();

    const prompt = `You are an expert software architect refining specifications based on feedback.

Current Capability: ${capability.name}
Purpose: ${capability.purpose}

Current Requirements:
${capability.requirements.map((r, i) => `${i + 1}. ${r.name}: ${r.content}`).join('\n')}

Feedback: ${feedback}

Suggest specific improvements:
1. What to add
2. What to modify
3. What to remove
4. What to clarify

Prioritize suggestions and explain the rationale.`;

    const result = await generateObject({
      model,
      schema: SpecRefinementSchema,
      prompt,
      temperature: AI_CONFIG.defaults.temperature,
      maxTokens: AI_CONFIG.defaults.maxTokens,
    });

    const usage = {
      inputTokens: result.usage?.promptTokens || 0,
      outputTokens: result.usage?.completionTokens || 0,
      totalTokens: result.usage?.totalTokens || 0,
    };

    const cost: CostEstimate = {
      inputTokens: usage.inputTokens,
      outputTokens: usage.outputTokens,
      totalTokens: usage.totalTokens,
      estimatedCost: calculateCost(provider, modelName, usage.inputTokens, usage.outputTokens),
      model: modelName,
      provider,
    };

    return {
      data: result.object,
      usage,
      cost,
      model: modelName,
      provider,
    };
  }

  /**
   * Generate markdown spec from capability
   */
  async generateSpecMarkdown(capability: SpecCapability): Promise<AIResult<string>> {
    const { provider, model, modelName } = getPreferredModel();

    const prompt = `You are an expert technical writer creating OpenSpec documentation.

Capability: ${capability.name}
Purpose: ${capability.purpose}

Requirements:
${capability.requirements
  .map(
    (r, i) =>
      `${i + 1}. ${r.name}
   ${r.content}

   Scenarios:
   ${r.scenarios
     .map(
       (s) =>
         `   - ${s.name}
     WHEN ${s.when}
     THEN ${s.then}
     ${s.and ? s.and.map((a) => `AND ${a}`).join('\n     ') : ''}`
     )
     .join('\n   ')}`
  )
  .join('\n\n')}

Generate a well-formatted OpenSpec markdown document with:
1. Clear hierarchy and structure
2. Proper markdown formatting
3. WHEN/THEN/AND scenarios formatted correctly
4. Professional technical writing

Return ONLY the markdown content, no explanations.`;

    const result = await generateText({
      model,
      prompt,
      temperature: AI_CONFIG.defaults.temperature,
      maxTokens: AI_CONFIG.defaults.maxTokens,
    });

    const usage = {
      inputTokens: result.usage?.promptTokens || 0,
      outputTokens: result.usage?.completionTokens || 0,
      totalTokens: result.usage?.totalTokens || 0,
    };

    const cost: CostEstimate = {
      inputTokens: usage.inputTokens,
      outputTokens: usage.outputTokens,
      totalTokens: usage.totalTokens,
      estimatedCost: calculateCost(provider, modelName, usage.inputTokens, usage.outputTokens),
      model: modelName,
      provider,
    };

    return {
      data: result.text,
      usage,
      cost,
      model: modelName,
      provider,
    };
  }
}

/**
 * Singleton instance of the AI service
 */
export const aiSpecService = new AISpecService();
