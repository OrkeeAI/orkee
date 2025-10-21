// ABOUTME: AI service layer for OpenSpec spec generation and analysis
// ABOUTME: Handles PRD analysis, task generation, spec suggestions, and validation

import { generateObject, generateText } from 'ai';
import { getPreferredModel } from './providers';
import { AI_CONFIG, calculateCost } from './config';
import { aiRateLimiter } from './rate-limiter';
import { aiCache } from './cache';
import {
  RateLimitError,
  TimeoutError,
  NetworkError,
  ValidationError,
  AIServiceError,
} from './errors';
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
import {
  validateContentSize,
  chunkText,
  createChunkPrompt,
  withTimeout,
  mergePRDAnalyses,
} from './utils';

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
   * Automatically handles large PRDs via chunking if needed
   */
  async analyzePRD(prdContent: string): Promise<AIResult<PRDAnalysis>> {
    try {
      // Check cache first
      const cachedResult = aiCache.get<AIResult<PRDAnalysis>>('analyzePRD', { prdContent });
      if (cachedResult) {
        return cachedResult;
      }

      // Check rate limits
      const rateLimitCheck = aiRateLimiter.canMakeCall('analyzePRD');
      if (!rateLimitCheck.allowed) {
        throw new RateLimitError(rateLimitCheck.reason!);
      }

      return await this._analyzePRDImpl(prdContent);
    } catch (error) {
      if (error instanceof RateLimitError) {
        throw error;
      }
      if (error instanceof TimeoutError) {
        throw new AIServiceError('PRD analysis timed out', 'analyzePRD', error, true);
      }
      if (error instanceof NetworkError) {
        throw new AIServiceError('Network error during PRD analysis', 'analyzePRD', error, error.retryable);
      }
      if (error instanceof ValidationError) {
        throw new AIServiceError('Invalid PRD analysis response', 'analyzePRD', error, false);
      }
      throw new AIServiceError('Failed to analyze PRD', 'analyzePRD', error, false);
    }
  }

  /**
   * Internal implementation of PRD analysis
   */
  private async _analyzePRDImpl(prdContent: string): Promise<AIResult<PRDAnalysis>> {

    const { provider, model, modelName } = getPreferredModel();
    const { maxPRDTokens, promptOverhead, timeoutMs } = AI_CONFIG.sizeLimits;

    // Validate size and check if chunking is needed
    const sizeCheck = validateContentSize(prdContent, maxPRDTokens, promptOverhead);

    if (!sizeCheck.valid) {
      console.warn(`PRD size check: ${sizeCheck.reason}`);
      console.warn(`Estimated tokens: ${sizeCheck.estimatedTokens}`);
      console.warn(`Will attempt to process via chunking...`);

      // Try chunking approach for large PRDs
      return this.analyzePRDChunked(prdContent);
    }

    // Log size info for transparency
    console.log(`Processing PRD: ~${sizeCheck.estimatedTokens} tokens`);

    const basePrompt = `You are an expert software architect analyzing a Product Requirements Document (PRD).

Your task is to:
1. Extract high-level capabilities (functional areas) from the PRD
2. For each capability, define specific requirements
3. For each requirement, create WHEN/THEN/AND scenarios
4. Suggest 5-10 actionable tasks to implement the capabilities
5. Identify dependencies and technical considerations

Important guidelines:
- Capability IDs must be kebab-case (e.g., "user-auth", "data-sync")
- Each requirement must have at least one scenario
- Scenarios must follow WHEN/THEN/AND structure
- Tasks should be specific, actionable, and include complexity scores (1-10)
- Tasks should reference the capability and requirement they implement
- Be specific and actionable
- Focus on testable behaviors

PRD Content:
${prdContent}`;

    const analysisPromise = generateObject({
      model,
      schema: PRDAnalysisSchema,
      prompt: basePrompt,
      temperature: AI_CONFIG.defaults.temperature,
      maxTokens: AI_CONFIG.defaults.maxTokens,
    });

    // Apply timeout
    const result = await withTimeout(
      analysisPromise,
      timeoutMs,
      `PRD analysis for ${modelName}`
    );

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

    // Record the call for rate limiting
    aiRateLimiter.recordCall('analyzePRD', cost.estimatedCost);

    const aiResult: AIResult<PRDAnalysis> = {
      data: result.object,
      usage,
      cost,
      model: modelName,
      provider,
    };

    // Cache the result
    aiCache.set('analyzePRD', { prdContent }, aiResult);

    return aiResult;
  }

  /**
   * Process a large PRD by chunking it and merging results
   */
  private async analyzePRDChunked(prdContent: string): Promise<AIResult<PRDAnalysis>> {
    const { provider, model, modelName } = getPreferredModel();
    const { chunkSize, timeoutMs } = AI_CONFIG.sizeLimits;

    // Split PRD into semantic chunks
    const chunks = chunkText(prdContent, chunkSize);
    console.log(`Processing large PRD in ${chunks.length} chunks`);

    const basePrompt = `You are an expert software architect analyzing a section of a Product Requirements Document (PRD).

Extract from this section:
1. Capabilities (functional areas)
2. Requirements for each capability
3. WHEN/THEN/AND scenarios for requirements
4. Task suggestions
5. Dependencies and technical considerations

Guidelines:
- Capability IDs must be kebab-case
- Each requirement needs at least one scenario
- Scenarios follow WHEN/THEN/AND structure
- Tasks should be actionable with complexity scores (1-10)`;

    // Process each chunk
    const chunkResults: PRDAnalysis[] = [];
    let totalInputTokens = 0;
    let totalOutputTokens = 0;
    let totalCost = 0;

    for (let i = 0; i < chunks.length; i++) {
      const chunkPrompt = createChunkPrompt(chunks[i], i, chunks.length, basePrompt);

      console.log(`Processing chunk ${i + 1}/${chunks.length}...`);

      const chunkPromise = generateObject({
        model,
        schema: PRDAnalysisSchema,
        prompt: chunkPrompt,
        temperature: AI_CONFIG.defaults.temperature,
        maxTokens: AI_CONFIG.defaults.maxTokens,
      });

      const result = await withTimeout(
        chunkPromise,
        timeoutMs,
        `PRD chunk ${i + 1}/${chunks.length} analysis`
      );

      chunkResults.push(result.object);

      totalInputTokens += result.usage?.promptTokens || 0;
      totalOutputTokens += result.usage?.completionTokens || 0;
      totalCost += calculateCost(
        provider,
        modelName,
        result.usage?.promptTokens || 0,
        result.usage?.completionTokens || 0
      );

      // Record rate limit for each chunk
      aiRateLimiter.recordCall(
        'analyzePRD',
        calculateCost(
          provider,
          modelName,
          result.usage?.promptTokens || 0,
          result.usage?.completionTokens || 0
        )
      );
    }

    // Merge all chunk results
    const mergedAnalysis = mergePRDAnalyses(chunkResults);

    const cost: CostEstimate = {
      inputTokens: totalInputTokens,
      outputTokens: totalOutputTokens,
      totalTokens: totalInputTokens + totalOutputTokens,
      estimatedCost: totalCost,
      model: modelName,
      provider,
    };

    const aiResult: AIResult<PRDAnalysis> = {
      data: mergedAnalysis,
      usage: {
        inputTokens: totalInputTokens,
        outputTokens: totalOutputTokens,
        totalTokens: totalInputTokens + totalOutputTokens,
      },
      cost,
      model: modelName,
      provider,
    };

    // Cache the merged result
    aiCache.set('analyzePRD', { prdContent }, aiResult);

    return aiResult;
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
    const { provider, model, modelName} = getPreferredModel();

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

  /**
   * Regenerate a PRD from existing specs and tasks
   */
  async regeneratePRD(
    capabilities: SpecCapability[],
    tasks?: Array<{ title: string; description: string; status: string }>
  ): Promise<AIResult<string>> {
    const { provider, model, modelName } = getPreferredModel();

    const capabilitiesSummary = capabilities
      .map(
        (c, i) =>
          `${i + 1}. ${c.name} (${c.id})
   Purpose: ${c.purpose}
   Requirements: ${c.requirements.length}
   ${c.requirements.map((r, ri) => `   ${ri + 1}. ${r.name}`).join('\n')}`
      )
      .join('\n\n');

    let prompt = `You are an expert product manager regenerating a Product Requirements Document (PRD) from existing specifications and tasks.

Existing Capabilities:
${capabilitiesSummary}`;

    if (tasks && tasks.length > 0) {
      const completedTasks = tasks.filter((t) => t.status === 'completed' || t.status === 'done');
      const inProgressTasks = tasks.filter((t) => t.status === 'in-progress' || t.status === 'in_progress');
      const pendingTasks = tasks.filter((t) => t.status === 'pending' || t.status === 'todo');

      prompt += `\n\nImplementation Progress:
- Completed tasks: ${completedTasks.length}
- In progress: ${inProgressTasks.length}
- Pending: ${pendingTasks.length}

Sample tasks:
${tasks.slice(0, 10).map((t) => `- [${t.status}] ${t.title}`).join('\n')}`;
    }

    prompt += `\n\nGenerate a comprehensive PRD document that:
1. Provides an executive summary of the product
2. Details each capability with clear objectives
3. Outlines technical requirements and constraints
4. Includes implementation status if tasks are provided
5. Identifies dependencies and risks
6. Sets clear success criteria

Format the PRD in professional markdown with:
- Clear section headers (## for main sections, ### for subsections)
- Bullet points for lists
- Tables for structured data
- Code blocks for technical details

Return ONLY the PRD markdown content, no explanations.`;

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
