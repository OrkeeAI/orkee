// ABOUTME: Zod schemas for type-safe AI-generated OpenSpec structures
// ABOUTME: Ensures AI outputs match database schema and provides validation

import { z } from 'zod';

// PRD Analysis Schema
export const PRDAnalysisSchema = z.object({
  title: z.string().describe('PRD title'),
  summary: z.string().describe('Executive summary of the PRD'),
  capabilities: z.array(z.object({
    name: z.string().describe('Capability name (e.g., "auth", "search")'),
    purpose: z.string().describe('Purpose of this capability'),
    priority: z.enum(['low', 'medium', 'high', 'critical']),
  })).describe('List of capabilities identified in the PRD'),
  estimatedComplexity: z.enum(['simple', 'moderate', 'complex', 'very-complex']),
  suggestedApproach: z.string().describe('Recommended implementation approach'),
});

export type PRDAnalysis = z.infer<typeof PRDAnalysisSchema>;

// Spec Scenario Schema (WHEN/THEN/AND)
export const ScenarioSchema = z.object({
  name: z.string().describe('Scenario name'),
  when: z.string().describe('WHEN condition'),
  then: z.string().describe('THEN expectation'),
  and: z.array(z.string()).optional().describe('Additional AND conditions'),
});

export type Scenario = z.infer<typeof ScenarioSchema>;

// Spec Requirement Schema
export const RequirementSchema = z.object({
  name: z.string().describe('Requirement name'),
  description: z.string().describe('Detailed requirement description'),
  scenarios: z.array(ScenarioSchema).describe('List of scenarios for this requirement'),
  acceptanceCriteria: z.array(z.string()).optional().describe('Acceptance criteria'),
});

export type Requirement = z.infer<typeof RequirementSchema>;

// Spec Capability Schema
export const CapabilitySchema = z.object({
  name: z.string().describe('Capability name matching PRD'),
  purpose: z.string().describe('Purpose of this capability'),
  requirements: z.array(RequirementSchema).describe('List of requirements'),
  design: z.string().optional().describe('Design notes or approach'),
});

export type Capability = z.infer<typeof CapabilitySchema>;

// Complete Spec Generation Schema
export const SpecGenerationSchema = z.object({
  capabilities: z.array(CapabilitySchema),
  metadata: z.object({
    totalRequirements: z.number(),
    totalScenarios: z.number(),
    estimatedEffort: z.string().describe('Estimated implementation effort'),
  }),
});

export type SpecGeneration = z.infer<typeof SpecGenerationSchema>;

// Change Delta Schema
export const ChangeDeltaSchema = z.object({
  capabilityName: z.string().describe('Affected capability'),
  type: z.enum(['added', 'modified', 'removed']),
  description: z.string().describe('What changed'),
  impact: z.enum(['low', 'medium', 'high']).describe('Impact level'),
  reasoning: z.string().describe('Why this change is needed'),
});

export type ChangeDelta = z.infer<typeof ChangeDeltaSchema>;

// Change Proposal Schema
export const ChangeProposalSchema = z.object({
  title: z.string().describe('Change proposal title'),
  summary: z.string().describe('Brief summary of the change'),
  motivation: z.string().describe('Why this change is needed'),
  deltas: z.array(ChangeDeltaSchema).describe('Specific capability changes'),
  tasks: z.array(z.object({
    title: z.string(),
    description: z.string(),
    estimatedHours: z.number().optional(),
    priority: z.enum(['low', 'medium', 'high', 'critical']),
    requirementIds: z.array(z.string()).optional().describe('Linked requirement IDs'),
  })).describe('Suggested implementation tasks'),
  risks: z.array(z.string()).optional().describe('Potential risks'),
  testing: z.string().optional().describe('Testing strategy'),
});

export type ChangeProposal = z.infer<typeof ChangeProposalSchema>;

// Task Suggestion Schema (from specs)
export const TaskSuggestionSchema = z.object({
  tasks: z.array(z.object({
    title: z.string().describe('Task title'),
    description: z.string().describe('Detailed task description'),
    acceptanceCriteria: z.string().optional().describe('What defines done'),
    estimatedHours: z.number().optional().describe('Estimated effort in hours'),
    priority: z.enum(['low', 'medium', 'high', 'critical']),
    tags: z.array(z.string()).optional(),
    requirementId: z.string().optional().describe('Linked requirement ID'),
    scenarioIds: z.array(z.string()).optional().describe('Linked scenario IDs'),
  })),
  orphanTasks: z.array(z.object({
    taskId: z.string(),
    suggestedCapability: z.string().optional(),
    suggestedRequirement: z.string().optional(),
    reasoning: z.string(),
  })).optional().describe('Tasks without spec coverage'),
});

export type TaskSuggestion = z.infer<typeof TaskSuggestionSchema>;

// Scenario Validation Schema
export const ScenarioValidationSchema = z.object({
  valid: z.boolean(),
  issues: z.array(z.object({
    type: z.enum(['missing', 'ambiguous', 'conflicting', 'incomplete']),
    description: z.string(),
    suggestion: z.string().optional(),
  })).optional(),
  coverage: z.number().describe('Percentage of requirements covered by scenarios (0-100)'),
  suggestions: z.array(z.object({
    requirementName: z.string(),
    suggestedScenario: ScenarioSchema,
  })).optional().describe('Suggested additional scenarios'),
});

export type ScenarioValidation = z.infer<typeof ScenarioValidationSchema>;

// Spec Sync Analysis Schema
export const SyncAnalysisSchema = z.object({
  direction: z.enum(['prd-to-spec', 'spec-to-prd', 'task-to-spec']),
  changes: z.array(z.object({
    type: z.enum(['add', 'update', 'delete']),
    entity: z.enum(['capability', 'requirement', 'scenario', 'task']),
    id: z.string().optional(),
    name: z.string(),
    description: z.string(),
    impact: z.enum(['low', 'medium', 'high']),
  })),
  conflicts: z.array(z.object({
    description: z.string(),
    resolution: z.string().optional(),
  })).optional(),
  summary: z.string(),
});

export type SyncAnalysis = z.infer<typeof SyncAnalysisSchema>;

// PRD to Spec Extraction Schema
export const PRDExtractionSchema = z.object({
  capabilities: z.array(z.object({
    name: z.string(),
    purpose: z.string(),
    priority: z.enum(['low', 'medium', 'high', 'critical']),
  })),
  globalRequirements: z.array(z.string()).optional().describe('Cross-cutting requirements'),
  assumptions: z.array(z.string()).optional(),
  constraints: z.array(z.string()).optional(),
});

export type PRDExtraction = z.infer<typeof PRDExtractionSchema>;

// Export all schemas as a collection for easy access
export const AISchemas = {
  PRDAnalysis: PRDAnalysisSchema,
  Scenario: ScenarioSchema,
  Requirement: RequirementSchema,
  Capability: CapabilitySchema,
  SpecGeneration: SpecGenerationSchema,
  ChangeDelta: ChangeDeltaSchema,
  ChangeProposal: ChangeProposalSchema,
  TaskSuggestion: TaskSuggestionSchema,
  ScenarioValidation: ScenarioValidationSchema,
  SyncAnalysis: SyncAnalysisSchema,
  PRDExtraction: PRDExtractionSchema,
} as const;
