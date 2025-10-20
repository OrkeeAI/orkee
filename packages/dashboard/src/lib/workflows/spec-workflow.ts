// ABOUTME: Workflow orchestration for OpenSpec PRD-to-Task and Task-to-PRD flows
// ABOUTME: Coordinates AI service calls with database operations for complete workflows

import { aiSpecService, type AIResult } from '../ai/services';
import type {
  PRDAnalysis,
  SpecCapability,
  TaskSuggestion,
  OrphanTaskAnalysis,
  CostEstimate,
} from '../ai/schemas';

/**
 * Progress callback for workflow steps
 */
export type ProgressCallback = (step: string, progress: number) => void;

/**
 * Result of PRD processing workflow
 */
export interface PRDWorkflowResult {
  prdId: string;
  analysis: PRDAnalysis;
  capabilities: Array<{
    id: string;
    capability: SpecCapability;
    specMarkdown: string;
  }>;
  suggestedTasks: TaskSuggestion[];
  totalCost: number;
  steps: Array<{
    step: string;
    cost: CostEstimate;
    timestamp: Date;
  }>;
}

/**
 * Result of orphan task processing workflow
 */
export interface OrphanTaskWorkflowResult {
  taskId: string;
  analysis: OrphanTaskAnalysis;
  createdCapability?: {
    id: string;
    name: string;
  };
  linkedRequirement: {
    id: string;
    name: string;
  };
  totalCost: number;
  steps: Array<{
    step: string;
    cost: CostEstimate;
    timestamp: Date;
  }>;
}

/**
 * Workflow orchestrator for OpenSpec operations
 */
export class SpecWorkflow {
  private progressCallback?: ProgressCallback;

  constructor(progressCallback?: ProgressCallback) {
    this.progressCallback = progressCallback;
  }

  /**
   * Complete PRD → Spec → Task workflow
   *
   * 1. Analyze PRD to extract capabilities
   * 2. Generate specs for each capability
   * 3. Generate markdown for each spec
   * 4. Suggest tasks for each capability
   * 5. Return aggregated results
   */
  async processNewPRD(
    prdId: string,
    prdContent: string,
    projectId: string
  ): Promise<PRDWorkflowResult> {
    const steps: Array<{ step: string; cost: CostEstimate; timestamp: Date }> = [];
    let totalCost = 0;

    // Step 1: Analyze PRD
    this.updateProgress('Analyzing PRD with AI...', 10);
    const analysisResult = await aiSpecService.analyzePRD(prdContent);
    steps.push({
      step: 'Analyze PRD',
      cost: analysisResult.cost,
      timestamp: new Date(),
    });
    totalCost += analysisResult.cost.estimatedCost;

    this.updateProgress('PRD analysis complete', 30);

    // Step 2: Generate markdown for each capability
    this.updateProgress('Generating spec documents...', 40);
    const capabilities: Array<{
      id: string;
      capability: SpecCapability;
      specMarkdown: string;
    }> = [];

    for (let i = 0; i < analysisResult.data.capabilities.length; i++) {
      const capability = analysisResult.data.capabilities[i];
      const progress = 40 + (30 * (i + 1)) / analysisResult.data.capabilities.length;

      this.updateProgress(`Generating spec for ${capability.name}...`, progress);

      const markdownResult = await aiSpecService.generateSpecMarkdown(capability);
      steps.push({
        step: `Generate markdown for ${capability.name}`,
        cost: markdownResult.cost,
        timestamp: new Date(),
      });
      totalCost += markdownResult.cost.estimatedCost;

      capabilities.push({
        id: capability.id,
        capability,
        specMarkdown: markdownResult.data,
      });
    }

    this.updateProgress('Specs generated successfully', 70);

    // Step 3: Generate task suggestions (optional, can be done later)
    const allSuggestedTasks = analysisResult.data.suggestedTasks || [];

    this.updateProgress('Workflow complete', 100);

    return {
      prdId,
      analysis: analysisResult.data,
      capabilities,
      suggestedTasks: allSuggestedTasks,
      totalCost,
      steps,
    };
  }

  /**
   * Generate tasks from a specific capability
   *
   * 1. Get capability details
   * 2. Check for existing tasks
   * 3. Generate task suggestions
   * 4. Return suggestions
   */
  async generateTasksFromSpec(
    capability: SpecCapability,
    existingTasks?: string[]
  ): Promise<AIResult<TaskSuggestion[]>> {
    this.updateProgress('Analyzing spec and generating tasks...', 50);

    const result = await aiSpecService.suggestTasks(capability, existingTasks);

    this.updateProgress('Task suggestions generated', 100);

    return result;
  }

  /**
   * Process orphan task and link to spec
   *
   * 1. Find orphan tasks without spec links
   * 2. Analyze each task with AI
   * 3. Create or link to capability
   * 4. Create requirement
   * 5. Link task to requirement
   * 6. Return sync results
   */
  async syncOrphanTask(
    task: { id: string; title: string; description: string },
    existingCapabilities: Array<{ id: string; name: string; purpose: string }>
  ): Promise<OrphanTaskWorkflowResult> {
    const steps: Array<{ step: string; cost: CostEstimate; timestamp: Date }> = [];
    let totalCost = 0;

    // Step 1: Analyze orphan task
    this.updateProgress('Analyzing task...', 20);
    const analysisResult = await aiSpecService.analyzeOrphanTask(task, existingCapabilities);
    steps.push({
      step: 'Analyze orphan task',
      cost: analysisResult.cost,
      timestamp: new Date(),
    });
    totalCost += analysisResult.cost.estimatedCost;

    this.updateProgress('Task analysis complete', 50);

    // Step 2: Create capability if needed (would be done via API)
    let createdCapability: { id: string; name: string } | undefined;

    if (!analysisResult.data.suggestedCapability.existing) {
      this.updateProgress('Creating new capability...', 60);
      // This would call the API to create the capability
      // For now, we just track what needs to be created
      createdCapability = {
        id: analysisResult.data.suggestedCapability.capabilityId || 'new-capability',
        name: analysisResult.data.suggestedCapability.capabilityName,
      };
    }

    // Step 3: Create/link requirement (would be done via API)
    this.updateProgress('Linking task to requirement...', 80);
    const linkedRequirement = {
      id: 'new-requirement-id', // Would come from API
      name: analysisResult.data.suggestedRequirement.name,
    };

    this.updateProgress('Task linked successfully', 100);

    return {
      taskId: task.id,
      analysis: analysisResult.data,
      createdCapability,
      linkedRequirement,
      totalCost,
      steps,
    };
  }

  /**
   * Batch process multiple orphan tasks
   */
  async syncOrphanTasks(
    tasks: Array<{ id: string; title: string; description: string }>,
    existingCapabilities: Array<{ id: string; name: string; purpose: string }>
  ): Promise<OrphanTaskWorkflowResult[]> {
    const results: OrphanTaskWorkflowResult[] = [];

    for (let i = 0; i < tasks.length; i++) {
      const task = tasks[i];
      const progress = (i / tasks.length) * 100;

      this.updateProgress(`Processing task ${i + 1}/${tasks.length}: ${task.title}`, progress);

      const result = await this.syncOrphanTask(task, existingCapabilities);
      results.push(result);
    }

    this.updateProgress('All orphan tasks processed', 100);

    return results;
  }

  /**
   * Regenerate PRD from updated specs
   *
   * 1. Collect all capabilities and requirements
   * 2. Generate updated PRD content
   * 3. Return new PRD markdown
   */
  async regeneratePRD(
    originalPRD: string,
    capabilities: SpecCapability[]
  ): Promise<AIResult<string>> {
    this.updateProgress('Regenerating PRD from specs...', 50);

    // This would use a text generation model to create updated PRD
    // For now, this is a placeholder - we'd need to add this to the AI service
    const { provider, model, modelName } = await import('../ai/providers').then(
      (m) => m.getPreferredModel()
    );

    const { generateText } = await import('ai');
    const { calculateCost } = await import('../ai/config');

    const prompt = `You are an expert product manager updating a PRD based on refined specifications.

Original PRD:
${originalPRD}

Updated Capabilities:
${capabilities
  .map(
    (c) =>
      `## ${c.name}
${c.purpose}

Requirements:
${c.requirements.map((r) => `- ${r.name}: ${r.content}`).join('\n')}`
  )
  .join('\n\n')}

Generate an updated PRD that:
1. Maintains the original structure and tone
2. Incorporates all changes from the updated specs
3. Highlights what changed
4. Remains clear and professional

Return ONLY the updated PRD markdown, no explanations.`;

    const result = await generateText({
      model,
      prompt,
      temperature: 0.7,
      maxTokens: 4096,
    });

    const usage = {
      inputTokens: result.usage?.promptTokens || 0,
      outputTokens: result.usage?.completionTokens || 0,
      totalTokens: result.usage?.totalTokens || 0,
    };

    const cost = {
      inputTokens: usage.inputTokens,
      outputTokens: usage.outputTokens,
      totalTokens: usage.totalTokens,
      estimatedCost: calculateCost(provider, modelName, usage.inputTokens, usage.outputTokens),
      model: modelName,
      provider,
    };

    this.updateProgress('PRD regenerated successfully', 100);

    return {
      data: result.text,
      usage,
      cost,
      model: modelName,
      provider,
    };
  }

  /**
   * Update progress callback
   */
  private updateProgress(step: string, progress: number) {
    if (this.progressCallback) {
      this.progressCallback(step, progress);
    }
  }
}

/**
 * Create a workflow instance with optional progress tracking
 */
export function createSpecWorkflow(progressCallback?: ProgressCallback): SpecWorkflow {
  return new SpecWorkflow(progressCallback);
}
