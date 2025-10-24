# Graph Tab Implementation Plan

## 🎯 Overview

This document provides a comprehensive implementation plan for adding a Graph tab to Orkee's project detail page. The Graph tab will provide interactive code visualization using Cytoscape.js, leveraging existing AST parsing and dependency graph infrastructure.

## 📊 Technology Stack

- **Frontend**: Cytoscape.js with react-cytoscapejs wrapper
- **Backend**: Existing Rust context module with new graph API endpoints
- **Rendering**: Canvas-based for performance
- **Data Source**: AST parser (Tree-sitter) + Dependency graph + Context metadata

## 🚀 Implementation Phases

---

## Phase 1: Backend Infrastructure

### 1.1 Graph Data Types
**Location**: `packages/projects/src/context/graph_types.rs`

- [x] Create graph type definitions
- [x] Define node types (File, Symbol, Module, Spec)
- [x] Define edge types (Import, Export, Reference, Implementation)
- [x] Create graph serialization structs

```rust
// packages/projects/src/context/graph_types.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub node_type: NodeType,
    pub metadata: NodeMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeType {
    File,
    Function,
    Class,
    Module,
    Spec,
    Requirement,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetadata {
    pub path: Option<String>,
    pub line_start: Option<usize>,
    pub line_end: Option<usize>,
    pub token_count: Option<usize>,
    pub complexity: Option<f32>,
    pub spec_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub edge_type: EdgeType,
    pub weight: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeType {
    Import,
    Export,
    Reference,
    Implementation,
    Dependency,
    Contains,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeGraph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub metadata: GraphMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMetadata {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub graph_type: String,
    pub generated_at: chrono::DateTime<chrono::Utc>,
    pub project_id: String,
}
```

### 1.2 Graph Builder Module
**Location**: `packages/projects/src/context/graph_builder.rs`

- [x] Create GraphBuilder struct
- [x] Implement file dependency graph generation
- [x] Implement symbol graph generation
- [x] Implement module hierarchy graph
- [ ] Add spec-to-code mapping graph
- [ ] Add caching layer

```rust
// packages/projects/src/context/graph_builder.rs
use super::graph_types::*;
use super::dependency_graph::DependencyGraph;
use super::ast_analyzer::AstAnalyzer;

pub struct GraphBuilder {
    dependency_graph: DependencyGraph,
    ast_analyzer: AstAnalyzer,
}

impl GraphBuilder {
    pub fn new() -> Self {
        Self {
            dependency_graph: DependencyGraph::new(),
            ast_analyzer: AstAnalyzer::new_typescript(),
        }
    }

    /// Build file dependency graph
    pub async fn build_dependency_graph(&self, project_path: &str) -> Result<CodeGraph, String> {
        // Implementation
    }

    /// Build symbol reference graph
    pub async fn build_symbol_graph(&self, project_path: &str) -> Result<CodeGraph, String> {
        // Implementation
    }

    /// Build module hierarchy graph
    pub async fn build_module_graph(&self, project_path: &str) -> Result<CodeGraph, String> {
        // Implementation
    }

    /// Build spec-to-code mapping graph
    pub async fn build_spec_mapping_graph(&self, project_id: &str) -> Result<CodeGraph, String> {
        // Implementation
    }
}
```

### 1.3 API Handlers
**Location**: `packages/projects/src/api/graph_handlers.rs`

- [x] Create graph API handlers
- [x] Implement GET `/api/projects/:id/graph/dependencies`
- [x] Implement GET `/api/projects/:id/graph/symbols`
- [x] Implement GET `/api/projects/:id/graph/modules`
- [x] Implement GET `/api/projects/:id/graph/spec-mapping`
- [ ] Add caching headers

```rust
// packages/projects/src/api/graph_handlers.rs
use axum::{
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Json},
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GraphQuery {
    pub max_depth: Option<usize>,
    pub filter: Option<String>,
    pub layout: Option<String>,
}

/// Get dependency graph for a project
pub async fn get_dependency_graph(
    Path(project_id): Path<String>,
    Query(params): Query<GraphQuery>,
    State(db): State<DbState>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    // Implementation
}

/// Get symbol graph for a project
pub async fn get_symbol_graph(
    Path(project_id): Path<String>,
    Query(params): Query<GraphQuery>,
    State(db): State<DbState>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    // Implementation
}
```

### 1.4 Route Registration
**Location**: `packages/projects/src/api/mod.rs`

- [x] Add graph routes to API router
- [ ] Add rate limiting for graph endpoints
- [ ] Add caching middleware

```rust
// Add to packages/projects/src/api/mod.rs
pub fn graph_routes() -> Router<AppState> {
    Router::new()
        .route("/api/projects/:id/graph/dependencies", get(get_dependency_graph))
        .route("/api/projects/:id/graph/symbols", get(get_symbol_graph))
        .route("/api/projects/:id/graph/modules", get(get_module_graph))
        .route("/api/projects/:id/graph/spec-mapping", get(get_spec_mapping_graph))
        .layer(middleware::from_fn(cache_middleware))
}
```

### Phase 1 Deferred Items

The following items were intentionally deferred for later implementation:

#### 1.2 Graph Builder - Spec-to-Code Mapping Graph
- **Status**: Deferred to Phase 4
- **Reason**: Requires OpenSpec integration to be fully implemented first
- **When to revisit**: After Phase 4 (Advanced Features) when spec mapping functionality is built out
- **Dependencies**: OpenSpec bridge enhancements, spec requirement tracking
- **Estimated effort**: 2-3 days
- **Location**: `packages/projects/src/context/graph_builder.rs`

#### 1.2 Graph Builder - Caching Layer
- **Status**: Optimization - defer until performance testing
- **Reason**: Current implementation generates graphs on-demand, which is acceptable for MVP
- **When to revisit**:
  - When graph generation time exceeds 2-3 seconds for typical projects
  - When usage metrics show repeated requests for same graph data
  - After Phase 5 performance testing
- **Implementation approach**: Consider Redis cache or in-memory LRU cache with project file hash as key
- **Estimated effort**: 1-2 days
- **Location**: `packages/projects/src/context/graph_builder.rs`

#### 1.3 API Handlers - Caching Headers
- **Status**: Optimization - defer until performance baseline established
- **Reason**: Need to establish usage patterns first to determine optimal cache duration
- **When to revisit**: After Phase 5 performance testing and user feedback
- **Implementation approach**: Add `Cache-Control`, `ETag`, and conditional request support
- **Estimated effort**: 4-6 hours
- **Location**: `packages/projects/src/api/graph_handlers.rs`

#### 1.4 Route Registration - Rate Limiting
- **Status**: Optional enhancement
- **Reason**: Graph endpoints are not public-facing and project access is already authenticated
- **When to revisit**: If abuse is detected or if opening API to third parties
- **Implementation approach**: Use existing rate limiting middleware pattern from other endpoints
- **Estimated effort**: 2-3 hours
- **Location**: `packages/projects/src/api/mod.rs`

#### 1.4 Route Registration - Caching Middleware
- **Status**: Optimization - defer with caching headers
- **Reason**: Couples with caching headers implementation
- **When to revisit**: Implement together with Phase 1.3 caching headers
- **Implementation approach**: Tower middleware layer for response caching
- **Estimated effort**: 4-6 hours
- **Location**: `packages/projects/src/api/mod.rs`

---

## Phase 2: Frontend Foundation

### 2.1 Install Dependencies

- [x] Add Cytoscape.js dependencies
- [x] Add TypeScript types
- [x] Add layout plugins

```bash
cd packages/dashboard
bun add cytoscape react-cytoscapejs cytoscape-fcose cytoscape-dagre
bun add -D @types/cytoscape
```

### 2.2 Graph Service Layer
**Location**: `packages/dashboard/src/services/graph.ts` and `packages/dashboard/src/hooks/useGraph.ts`

- [x] Create graph API client
- [x] Add React Query hooks
- [x] Implement caching strategy (React Query provides built-in caching with 5-minute staleTime)

```typescript
// packages/dashboard/src/services/graph.ts
import { useQuery } from '@tanstack/react-query';
import { api } from './api';

export interface GraphNode {
  id: string;
  label: string;
  node_type: 'file' | 'function' | 'class' | 'module' | 'spec';
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
  edge_type: 'import' | 'export' | 'reference' | 'implementation';
  weight?: number;
}

export interface CodeGraph {
  nodes: GraphNode[];
  edges: GraphEdge[];
  metadata: {
    total_nodes: number;
    total_edges: number;
    graph_type: string;
    generated_at: string;
  };
}

export const useProjectGraph = (
  projectId: string,
  graphType: 'dependencies' | 'symbols' | 'modules' | 'spec-mapping',
  options?: { max_depth?: number; filter?: string }
) => {
  return useQuery({
    queryKey: ['project-graph', projectId, graphType, options],
    queryFn: () => api.get<CodeGraph>(`/projects/${projectId}/graph/${graphType}`, { params: options }),
    staleTime: 5 * 60 * 1000, // 5 minutes
  });
};
```

### 2.3 Graph Tab Component
**Location**: `packages/dashboard/src/components/graph/GraphTab.tsx`

- [x] Create main GraphTab component
- [x] Add graph type selector
- [x] Implement loading states
- [x] Add error handling

```tsx
// packages/dashboard/src/components/graph/GraphTab.tsx
import React, { useState } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { DependencyGraph } from './DependencyGraph';
import { SymbolGraph } from './SymbolGraph';
import { ModuleGraph } from './ModuleGraph';
import { SpecMappingGraph } from './SpecMappingGraph';
import { GraphControls } from './GraphControls';
import { GraphSidebar } from './GraphSidebar';

interface GraphTabProps {
  projectId: string;
  projectPath: string;
}

export function GraphTab({ projectId, projectPath }: GraphTabProps) {
  const [selectedNode, setSelectedNode] = useState<string | null>(null);
  const [layout, setLayout] = useState<string>('hierarchical');
  const [filters, setFilters] = useState<GraphFilters>({});

  return (
    <div className="flex gap-4 h-[800px]">
      <div className="flex-1 flex flex-col gap-4">
        <GraphControls
          layout={layout}
          onLayoutChange={setLayout}
          filters={filters}
          onFiltersChange={setFilters}
        />

        <Tabs defaultValue="dependencies" className="flex-1">
          <TabsList>
            <TabsTrigger value="dependencies">File Dependencies</TabsTrigger>
            <TabsTrigger value="symbols">Symbol Graph</TabsTrigger>
            <TabsTrigger value="modules">Module Architecture</TabsTrigger>
            <TabsTrigger value="spec-mapping">Spec Mapping</TabsTrigger>
          </TabsList>

          <TabsContent value="dependencies" className="flex-1">
            <DependencyGraph
              projectId={projectId}
              layout={layout}
              filters={filters}
              onNodeSelect={setSelectedNode}
            />
          </TabsContent>

          <TabsContent value="symbols" className="flex-1">
            <SymbolGraph
              projectId={projectId}
              layout={layout}
              filters={filters}
              onNodeSelect={setSelectedNode}
            />
          </TabsContent>

          <TabsContent value="modules" className="flex-1">
            <ModuleGraph
              projectId={projectId}
              layout={layout}
              filters={filters}
              onNodeSelect={setSelectedNode}
            />
          </TabsContent>

          <TabsContent value="spec-mapping" className="flex-1">
            <SpecMappingGraph
              projectId={projectId}
              layout={layout}
              filters={filters}
              onNodeSelect={setSelectedNode}
            />
          </TabsContent>
        </Tabs>
      </div>

      {selectedNode && (
        <GraphSidebar
          nodeId={selectedNode}
          projectId={projectId}
          onClose={() => setSelectedNode(null)}
        />
      )}
    </div>
  );
}
```

### Phase 2 Deferred Items

The following items were intentionally deferred for later phases:

#### 2.2 Graph Service - React Query Hooks
- **Status**: Deferred to Phase 3
- **Reason**: Will implement as part of actual graph visualization components that consume the API
- **When to revisit**: During Phase 3.1 (DependencyGraph component implementation)
- **Implementation approach**:
  ```typescript
  export const useProjectGraph = (
    projectId: string,
    graphType: GraphType,
    options?: GraphQueryOptions
  ) => {
    return useQuery({
      queryKey: ['project-graph', projectId, graphType, options],
      queryFn: () => graphService.getGraph(projectId, graphType, options),
      staleTime: 5 * 60 * 1000, // 5 minutes
      enabled: !!projectId,
    });
  };
  ```
- **Estimated effort**: 2-3 hours
- **Location**: `packages/dashboard/src/services/graph.ts` or new `packages/dashboard/src/hooks/useGraph.ts`

#### 2.2 Graph Service - Caching Strategy
- **Status**: Optimization - defer until usage patterns established
- **Reason**: React Query provides built-in caching; additional caching may be premature
- **When to revisit**:
  - After Phase 3 completion when graph interactions are tested
  - If users report slow graph switching or excessive API calls
- **Implementation approach**: Adjust React Query `staleTime`, `cacheTime`, and consider `keepPreviousData`
- **Estimated effort**: 1-2 hours
- **Location**: React Query configuration or custom hook

#### 2.3 GraphTab - Loading States
- **Status**: Deferred to Phase 3
- **Reason**: Will implement when connecting to actual graph API with React Query
- **When to revisit**: Phase 3.1 when implementing first graph visualization
- **Implementation approach**: Use React Query's `isLoading`, `isFetching` states with skeleton loaders
- **Estimated effort**: 2-3 hours
- **Location**: `packages/dashboard/src/components/graph/GraphTab.tsx` and individual graph components

#### 2.3 GraphTab - Error Handling
- **Status**: Deferred to Phase 3
- **Reason**: Will implement when connecting to actual graph API with React Query
- **When to revisit**: Phase 3.1 when implementing first graph visualization
- **Implementation approach**: Use React Query's `isError`, `error` states with Alert components
- **Estimated effort**: 2-3 hours
- **Location**: `packages/dashboard/src/components/graph/GraphTab.tsx` and individual graph components

---

## Phase 3: Graph Visualization Components

### 3.1 Dependency Graph Component
**Location**: `packages/dashboard/src/components/graph/DependencyGraph.tsx` and `packages/dashboard/src/components/graph/GraphVisualization.tsx`

- [x] Create Cytoscape wrapper component
- [x] Implement node styling
- [x] Add edge styling
- [x] Handle interactions (click, hover, zoom)
- [ ] Add context menu (deferred - not critical for MVP)

```tsx
// packages/dashboard/src/components/graph/DependencyGraph.tsx
import React, { useEffect, useRef } from 'react';
import CytoscapeComponent from 'react-cytoscapejs';
import cytoscape from 'cytoscape';
import fcose from 'cytoscape-fcose';
import dagre from 'cytoscape-dagre';
import { useProjectGraph } from '@/services/graph';

cytoscape.use(fcose);
cytoscape.use(dagre);

interface DependencyGraphProps {
  projectId: string;
  layout: string;
  filters: GraphFilters;
  onNodeSelect: (nodeId: string) => void;
}

export function DependencyGraph({
  projectId,
  layout,
  filters,
  onNodeSelect
}: DependencyGraphProps) {
  const cyRef = useRef<cytoscape.Core | null>(null);
  const { data: graph, isLoading, error } = useProjectGraph(projectId, 'dependencies', filters);

  const elements = React.useMemo(() => {
    if (!graph) return [];

    const nodes = graph.nodes.map(node => ({
      data: {
        id: node.id,
        label: node.label,
        type: node.node_type,
        ...node.metadata
      },
      classes: node.node_type
    }));

    const edges = graph.edges.map(edge => ({
      data: {
        id: edge.id,
        source: edge.source,
        target: edge.target,
        type: edge.edge_type,
        weight: edge.weight
      },
      classes: edge.edge_type
    }));

    return [...nodes, ...edges];
  }, [graph]);

  const stylesheet = [
    {
      selector: 'node',
      style: {
        'background-color': '#666',
        'label': 'data(label)',
        'text-valign': 'center',
        'text-halign': 'center',
        'font-size': '12px',
        'width': 30,
        'height': 30
      }
    },
    {
      selector: 'node.file',
      style: {
        'background-color': '#4F46E5',
        'shape': 'round-rectangle'
      }
    },
    {
      selector: 'node.function',
      style: {
        'background-color': '#10B981',
        'shape': 'ellipse'
      }
    },
    {
      selector: 'node.class',
      style: {
        'background-color': '#F59E0B',
        'shape': 'diamond'
      }
    },
    {
      selector: 'edge',
      style: {
        'width': 2,
        'line-color': '#ccc',
        'target-arrow-color': '#ccc',
        'target-arrow-shape': 'triangle',
        'curve-style': 'bezier'
      }
    },
    {
      selector: 'edge.import',
      style: {
        'line-color': '#4F46E5',
        'target-arrow-color': '#4F46E5'
      }
    },
    {
      selector: ':selected',
      style: {
        'background-color': '#EF4444',
        'line-color': '#EF4444',
        'target-arrow-color': '#EF4444'
      }
    }
  ];

  const layoutConfig = {
    name: layout,
    ...getLayoutOptions(layout)
  };

  return (
    <Card className="flex-1">
      <CardContent className="p-0 h-full">
        {isLoading && (
          <div className="flex items-center justify-center h-full">
            <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary" />
          </div>
        )}

        {error && (
          <div className="flex items-center justify-center h-full">
            <p className="text-destructive">Failed to load graph</p>
          </div>
        )}

        {graph && (
          <CytoscapeComponent
            elements={elements}
            style={{ width: '100%', height: '100%' }}
            stylesheet={stylesheet}
            layout={layoutConfig}
            cy={(cy) => {
              cyRef.current = cy;

              cy.on('tap', 'node', (evt) => {
                const node = evt.target;
                onNodeSelect(node.id());
              });

              cy.on('mouseover', 'node', (evt) => {
                document.body.style.cursor = 'pointer';
              });

              cy.on('mouseout', 'node', (evt) => {
                document.body.style.cursor = 'default';
              });
            }}
          />
        )}
      </CardContent>
    </Card>
  );
}

function getLayoutOptions(layoutName: string) {
  switch (layoutName) {
    case 'hierarchical':
      return {
        name: 'dagre',
        rankDir: 'TB',
        animate: true,
        animationDuration: 500,
        nodeSep: 50,
        rankSep: 100
      };
    case 'force':
      return {
        name: 'fcose',
        animate: true,
        animationDuration: 500,
        nodeRepulsion: 4500,
        idealEdgeLength: 50
      };
    case 'circular':
      return {
        name: 'circle',
        animate: true,
        animationDuration: 500
      };
    default:
      return {};
  }
}
```

### 3.2 Graph Controls Component
**Location**: `packages/dashboard/src/components/graph/GraphControls.tsx`

- [x] Create layout selector
- [x] Add zoom controls
- [x] Implement search functionality
- [x] Add filter controls
- [x] Export functionality

```tsx
// packages/dashboard/src/components/graph/GraphControls.tsx
import React from 'react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { ZoomIn, ZoomOut, Maximize, Download, Search, Filter } from 'lucide-react';

interface GraphControlsProps {
  layout: string;
  onLayoutChange: (layout: string) => void;
  filters: GraphFilters;
  onFiltersChange: (filters: GraphFilters) => void;
}

export function GraphControls({
  layout,
  onLayoutChange,
  filters,
  onFiltersChange
}: GraphControlsProps) {
  return (
    <Card>
      <CardContent className="p-4">
        <div className="flex items-center justify-between gap-4">
          <div className="flex items-center gap-2">
            <Select value={layout} onValueChange={onLayoutChange}>
              <SelectTrigger className="w-[180px]">
                <SelectValue placeholder="Select layout" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="hierarchical">Hierarchical</SelectItem>
                <SelectItem value="force">Force Directed</SelectItem>
                <SelectItem value="circular">Circular</SelectItem>
                <SelectItem value="grid">Grid</SelectItem>
              </SelectContent>
            </Select>

            <div className="flex items-center gap-1">
              <Button variant="outline" size="icon">
                <ZoomIn className="h-4 w-4" />
              </Button>
              <Button variant="outline" size="icon">
                <ZoomOut className="h-4 w-4" />
              </Button>
              <Button variant="outline" size="icon">
                <Maximize className="h-4 w-4" />
              </Button>
            </div>
          </div>

          <div className="flex items-center gap-2">
            <div className="relative">
              <Search className="absolute left-2 top-2.5 h-4 w-4 text-muted-foreground" />
              <Input
                placeholder="Search nodes..."
                className="pl-8 w-[200px]"
                value={filters.search || ''}
                onChange={(e) => onFiltersChange({ ...filters, search: e.target.value })}
              />
            </div>

            <Button variant="outline" size="icon">
              <Filter className="h-4 w-4" />
            </Button>

            <Button variant="outline" size="icon">
              <Download className="h-4 w-4" />
            </Button>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
```

### 3.3 Graph Sidebar Component
**Location**: `packages/dashboard/src/components/graph/GraphSidebar.tsx`

- [x] Display node details
- [ ] Show file content preview (deferred - requires additional backend API)
- [x] Add navigation actions
- [ ] Context integration (Phase 4 feature)

```tsx
// packages/dashboard/src/components/graph/GraphSidebar.tsx
import React from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { X, ExternalLink, FileText, Code } from 'lucide-react';

interface GraphSidebarProps {
  nodeId: string;
  projectId: string;
  onClose: () => void;
}

export function GraphSidebar({ nodeId, projectId, onClose }: GraphSidebarProps) {
  // Fetch node details
  const { data: nodeDetails } = useNodeDetails(projectId, nodeId);

  return (
    <Card className="w-[400px] h-full overflow-y-auto">
      <CardHeader className="flex flex-row items-center justify-between">
        <CardTitle>Node Details</CardTitle>
        <Button variant="ghost" size="icon" onClick={onClose}>
          <X className="h-4 w-4" />
        </Button>
      </CardHeader>

      <CardContent className="space-y-4">
        {nodeDetails && (
          <>
            <div>
              <h3 className="font-semibold mb-2">{nodeDetails.label}</h3>
              <Badge>{nodeDetails.type}</Badge>
            </div>

            {nodeDetails.path && (
              <div>
                <p className="text-sm text-muted-foreground">Path</p>
                <p className="text-sm font-mono">{nodeDetails.path}</p>
              </div>
            )}

            {nodeDetails.dependencies && (
              <div>
                <p className="text-sm text-muted-foreground mb-2">Dependencies ({nodeDetails.dependencies.length})</p>
                <div className="space-y-1">
                  {nodeDetails.dependencies.map(dep => (
                    <div key={dep.id} className="text-sm">
                      • {dep.label}
                    </div>
                  ))}
                </div>
              </div>
            )}

            <div className="flex gap-2">
              <Button variant="outline" size="sm">
                <FileText className="h-4 w-4 mr-2" />
                Open File
              </Button>
              <Button variant="outline" size="sm">
                <Code className="h-4 w-4 mr-2" />
                Add to Context
              </Button>
            </div>
          </>
        )}
      </CardContent>
    </Card>
  );
}
```

### Phase 3 Implementation Summary

**Status**: ✅ Phase 3 Complete (with minor deferrals noted above)

**Actual Implementation** (committed 2025-01-XX):

Created a unified, reusable graph visualization system:

1. **GraphVisualization.tsx** - Reusable Cytoscape wrapper supporting all 4 graph types
   - Supports dependencies, symbols, modules, and spec-mapping graphs
   - Multiple layout algorithms: hierarchical (dagre), force-directed (fcose), circular, grid
   - Interactive features: node selection, hover states, double-click to fit
   - Search filtering with connected node inclusion
   - Comprehensive node and edge styling for all types

2. **GraphControls.tsx** - Unified control panel
   - Layout selector dropdown (4 layouts)
   - Zoom controls (in, out, fit-to-view)
   - Live search with filtering
   - PNG export functionality
   - Clean, accessible UI

3. **GraphSidebar.tsx** - Node details panel
   - Displays node metadata (path, lines, complexity, token count)
   - Shows incoming dependencies and outgoing dependents
   - Lists connection types with badges
   - Action buttons (ready for backend integration)
   - Scrollable design for large connection lists

4. **DependencyGraph.tsx** - Original dependency-specific implementation (kept for reference)

5. **useGraph.ts** - React Query hooks
   - Generic `useProjectGraph()` for any graph type
   - Type-specific hooks for each graph type
   - 5-minute cache with automatic refetch
   - Error handling and retry logic

6. **GraphTab.tsx** - Fully integrated tab interface
   - All 4 graph types accessible via tabs
   - Shared controls and sidebar
   - Proper state management
   - Clean, responsive layout

**Key Features Delivered**:
- ✅ All 4 graph types supported
- ✅ Multiple layout algorithms
- ✅ Interactive node selection
- ✅ Real-time search filtering
- ✅ Zoom and pan controls
- ✅ PNG export
- ✅ Loading states and error handling
- ✅ Type-safe with full TypeScript support
- ✅ All linting checks passing

**Ready for Backend Integration**: Frontend is fully implemented and will automatically work once the backend graph API endpoints (`/api/projects/:id/graph/{dependencies,symbols,modules,spec-mapping}`) are available.

---

## Phase 4: Advanced Features

### 4.1 Context Integration

- [ ] Add "Add to Context" action for nodes
- [ ] Highlight nodes already in context
- [ ] Show token count contribution
- [ ] Batch selection for context

### 4.2 Spec Mapping Visualization

- [ ] Color code by implementation status
- [ ] Show spec requirements on hover
- [ ] Link to spec details
- [ ] Coverage metrics overlay

### 4.3 Performance Optimizations

- [ ] Implement virtualization for large graphs
- [ ] Add progressive loading
- [ ] Cache computed layouts
- [ ] Web Worker for layout computation

### 4.4 Export & Sharing

- [ ] Export as PNG/SVG
- [ ] Export graph data as JSON
- [ ] Share graph state via URL
- [ ] Save custom views

---

## Phase 5: Testing & Polish

### 5.1 Unit Tests

- [ ] Test graph builder logic
- [ ] Test API handlers
- [ ] Test React components
- [ ] Test graph interactions

### 5.2 Integration Tests

- [ ] Test end-to-end graph generation
- [ ] Test with large codebases
- [ ] Test error scenarios
- [ ] Test caching behavior

### 5.3 Performance Testing

- [ ] Benchmark with 1000+ node graphs
- [ ] Measure rendering performance
- [ ] Test memory usage
- [ ] Optimize bottlenecks

### 5.4 UI Polish

- [ ] Add loading skeletons
- [ ] Improve error messages
- [ ] Add tooltips and help text
- [ ] Keyboard shortcuts

---

## 📋 Integration Checklist

### Add Graph Tab to ProjectDetail.tsx

- [ ] Import GraphTab component
- [ ] Add tab trigger after Context tab
- [ ] Add tab content panel
- [ ] Pass required props

```tsx
// In ProjectDetail.tsx
import { Network } from 'lucide-react'; // Graph icon
import { GraphTab } from '@/components/graph/GraphTab';

// In tabs list
<TabsTrigger value="graph" className="flex items-center gap-2">
  <Network className="h-4 w-4" />
  Graph
</TabsTrigger>

// In tabs content
<TabsContent value="graph" className="space-y-4">
  <GraphTab projectId={project.id} projectPath={project.projectRoot} />
</TabsContent>
```

### Update API Router

- [ ] Import graph routes
- [ ] Register with main router
- [ ] Add to API documentation

```rust
// In packages/projects/src/api/mod.rs
mod graph_handlers;

pub fn create_api_router(state: AppState) -> Router {
    Router::new()
        // ... existing routes
        .merge(graph_routes())
        .with_state(state)
}
```

---

## 🎨 Styling Guide

### Node Colors
- **Files**: `#4F46E5` (Indigo)
- **Functions**: `#10B981` (Emerald)
- **Classes**: `#F59E0B` (Amber)
- **Modules**: `#8B5CF6` (Violet)
- **Specs**: `#EC4899` (Pink)

### Edge Styles
- **Import/Export**: Solid line with arrow
- **Reference**: Dashed line
- **Implementation**: Dotted line
- **Contains**: Thick line (for module hierarchy)

### Interaction States
- **Hover**: Brighten color, show tooltip
- **Selected**: Red highlight (`#EF4444`)
- **In Context**: Green border
- **Related**: Orange glow

---

## 🚀 Performance Targets

- **Initial Load**: < 1 second for graphs up to 500 nodes
- **Layout Change**: < 500ms animation
- **Interaction**: < 100ms response time
- **Search**: < 50ms for filtering
- **Memory**: < 100MB for 1000 node graph

---

## 📝 Notes & Considerations

1. **Incremental Updates**: Leverage existing AST cache from `incremental_parser.rs`
2. **Large Codebases**: Consider pagination or clustering for 1000+ files
3. **Real-time Updates**: Use SSE endpoint for file change notifications
4. **Mobile Support**: Graph visualization should be view-only on mobile
5. **Accessibility**: Provide table view alternative for screen readers

---

## 🔗 Related Documentation

- [Context Tab Implementation](context.md)
- [AST Parser Documentation](packages/projects/src/context/ast_analyzer.rs)
- [Dependency Graph Module](packages/projects/src/context/dependency_graph.rs)
- [Cytoscape.js Documentation](https://js.cytoscape.org/)
- [React Cytoscape Component](https://github.com/plotly/react-cytoscapejs)

---

## 📊 Success Metrics

- [ ] Handle projects with 1000+ files smoothly
- [ ] Sub-second graph rendering
- [ ] Interactive operations < 100ms
- [ ] Integration with Context tab functional
- [ ] All graph types implemented
- [ ] Export functionality working
- [ ] Tests passing with > 80% coverage

---

## 🎉 Completion Criteria

The Graph tab is considered complete when:

1. All four graph types are functional (dependencies, symbols, modules, spec-mapping)
2. Performance targets are met for large codebases
3. Context integration is working bidirectionally
4. Export functionality is implemented
5. All tests are passing
6. Documentation is complete
7. UI is polished with proper loading states and error handling

---

*Last Updated: [Current Date]*
*Status: Ready for Implementation*