// ABOUTME: Tests for AITestDialog component
// ABOUTME: Validates AI integration testing with PRD analysis and full workflow functionality

import React from 'react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { AITestDialog } from './AITestDialog';

// Mock hooks with configurable state
const mockAnalyzePRD = vi.fn();
const mockPRDWorkflow = vi.fn();
const mockAnalyzePRDState = {
  isPending: false,
  data: null as any,
  error: null as any,
};
const mockPRDWorkflowState = {
  isPending: false,
  data: null as any,
  error: null as any,
};

vi.mock('@/hooks/useAI', () => ({
  useAIConfiguration: () => ({
    data: {
      isConfigured: true,
      preferredProvider: 'anthropic',
      openaiConfigured: false,
      anthropicConfigured: true,
    },
  }),
  useAnalyzePRD: () => ({
    mutate: mockAnalyzePRD,
    isPending: mockAnalyzePRDState.isPending,
    data: mockAnalyzePRDState.data,
    error: mockAnalyzePRDState.error,
  }),
  usePRDWorkflow: () => ({
    mutate: mockPRDWorkflow,
    isPending: mockPRDWorkflowState.isPending,
    data: mockPRDWorkflowState.data,
    error: mockPRDWorkflowState.error,
  }),
}));

// Mock Dialog components
vi.mock('@/components/ui/dialog', () => ({
  Dialog: ({ children, open }: { children: React.ReactNode; open: boolean }) =>
    open ? <div data-testid="dialog">{children}</div> : null,
  DialogContent: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
  DialogDescription: ({ children }: { children: React.ReactNode }) => <p>{children}</p>,
  DialogHeader: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
  DialogTitle: ({ children }: { children: React.ReactNode }) => <h2>{children}</h2>,
}));

// Mock Textarea component
vi.mock('@/components/ui/textarea', () => ({
  Textarea: (props: any) => <textarea {...props} />,
}));

// Mock Label component
vi.mock('@/components/ui/label', () => ({
  Label: ({ children, htmlFor }: { children: React.ReactNode; htmlFor?: string }) => (
    <label htmlFor={htmlFor}>{children}</label>
  ),
}));

// Mock Button component
vi.mock('@/components/ui/button', () => ({
  Button: ({ children, onClick, disabled }: any) => (
    <button onClick={onClick} disabled={disabled}>
      {children}
    </button>
  ),
}));

// Mock Tabs components
vi.mock('@/components/ui/tabs', () => ({
  Tabs: ({ children, defaultValue }: any) => <div data-default-tab={defaultValue}>{children}</div>,
  TabsContent: ({ children, value }: { children: React.ReactNode; value: string }) => (
    <div data-tab-content={value}>{children}</div>
  ),
  TabsList: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
  TabsTrigger: ({ children, value }: any) => <button data-tab={value}>{children}</button>,
}));

// Mock Alert components
vi.mock('@/components/ui/alert', () => ({
  Alert: ({ children }: { children: React.ReactNode }) => <div role="alert">{children}</div>,
  AlertDescription: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
}));

// Mock Progress component
vi.mock('@/components/ui/progress', () => ({
  Progress: ({ value }: { value: number }) => <div role="progressbar" aria-valuenow={value} />,
}));

// Mock lucide-react icons
vi.mock('lucide-react', () => ({
  Sparkles: () => <span>Sparkles</span>,
  Loader2: ({ className }: { className?: string }) => <span className={className}>Loader2</span>,
  CheckCircle: () => <span>CheckCircle</span>,
  XCircle: () => <span>XCircle</span>,
  DollarSign: () => <span>DollarSign</span>,
}));

describe('AITestDialog', () => {
  const defaultProps = {
    open: true,
    onOpenChange: vi.fn(),
  };

  beforeEach(() => {
    vi.clearAllMocks();
    mockAnalyzePRDState.isPending = false;
    mockAnalyzePRDState.data = null;
    mockAnalyzePRDState.error = null;
    mockPRDWorkflowState.isPending = false;
    mockPRDWorkflowState.data = null;
    mockPRDWorkflowState.error = null;
  });

  describe('Initial render', () => {
    it('should render dialog when open', () => {
      render(<AITestDialog {...defaultProps} />);

      expect(screen.getByTestId('dialog')).toBeInTheDocument();
      expect(screen.getByText('AI Integration Test')).toBeInTheDocument();
    });

    it('should not render dialog when closed', () => {
      render(<AITestDialog {...defaultProps} open={false} />);

      expect(screen.queryByTestId('dialog')).not.toBeInTheDocument();
    });

    it('should render description', () => {
      render(<AITestDialog {...defaultProps} />);

      expect(
        screen.getByText('Test AI integration with PRD analysis and spec generation.')
      ).toBeInTheDocument();
    });

    it('should render two tabs', () => {
      render(<AITestDialog {...defaultProps} />);

      expect(screen.getByText('Quick Analysis')).toBeInTheDocument();
      expect(screen.getByText('Full Workflow')).toBeInTheDocument();
    });
  });

  describe('AI Configuration status', () => {
    it('should show configured status when AI is configured', () => {
      render(<AITestDialog {...defaultProps} />);

      expect(screen.getByText('✓ AI is configured')).toBeInTheDocument();
      expect(screen.getByText(/Anthropic Claude/)).toBeInTheDocument();
    });

    it('should show provider information', () => {
      render(<AITestDialog {...defaultProps} />);

      expect(screen.getByText(/Provider: Anthropic Claude/)).toBeInTheDocument();
    });

    it('should show alert with configuration status', () => {
      render(<AITestDialog {...defaultProps} />);

      const alerts = screen.getAllByRole('alert');
      expect(alerts.length).toBeGreaterThan(0);
    });
  });

  describe('Default PRD content', () => {
    it('should have default PRD content pre-filled', () => {
      render(<AITestDialog {...defaultProps} />);

      const textarea = screen.getAllByLabelText('PRD Content')[0] as HTMLTextAreaElement;
      expect(textarea.value).toContain('User Authentication System');
      expect(textarea.value).toContain('OAuth login with Google and GitHub');
    });

    it('should allow editing PRD content', () => {
      render(<AITestDialog {...defaultProps} />);

      const textarea = screen.getAllByLabelText('PRD Content')[0];
      fireEvent.change(textarea, { target: { value: '# New Content' } });

      expect((textarea as HTMLTextAreaElement).value).toBe('# New Content');
    });
  });

  describe('Quick Analysis tab', () => {
    it('should render analyze button', () => {
      render(<AITestDialog {...defaultProps} />);

      expect(screen.getByText('Analyze PRD')).toBeInTheDocument();
    });

    it('should trigger analysis on button click', () => {
      render(<AITestDialog {...defaultProps} />);

      const analyzeButton = screen.getByText('Analyze PRD');
      fireEvent.click(analyzeButton);

      expect(mockAnalyzePRD).toHaveBeenCalled();
    });

    it('should disable analyze button when content is empty', () => {
      render(<AITestDialog {...defaultProps} />);

      const textarea = screen.getAllByLabelText('PRD Content')[0];
      fireEvent.change(textarea, { target: { value: '' } });

      const analyzeButton = screen.getByText('Analyze PRD');
      expect(analyzeButton).toBeDisabled();
    });

    it('should disable analyze button when content is only whitespace', () => {
      render(<AITestDialog {...defaultProps} />);

      const textarea = screen.getAllByLabelText('PRD Content')[0];
      fireEvent.change(textarea, { target: { value: '   \n\n  ' } });

      const analyzeButton = screen.getByText('Analyze PRD');
      expect(analyzeButton).toBeDisabled();
    });
  });

  describe('Full Workflow tab', () => {
    it('should render workflow button', () => {
      render(<AITestDialog {...defaultProps} />);

      expect(screen.getByText('Run Full PRD → Spec → Task Workflow')).toBeInTheDocument();
    });

    it('should trigger workflow on button click', () => {
      render(<AITestDialog {...defaultProps} />);

      const workflowButton = screen.getByText('Run Full PRD → Spec → Task Workflow');
      fireEvent.click(workflowButton);

      expect(mockPRDWorkflow).toHaveBeenCalled();
    });

    it('should pass content to workflow', () => {
      render(<AITestDialog {...defaultProps} />);

      const textarea = screen.getAllByLabelText('PRD Content')[1];
      fireEvent.change(textarea, { target: { value: '# Workflow Test' } });

      const workflowButton = screen.getByText('Run Full PRD → Spec → Task Workflow');
      fireEvent.click(workflowButton);

      expect(mockPRDWorkflow).toHaveBeenCalledWith(
        expect.objectContaining({
          prdContent: '# Workflow Test',
        })
      );
    });

    it('should disable workflow button when content is empty', () => {
      render(<AITestDialog {...defaultProps} />);

      const textarea = screen.getAllByLabelText('PRD Content')[1];
      fireEvent.change(textarea, { target: { value: '' } });

      const workflowButton = screen.getByText('Run Full PRD → Spec → Task Workflow');
      expect(workflowButton).toBeDisabled();
    });
  });

  describe('Analysis results display', () => {
    it('should display analysis results when available', () => {
      mockAnalyzePRDState.data = {
        data: {
          summary: 'Authentication system with OAuth support',
          capabilities: [
            {
              id: 'cap-1',
              name: 'User Registration',
              purpose: 'Allow new users to register',
              requirements: [
                { scenarios: ['Valid email', 'Invalid email'] },
                { scenarios: ['Password strength'] },
              ],
            },
          ],
          dependencies: ['bcrypt', 'jsonwebtoken'],
        },
        cost: {
          estimatedCost: 0.0234,
        },
        usage: {
          totalTokens: 1500,
        },
      };

      render(<AITestDialog {...defaultProps} />);

      expect(screen.getByText('Analysis Results')).toBeInTheDocument();
      expect(screen.getByText('Authentication system with OAuth support')).toBeInTheDocument();
      expect(screen.getByText('User Registration')).toBeInTheDocument();
      expect(screen.getByText('$0.0234')).toBeInTheDocument();
      expect(screen.getByText('(1500 tokens)')).toBeInTheDocument();
    });

    it('should display capabilities count', () => {
      mockAnalyzePRDState.data = {
        data: {
          summary: 'Summary',
          capabilities: [
            { id: 'cap-1', name: 'Cap 1', purpose: 'Purpose 1', requirements: [] },
            { id: 'cap-2', name: 'Cap 2', purpose: 'Purpose 2', requirements: [] },
            { id: 'cap-3', name: 'Cap 3', purpose: 'Purpose 3', requirements: [] },
          ],
          dependencies: [],
        },
        cost: { estimatedCost: 0 },
        usage: { totalTokens: 0 },
      };

      render(<AITestDialog {...defaultProps} />);

      expect(screen.getByText('Capabilities (3)')).toBeInTheDocument();
    });

    it('should display scenario count across requirements', () => {
      mockAnalyzePRDState.data = {
        data: {
          summary: 'Summary',
          capabilities: [
            {
              id: 'cap-1',
              name: 'Feature',
              purpose: 'Purpose',
              requirements: [{ scenarios: ['S1', 'S2'] }, { scenarios: ['S3', 'S4', 'S5'] }],
            },
          ],
          dependencies: [],
        },
        cost: { estimatedCost: 0 },
        usage: { totalTokens: 0 },
      };

      render(<AITestDialog {...defaultProps} />);

      // 2 + 3 = 5 scenarios
      expect(screen.getByText(/5 scenarios/)).toBeInTheDocument();
    });

    it('should display dependencies when present', () => {
      mockAnalyzePRDState.data = {
        data: {
          summary: 'Summary',
          capabilities: [],
          dependencies: ['react', 'express', 'postgres'],
        },
        cost: { estimatedCost: 0 },
        usage: { totalTokens: 0 },
      };

      render(<AITestDialog {...defaultProps} />);

      expect(screen.getByText('Dependencies')).toBeInTheDocument();
      expect(screen.getByText('react, express, postgres')).toBeInTheDocument();
    });

    it('should not display dependencies section when empty', () => {
      mockAnalyzePRDState.data = {
        data: {
          summary: 'Summary',
          capabilities: [],
          dependencies: [],
        },
        cost: { estimatedCost: 0 },
        usage: { totalTokens: 0 },
      };

      render(<AITestDialog {...defaultProps} />);

      expect(screen.queryByText('Dependencies')).not.toBeInTheDocument();
    });
  });

  describe('Workflow results display', () => {
    it('should display workflow completion status', () => {
      mockPRDWorkflowState.data = {
        capabilities: [{ id: 'cap-1', capability: { id: 'c1', name: 'Cap 1' }, specMarkdown: '' }],
        suggestedTasks: [],
        totalCost: 0.05,
        steps: ['Step 1', 'Step 2'],
      };

      render(<AITestDialog {...defaultProps} />);

      expect(screen.getByText('Workflow Complete')).toBeInTheDocument();
      expect(screen.getByText('CheckCircle')).toBeInTheDocument();
    });

    it('should display workflow statistics', () => {
      mockPRDWorkflowState.data = {
        capabilities: [
          { id: 'cap-1', capability: { id: 'c1', name: 'Cap 1' }, specMarkdown: '' },
          { id: 'cap-2', capability: { id: 'c2', name: 'Cap 2' }, specMarkdown: '' },
        ],
        suggestedTasks: [
          { title: 'Task 1', description: 'Desc 1', complexity: 5, capabilityId: 'c1' },
          { title: 'Task 2', description: 'Desc 2', complexity: 3, capabilityId: 'c1' },
          { title: 'Task 3', description: 'Desc 3', complexity: 7, capabilityId: 'c2' },
        ],
        totalCost: 0.0875,
        steps: ['Analyze', 'Generate', 'Suggest'],
      };

      render(<AITestDialog {...defaultProps} />);

      // Check for capabilities and tasks count in statistics section
      const capabilitiesCount = screen.getAllByText('2');
      expect(capabilitiesCount.length).toBeGreaterThan(0);
      const tasksCount = screen.getAllByText('3');
      expect(tasksCount.length).toBeGreaterThan(0);
      expect(screen.getByText('$0.0875')).toBeInTheDocument(); // Cost
    });

    it('should show workflow progress when running', () => {
      mockPRDWorkflowState.isPending = true;

      render(<AITestDialog {...defaultProps} />);

      expect(screen.getByText('Running Workflow...')).toBeInTheDocument();
    });
  });

  describe('Error handling', () => {
    it('should display analysis error', () => {
      mockAnalyzePRDState.error = { message: 'API key invalid' };

      render(<AITestDialog {...defaultProps} />);

      expect(screen.getByText('Error: API key invalid')).toBeInTheDocument();
      expect(screen.getByText('XCircle')).toBeInTheDocument();
    });

    it('should display workflow error', () => {
      mockPRDWorkflowState.error = { message: 'Workflow failed at step 2' };

      render(<AITestDialog {...defaultProps} />);

      expect(screen.getByText('Error: Workflow failed at step 2')).toBeInTheDocument();
    });
  });

  describe('Cost formatting', () => {
    it('should format costs with 4 decimal places', () => {
      mockAnalyzePRDState.data = {
        data: { summary: '', capabilities: [], dependencies: [] },
        cost: { estimatedCost: 0.123456 },
        usage: { totalTokens: 1000 },
      };

      render(<AITestDialog {...defaultProps} />);

      expect(screen.getByText('$0.1235')).toBeInTheDocument();
    });
  });
});
