// ABOUTME: Tests for ScenarioTestRunner component
// ABOUTME: Validates scenario testing functionality with implementation validation and result display

import React from 'react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { ScenarioTestRunner } from './ScenarioTestRunner';

// Mock useMutation
const mockMutate = vi.fn();
const mockMutationState = {
  mutate: mockMutate,
  isPending: false,
  isError: false,
  isSuccess: false,
  error: null as any,
  data: null as any,
};

vi.mock('@tanstack/react-query', () => ({
  useMutation: (options: any) => {
    // Store the mutationFn for testing
    (mockMutate as any).mutationFn = options.mutationFn;
    (mockMutate as any).onSuccess = options.onSuccess;
    return mockMutationState;
  },
}));

// Mock AI service
vi.mock('@/lib/ai/services', () => ({
  aiSpecService: {
    validateTaskCompletion: vi.fn(),
  },
}));

// Mock Card components
vi.mock('@/components/ui/card', () => ({
  Card: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
  CardContent: ({ children, className }: { children: React.ReactNode; className?: string }) => (
    <div className={className}>{children}</div>
  ),
  CardDescription: ({ children }: { children: React.ReactNode }) => <p>{children}</p>,
  CardHeader: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
  CardTitle: ({ children }: { children: React.ReactNode }) => <h3>{children}</h3>,
}));

// Mock Button component
vi.mock('@/components/ui/button', () => ({
  Button: ({ children, onClick, disabled, size }: any) => (
    <button onClick={onClick} disabled={disabled} data-size={size}>
      {children}
    </button>
  ),
}));

// Mock Textarea component
vi.mock('@/components/ui/textarea', () => ({
  Textarea: (props: any) => <textarea {...props} />,
}));

// Mock Alert components
vi.mock('@/components/ui/alert', () => ({
  Alert: ({ children, variant }: { children: React.ReactNode; variant?: string }) => (
    <div role="alert" data-variant={variant}>{children}</div>
  ),
  AlertDescription: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
}));

// Mock Badge component
vi.mock('@/components/ui/badge', () => ({
  Badge: ({ children, variant }: { children: React.ReactNode; variant?: string }) => (
    <span data-variant={variant}>{children}</span>
  ),
}));

// Mock ValidationResultsPanel
vi.mock('./ValidationResultsPanel', () => ({
  ValidationResultsPanel: ({ validation }: any) => (
    <div data-testid="validation-results">
      {validation ? JSON.stringify(validation) : null}
    </div>
  ),
}));

// Mock lucide-react icons
vi.mock('lucide-react', () => ({
  Play: () => <span>Play</span>,
  Loader2: ({ className }: { className?: string }) => <span className={className}>Loader2</span>,
  AlertCircle: () => <span>AlertCircle</span>,
  Info: () => <span>Info</span>,
}));

describe('ScenarioTestRunner', () => {
  const mockScenarios = [
    {
      name: 'Valid login',
      when: 'user enters valid credentials',
      then: 'system authenticates and redirects to dashboard',
      and: ['session is created', 'user preferences are loaded'],
    },
    {
      name: 'Invalid login',
      when: 'user enters invalid credentials',
      then: 'system shows error message',
    },
  ];

  const defaultProps = {
    taskId: 'task-123',
    taskTitle: 'Implement User Authentication',
    taskDescription: 'Users should be able to log in with email and password',
    scenarios: mockScenarios,
  };

  beforeEach(async () => {
    vi.clearAllMocks();
    mockMutationState.isPending = false;
    mockMutationState.isError = false;
    mockMutationState.isSuccess = false;
    mockMutationState.error = null;
    mockMutationState.data = null;

    // Import and mock the service
    const { aiSpecService } = await import('@/lib/ai/services');
    vi.mocked(aiSpecService.validateTaskCompletion).mockResolvedValue({
      data: { overallAssessment: { passed: true }, scenarioResults: [], recommendations: [] },
      cost: { estimatedCost: 0.05 },
    } as any);
  });

  describe('Initial render', () => {
    it('should render title and description', () => {
      render(<ScenarioTestRunner {...defaultProps} />);

      expect(screen.getByText('Scenario Test Runner')).toBeInTheDocument();
      expect(screen.getByText(/Test task implementation against 2 scenarios/)).toBeInTheDocument();
    });

    it('should display task information', () => {
      render(<ScenarioTestRunner {...defaultProps} />);

      expect(screen.getByText('Implement User Authentication')).toBeInTheDocument();
      expect(screen.getByText('Users should be able to log in with email and password')).toBeInTheDocument();
    });

    it('should display singular scenario text when one scenario', () => {
      render(
        <ScenarioTestRunner
          {...defaultProps}
          scenarios={[mockScenarios[0]]}
        />
      );

      expect(screen.getByText(/Test task implementation against 1 scenario/)).toBeInTheDocument();
    });
  });

  describe('Scenarios display', () => {
    it('should display all scenarios', () => {
      render(<ScenarioTestRunner {...defaultProps} />);

      expect(screen.getByText('Valid login')).toBeInTheDocument();
      expect(screen.getByText('Invalid login')).toBeInTheDocument();
    });

    it('should display WHEN clauses', () => {
      render(<ScenarioTestRunner {...defaultProps} />);

      expect(screen.getByText('user enters valid credentials')).toBeInTheDocument();
      expect(screen.getByText('user enters invalid credentials')).toBeInTheDocument();
    });

    it('should display THEN clauses', () => {
      render(<ScenarioTestRunner {...defaultProps} />);

      expect(screen.getByText('system authenticates and redirects to dashboard')).toBeInTheDocument();
      expect(screen.getByText('system shows error message')).toBeInTheDocument();
    });

    it('should display AND clauses when present', () => {
      render(<ScenarioTestRunner {...defaultProps} />);

      expect(screen.getByText('session is created')).toBeInTheDocument();
      expect(screen.getByText('user preferences are loaded')).toBeInTheDocument();
    });

    it('should not display AND clauses when not present', () => {
      render(
        <ScenarioTestRunner
          {...defaultProps}
          scenarios={[mockScenarios[1]]}
        />
      );

      const andClauses = screen.queryByText(/AND/);
      expect(andClauses).not.toBeInTheDocument();
    });
  });

  describe('Implementation input', () => {
    it('should render implementation textarea', () => {
      render(<ScenarioTestRunner {...defaultProps} />);

      const textarea = screen.getByPlaceholderText(/Describe what you implemented/);
      expect(textarea).toBeInTheDocument();
    });

    it('should allow entering implementation details', () => {
      render(<ScenarioTestRunner {...defaultProps} />);

      const textarea = screen.getByPlaceholderText(/Describe what you implemented/) as HTMLTextAreaElement;
      fireEvent.change(textarea, { target: { value: 'Implemented JWT authentication' } });

      expect(textarea.value).toBe('Implemented JWT authentication');
    });

    it('should display helper text', () => {
      render(<ScenarioTestRunner {...defaultProps} />);

      expect(screen.getByText('Provide implementation details to validate against scenarios')).toBeInTheDocument();
    });
  });

  describe('Run button', () => {
    it('should display Run button', () => {
      render(<ScenarioTestRunner {...defaultProps} />);

      expect(screen.getByText('Run Scenario Tests')).toBeInTheDocument();
    });

    it('should disable button when implementation is empty', () => {
      render(<ScenarioTestRunner {...defaultProps} />);

      const button = screen.getByText('Run Scenario Tests').closest('button')!;
      expect(button).toBeDisabled();
    });

    it('should disable button when implementation is only whitespace', () => {
      render(<ScenarioTestRunner {...defaultProps} />);

      const textarea = screen.getByPlaceholderText(/Describe what you implemented/);
      fireEvent.change(textarea, { target: { value: '   \n\n  ' } });

      const button = screen.getByText('Run Scenario Tests').closest('button')!;
      expect(button).toBeDisabled();
    });

    it('should enable button when implementation has content', () => {
      render(<ScenarioTestRunner {...defaultProps} />);

      const textarea = screen.getByPlaceholderText(/Describe what you implemented/);
      fireEvent.change(textarea, { target: { value: 'Implemented JWT authentication' } });

      const button = screen.getByText('Run Scenario Tests').closest('button')!;
      expect(button).not.toBeDisabled();
    });

    it('should call mutation when button is clicked', () => {
      render(<ScenarioTestRunner {...defaultProps} />);

      const textarea = screen.getByPlaceholderText(/Describe what you implemented/);
      fireEvent.change(textarea, { target: { value: 'Implemented JWT authentication' } });

      const button = screen.getByText('Run Scenario Tests').closest('button')!;
      fireEvent.click(button);

      expect(mockMutate).toHaveBeenCalledWith('Implemented JWT authentication');
    });

    it('should display loading state when mutation is pending', () => {
      mockMutationState.isPending = true;

      render(<ScenarioTestRunner {...defaultProps} />);

      expect(screen.getByText('Running Tests...')).toBeInTheDocument();
      expect(screen.getByText('Loader2')).toBeInTheDocument();
    });

    it('should disable button when mutation is pending', () => {
      mockMutationState.isPending = true;

      render(<ScenarioTestRunner {...defaultProps} />);

      const textarea = screen.getByPlaceholderText(/Describe what you implemented/);
      fireEvent.change(textarea, { target: { value: 'Implemented JWT authentication' } });

      const button = screen.getByText('Running Tests...').closest('button')!;
      expect(button).toBeDisabled();
    });
  });

  describe('Validation results', () => {
    it('should not display results initially', () => {
      render(<ScenarioTestRunner {...defaultProps} />);

      expect(screen.queryByTestId('validation-results')).not.toBeInTheDocument();
    });

    it('should display ValidationResultsPanel when validation completes', async () => {
      const validationResult = {
        overallAssessment: { passed: true, notes: 'All tests passed' },
        scenarioResults: [],
        recommendations: [],
      };

      render(<ScenarioTestRunner {...defaultProps} />);

      const textarea = screen.getByPlaceholderText(/Describe what you implemented/);
      fireEvent.change(textarea, { target: { value: 'Implementation' } });

      const button = screen.getByText('Run Scenario Tests').closest('button')!;
      fireEvent.click(button);

      // Simulate onSuccess callback
      if ((mockMutate as any).onSuccess) {
        (mockMutate as any).onSuccess({ data: validationResult });
      }

      await waitFor(() => {
        expect(screen.getByTestId('validation-results')).toBeInTheDocument();
      });
    });
  });

  describe('Error handling', () => {
    it('should display error message when mutation fails', () => {
      mockMutationState.isError = true;
      mockMutationState.error = new Error('API key invalid');

      render(<ScenarioTestRunner {...defaultProps} />);

      expect(screen.getByText('API key invalid')).toBeInTheDocument();
      expect(screen.getByText('AlertCircle')).toBeInTheDocument();
    });

    it('should display generic error when error is not an Error object', () => {
      mockMutationState.isError = true;
      mockMutationState.error = { message: 'Some error' };

      render(<ScenarioTestRunner {...defaultProps} />);

      expect(screen.getByText('Failed to validate task implementation')).toBeInTheDocument();
    });
  });

  describe('Cost information', () => {
    it('should not display cost initially', () => {
      render(<ScenarioTestRunner {...defaultProps} />);

      expect(screen.queryByText(/AI validation completed/)).not.toBeInTheDocument();
    });

    it('should display cost when validation succeeds', () => {
      mockMutationState.isSuccess = true;
      mockMutationState.data = {
        data: {
          overallAssessment: { passed: true },
          scenarioResults: [],
          recommendations: [],
        },
        cost: { estimatedCost: 0.0456 },
      };

      render(<ScenarioTestRunner {...defaultProps} />);

      expect(screen.getByText('AI validation completed')).toBeInTheDocument();
      expect(screen.getByText('Cost: $0.0456')).toBeInTheDocument();
    });

    it('should format cost with 4 decimal places', () => {
      mockMutationState.isSuccess = true;
      mockMutationState.data = {
        data: {
          overallAssessment: { passed: true },
          scenarioResults: [],
        },
        cost: { estimatedCost: 0.123456 },
      };

      render(<ScenarioTestRunner {...defaultProps} />);

      expect(screen.getByText('Cost: $0.1235')).toBeInTheDocument();
    });
  });

  describe('Edge cases', () => {
    it('should handle empty scenarios array', () => {
      render(<ScenarioTestRunner {...defaultProps} scenarios={[]} />);

      expect(screen.getByText(/Test task implementation against 0 scenarios/)).toBeInTheDocument();
    });

    it('should handle scenarios without AND clauses', () => {
      const scenariosWithoutAnd = [
        {
          name: 'Test scenario',
          when: 'condition',
          then: 'outcome',
        },
      ];

      render(<ScenarioTestRunner {...defaultProps} scenarios={scenariosWithoutAnd} />);

      expect(screen.getByText('Test scenario')).toBeInTheDocument();
      expect(screen.getByText('condition')).toBeInTheDocument();
      expect(screen.getByText('outcome')).toBeInTheDocument();
    });

    it('should handle scenarios with empty AND array', () => {
      const scenariosWithEmptyAnd = [
        {
          name: 'Test scenario',
          when: 'condition',
          then: 'outcome',
          and: [],
        },
      ];

      render(<ScenarioTestRunner {...defaultProps} scenarios={scenariosWithEmptyAnd} />);

      expect(screen.getByText('Test scenario')).toBeInTheDocument();
    });
  });
});
