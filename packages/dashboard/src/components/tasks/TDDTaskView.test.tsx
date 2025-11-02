// ABOUTME: Tests for TDDTaskView component
// ABOUTME: Validates TDD task execution steps, test strategies, and file references

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { TDDTaskView } from './TDDTaskView';
import type { TaskExecutionSteps } from '@/services/tasks';

describe('TDDTaskView', () => {
  const mockExecutionSteps: TaskExecutionSteps = {
    testStrategy: 'Write unit tests first, then implement functionality following TDD principles.',
    acceptanceCriteria: [
      'All tests pass',
      'Code coverage above 80%',
      'No linting errors',
    ],
    steps: [
      {
        stepNumber: 1,
        action: 'Write failing test for user authentication',
        expectedOutput: 'Test fails with expected error',
        estimatedMinutes: 5,
        testCommand: 'npm test auth.test.ts',
      },
      {
        stepNumber: 2,
        action: 'Implement authentication logic',
        expectedOutput: 'Test passes',
        estimatedMinutes: 15,
        testCommand: 'npm test auth.test.ts',
      },
      {
        stepNumber: 3,
        action: 'Refactor code for better readability',
        expectedOutput: 'All tests still pass',
        estimatedMinutes: 10,
      },
    ],
    relevantFiles: [
      {
        path: 'src/auth/auth.ts',
        operation: 'create',
        reason: 'Main authentication logic',
      },
      {
        path: 'src/auth/auth.test.ts',
        operation: 'create',
        reason: 'Test file for authentication',
      },
      {
        path: 'src/types/user.ts',
        operation: 'modify',
        reason: 'Add authentication types',
      },
    ],
    similarImplementations: [
      'src/utils/validation.ts',
      'src/services/api.ts',
    ],
  };

  const defaultProps = {
    executionSteps: mockExecutionSteps,
    onGenerateSteps: vi.fn(),
    isGenerating: false,
  };

  // Mock clipboard API
  beforeEach(() => {
    Object.assign(navigator, {
      clipboard: {
        writeText: vi.fn().mockResolvedValue(undefined),
      },
    });
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  describe('Null State', () => {
    it('should display message when executionSteps is null', () => {
      render(<TDDTaskView executionSteps={null} />);

      expect(screen.getByText('No execution steps available')).toBeInTheDocument();
    });

    it('should display Terminal icon when executionSteps is null', () => {
      const { container } = render(<TDDTaskView executionSteps={null} />);

      const icon = container.querySelector('svg.lucide-terminal');
      expect(icon).toBeInTheDocument();
    });

    it('should not show generate button when onGenerateSteps is not provided', () => {
      render(<TDDTaskView executionSteps={null} />);

      expect(screen.queryByRole('button')).not.toBeInTheDocument();
    });

    it('should show generate button when onGenerateSteps is provided', () => {
      const onGenerateSteps = vi.fn();
      render(
        <TDDTaskView
          executionSteps={null}
          onGenerateSteps={onGenerateSteps}
        />
      );

      expect(screen.getByRole('button', { name: /Generate Execution Steps/i })).toBeInTheDocument();
    });

    it('should call onGenerateSteps when generate button is clicked', async () => {
      const user = userEvent.setup();
      const onGenerateSteps = vi.fn();
      render(
        <TDDTaskView
          executionSteps={null}
          onGenerateSteps={onGenerateSteps}
        />
      );

      const generateButton = screen.getByRole('button', { name: /Generate Execution Steps/i });
      await user.click(generateButton);

      expect(onGenerateSteps).toHaveBeenCalledTimes(1);
    });

    it('should disable generate button when generating', () => {
      render(
        <TDDTaskView
          executionSteps={null}
          onGenerateSteps={vi.fn()}
          isGenerating={true}
        />
      );

      const generateButton = screen.getByRole('button', { name: /Generating.../i });
      expect(generateButton).toBeDisabled();
    });
  });

  describe('Test Strategy Card', () => {
    it('should render test strategy title', () => {
      render(<TDDTaskView {...defaultProps} />);

      expect(screen.getByText('Test Strategy')).toBeInTheDocument();
    });

    it('should display test strategy content', () => {
      render(<TDDTaskView {...defaultProps} />);

      expect(screen.getByText(mockExecutionSteps.testStrategy)).toBeInTheDocument();
    });

    it('should preserve whitespace in test strategy', () => {
      const stepsWithMultilineStrategy = {
        ...mockExecutionSteps,
        testStrategy: 'Step 1\n\nStep 2\nStep 3',
      };
      render(<TDDTaskView {...defaultProps} executionSteps={stepsWithMultilineStrategy} />);

      const strategyElement = screen.getByText(/Step 1/);
      expect(strategyElement).toHaveClass('whitespace-pre-wrap');
    });
  });

  describe('Acceptance Criteria Card', () => {
    it('should render acceptance criteria title', () => {
      render(<TDDTaskView {...defaultProps} />);

      expect(screen.getByText('Acceptance Criteria')).toBeInTheDocument();
    });

    it('should display all acceptance criteria', () => {
      render(<TDDTaskView {...defaultProps} />);

      expect(screen.getByText('All tests pass')).toBeInTheDocument();
      expect(screen.getByText('Code coverage above 80%')).toBeInTheDocument();
      expect(screen.getByText('No linting errors')).toBeInTheDocument();
    });

    it('should display checkmark icons for acceptance criteria', () => {
      const { container } = render(<TDDTaskView {...defaultProps} />);

      const checkIcons = container.querySelectorAll('svg.lucide-check-circle-2');
      expect(checkIcons.length).toBeGreaterThan(0);
    });

    it('should not render acceptance criteria card when empty', () => {
      const stepsWithoutCriteria = {
        ...mockExecutionSteps,
        acceptanceCriteria: [],
      };
      render(<TDDTaskView {...defaultProps} executionSteps={stepsWithoutCriteria} />);

      expect(screen.queryByText('Acceptance Criteria')).not.toBeInTheDocument();
    });
  });

  describe('Execution Steps Card', () => {
    it('should render execution steps title', () => {
      render(<TDDTaskView {...defaultProps} />);

      expect(screen.getByText('Execution Steps')).toBeInTheDocument();
    });

    it('should display total estimated time', () => {
      render(<TDDTaskView {...defaultProps} />);

      // 5 + 15 + 10 = 30 minutes
      expect(screen.getByText('~30 min total')).toBeInTheDocument();
    });

    it('should display all execution steps', () => {
      render(<TDDTaskView {...defaultProps} />);

      expect(screen.getByText('Write failing test for user authentication')).toBeInTheDocument();
      expect(screen.getByText('Implement authentication logic')).toBeInTheDocument();
      expect(screen.getByText('Refactor code for better readability')).toBeInTheDocument();
    });

    it('should display step numbers', () => {
      render(<TDDTaskView {...defaultProps} />);

      expect(screen.getByText('1')).toBeInTheDocument();
      expect(screen.getByText('2')).toBeInTheDocument();
      expect(screen.getByText('3')).toBeInTheDocument();
    });

    it('should display expected output for each step', () => {
      render(<TDDTaskView {...defaultProps} />);

      expect(screen.getByText('Expected: Test fails with expected error')).toBeInTheDocument();
      expect(screen.getByText('Expected: Test passes')).toBeInTheDocument();
      expect(screen.getByText('Expected: All tests still pass')).toBeInTheDocument();
    });

    it('should display estimated time for each step', () => {
      render(<TDDTaskView {...defaultProps} />);

      expect(screen.getByText('~5m')).toBeInTheDocument();
      expect(screen.getByText('~15m')).toBeInTheDocument();
      expect(screen.getByText('~10m')).toBeInTheDocument();
    });
  });

  describe('Test Commands', () => {
    it('should display test command when available', () => {
      render(<TDDTaskView {...defaultProps} />);

      expect(screen.getAllByText('npm test auth.test.ts')).toHaveLength(2);
    });

    it('should not display command section when testCommand is missing', () => {
      render(<TDDTaskView {...defaultProps} />);

      const commandLabels = screen.getAllByText('Command');
      // Only 2 steps have commands (step 1 and 2), step 3 doesn't
      expect(commandLabels).toHaveLength(2);
    });

    it('should display copy button for each command', () => {
      render(<TDDTaskView {...defaultProps} />);

      const copyButtons = screen.getAllByRole('button', { name: /Copy/i });
      expect(copyButtons).toHaveLength(2);
    });

    it('should copy command to clipboard when copy button is clicked', async () => {
      const user = userEvent.setup();
      render(<TDDTaskView {...defaultProps} />);

      const copyButtons = screen.getAllByRole('button', { name: /Copy/i });
      await user.click(copyButtons[0]);

      expect(navigator.clipboard.writeText).toHaveBeenCalledWith('npm test auth.test.ts');
    });

    it('should show "Copied" state after copying', async () => {
      const user = userEvent.setup();
      render(<TDDTaskView {...defaultProps} />);

      const copyButtons = screen.getAllByRole('button', { name: /Copy/i });
      await user.click(copyButtons[0]);

      await waitFor(() => {
        expect(screen.getByText('Copied')).toBeInTheDocument();
      });
    });

    it('should reset "Copied" state after 2 seconds', async () => {
      const user = userEvent.setup();
      vi.useFakeTimers();
      render(<TDDTaskView {...defaultProps} />);

      const copyButtons = screen.getAllByRole('button', { name: /Copy/i });
      await user.click(copyButtons[0]);

      expect(screen.getByText('Copied')).toBeInTheDocument();

      vi.advanceTimersByTime(2000);

      await waitFor(() => {
        expect(screen.queryByText('Copied')).not.toBeInTheDocument();
      });

      vi.useRealTimers();
    });

    it('should handle clipboard copy failure gracefully', async () => {
      const user = userEvent.setup();
      const consoleError = vi.spyOn(console, 'error').mockImplementation(() => {});
      (navigator.clipboard.writeText as any).mockRejectedValue(new Error('Clipboard error'));

      render(<TDDTaskView {...defaultProps} />);

      const copyButtons = screen.getAllByRole('button', { name: /Copy/i });
      await user.click(copyButtons[0]);

      await waitFor(() => {
        expect(consoleError).toHaveBeenCalledWith('Failed to copy command:', expect.any(Error));
      });

      consoleError.mockRestore();
    });
  });

  describe('File References Card', () => {
    it('should render file references title', () => {
      render(<TDDTaskView {...defaultProps} />);

      expect(screen.getByText('Files to Work On')).toBeInTheDocument();
    });

    it('should display all relevant files', () => {
      render(<TDDTaskView {...defaultProps} />);

      expect(screen.getByText('src/auth/auth.ts')).toBeInTheDocument();
      expect(screen.getByText('src/auth/auth.test.ts')).toBeInTheDocument();
      expect(screen.getByText('src/types/user.ts')).toBeInTheDocument();
    });

    it('should display operation badges with correct colors', () => {
      const { container } = render(<TDDTaskView {...defaultProps} />);

      const createBadges = container.querySelectorAll('.bg-green-500');
      expect(createBadges.length).toBeGreaterThan(0);

      const modifyBadges = container.querySelectorAll('.bg-blue-500');
      expect(modifyBadges.length).toBeGreaterThan(0);
    });

    it('should display file operation reasons', () => {
      render(<TDDTaskView {...defaultProps} />);

      expect(screen.getByText('Main authentication logic')).toBeInTheDocument();
      expect(screen.getByText('Test file for authentication')).toBeInTheDocument();
      expect(screen.getByText('Add authentication types')).toBeInTheDocument();
    });

    it('should handle delete operation color', () => {
      const stepsWithDeleteOp = {
        ...mockExecutionSteps,
        relevantFiles: [
          {
            path: 'old-file.ts',
            operation: 'delete',
            reason: 'Remove deprecated file',
          },
        ],
      };
      const { container } = render(<TDDTaskView {...defaultProps} executionSteps={stepsWithDeleteOp} />);

      const deleteBadge = container.querySelector('.bg-red-500');
      expect(deleteBadge).toBeInTheDocument();
    });

    it('should not render file references card when empty', () => {
      const stepsWithoutFiles = {
        ...mockExecutionSteps,
        relevantFiles: [],
      };
      render(<TDDTaskView {...defaultProps} executionSteps={stepsWithoutFiles} />);

      expect(screen.queryByText('Files to Work On')).not.toBeInTheDocument();
    });
  });

  describe('Similar Implementations Card', () => {
    it('should render similar implementations title', () => {
      render(<TDDTaskView {...defaultProps} />);

      expect(screen.getByText('Similar Implementations')).toBeInTheDocument();
    });

    it('should display reference message', () => {
      render(<TDDTaskView {...defaultProps} />);

      expect(screen.getByText('Reference these existing implementations for guidance:')).toBeInTheDocument();
    });

    it('should display all similar implementations', () => {
      render(<TDDTaskView {...defaultProps} />);

      expect(screen.getByText('• src/utils/validation.ts')).toBeInTheDocument();
      expect(screen.getByText('• src/services/api.ts')).toBeInTheDocument();
    });

    it('should not render similar implementations card when empty', () => {
      const stepsWithoutSimilar = {
        ...mockExecutionSteps,
        similarImplementations: [],
      };
      render(<TDDTaskView {...defaultProps} executionSteps={stepsWithoutSimilar} />);

      expect(screen.queryByText('Similar Implementations')).not.toBeInTheDocument();
    });
  });

  describe('Edge Cases', () => {
    it('should handle empty test strategy', () => {
      const stepsWithEmptyStrategy = {
        ...mockExecutionSteps,
        testStrategy: '',
      };
      render(<TDDTaskView {...defaultProps} executionSteps={stepsWithEmptyStrategy} />);

      expect(screen.getByText('Test Strategy')).toBeInTheDocument();
    });

    it('should calculate total time correctly with single step', () => {
      const stepsWithOneStep = {
        ...mockExecutionSteps,
        steps: [mockExecutionSteps.steps[0]],
      };
      render(<TDDTaskView {...defaultProps} executionSteps={stepsWithOneStep} />);

      expect(screen.getByText('~5 min total')).toBeInTheDocument();
    });

    it('should handle zero estimated minutes', () => {
      const stepsWithZeroTime = {
        ...mockExecutionSteps,
        steps: [
          {
            ...mockExecutionSteps.steps[0],
            estimatedMinutes: 0,
          },
        ],
      };
      render(<TDDTaskView {...defaultProps} executionSteps={stepsWithZeroTime} />);

      expect(screen.getByText('~0 min total')).toBeInTheDocument();
    });

    it('should handle unknown file operation', () => {
      const stepsWithUnknownOp = {
        ...mockExecutionSteps,
        relevantFiles: [
          {
            path: 'file.ts',
            operation: 'unknown',
            reason: 'Unknown operation',
          },
        ],
      };
      const { container } = render(<TDDTaskView {...defaultProps} executionSteps={stepsWithUnknownOp} />);

      const unknownBadge = container.querySelector('.bg-gray-500');
      expect(unknownBadge).toBeInTheDocument();
    });

    it('should handle step with no expected output', () => {
      const stepsWithNoOutput = {
        ...mockExecutionSteps,
        steps: [
          {
            ...mockExecutionSteps.steps[0],
            expectedOutput: '',
          },
        ],
      };
      render(<TDDTaskView {...defaultProps} executionSteps={stepsWithNoOutput} />);

      expect(screen.getByText('Expected:')).toBeInTheDocument();
    });
  });

  describe('Component Layout', () => {
    it('should render all cards in correct order when all data is present', () => {
      const { container } = render(<TDDTaskView {...defaultProps} />);

      expect(screen.getByText('Test Strategy')).toBeInTheDocument();
      expect(screen.getByText('Acceptance Criteria')).toBeInTheDocument();
      expect(screen.getByText('Execution Steps')).toBeInTheDocument();
      expect(screen.getByText('Files to Work On')).toBeInTheDocument();
      expect(screen.getByText('Similar Implementations')).toBeInTheDocument();
    });

    it('should render minimum cards when optional data is missing', () => {
      const minimalSteps = {
        testStrategy: 'Basic strategy',
        acceptanceCriteria: [],
        steps: [
          {
            stepNumber: 1,
            action: 'Do something',
            expectedOutput: 'Something happens',
            estimatedMinutes: 5,
          },
        ],
        relevantFiles: [],
        similarImplementations: [],
      };
      render(<TDDTaskView {...defaultProps} executionSteps={minimalSteps} />);

      expect(screen.getByText('Test Strategy')).toBeInTheDocument();
      expect(screen.getByText('Execution Steps')).toBeInTheDocument();
      expect(screen.queryByText('Acceptance Criteria')).not.toBeInTheDocument();
      expect(screen.queryByText('Files to Work On')).not.toBeInTheDocument();
      expect(screen.queryByText('Similar Implementations')).not.toBeInTheDocument();
    });
  });
});
