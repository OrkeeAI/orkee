// ABOUTME: API service for code graph visualization
// ABOUTME: Provides methods for fetching dependency, symbol, module, and spec-mapping graphs

import { ApiClient } from './api';

const api = new ApiClient();

export interface GraphNode {
  id: string;
  label: string;
  node_type: 'file' | 'function' | 'class' | 'module' | 'spec' | 'requirement';
  metadata: {
    path?: string;
    line_start?: number;
    line_end?: number;
    token_count?: number;
    complexity?: number;
    spec_id?: string;
  };
}

export interface GraphEdge {
  id: string;
  source: string;
  target: string;
  edge_type: 'import' | 'export' | 'reference' | 'implementation' | 'dependency' | 'contains';
  weight?: number;
}

export interface GraphMetadata {
  total_nodes: number;
  total_edges: number;
  graph_type: string;
  generated_at: string;
  project_id: string;
}

export interface CodeGraph {
  nodes: GraphNode[];
  edges: GraphEdge[];
  metadata: GraphMetadata;
}

export interface GraphResponse {
  success: boolean;
  data?: CodeGraph;
  error?: string;
}

export type GraphType = 'dependencies' | 'symbols' | 'modules' | 'spec-mapping';

export interface GraphQueryOptions {
  max_depth?: number;
  filter?: string;
  layout?: string;
}

export const graphService = {
  /**
   * Get dependency graph for a project
   */
  async getDependencyGraph(
    projectId: string,
    options?: GraphQueryOptions
  ): Promise<CodeGraph> {
    const response = await api.get<GraphResponse>(
      `/projects/${projectId}/graph/dependencies`,
      options ? { params: options } : undefined
    );

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to fetch dependency graph');
    }

    return response.data;
  },

  /**
   * Get symbol graph for a project
   */
  async getSymbolGraph(
    projectId: string,
    options?: GraphQueryOptions
  ): Promise<CodeGraph> {
    const response = await api.get<GraphResponse>(
      `/projects/${projectId}/graph/symbols`,
      options ? { params: options } : undefined
    );

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to fetch symbol graph');
    }

    return response.data;
  },

  /**
   * Get module graph for a project
   */
  async getModuleGraph(
    projectId: string,
    options?: GraphQueryOptions
  ): Promise<CodeGraph> {
    const response = await api.get<GraphResponse>(
      `/projects/${projectId}/graph/modules`,
      options ? { params: options } : undefined
    );

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to fetch module graph');
    }

    return response.data;
  },

  /**
   * Get spec-mapping graph for a project
   */
  async getSpecMappingGraph(
    projectId: string,
    options?: GraphQueryOptions
  ): Promise<CodeGraph> {
    const response = await api.get<GraphResponse>(
      `/projects/${projectId}/graph/spec-mapping`,
      options ? { params: options } : undefined
    );

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to fetch spec-mapping graph');
    }

    return response.data;
  },

  /**
   * Get graph by type (generic method)
   */
  async getGraph(
    projectId: string,
    graphType: GraphType,
    options?: GraphQueryOptions
  ): Promise<CodeGraph> {
    const response = await api.get<GraphResponse>(
      `/projects/${projectId}/graph/${graphType}`,
      options ? { params: options } : undefined
    );

    if (!response.success || !response.data) {
      throw new Error(response.error || `Failed to fetch ${graphType} graph`);
    }

    return response.data;
  },
};
