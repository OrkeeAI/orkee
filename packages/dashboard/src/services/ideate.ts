// ABOUTME: Ideate session service layer for PRD ideation API integration
// ABOUTME: Handles session CRUD, mode selection, and section skip functionality

import { apiClient } from './api';
import { usersService } from './users';
import { createAIService } from './ai';

export type IdeateMode = 'quick' | 'guided' | 'comprehensive';
export type IdeateStatus = 'draft' | 'in_progress' | 'ready_for_prd' | 'completed';

export interface IdeateSession {
  id: string;
  project_id: string;
  initial_description: string;
  mode: IdeateMode;
  status: IdeateStatus;
  skipped_sections: string[] | null;
  current_section: string | null;
  created_at: string;
  updated_at: string;
}

export interface CreateIdeateInput {
  projectId: string;
  initialDescription: string;
  mode: IdeateMode;
  templateId?: string;
}

export interface UpdateIdeateInput {
  initialDescription?: string;
  mode?: IdeateMode;
  status?: IdeateStatus;
  skippedSections?: string[];
}

export interface SkipSectionInput {
  section: string;
  ai_fill: boolean;
}

export interface SessionCompletionStatus {
  session_id: string;
  total_sections: number;
  completed_sections: number;
  skipped_sections: string[];
  is_ready_for_prd: boolean;
  missing_required_sections: string[];
}

export interface QuickGenerateInput {
  sections?: string[];
  provider?: string;
  model?: string;
}

export interface QuickExpandInput {
  sections: string[];
}

export interface GeneratedPRD {
  session_id: string;
  content: string;
  sections: Record<string, string>;
  generated_at: string;
}

export interface SavePRDResult {
  prd_id: string;
  success: boolean;
}

// Section types for Guided Mode
export interface IdeateOverview {
  id: string;
  session_id: string;
  problem_statement: string | null;
  target_audience: string | null;
  value_proposition: string | null;
  one_line_pitch: string | null;
  ai_generated: boolean;
  created_at: string;
}

export interface Persona {
  name: string;
  role: string;
  goals: string[];
  pain_points: string[];
}

export interface FlowStep {
  action: string;
  screen: string;
  notes: string | null;
}

export interface UserFlow {
  name: string;
  steps: FlowStep[];
  touchpoints: string[];
}

export interface IdeateUX {
  id: string;
  session_id: string;
  personas: Persona[] | null;
  user_flows: UserFlow[] | null;
  ui_considerations: string | null;
  ux_principles: string | null;
  ai_generated: boolean;
  created_at: string;
}

export interface Component {
  name: string;
  purpose: string;
  technology: string | null;
}

export interface Field {
  name: string;
  field_type: string;
  required: boolean;
}

export interface DataModel {
  name: string;
  fields: Field[];
}

export interface API {
  name: string;
  purpose: string;
  endpoints: string[];
}

export interface Infrastructure {
  hosting: string | null;
  database: string | null;
  caching: string | null;
  file_storage: string | null;
}

export interface IdeateTechnical {
  id: string;
  session_id: string;
  components: Component[] | null;
  data_models: DataModel[] | null;
  apis: API[] | null;
  infrastructure: Infrastructure | null;
  tech_stack_quick: string | null;
  ai_generated: boolean;
  created_at: string;
}

export interface Phase {
  name: string;
  features: string[];
  goals: string[];
}

export interface IdeateRoadmap {
  id: string;
  session_id: string;
  mvp_scope: string[] | null;
  future_phases: Phase[] | null;
  ai_generated: boolean;
  created_at: string;
}

export interface GraphNode {
  id: string;
  label: string;
  phase: number;
}

export interface GraphEdge {
  from: string;
  to: string;
}

export interface DependencyGraph {
  nodes: GraphNode[];
  edges: GraphEdge[];
}

export interface IdeateDependencies {
  id: string;
  session_id: string;
  foundation_features: string[] | null;
  visible_features: string[] | null;
  enhancement_features: string[] | null;
  dependency_graph: DependencyGraph | null;
  ai_generated: boolean;
  created_at: string;
}

export interface Risk {
  description: string;
  severity: string;
  probability: string;
}

export interface Mitigation {
  risk: string;
  strategy: string;
  owner: string | null;
}

export interface IdeateRisks {
  id: string;
  session_id: string;
  technical_risks: Risk[] | null;
  mvp_scoping_risks: Risk[] | null;
  resource_risks: Risk[] | null;
  mitigations: Mitigation[] | null;
  ai_generated: boolean;
  created_at: string;
}

export interface Competitor {
  name: string;
  url: string;
  strengths: string[];
  gaps: string[];
  features: string[];
}

export interface SimilarProject {
  name: string;
  url: string;
  positive_aspects: string[];
  negative_aspects: string[];
  patterns_to_adopt: string[];
}

export interface Reference {
  title: string;
  url: string;
  notes: string | null;
}

export interface IdeateResearch {
  id: string;
  session_id: string;
  competitors: Competitor[] | null;
  similar_projects: SimilarProject[] | null;
  research_findings: string | null;
  technical_specs: string | null;
  reference_links: Reference[] | null;
  ai_generated: boolean;
  created_at: string;
}

// Phase 5: Research Analysis types

export interface UIPattern {
  pattern_type: string; // layout, navigation, interaction, visual, content
  name: string;
  description: string;
  benefits: string;
  adoption_notes: string;
}

export interface Opportunity {
  opportunity_type: string; // differentiation, improvement, gap
  title: string;
  description: string;
  competitor_context: string;
  recommendation: string;
}

export interface GapAnalysis {
  opportunities: Opportunity[];
  summary: string;
}

export interface Lesson {
  category: string; // design, implementation, feature, ux, technical
  insight: string;
  application: string;
  priority: string; // high, medium, low
}

export interface ResearchSynthesis {
  key_findings: string[];
  market_position: string;
  differentiators: string[];
  risks: string[];
  recommendations: string[];
}

// Phase 4: Dependency Intelligence types

export type DependencyType = 'technical' | 'logical' | 'business';
export type DependencyStrength = 'required' | 'recommended' | 'optional';
export type OptimizationStrategy = 'fastest' | 'balanced' | 'safest';

export interface FeatureDependency {
  id: string;
  from_feature_id: string;
  to_feature_id: string;
  dependency_type: DependencyType;
  strength: DependencyStrength;
  reason: string | null;
  auto_detected: boolean;
  created_at: string;
}

export interface CreateFeatureDependencyInput {
  fromFeatureId: string;
  toFeatureId: string;
  dependencyType: DependencyType;
  strength: DependencyStrength;
  reason?: string;
}

export interface DependencyAnalysisResult {
  session_id: string;
  detected_dependencies: FeatureDependency[];
  analysis_summary: string;
  cached: boolean;
}

export interface ParallelGroup {
  features: string[];
  estimated_time: number;
}

export interface BuildOrderResult {
  session_id: string;
  build_order: string[];
  parallel_groups: ParallelGroup[];
  critical_path: string[];
  strategy: OptimizationStrategy;
  optimization_notes: string;
}

export interface CircularDependency {
  cycle: string[];
  severity: string;
  suggestion: string;
}

// ============================================================================
// ROUNDTABLE TYPES (Phase 6)
// ============================================================================

export type RoundtableStatus = 'setup' | 'discussing' | 'completed' | 'cancelled';
export type MessageRole = 'expert' | 'user' | 'moderator' | 'system';
export type InsightPriority = 'low' | 'medium' | 'high' | 'critical';

export interface ExpertPersona {
  id: string;
  name: string;
  role: string;
  expertise: string[];
  system_prompt: string;
  bio: string | null;
  is_default: boolean;
  created_at: string;
}

export interface CreateExpertPersonaInput {
  name: string;
  role: string;
  expertise: string[];
  system_prompt: string;
  bio?: string;
}

export interface ExpertSuggestion {
  id: string;
  session_id: string;
  expert_name: string;
  role: string;
  expertise_area: string;
  reason: string;
  relevance_score: number | null;
  created_at: string;
}

export interface SuggestExpertsRequest {
  projectDescription: string;
  existingContent?: string;
  numExperts?: number;
}

export interface RoundtableSession {
  id: string;
  session_id: string;
  status: RoundtableStatus;
  topic: string;
  num_experts: number;
  moderator_persona: string | null;
  started_at: string | null;
  completed_at: string | null;
  created_at: string;
}

export interface CreateRoundtableRequest {
  topic: string;
  numExperts: number;
}

export interface RoundtableWithParticipants {
  session: RoundtableSession;
  participants: ExpertPersona[];
}

export interface AddParticipantsRequest {
  expertIds: string[];
}

export interface StartRoundtableRequest {
  topic: string;
  expertIds: string[];
  durationMinutes?: number;
}

export interface RoundtableMessage {
  id: string;
  roundtable_id: string;
  message_order: number;
  role: MessageRole;
  expert_id: string | null;
  expert_name: string | null;
  content: string;
  metadata: string | null;
  created_at: string;
}

export interface UserInterjectionInput {
  message: string;
}

export interface UserInterjectionResponse {
  message_id: string;
  acknowledged: boolean;
}

export interface RoundtableInsight {
  id: string;
  roundtable_id: string;
  insight_text: string;
  category: string;
  priority: InsightPriority;
  source_experts: string[];
  source_message_ids: string[] | null;
  created_at: string;
}

export interface ExtractInsightsRequest {
  categories?: string[];
}

export interface ExtractInsightsResponse {
  insights: RoundtableInsight[];
  summary: string | null;
}

export interface InsightsByCategory {
  category: string;
  insights: RoundtableInsight[];
}

export interface RoundtableStatistics {
  roundtable_id: string;
  message_count: number;
  expert_count: number;
  user_interjection_count: number;
  insight_count: number;
  duration_seconds: number | null;
  started_at: string | null;
  completed_at: string | null;
}

export interface RoundtableEvent {
  type: 'connected' | 'started' | 'message' | 'typing' | 'interjection_acknowledged' | 'completed' | 'error' | 'heartbeat';
  data?: unknown;
}

// =============================================================================
// Phase 8: Templates & Intelligence Types
// =============================================================================

export type ProjectType = 'saas' | 'mobile' | 'api' | 'marketplace' | 'internal-tool';

export interface PRDTemplate {
  id: string;
  name: string;
  description: string | null;
  project_type: ProjectType | null;
  one_liner_prompts: string[] | null;
  default_features: string[] | null;
  default_dependencies: Record<string, unknown> | null;
  is_system: boolean;
  created_at: string;
}

export interface SuggestTemplateRequest {
  description: string;
}

// =============================================================================
// Phase 7: PRD Generation & Export Types
// =============================================================================

export type ExportFormat = 'markdown' | 'html' | 'pdf' | 'docx';

export interface ExportOptions {
  format: ExportFormat;
  includeToc?: boolean;
  includeMetadata?: boolean;
  includePageNumbers?: boolean;
  customCss?: string;
  title?: string;
}

export interface ExportResult {
  format: string;
  content: string;
  fileName: string;
  mimeType: string;
  sizeBytes: number;
}

export interface CompletenessMetrics {
  total_sections: number;
  completed_sections: number;
  skipped_sections: number;
  ai_filled_sections: number;
  completeness_percentage: number;
  missing_required: string[];
}

export interface AggregatedPRDData {
  session: IdeateSession;
  overview: IdeateOverview | null;
  ux: IdeateUX | null;
  technical: IdeateTechnical | null;
  roadmap: IdeateRoadmap | null;
  dependencies: IdeateDependencies | null;
  risks: IdeateRisks | null;
  research: IdeateResearch | null;
  roundtableInsights: RoundtableInsight[] | null;
  skippedSections: string[];
  completeness: CompletenessMetrics;
}

export interface GenerationHistoryItem {
  id: string;
  version: number;
  generationMethod: string;
  validationStatus: string;
  createdAt: string;
}

export interface ValidationIssue {
  rule: string;
  section: string | null;
  message: string;
}

export interface ValidationResponse {
  status: string;
  errors: ValidationIssue[];
  warnings: ValidationIssue[];
}

class IdeateService {
  /**
   * Create a new ideate session
   */
  async createSession(input: CreateIdeateInput): Promise<IdeateSession> {
    const response = await apiClient.post<{ success: boolean; data: IdeateSession }>(
      '/api/ideate/start',
      input
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to create ideate session');
    }

    return response.data.data;
  }

  /**
   * Get a ideate session by ID
   */
  async getSession(sessionId: string): Promise<IdeateSession> {
    const response = await apiClient.get<{ success: boolean; data: IdeateSession }>(
      `/api/ideate/${sessionId}`
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to fetch ideate session');
    }

    return response.data.data;
  }

  /**
   * List all ideate sessions for a project
   */
  async listSessions(projectId: string): Promise<IdeateSession[]> {
    const response = await apiClient.get<{ success: boolean; data: IdeateSession[] }>(
      `/api/${projectId}/ideate/sessions`
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to fetch ideate sessions');
    }

    return response.data.data;
  }

  /**
   * Update a ideate session
   */
  async updateSession(sessionId: string, input: UpdateIdeateInput): Promise<void> {
    const response = await apiClient.put<{ success: boolean }>(
      `/api/ideate/${sessionId}`,
      input
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to update ideate session');
    }
  }

  /**
   * Delete a ideate session
   */
  async deleteSession(sessionId: string): Promise<void> {
    const response = await apiClient.delete<{ success: boolean }>(
      `/api/ideate/${sessionId}`
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to delete ideate session');
    }
  }

  /**
   * Skip a section with optional AI fill
   */
  async skipSection(sessionId: string, input: SkipSectionInput): Promise<void> {
    const response = await apiClient.post<{ success: boolean }>(
      `/api/ideate/${sessionId}/skip-section`,
      input
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to skip section');
    }
  }

  /**
   * Get session completion status
   */
  async getCompletionStatus(sessionId: string): Promise<SessionCompletionStatus> {
    const response = await apiClient.get<{ success: boolean; data: SessionCompletionStatus }>(
      `/api/ideate/${sessionId}/status`
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to fetch completion status');
    }

    return response.data.data;
  }

  /**
   * Generate PRD from session description (Quick Mode)
   */
  async quickGenerate(sessionId: string, input?: QuickGenerateInput): Promise<GeneratedPRD> {
    // Get the session to extract description
    const session = await this.getSession(sessionId);

    // Create AI service with config (no API key needed - proxy handles it)
    const aiService = createAIService({
      apiKey: '', // Not used - proxy fetches from database
      model: input?.model || 'claude-sonnet-4-20250514',
      maxTokens: 64000,
      temperature: 0.7,
    });

    console.log(`[quickGenerate] Generating PRD for session ${sessionId}`);
    console.log(`[quickGenerate] Model: ${aiService.getModel()}`);
    console.log(`[quickGenerate] Description: ${session.initial_description.substring(0, 100)}...`);

    // Generate complete PRD
    const result = await aiService.generateCompletePRD(session.initial_description);

    console.log(`[quickGenerate] Generation complete`);
    console.log(`[quickGenerate] Tokens: ${result.usage.totalTokens} (${result.usage.inputTokens} in, ${result.usage.outputTokens} out)`);

    // Transform structured data to markdown sections
    const sections: Record<string, string> = {
      overview: JSON.stringify(result.data.overview, null, 2),
      features: JSON.stringify(result.data.features, null, 2),
      ux: JSON.stringify(result.data.ux, null, 2),
      technical: JSON.stringify(result.data.technical, null, 2),
      roadmap: JSON.stringify(result.data.roadmap, null, 2),
      dependencies: JSON.stringify(result.data.dependencies, null, 2),
      risks: JSON.stringify(result.data.risks, null, 2),
      research: JSON.stringify(result.data.research, null, 2),
    };

    // Generate complete markdown content
    const content = Object.entries(sections)
      .map(([section, data]) => `## ${section}\n\n${data}\n\n`)
      .join('');

    return {
      session_id: sessionId,
      content,
      sections,
      generated_at: new Date().toISOString(),
    };
  }

  /**
   * Expand specific PRD sections (Quick Mode)
   */
  async quickExpand(sessionId: string, input: QuickExpandInput): Promise<GeneratedPRD> {
    const response = await apiClient.post<{ success: boolean; data: GeneratedPRD }>(
      `/api/ideate/${sessionId}/quick-expand`,
      input
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to expand PRD sections');
    }

    return response.data.data;
  }

  /**
   * Preview PRD before saving
   */
  async previewPRD(sessionId: string): Promise<GeneratedPRD> {
    const response = await apiClient.get<{ success: boolean; data: GeneratedPRD }>(
      `/api/ideate/${sessionId}/preview`
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to preview PRD');
    }

    return response.data.data;
  }

  /**
   * Save generated PRD to OpenSpec system
   */
  async saveAsPRD(sessionId: string): Promise<SavePRDResult> {
    const response = await apiClient.post<{ success: boolean; data: SavePRDResult }>(
      `/api/ideate/${sessionId}/save-as-prd`,
      {}
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to save PRD');
    }

    return response.data.data;
  }

  // Section CRUD methods for Guided Mode

  /**
   * Save overview section
   */
  async saveOverview(sessionId: string, overview: Omit<IdeateOverview, 'id' | 'created_at'>): Promise<IdeateOverview> {
    const response = await apiClient.post<{ success: boolean; data: IdeateOverview }>(
      `/api/ideate/${sessionId}/overview`,
      overview
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to save overview section');
    }

    return response.data.data;
  }

  /**
   * Get overview section
   */
  async getOverview(sessionId: string): Promise<IdeateOverview | null> {
    const response = await apiClient.get<{ success: boolean; data: IdeateOverview | null }>(
      `/api/ideate/${sessionId}/overview`
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to get overview section');
    }

    return response.data.data;
  }

  /**
   * Delete overview section
   */
  async deleteOverview(sessionId: string): Promise<void> {
    const response = await apiClient.delete<{ success: boolean }>(
      `/api/ideate/${sessionId}/overview`
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to delete overview section');
    }
  }

  /**
   * Save UX section
   */
  async saveUX(sessionId: string, ux: Omit<IdeateUX, 'id' | 'created_at'>): Promise<IdeateUX> {
    const response = await apiClient.post<{ success: boolean; data: IdeateUX }>(
      `/api/ideate/${sessionId}/ux`,
      ux
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to save UX section');
    }

    return response.data.data;
  }

  /**
   * Get UX section
   */
  async getUX(sessionId: string): Promise<IdeateUX | null> {
    const response = await apiClient.get<{ success: boolean; data: IdeateUX | null }>(
      `/api/ideate/${sessionId}/ux`
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to get UX section');
    }

    return response.data.data;
  }

  /**
   * Delete UX section
   */
  async deleteUX(sessionId: string): Promise<void> {
    const response = await apiClient.delete<{ success: boolean }>(
      `/api/ideate/${sessionId}/ux`
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to delete UX section');
    }
  }

  /**
   * Save technical section
   */
  async saveTechnical(sessionId: string, technical: Omit<IdeateTechnical, 'id' | 'created_at'>): Promise<IdeateTechnical> {
    const response = await apiClient.post<{ success: boolean; data: IdeateTechnical }>(
      `/api/ideate/${sessionId}/technical`,
      technical
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to save technical section');
    }

    return response.data.data;
  }

  /**
   * Get technical section
   */
  async getTechnical(sessionId: string): Promise<IdeateTechnical | null> {
    const response = await apiClient.get<{ success: boolean; data: IdeateTechnical | null }>(
      `/api/ideate/${sessionId}/technical`
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to get technical section');
    }

    return response.data.data;
  }

  /**
   * Delete technical section
   */
  async deleteTechnical(sessionId: string): Promise<void> {
    const response = await apiClient.delete<{ success: boolean }>(
      `/api/ideate/${sessionId}/technical`
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to delete technical section');
    }
  }

  /**
   * Save roadmap section
   */
  async saveRoadmap(sessionId: string, roadmap: Omit<IdeateRoadmap, 'id' | 'created_at'>): Promise<IdeateRoadmap> {
    const response = await apiClient.post<{ success: boolean; data: IdeateRoadmap }>(
      `/api/ideate/${sessionId}/roadmap`,
      roadmap
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to save roadmap section');
    }

    return response.data.data;
  }

  /**
   * Get roadmap section
   */
  async getRoadmap(sessionId: string): Promise<IdeateRoadmap | null> {
    const response = await apiClient.get<{ success: boolean; data: IdeateRoadmap | null }>(
      `/api/ideate/${sessionId}/roadmap`
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to get roadmap section');
    }

    return response.data.data;
  }

  /**
   * Delete roadmap section
   */
  async deleteRoadmap(sessionId: string): Promise<void> {
    const response = await apiClient.delete<{ success: boolean }>(
      `/api/ideate/${sessionId}/roadmap`
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to delete roadmap section');
    }
  }

  /**
   * Save dependencies section
   */
  async saveDependencies(sessionId: string, dependencies: Omit<IdeateDependencies, 'id' | 'created_at'>): Promise<IdeateDependencies> {
    const response = await apiClient.post<{ success: boolean; data: IdeateDependencies }>(
      `/api/ideate/${sessionId}/dependencies`,
      dependencies
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to save dependencies section');
    }

    return response.data.data;
  }

  /**
   * Get dependencies section
   */
  async getDependencies(sessionId: string): Promise<IdeateDependencies | null> {
    const response = await apiClient.get<{ success: boolean; data: IdeateDependencies | null }>(
      `/api/ideate/${sessionId}/dependencies`
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to get dependencies section');
    }

    return response.data.data;
  }

  /**
   * Delete dependencies section
   */
  async deleteDependencies(sessionId: string): Promise<void> {
    const response = await apiClient.delete<{ success: boolean }>(
      `/api/ideate/${sessionId}/dependencies`
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to delete dependencies section');
    }
  }

  /**
   * Save risks section
   */
  async saveRisks(sessionId: string, risks: Omit<IdeateRisks, 'id' | 'created_at'>): Promise<IdeateRisks> {
    const response = await apiClient.post<{ success: boolean; data: IdeateRisks }>(
      `/api/ideate/${sessionId}/risks`,
      risks
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to save risks section');
    }

    return response.data.data;
  }

  /**
   * Get risks section
   */
  async getRisks(sessionId: string): Promise<IdeateRisks | null> {
    const response = await apiClient.get<{ success: boolean; data: IdeateRisks | null }>(
      `/api/ideate/${sessionId}/risks`
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to get risks section');
    }

    return response.data.data;
  }

  /**
   * Delete risks section
   */
  async deleteRisks(sessionId: string): Promise<void> {
    const response = await apiClient.delete<{ success: boolean }>(
      `/api/ideate/${sessionId}/risks`
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to delete risks section');
    }
  }

  /**
   * Save research section
   */
  async saveResearch(sessionId: string, research: Omit<IdeateResearch, 'id' | 'created_at'>): Promise<IdeateResearch> {
    const response = await apiClient.post<{ success: boolean; data: IdeateResearch }>(
      `/api/ideate/${sessionId}/research`,
      research
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to save research section');
    }

    return response.data.data;
  }

  /**
   * Get research section
   */
  async getResearch(sessionId: string): Promise<IdeateResearch | null> {
    const response = await apiClient.get<{ success: boolean; data: IdeateResearch | null }>(
      `/api/ideate/${sessionId}/research`
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to get research section');
    }

    return response.data.data;
  }

  /**
   * Delete research section
   */
  async deleteResearch(sessionId: string): Promise<void> {
    const response = await apiClient.delete<{ success: boolean }>(
      `/api/ideate/${sessionId}/research`
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to delete research section');
    }
  }

  // Phase 5: Research Analysis methods

  /**
   * Analyze a competitor URL
   */
  async analyzeCompetitor(sessionId: string, url: string, projectDescription?: string): Promise<Competitor> {
    const response = await apiClient.post<{ success: boolean; data: Competitor }>(
      `/api/ideate/${sessionId}/research/competitors/analyze`,
      { url, projectDescription }
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to analyze competitor');
    }

    return response.data.data;
  }

  /**
   * Get all analyzed competitors
   */
  async getCompetitors(sessionId: string): Promise<Competitor[]> {
    const response = await apiClient.get<{ success: boolean; data: Competitor[] }>(
      `/api/ideate/${sessionId}/research/competitors`
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to get competitors');
    }

    return response.data.data;
  }

  /**
   * Perform gap analysis against competitors
   */
  async analyzeGaps(sessionId: string, yourFeatures: string[]): Promise<GapAnalysis> {
    const response = await apiClient.post<{ success: boolean; data: GapAnalysis }>(
      `/api/ideate/${sessionId}/research/gaps/analyze`,
      { yourFeatures }
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to analyze gaps');
    }

    return response.data.data;
  }

  /**
   * Extract UI/UX patterns from a URL
   */
  async extractPatterns(sessionId: string, url: string, projectDescription?: string): Promise<UIPattern[]> {
    const response = await apiClient.post<{ success: boolean; data: UIPattern[] }>(
      `/api/ideate/${sessionId}/research/patterns/extract`,
      { url, projectDescription }
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to extract patterns');
    }

    return response.data.data;
  }

  /**
   * Add a similar project reference
   */
  async addSimilarProject(sessionId: string, project: SimilarProject): Promise<void> {
    const response = await apiClient.post<{ success: boolean }>(
      `/api/ideate/${sessionId}/research/similar-projects`,
      project
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to add similar project');
    }
  }

  /**
   * Get all similar projects
   */
  async getSimilarProjects(sessionId: string): Promise<SimilarProject[]> {
    const response = await apiClient.get<{ success: boolean; data: SimilarProject[] }>(
      `/api/ideate/${sessionId}/research/similar-projects`
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to get similar projects');
    }

    return response.data.data;
  }

  /**
   * Extract lessons from a similar project
   */
  async extractLessons(sessionId: string, projectName: string, projectDescription?: string): Promise<Lesson[]> {
    const response = await apiClient.post<{ success: boolean; data: Lesson[] }>(
      `/api/ideate/${sessionId}/research/lessons/extract`,
      { projectName, projectDescription }
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to extract lessons');
    }

    return response.data.data;
  }

  /**
   * Synthesize all research findings
   */
  async synthesizeResearch(sessionId: string): Promise<ResearchSynthesis> {
    const response = await apiClient.post<{ success: boolean; data: ResearchSynthesis }>(
      `/api/ideate/${sessionId}/research/synthesize`,
      {}
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to synthesize research');
    }

    return response.data.data;
  }

  // Phase 4: Dependency Intelligence methods

  /**
   * Get all feature dependencies for a session
   */
  async getFeatureDependencies(sessionId: string): Promise<FeatureDependency[]> {
    const response = await apiClient.get<{ success: boolean; data: FeatureDependency[] }>(
      `/api/ideate/${sessionId}/features/dependencies`
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to fetch feature dependencies');
    }

    return response.data.data;
  }

  /**
   * Create a manual feature dependency
   */
  async createFeatureDependency(sessionId: string, input: CreateFeatureDependencyInput): Promise<FeatureDependency> {
    const response = await apiClient.post<{ success: boolean; data: FeatureDependency }>(
      `/api/ideate/${sessionId}/features/dependencies`,
      input
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to create feature dependency');
    }

    return response.data.data;
  }

  /**
   * Delete a feature dependency
   */
  async deleteFeatureDependency(sessionId: string, dependencyId: string): Promise<void> {
    const response = await apiClient.delete<{ success: boolean }>(
      `/api/ideate/${sessionId}/features/dependencies/${dependencyId}`
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to delete feature dependency');
    }
  }

  /**
   * Analyze dependencies using AI
   */
  async analyzeDependencies(sessionId: string): Promise<DependencyAnalysisResult> {
    const response = await apiClient.post<{ success: boolean; data: DependencyAnalysisResult }>(
      `/api/ideate/${sessionId}/dependencies/analyze`,
      {}
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to analyze dependencies');
    }

    return response.data.data;
  }

  /**
   * Optimize build order
   */
  async optimizeBuildOrder(sessionId: string, strategy: OptimizationStrategy): Promise<BuildOrderResult> {
    const response = await apiClient.post<{ success: boolean; data: BuildOrderResult }>(
      `/api/ideate/${sessionId}/dependencies/optimize`,
      { strategy }
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to optimize build order');
    }

    return response.data.data;
  }

  /**
   * Get current build order
   */
  async getBuildOrder(sessionId: string): Promise<BuildOrderResult> {
    const response = await apiClient.get<{ success: boolean; data: BuildOrderResult }>(
      `/api/ideate/${sessionId}/dependencies/build-order`
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to fetch build order');
    }

    return response.data.data;
  }

  /**
   * Get circular dependencies
   */
  async getCircularDependencies(sessionId: string): Promise<CircularDependency[]> {
    const response = await apiClient.get<{ success: boolean; data: CircularDependency[] }>(
      `/api/ideate/${sessionId}/dependencies/circular`
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to fetch circular dependencies');
    }

    return response.data.data;
  }

  /**
   * Suggest quick-win features (high value, low dependency)
   */
  async suggestQuickWins(sessionId: string): Promise<string[]> {
    const response = await apiClient.get<{ success: boolean; data: { quick_wins: string[] } }>(
      `/api/ideate/${sessionId}/features/suggest-visible`
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to suggest quick wins');
    }

    return response.data.data.quick_wins;
  }

  // Navigation methods

  /**
   * Get next incomplete section
   */
  async getNextSection(sessionId: string): Promise<string | null> {
    const response = await apiClient.get<{ success: boolean; data: string | null }>(
      `/api/ideate/${sessionId}/next-section`
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to get next section');
    }

    return response.data.data;
  }

  /**
   * Navigate to a specific section
   */
  async navigateTo(sessionId: string, section: string): Promise<IdeateSession> {
    const response = await apiClient.post<{ success: boolean; data: IdeateSession }>(
      `/api/ideate/${sessionId}/navigate`,
      { section }
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to navigate to section');
    }

    return response.data.data;
  }

  // ============================================================================
  // ROUNDTABLE METHODS (Phase 6)
  // ============================================================================

  /**
   * List all expert personas (default + custom)
   */
  async listExperts(sessionId: string): Promise<ExpertPersona[]> {
    const response = await apiClient.get<{ success: boolean; data: ExpertPersona[] }>(
      `/api/ideate/${sessionId}/experts`
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to list experts');
    }

    return response.data.data;
  }

  /**
   * Create a custom expert persona
   */
  async createExpert(sessionId: string, input: CreateExpertPersonaInput): Promise<ExpertPersona> {
    const response = await apiClient.post<{ success: boolean; data: ExpertPersona }>(
      `/api/ideate/${sessionId}/experts`,
      input
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to create expert');
    }

    return response.data.data;
  }

  /**
   * Get AI-suggested experts for session
   */
  async suggestExperts(sessionId: string, request: SuggestExpertsRequest): Promise<ExpertSuggestion[]> {
    const response = await apiClient.post<{ success: boolean; data: { suggestions: ExpertSuggestion[] } }>(
      `/api/ideate/${sessionId}/experts/suggest`,
      request
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to suggest experts');
    }

    return response.data.data.suggestions;
  }

  /**
   * Create a roundtable session
   */
  async createRoundtable(sessionId: string, request: CreateRoundtableRequest): Promise<RoundtableSession> {
    const response = await apiClient.post<{ success: boolean; data: RoundtableSession }>(
      `/api/ideate/${sessionId}/roundtable`,
      request
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to create roundtable');
    }

    return response.data.data;
  }

  /**
   * List roundtables for a session
   */
  async listRoundtables(sessionId: string): Promise<RoundtableSession[]> {
    const response = await apiClient.get<{ success: boolean; data: RoundtableSession[] }>(
      `/api/ideate/${sessionId}/roundtables`
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to list roundtables');
    }

    return response.data.data;
  }

  /**
   * Get roundtable with participants
   */
  async getRoundtable(roundtableId: string): Promise<RoundtableWithParticipants> {
    const response = await apiClient.get<{ success: boolean; data: RoundtableWithParticipants }>(
      `/api/ideate/roundtable/${roundtableId}`
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to get roundtable');
    }

    return response.data.data;
  }

  /**
   * Add participants to roundtable
   */
  async addParticipants(roundtableId: string, request: AddParticipantsRequest): Promise<void> {
    const response = await apiClient.post<{ success: boolean }>(
      `/api/ideate/roundtable/${roundtableId}/participants`,
      request
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to add participants');
    }
  }

  /**
   * Get participants for roundtable
   */
  async getParticipants(roundtableId: string): Promise<ExpertPersona[]> {
    const response = await apiClient.get<{ success: boolean; data: ExpertPersona[] }>(
      `/api/ideate/roundtable/${roundtableId}/participants`
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to get participants');
    }

    return response.data.data;
  }

  /**
   * Start roundtable discussion (async - returns immediately)
   */
  async startDiscussion(roundtableId: string, request: StartRoundtableRequest): Promise<void> {
    const response = await apiClient.post<{ success: boolean }>(
      `/api/ideate/roundtable/${roundtableId}/start`,
      request
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to start discussion');
    }
  }

  /**
   * Get SSE stream URL for real-time discussion updates
   * Note: This returns the URL, actual SSE connection should be managed by React hook
   */
  getRoundtableStreamUrl(roundtableId: string): string {
    return `/api/ideate/roundtable/${roundtableId}/stream`;
  }

  /**
   * Send user interjection during discussion
   */
  async sendInterjection(roundtableId: string, input: UserInterjectionInput): Promise<UserInterjectionResponse> {
    const response = await apiClient.post<{ success: boolean; data: UserInterjectionResponse }>(
      `/api/ideate/roundtable/${roundtableId}/interjection`,
      input
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to send interjection');
    }

    return response.data.data;
  }

  /**
   * Get all messages for roundtable
   */
  async getRoundtableMessages(roundtableId: string): Promise<RoundtableMessage[]> {
    const response = await apiClient.get<{ success: boolean; data: RoundtableMessage[] }>(
      `/api/ideate/roundtable/${roundtableId}/messages`
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to get messages');
    }

    return response.data.data;
  }

  /**
   * Extract insights from roundtable discussion
   */
  async extractInsights(roundtableId: string, request: ExtractInsightsRequest): Promise<ExtractInsightsResponse> {
    const response = await apiClient.post<{ success: boolean; data: ExtractInsightsResponse }>(
      `/api/ideate/roundtable/${roundtableId}/insights/extract`,
      request
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to extract insights');
    }

    return response.data.data;
  }

  /**
   * Get insights grouped by category
   */
  async getInsights(roundtableId: string): Promise<InsightsByCategory[]> {
    const response = await apiClient.get<{ success: boolean; data: InsightsByCategory[] }>(
      `/api/ideate/roundtable/${roundtableId}/insights`
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to get insights');
    }

    return response.data.data;
  }

  /**
   * Get roundtable statistics
   */
  async getRoundtableStatistics(roundtableId: string): Promise<RoundtableStatistics> {
    const response = await apiClient.get<{ success: boolean; data: RoundtableStatistics }>(
      `/api/ideate/roundtable/${roundtableId}/statistics`
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to get statistics');
    }

    return response.data.data;
  }

  // =============================================================================
  // Phase 7: PRD Generation & Export
  // =============================================================================

  /**
   * Generate PRD from collected session data
   */
  async generatePRD(sessionId: string, includeSkipped = false): Promise<GeneratedPRD> {
    const response = await apiClient.post<{ success: boolean; data: GeneratedPRD }>(
      `/api/ideate/${sessionId}/prd/generate`,
      { includeSkipped }
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to generate PRD');
    }

    return response.data.data;
  }

  /**
   * AI-fill skipped sections with context
   */
  async fillSkippedSections(sessionId: string, sections: string[]): Promise<Record<string, unknown>> {
    const response = await apiClient.post<{ success: boolean; data: Record<string, unknown> }>(
      `/api/ideate/${sessionId}/prd/fill-sections`,
      { sections }
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to fill skipped sections');
    }

    return response.data.data;
  }

  /**
   * Regenerate specific section with full context
   */
  async regenerateSection(sessionId: string, section: string): Promise<unknown> {
    const response = await apiClient.post<{ success: boolean; data: unknown }>(
      `/api/ideate/${sessionId}/prd/regenerate-section`,
      { section }
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to regenerate section');
    }

    return response.data.data;
  }

  /**
   * Get PRD preview (aggregated data)
   */
  async getPRDPreview(sessionId: string): Promise<AggregatedPRDData> {
    const response = await apiClient.get<{ success: boolean; data: AggregatedPRDData }>(
      `/api/ideate/${sessionId}/prd/preview`
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to get PRD preview');
    }

    return response.data.data;
  }

  /**
   * Export PRD in specified format
   */
  async exportPRD(sessionId: string, options: ExportOptions): Promise<ExportResult> {
    const response = await apiClient.post<{ success: boolean; data: ExportResult }>(
      `/api/ideate/${sessionId}/prd/export`,
      options
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to export PRD');
    }

    return response.data.data;
  }

  /**
   * Get completeness metrics for session
   */
  async getCompleteness(sessionId: string): Promise<CompletenessMetrics> {
    const response = await apiClient.get<{ success: boolean; data: CompletenessMetrics }>(
      `/api/ideate/${sessionId}/prd/completeness`
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to get completeness metrics');
    }

    return response.data.data;
  }

  /**
   * Get PRD generation history
   */
  async getGenerationHistory(sessionId: string): Promise<GenerationHistoryItem[]> {
    const response = await apiClient.get<{ success: boolean; data: GenerationHistoryItem[] }>(
      `/api/ideate/${sessionId}/prd/history`
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to get generation history');
    }

    return response.data.data;
  }

  /**
   * Validate PRD against rules
   */
  async validatePRD(sessionId: string): Promise<ValidationResponse> {
    const response = await apiClient.get<{ success: boolean; data: ValidationResponse }>(
      `/api/ideate/${sessionId}/prd/validation`
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to validate PRD');
    }

    return response.data.data;
  }

  // ===========================================================================
  // Phase 8: Template Methods
  // ===========================================================================

  /**
   * Get all available templates
   */
  async getTemplates(): Promise<PRDTemplate[]> {
    const response = await apiClient.get<{ success: boolean; data: PRDTemplate[] }>(
      '/api/ideate/templates'
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to get templates');
    }

    return response.data.data;
  }

  /**
   * Get a specific template by ID
   */
  async getTemplate(templateId: string): Promise<PRDTemplate> {
    const response = await apiClient.get<{ success: boolean; data: PRDTemplate }>(
      `/api/ideate/templates/${templateId}`
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to get template');
    }

    return response.data.data;
  }

  /**
   * Get templates filtered by project type
   */
  async getTemplatesByType(projectType: ProjectType): Promise<PRDTemplate[]> {
    const response = await apiClient.get<{ success: boolean; data: PRDTemplate[] }>(
      `/api/ideate/templates/by-type/${projectType}`
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to get templates by type');
    }

    return response.data.data;
  }

  /**
   * Suggest best matching template based on description
   */
  async suggestTemplate(description: string): Promise<PRDTemplate | null> {
    const response = await apiClient.post<{ success: boolean; data: PRDTemplate | null }>(
      '/api/ideate/templates/suggest',
      { description }
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to suggest template');
    }

    return response.data.data;
  }
}

export const ideateService = new IdeateService();
