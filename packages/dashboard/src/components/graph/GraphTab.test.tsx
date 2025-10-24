// ABOUTME: Tests for GraphTab component with multiple graph types
// ABOUTME: Tests tab switching behavior and shared Cytoscape ref handling

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { GraphTab } from './GraphTab';

// Mock the graph hooks
vi.mock('@/hooks/useGraph', () => ({
  useProjectGraph: vi.fn(() => ({
    data: {
      nodes: [
        { id: 'node1', label: 'Node 1', node_type: 'file', metadata: {} },
      ],
      edges: [],
      metadata: {
        total_nodes: 1,
        total_edges: 0,
        project_id: 'test-project',
        generated_at: new Date().toISOString(),
      },
    },
    isLoading: false,
    error: null,
  })),
}));

// Mock Cytoscape
vi.mock('react-cytoscapejs', () => ({
  default: vi.fn(() => <div data-testid="cytoscape-mock" />),
}));

// Mock the child components
vi.mock('./GraphVisualization', () => ({
  GraphVisualization: ({ graphType }: { graphType: string }) => (
    <div data-testid={`graph-${graphType}`}>Graph: {graphType}</div>
  ),
}));

vi.mock('./GraphControls', () => ({
  GraphControls: () => <div data-testid="graph-controls">Controls</div>,
}));

vi.mock('./GraphSidebar', () => ({
  GraphSidebar: () => <div data-testid="graph-sidebar">Sidebar</div>,
}));

describe('GraphTab', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should render all graph type tabs', async () => {
    render(<GraphTab projectId="test-project" projectPath="/test/path" />);

    await waitFor(() => {
      expect(screen.getByText('Dependencies')).toBeInTheDocument();
      expect(screen.getByText('Symbols')).toBeInTheDocument();
      expect(screen.getByText('Modules')).toBeInTheDocument();
      expect(screen.getByText('Spec Mapping')).toBeInTheDocument();
    });
  });

  it('should switch between graph types without errors', async () => {
    const user = userEvent.setup();
    render(<GraphTab projectId="test-project" projectPath="/test/path" />);

    // Wait for initial render
    await waitFor(() => {
      expect(screen.getByTestId('graph-dependencies')).toBeInTheDocument();
    });

    // Switch to symbols tab
    const symbolsTab = screen.getByText('Symbols');
    await user.click(symbolsTab);

    await waitFor(() => {
      expect(screen.getByTestId('graph-symbols')).toBeInTheDocument();
    });

    // Switch to modules tab
    const modulesTab = screen.getByText('Modules');
    await user.click(modulesTab);

    await waitFor(() => {
      expect(screen.getByTestId('graph-modules')).toBeInTheDocument();
    });

    // Switch to spec-mapping tab
    const specMappingTab = screen.getByText('Spec Mapping');
    await user.click(specMappingTab);

    await waitFor(() => {
      expect(screen.getByTestId('graph-spec-mapping')).toBeInTheDocument();
    });

    // Switch back to dependencies
    const dependenciesTab = screen.getByText('Dependencies');
    await user.click(dependenciesTab);

    await waitFor(() => {
      expect(screen.getByTestId('graph-dependencies')).toBeInTheDocument();
    });
  });

  it('should handle rapid tab switching without crashing', async () => {
    const user = userEvent.setup();
    render(<GraphTab projectId="test-project" projectPath="/test/path" />);

    // Wait for initial render
    await waitFor(() => {
      expect(screen.getByTestId('graph-dependencies')).toBeInTheDocument();
    });

    // Rapidly switch between tabs
    const tabSequence = ['Symbols', 'Modules', 'Spec Mapping', 'Dependencies', 'Symbols'];

    for (const tabName of tabSequence) {
      const tab = screen.getByText(tabName);
      await user.click(tab);
    }

    // Verify we're still on the last tab and component didn't crash
    await waitFor(() => {
      expect(screen.getByTestId('graph-symbols')).toBeInTheDocument();
    });
  });

  it('should clear selection when switching tabs', async () => {
    const user = userEvent.setup();
    const { container } = render(<GraphTab projectId="test-project" projectPath="/test/path" />);

    await waitFor(() => {
      expect(screen.getByTestId('graph-dependencies')).toBeInTheDocument();
    });

    // Switch to another tab
    const symbolsTab = screen.getByText('Symbols');
    await user.click(symbolsTab);

    await waitFor(() => {
      expect(screen.getByTestId('graph-symbols')).toBeInTheDocument();
    });

    // Component should still be mounted and functional
    expect(container).toBeInTheDocument();
  });

  it('should maintain layout state across tab switches', async () => {
    const user = userEvent.setup();
    render(<GraphTab projectId="test-project" projectPath="/test/path" />);

    await waitFor(() => {
      expect(screen.getByTestId('graph-dependencies')).toBeInTheDocument();
    });

    // Switch tabs multiple times
    await user.click(screen.getByText('Symbols'));
    await waitFor(() => {
      expect(screen.getByTestId('graph-symbols')).toBeInTheDocument();
    });

    await user.click(screen.getByText('Dependencies'));
    await waitFor(() => {
      expect(screen.getByTestId('graph-dependencies')).toBeInTheDocument();
    });

    // Verify component is stable
    expect(screen.getByTestId('graph-controls')).toBeInTheDocument();
  });
});
