// ABOUTME: Ideate session service layer for PRD ideation API integration
// ABOUTME: Handles session CRUD, mode selection, and section skip functionality

import { apiClient } from './api';

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
    const response = await apiClient.post<{ success: boolean; data: GeneratedPRD }>(
      `/api/ideate/${sessionId}/quick-generate`,
      input || {}
    );

    if (response.error || !response.data.success) {
      throw new Error(response.error || 'Failed to generate PRD');
    }

    return response.data.data;
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
}

export const ideateService = new IdeateService();
