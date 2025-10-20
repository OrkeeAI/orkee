// ABOUTME: Zod schemas for type-safe AI-generated outputs
// ABOUTME: Validates structured responses from LLMs for spec generation and analysis

import { z } from 'zod';

/**
 * Schema for a scenario with WHEN/THEN/AND structure
 */
export const SpecScenarioSchema = z.object({
  name: z.string().min(1).describe('Scenario name'),
  when: z.string().min(1).describe('WHEN condition that triggers this scenario'),
  then: z.string().min(1).describe('THEN expected outcome'),
  and: z.array(z.string()).optional().describe('Additional AND conditions'),
});

export type SpecScenario = z.infer<typeof SpecScenarioSchema>;

/**
 * Schema for a requirement with scenarios
 */
export const SpecRequirementSchema = z.object({
  name: z.string().min(1).describe('Requirement name, e.g., "User Authentication"'),
  content: z.string().min(1).describe('Detailed requirement description in markdown'),
  scenarios: z.array(SpecScenarioSchema).min(1).describe('At least one scenario required'),
});

export type SpecRequirement = z.infer<typeof SpecRequirementSchema>;

/**
 * Schema for a capability (collection of requirements)
 */
export const SpecCapabilitySchema = z.object({
  id: z
    .string()
    .regex(/^[a-z0-9-]+$/)
    .describe('Capability ID in kebab-case'),
  name: z.string().min(1).describe('Human-readable capability name'),
  purpose: z.string().min(1).describe('Purpose and context of this capability'),
  requirements: z
    .array(SpecRequirementSchema)
    .min(1)
    .describe('At least one requirement required'),
});

export type SpecCapability = z.infer<typeof SpecCapabilitySchema>;

/**
 * Schema for a task suggestion from AI
 */
export const TaskSuggestionSchema = z.object({
  title: z.string().min(1).describe('Task title'),
  description: z.string().min(1).describe('Task description'),
  capabilityId: z.string().describe('Related capability ID'),
  requirementName: z.string().describe('Related requirement name'),
  complexity: z.number().min(1).max(10).describe('Complexity score 1-10'),
  estimatedHours: z.number().optional().describe('Estimated hours to complete'),
  priority: z.enum(['low', 'medium', 'high']).default('medium').describe('Task priority'),
});

export type TaskSuggestion = z.infer<typeof TaskSuggestionSchema>;

/**
 * Schema for PRD analysis output
 */
export const PRDAnalysisSchema = z.object({
  summary: z.string().min(1).describe('Executive summary of the PRD'),
  capabilities: z
    .array(SpecCapabilitySchema)
    .min(1)
    .describe('Extracted capabilities from the PRD'),
  suggestedTasks: z
    .array(TaskSuggestionSchema)
    .optional()
    .describe('Optional task suggestions'),
  dependencies: z
    .array(z.string())
    .optional()
    .describe('External dependencies identified'),
  technicalConsiderations: z
    .array(z.string())
    .optional()
    .describe('Technical considerations and constraints'),
});

export type PRDAnalysis = z.infer<typeof PRDAnalysisSchema>;

/**
 * Schema for a spec delta (change to a capability)
 */
export const SpecDeltaSchema = z.object({
  deltaType: z.enum(['added', 'modified', 'removed']).describe('Type of change'),
  capabilityId: z.string().describe('Capability being changed'),
  capabilityName: z.string().describe('Capability name'),
  requirements: z.array(SpecRequirementSchema).describe('Changed requirements'),
  rationale: z.string().min(1).describe('Why this change is needed'),
});

export type SpecDelta = z.infer<typeof SpecDeltaSchema>;

/**
 * Schema for a change proposal
 */
export const ChangeProposalSchema = z.object({
  changeId: z
    .string()
    .regex(/^[a-z0-9-]+$/)
    .describe('Change ID like add-2fa'),
  title: z.string().min(1).describe('Change title'),
  proposal: z.string().min(1).describe('Proposal markdown content'),
  deltas: z.array(SpecDeltaSchema).min(1).describe('Spec deltas for this change'),
  tasks: z.array(TaskSuggestionSchema).describe('Suggested tasks for implementing this change'),
  impact: z
    .enum(['low', 'medium', 'high'])
    .default('medium')
    .describe('Impact level of this change'),
});

export type ChangeProposal = z.infer<typeof ChangeProposalSchema>;

/**
 * Schema for spec validation results
 */
export const SpecValidationSchema = z.object({
  valid: z.boolean().describe('Whether the spec is valid'),
  errors: z.array(z.string()).describe('Validation errors'),
  warnings: z.array(z.string()).describe('Validation warnings'),
  suggestions: z.array(z.string()).optional().describe('Improvement suggestions'),
});

export type SpecValidation = z.infer<typeof SpecValidationSchema>;

/**
 * Schema for orphan task analysis
 */
export const OrphanTaskAnalysisSchema = z.object({
  taskId: z.string().describe('Task ID'),
  suggestedCapability: z.object({
    existing: z.boolean().describe('Whether to use an existing capability'),
    capabilityId: z.string().optional().describe('Existing capability ID if applicable'),
    capabilityName: z.string().describe('Capability name (existing or new)'),
  }),
  suggestedRequirement: z.object({
    name: z.string().describe('Requirement name'),
    content: z.string().describe('Requirement description'),
    scenarios: z.array(SpecScenarioSchema).describe('Suggested scenarios'),
  }),
  confidence: z.number().min(0).max(1).describe('Confidence score for this suggestion'),
  rationale: z.string().describe('Why this mapping makes sense'),
});

export type OrphanTaskAnalysis = z.infer<typeof OrphanTaskAnalysisSchema>;

/**
 * Schema for task validation against spec
 */
export const TaskValidationSchema = z.object({
  taskId: z.string().describe('Task ID'),
  scenarioResults: z.array(
    z.object({
      scenarioName: z.string().describe('Scenario name'),
      passed: z.boolean().describe('Whether scenario passed'),
      confidence: z.number().min(0).max(1).describe('Confidence in the result'),
      notes: z.string().optional().describe('Additional notes'),
    })
  ),
  overallPassed: z.boolean().describe('Whether all scenarios passed'),
  recommendations: z.array(z.string()).optional().describe('Recommendations for improvement'),
});

export type TaskValidation = z.infer<typeof TaskValidationSchema>;

/**
 * Schema for spec refinement suggestions
 */
export const SpecRefinementSchema = z.object({
  capabilityId: z.string().describe('Capability ID'),
  suggestions: z.array(
    z.object({
      type: z.enum(['add', 'modify', 'remove', 'clarify']).describe('Type of suggestion'),
      target: z.string().describe('What to change (requirement, scenario, etc.)'),
      description: z.string().describe('Description of the suggested change'),
      rationale: z.string().describe('Why this change would improve the spec'),
      priority: z.enum(['low', 'medium', 'high']).describe('Priority of this suggestion'),
    })
  ),
});

export type SpecRefinement = z.infer<typeof SpecRefinementSchema>;

/**
 * Schema for cost estimation
 */
export const CostEstimateSchema = z.object({
  inputTokens: z.number().int().describe('Number of input tokens'),
  outputTokens: z.number().int().describe('Number of output tokens'),
  totalTokens: z.number().int().describe('Total tokens'),
  estimatedCost: z.number().describe('Estimated cost in USD'),
  model: z.string().describe('Model used'),
  provider: z.enum(['openai', 'anthropic']).describe('AI provider'),
});

export type CostEstimate = z.infer<typeof CostEstimateSchema>;
