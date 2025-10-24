// ABOUTME: Reusable Cytoscape graph visualization component
// ABOUTME: Used for dependency, symbol, module, and spec-mapping graphs

import { useEffect, useRef, useMemo } from 'react';
import CytoscapeComponent from 'react-cytoscapejs';
import cytoscape from 'cytoscape';
import fcose from 'cytoscape-fcose';
import dagre from 'cytoscape-dagre';
import { useProjectGraph } from '@/hooks/useGraph';
import { Card, CardContent } from '@/components/ui/card';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { AlertCircle } from 'lucide-react';
import type { GraphType } from '@/services/graph';
import type { GraphFilters } from './DependencyGraph';

// Register layout extensions
cytoscape.use(fcose);
cytoscape.use(dagre);

interface GraphVisualizationProps {
  projectId: string;
  graphType: GraphType;
  layout: string;
  filters: GraphFilters;
  onNodeSelect: (nodeId: string) => void;
  cytoscapeRef?: React.MutableRefObject<cytoscape.Core | null>;
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
        rankSep: 100,
      };
    case 'force':
      return {
        name: 'fcose',
        animate: true,
        animationDuration: 500,
        nodeRepulsion: 4500,
        idealEdgeLength: 50,
      };
    case 'circular':
      return {
        name: 'circle',
        animate: true,
        animationDuration: 500,
      };
    case 'grid':
      return {
        name: 'grid',
        animate: true,
        animationDuration: 500,
      };
    default:
      return {
        name: 'dagre',
        rankDir: 'TB',
      };
  }
}

export function GraphVisualization({
  projectId,
  graphType,
  layout,
  filters,
  onNodeSelect,
  cytoscapeRef: externalCyRef,
}: GraphVisualizationProps) {
  const internalCyRef = useRef<cytoscape.Core | null>(null);
  const cyRef = externalCyRef || internalCyRef;

  const { data: graph, isLoading, error } = useProjectGraph(projectId, graphType, {
    max_depth: filters.max_depth,
    filter: filters.filter,
  });

  const elements = useMemo(() => {
    if (!graph) return [];

    let nodes = graph.nodes.map((node) => ({
      data: {
        id: node.id,
        label: node.label,
        type: node.node_type,
        ...node.metadata,
      },
      classes: node.node_type,
    }));

    let edges = graph.edges.map((edge) => ({
      data: {
        id: edge.id,
        source: edge.source,
        target: edge.target,
        type: edge.edge_type,
        weight: edge.weight,
      },
      classes: edge.edge_type,
    }));

    // Apply search filter
    if (filters.search) {
      const searchLower = filters.search.toLowerCase();
      const matchingNodeIds = new Set(
        nodes
          .filter((node) => node.data.label.toLowerCase().includes(searchLower))
          .map((node) => node.data.id)
      );

      // Include connected nodes
      const connectedNodeIds = new Set(matchingNodeIds);
      edges.forEach((edge) => {
        if (matchingNodeIds.has(edge.data.source) || matchingNodeIds.has(edge.data.target)) {
          connectedNodeIds.add(edge.data.source);
          connectedNodeIds.add(edge.data.target);
        }
      });

      nodes = nodes.filter((node) => connectedNodeIds.has(node.data.id));
      edges = edges.filter(
        (edge) =>
          connectedNodeIds.has(edge.data.source) && connectedNodeIds.has(edge.data.target)
      );
    }

    return [...nodes, ...edges];
  }, [graph, filters.search]);

  const stylesheet: cytoscape.Stylesheet[] = [
    {
      selector: 'node',
      style: {
        'background-color': '#666',
        label: 'data(label)',
        'text-valign': 'center',
        'text-halign': 'center',
        'font-size': '12px',
        width: 30,
        height: 30,
        'text-wrap': 'wrap',
        'text-max-width': '100px',
      },
    },
    {
      selector: 'node.file',
      style: {
        'background-color': '#4F46E5',
        shape: 'round-rectangle',
      },
    },
    {
      selector: 'node.function',
      style: {
        'background-color': '#10B981',
        shape: 'ellipse',
      },
    },
    {
      selector: 'node.class',
      style: {
        'background-color': '#F59E0B',
        shape: 'diamond',
      },
    },
    {
      selector: 'node.module',
      style: {
        'background-color': '#8B5CF6',
        shape: 'round-rectangle',
      },
    },
    {
      selector: 'node.spec',
      style: {
        'background-color': '#EC4899',
        shape: 'round-rectangle',
      },
    },
    {
      selector: 'node.requirement',
      style: {
        'background-color': '#F472B6',
        shape: 'hexagon',
      },
    },
    {
      selector: 'edge',
      style: {
        width: 2,
        'line-color': '#ccc',
        'target-arrow-color': '#ccc',
        'target-arrow-shape': 'triangle',
        'curve-style': 'bezier',
      },
    },
    {
      selector: 'edge.import',
      style: {
        'line-color': '#4F46E5',
        'target-arrow-color': '#4F46E5',
      },
    },
    {
      selector: 'edge.export',
      style: {
        'line-color': '#10B981',
        'target-arrow-color': '#10B981',
        'line-style': 'dashed',
      },
    },
    {
      selector: 'edge.reference',
      style: {
        'line-color': '#F59E0B',
        'target-arrow-color': '#F59E0B',
        'line-style': 'dashed',
      },
    },
    {
      selector: 'edge.dependency',
      style: {
        'line-color': '#8B5CF6',
        'target-arrow-color': '#8B5CF6',
      },
    },
    {
      selector: 'edge.implementation',
      style: {
        'line-color': '#EC4899',
        'target-arrow-color': '#EC4899',
        'line-style': 'dotted',
      },
    },
    {
      selector: 'edge.contains',
      style: {
        'line-color': '#64748b',
        'target-arrow-color': '#64748b',
        width: 3,
      },
    },
    {
      selector: ':selected',
      style: {
        'background-color': '#EF4444',
        'line-color': '#EF4444',
        'target-arrow-color': '#EF4444',
        'border-width': 3,
        'border-color': '#EF4444',
      },
    },
    {
      selector: 'node:active',
      style: {
        'overlay-color': '#EF4444',
        'overlay-padding': 5,
        'overlay-opacity': 0.25,
      },
    },
  ];

  const layoutConfig = useMemo(() => getLayoutOptions(layout), [layout]);

  // Re-run layout when layout type changes
  useEffect(() => {
    if (cyRef.current) {
      const cy = cyRef.current;
      cy.layout(layoutConfig).run();
    }
  }, [layoutConfig, cyRef]);

  if (isLoading) {
    return (
      <Card className="flex-1">
        <CardContent className="p-0 h-[600px] flex items-center justify-center">
          <div className="flex flex-col items-center gap-2">
            <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary" />
            <p className="text-sm text-muted-foreground">Loading {graphType} graph...</p>
          </div>
        </CardContent>
      </Card>
    );
  }

  if (error) {
    return (
      <Card className="flex-1">
        <CardContent className="p-0 h-[600px] flex items-center justify-center">
          <Alert variant="destructive" className="max-w-md">
            <AlertCircle className="h-4 w-4" />
            <AlertDescription>
              Failed to load graph: {error.message}
            </AlertDescription>
          </Alert>
        </CardContent>
      </Card>
    );
  }

  if (!graph || elements.length === 0) {
    return (
      <Card className="flex-1">
        <CardContent className="p-0 h-[600px] flex items-center justify-center">
          <div className="text-center">
            <p className="text-sm text-muted-foreground">
              No {graphType} data available for this project
            </p>
          </div>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card className="flex-1">
      <CardContent className="p-0 h-[600px]">
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

            cy.on('mouseover', 'node', () => {
              document.body.style.cursor = 'pointer';
            });

            cy.on('mouseout', 'node', () => {
              document.body.style.cursor = 'default';
            });

            // Double-click to fit
            cy.on('dblclick', (evt) => {
              if (evt.target === cy) {
                cy.fit(undefined, 50);
              }
            });
          }}
        />
      </CardContent>
    </Card>
  );
}
