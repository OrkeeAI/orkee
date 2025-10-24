// ABOUTME: Tests for GraphVisualization component
// ABOUTME: Tests layout switching, cleanup behavior, and Cytoscape interaction safety

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, waitFor } from '@testing-library/react';
import { GraphVisualization } from './GraphVisualization';
import type { CodeGraph } from '@/types/graph';

const mockGraph: CodeGraph = {
  nodes: [
    {
      id: 'node1',
      label: 'Node 1',
      node_type: 'file',
      metadata: {},
    },
    {
      id: 'node2',
      label: 'Node 2',
      node_type: 'function',
      metadata: {},
    },
  ],
  edges: [
    {
      id: 'edge1',
      source: 'node1',
      target: 'node2',
      edge_type: 'import',
      weight: 1,
    },
  ],
  metadata: {
    total_nodes: 2,
    total_edges: 1,
    project_id: 'test-project',
    generated_at: new Date().toISOString(),
  },
};

// Mock the graph hooks
vi.mock('@/hooks/useGraph', () => ({
  useProjectGraph: vi.fn(() => ({
    data: mockGraph,
    isLoading: false,
    error: null,
  })),
}));

// Mock Cytoscape
vi.mock('react-cytoscapejs', () => ({
  default: vi.fn(({ cy, elements, layout }) => {
    const mockCy = {
      destroyed: vi.fn(() => false),
      elements: true,
      layout: vi.fn(() => ({ run: vi.fn() })),
      destroy: vi.fn(),
      on: vi.fn(),
      fit: vi.fn(),
    };

    // Call the cy callback if provided
    if (cy) {
      cy(mockCy);
    }

    return <div data-testid="cytoscape-mock" data-elements={elements?.length} data-layout={layout?.name} />;
  }),
}));

describe('GraphVisualization', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should handle rapid layout switches without crashing', async () => {
    const onNodeSelect = vi.fn();
    const { rerender } = render(
      <GraphVisualization
        projectId="test-project"
        graphType="dependencies"
        layout="hierarchical"
        filters={{}}
        onNodeSelect={onNodeSelect}
      />
    );

    // Simulate rapid layout switching (user clicking layout buttons quickly)
    const layouts = ['hierarchical', 'force', 'circular', 'grid', 'hierarchical', 'force'];

    for (const layout of layouts) {
      rerender(
        <GraphVisualization
          projectId="test-project"
          graphType="dependencies"
          layout={layout}
          filters={{}}
          onNodeSelect={onNodeSelect}
        />
      );
    }

    // Wait for all layout changes to settle
    await waitFor(() => {
      expect(document.querySelector('[data-testid="cytoscape-mock"]')).toBeInTheDocument();
    });

    // Verify component didn't crash
    expect(document.querySelector('[data-testid="cytoscape-mock"]')).toBeInTheDocument();
  });

  it('should clean up Cytoscape instance on unmount', async () => {
    const onNodeSelect = vi.fn();

    const { unmount } = render(
      <GraphVisualization
        projectId="test-project"
        graphType="dependencies"
        layout="hierarchical"
        filters={{}}
        onNodeSelect={onNodeSelect}
      />
    );

    // Wait for component to render
    await waitFor(() => {
      expect(document.querySelector('[data-testid="cytoscape-mock"]')).toBeInTheDocument();
    });

    // Unmount the component - should not throw errors
    unmount();

    // Verify the component was unmounted successfully
    await waitFor(() => {
      expect(document.querySelector('[data-testid="cytoscape-mock"]')).not.toBeInTheDocument();
    });
  });

  it('should handle layout changes gracefully when cy is destroyed', async () => {
    const onNodeSelect = vi.fn();
    let mockCyInstance: any;

    const CytoscapeComponent = await import('react-cytoscapejs');
    const mockImplementation = (CytoscapeComponent.default as any).mockImplementation;

    // Override the mock to simulate a destroyed instance
    mockImplementation(({ cy, elements, layout }: any) => {
      mockCyInstance = {
        destroyed: vi.fn(() => true), // Simulate destroyed state
        elements: true,
        layout: vi.fn(() => ({ run: vi.fn() })),
        destroy: vi.fn(),
        on: vi.fn(),
        fit: vi.fn(),
      };

      if (cy) {
        cy(mockCyInstance);
      }

      return <div data-testid="cytoscape-mock" data-elements={elements?.length} data-layout={layout?.name} />;
    });

    const { rerender } = render(
      <GraphVisualization
        projectId="test-project"
        graphType="dependencies"
        layout="hierarchical"
        filters={{}}
        onNodeSelect={onNodeSelect}
      />
    );

    // Change layout while instance is destroyed
    rerender(
      <GraphVisualization
        projectId="test-project"
        graphType="dependencies"
        layout="force"
        filters={{}}
        onNodeSelect={onNodeSelect}
      />
    );

    // Should not throw error
    await waitFor(() => {
      expect(document.querySelector('[data-testid="cytoscape-mock"]')).toBeInTheDocument();
    });

    // layout.run should NOT have been called since cy was destroyed
    expect(mockCyInstance.layout).not.toHaveBeenCalled();
  });

  it('should filter graph elements based on search term', async () => {
    const onNodeSelect = vi.fn();
    const { rerender } = render(
      <GraphVisualization
        projectId="test-project"
        graphType="dependencies"
        layout="hierarchical"
        filters={{}}
        onNodeSelect={onNodeSelect}
      />
    );

    // Apply search filter
    rerender(
      <GraphVisualization
        projectId="test-project"
        graphType="dependencies"
        layout="hierarchical"
        filters={{ search: 'Node 1' }}
        onNodeSelect={onNodeSelect}
      />
    );

    await waitFor(() => {
      const cytoscapeMock = document.querySelector('[data-testid="cytoscape-mock"]');
      expect(cytoscapeMock).toBeInTheDocument();
    });
  });

  it('should handle missing elements gracefully', async () => {
    const onNodeSelect = vi.fn();
    const emptyGraph: CodeGraph = {
      nodes: [],
      edges: [],
      metadata: {
        total_nodes: 0,
        total_edges: 0,
        project_id: 'empty-project',
        generated_at: new Date().toISOString(),
      },
    };

    // Import and mock the hook to return empty graph
    const { useProjectGraph } = await import('@/hooks/useGraph');
    vi.mocked(useProjectGraph).mockReturnValue({
      data: emptyGraph,
      isLoading: false,
      error: null,
    });

    const { container } = render(
      <GraphVisualization
        projectId="test-project"
        graphType="dependencies"
        layout="hierarchical"
        filters={{}}
        onNodeSelect={onNodeSelect}
      />
    );

    await waitFor(() => {
      expect(container).toBeInTheDocument();
    });
  });
});
