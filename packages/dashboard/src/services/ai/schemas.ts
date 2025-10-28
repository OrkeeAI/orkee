// ABOUTME: Zod schemas for AI-generated PRD structures
// ABOUTME: Type-safe validation for Vercel AI SDK generateObject responses

import { z } from 'zod';

// ===== Overview Section =====
export const IdeateOverviewSchema = z.object({
  problemStatement: z.string().optional(),
  targetAudience: z.string().optional(),
  valueProposition: z.string().optional(),
  oneLinePitch: z.string().optional(),
});

export type IdeateOverview = z.infer<typeof IdeateOverviewSchema>;

// ===== Features Section =====
export const IdeateFeatureSchema = z.object({
  name: z.string(),
  what: z.string().optional(),
  why: z.string().optional(),
  how: z.string().optional(),
  dependsOn: z.array(z.string()).optional(),
  enables: z.array(z.string()).optional(),
  buildPhase: z.number().int().min(1).max(3), // 1=foundation, 2=visible, 3=enhancement
  isVisible: z.boolean(),
});

export type IdeateFeature = z.infer<typeof IdeateFeatureSchema>;

export const FeaturesResponseSchema = z.object({
  features: z.array(IdeateFeatureSchema),
});

// ===== UX Section =====
export const PersonaSchema = z.object({
  name: z.string(),
  role: z.string(),
  goals: z.array(z.string()),
  painPoints: z.array(z.string()),
});

export type Persona = z.infer<typeof PersonaSchema>;

export const FlowStepSchema = z.object({
  action: z.string(),
  screen: z.string(),
  notes: z.string().optional(),
});

export type FlowStep = z.infer<typeof FlowStepSchema>;

export const UserFlowSchema = z.object({
  name: z.string(),
  steps: z.array(FlowStepSchema),
  touchpoints: z.array(z.string()),
});

export type UserFlow = z.infer<typeof UserFlowSchema>;

export const IdeateUXSchema = z.object({
  personas: z.array(PersonaSchema).optional(),
  userFlows: z.array(UserFlowSchema).optional(),
  uiConsiderations: z.string().optional(),
  uxPrinciples: z.string().optional(),
});

export type IdeateUX = z.infer<typeof IdeateUXSchema>;

// ===== Technical Section =====
export const ComponentSchema = z.object({
  name: z.string(),
  purpose: z.string(),
  technology: z.string().optional(),
});

export type Component = z.infer<typeof ComponentSchema>;

export const FieldSchema = z.object({
  name: z.string(),
  type: z.string(),
  required: z.boolean(),
});

export type Field = z.infer<typeof FieldSchema>;

export const DataModelSchema = z.object({
  name: z.string(),
  fields: z.array(FieldSchema),
});

export type DataModel = z.infer<typeof DataModelSchema>;

export const APISchema = z.object({
  name: z.string(),
  purpose: z.string(),
  endpoints: z.array(z.string()),
});

export type API = z.infer<typeof APISchema>;

export const InfrastructureSchema = z.object({
  hosting: z.string().optional(),
  database: z.string().optional(),
  caching: z.string().optional(),
  fileStorage: z.string().optional(),
});

export type Infrastructure = z.infer<typeof InfrastructureSchema>;

export const IdeateTechnicalSchema = z.object({
  components: z.array(ComponentSchema).optional(),
  dataModels: z.array(DataModelSchema).optional(),
  apis: z.array(APISchema).optional(),
  infrastructure: InfrastructureSchema.optional(),
  techStackQuick: z.string().optional(),
});

export type IdeateTechnical = z.infer<typeof IdeateTechnicalSchema>;

// ===== Roadmap Section =====
export const PhaseSchema = z.object({
  name: z.string(),
  features: z.array(z.string()),
  goals: z.array(z.string()),
});

export type Phase = z.infer<typeof PhaseSchema>;

export const IdeateRoadmapSchema = z.object({
  mvpScope: z.array(z.string()).optional(),
  futurePhases: z.array(PhaseSchema).optional(),
});

export type IdeateRoadmap = z.infer<typeof IdeateRoadmapSchema>;

// ===== Dependencies Section =====
export const GraphNodeSchema = z.object({
  id: z.string(),
  label: z.string(),
  phase: z.number().int().min(1).max(3),
  type: z.string().optional(),
});

export type GraphNode = z.infer<typeof GraphNodeSchema>;

export const GraphEdgeSchema = z.object({
  from: z.string(),
  to: z.string(),
  type: z.string().optional(),
});

export type GraphEdge = z.infer<typeof GraphEdgeSchema>;

export const DependencyGraphSchema = z.object({
  nodes: z.array(GraphNodeSchema),
  edges: z.array(GraphEdgeSchema),
});

export type DependencyGraph = z.infer<typeof DependencyGraphSchema>;

export const IdeateDependenciesSchema = z.object({
  foundationFeatures: z.array(z.string()).optional(),
  visibleFeatures: z.array(z.string()).optional(),
  enhancementFeatures: z.array(z.string()).optional(),
  dependencyGraph: DependencyGraphSchema.optional(),
});

export type IdeateDependencies = z.infer<typeof IdeateDependenciesSchema>;

// ===== Risks Section =====
export const RiskSchema = z.object({
  description: z.string(),
  severity: z.enum(['low', 'medium', 'high', 'critical']),
  probability: z.enum(['low', 'medium', 'high']),
});

export type Risk = z.infer<typeof RiskSchema>;

export const MitigationSchema = z.object({
  risk: z.string(),
  strategy: z.string(),
  owner: z.string().optional(),
});

export type Mitigation = z.infer<typeof MitigationSchema>;

export const IdeateRisksSchema = z.object({
  technicalRisks: z.array(RiskSchema).optional(),
  mvpScopingRisks: z.array(RiskSchema).optional(),
  resourceRisks: z.array(RiskSchema).optional(),
  mitigations: z.array(MitigationSchema).optional(),
});

export type IdeateRisks = z.infer<typeof IdeateRisksSchema>;

// ===== Research Section =====
export const CompetitorSchema = z.object({
  name: z.string(),
  url: z.string().optional(),
  strengths: z.array(z.string()),
  gaps: z.array(z.string()),
  features: z.array(z.string()),
});

export type Competitor = z.infer<typeof CompetitorSchema>;

export const SimilarProjectSchema = z.object({
  name: z.string(),
  url: z.string().optional(),
  positiveAspects: z.array(z.string()),
  negativeAspects: z.array(z.string()),
  patternsToAdopt: z.array(z.string()),
});

export type SimilarProject = z.infer<typeof SimilarProjectSchema>;

export const ReferenceSchema = z.object({
  title: z.string(),
  url: z.string().optional(),
  notes: z.string().optional(),
});

export type Reference = z.infer<typeof ReferenceSchema>;

export const IdeateResearchSchema = z.object({
  competitors: z.array(CompetitorSchema).optional(),
  similarProjects: z.array(SimilarProjectSchema).optional(),
  researchFindings: z.string().optional(),
  technicalSpecs: z.string().optional(),
  referenceLinks: z.array(ReferenceSchema).optional(),
});

export type IdeateResearch = z.infer<typeof IdeateResearchSchema>;

// ===== Complete PRD Schema =====
export const CompletePRDSchema = z.object({
  overview: IdeateOverviewSchema,
  features: z.array(IdeateFeatureSchema),
  ux: IdeateUXSchema,
  technical: IdeateTechnicalSchema,
  roadmap: IdeateRoadmapSchema,
  dependencies: IdeateDependenciesSchema,
  risks: IdeateRisksSchema,
  research: IdeateResearchSchema,
});

export type CompletePRD = z.infer<typeof CompletePRDSchema>;
