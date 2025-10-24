// ABOUTME: Main Graph tab component for code visualization
// ABOUTME: Provides tabbed interface for dependency, symbol, module, and spec-mapping graphs

import { useState, useRef } from 'react';
import cytoscape from 'cytoscape';
import { Card, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Network, GitBranch, Layers, FileCode } from 'lucide-react';
import { GraphVisualization } from './GraphVisualization';
import { GraphControls } from './GraphControls';
import { GraphSidebar } from './GraphSidebar';
import { useProjectGraph } from '@/hooks/useGraph';
import type { GraphFilters } from './DependencyGraph';
import type { GraphType } from '@/services/graph';

interface GraphTabProps {
  projectId: string;
  projectPath: string;
}

export function GraphTab({ projectId }: GraphTabProps) {
  const [selectedTab, setSelectedTab] = useState<GraphType>('dependencies');
  const [selectedNode, setSelectedNode] = useState<string | null>(null);
  const [layout, setLayout] = useState<string>('hierarchical');
  const [filters, setFilters] = useState<GraphFilters>({});
  const cytoscapeRef = useRef<cytoscape.Core | null>(null);

  // Fetch current graph data for sidebar
  const { data: currentGraphData } = useProjectGraph(projectId, selectedTab, {
    max_depth: filters.max_depth,
    filter: filters.filter,
  });

  return (
    <div className="flex flex-col gap-4">
      {/* Header */}
      <Card>
        <CardHeader>
          <div className="flex items-center gap-2">
            <Network className="h-5 w-5" />
            <CardTitle>Code Graph Visualization</CardTitle>
          </div>
          <CardDescription>
            Explore your codebase structure through interactive dependency, symbol, module, and spec-mapping graphs
          </CardDescription>
        </CardHeader>
      </Card>

      {/* Controls */}
      <GraphControls
        layout={layout}
        onLayoutChange={setLayout}
        filters={filters}
        onFiltersChange={setFilters}
        cytoscapeRef={cytoscapeRef}
      />

      {/* Graph Tabs with Sidebar */}
      <div className="flex gap-4">
        <div className="flex-1">
          <Tabs
            value={selectedTab}
            onValueChange={(value) => {
              setSelectedTab(value as GraphType);
              setSelectedNode(null); // Clear selection when switching tabs
            }}
            className="flex-1"
          >
            <TabsList className="grid w-full grid-cols-4">
              <TabsTrigger value="dependencies" className="flex items-center gap-2">
                <GitBranch className="h-4 w-4" />
                Dependencies
              </TabsTrigger>
              <TabsTrigger value="symbols" className="flex items-center gap-2">
                <FileCode className="h-4 w-4" />
                Symbols
              </TabsTrigger>
              <TabsTrigger value="modules" className="flex items-center gap-2">
                <Layers className="h-4 w-4" />
                Modules
              </TabsTrigger>
              <TabsTrigger value="spec-mapping" className="flex items-center gap-2">
                <Network className="h-4 w-4" />
                Spec Mapping
              </TabsTrigger>
            </TabsList>

            <TabsContent value="dependencies" className="mt-4">
              <GraphVisualization
                projectId={projectId}
                graphType="dependencies"
                layout={layout}
                filters={filters}
                onNodeSelect={setSelectedNode}
                cytoscapeRef={cytoscapeRef}
              />
            </TabsContent>

            <TabsContent value="symbols" className="mt-4">
              <GraphVisualization
                projectId={projectId}
                graphType="symbols"
                layout={layout}
                filters={filters}
                onNodeSelect={setSelectedNode}
                cytoscapeRef={cytoscapeRef}
              />
            </TabsContent>

            <TabsContent value="modules" className="mt-4">
              <GraphVisualization
                projectId={projectId}
                graphType="modules"
                layout={layout}
                filters={filters}
                onNodeSelect={setSelectedNode}
                cytoscapeRef={cytoscapeRef}
              />
            </TabsContent>

            <TabsContent value="spec-mapping" className="mt-4">
              <GraphVisualization
                projectId={projectId}
                graphType="spec-mapping"
                layout={layout}
                filters={filters}
                onNodeSelect={setSelectedNode}
                cytoscapeRef={cytoscapeRef}
              />
            </TabsContent>
          </Tabs>
        </div>

        {/* Sidebar */}
        {selectedNode && (
          <GraphSidebar
            nodeId={selectedNode}
            graphData={currentGraphData}
            onClose={() => setSelectedNode(null)}
          />
        )}
      </div>
    </div>
  );
}
