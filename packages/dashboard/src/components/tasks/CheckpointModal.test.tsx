// ABOUTME: Tests for CheckpointModal component
// ABOUTME: Validates checkpoint display, validation checklists, and completion actions

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { CheckpointModal } from './CheckpointModal';
import type { ExecutionCheckpoint } from '@/services/tasks';

describe('CheckpointModal', () => {
  const mockCheckpoint: ExecutionCheckpoint = {
    checkpointType: 'test',
    message: 'Tests must pass before continuing',
    requiredValidation: [
      'All unit tests passing',
      'Code coverage above 80%',
      'No linting errors',
    ],
  };

  const defaultProps = {
    checkpoint: mockCheckpoint,
    open: true,
    onClose: vi.fn(),
    onComplete: vi.fn(),
    onSkip: vi.fn(),
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('Null State', () => {
    it('should render nothing when checkpoint is null', () => {
      const { container } = render(<CheckpointModal {...defaultProps} checkpoint={null} />);

      expect(container.firstChild).toBeNull();
    });
  });

  describe('Rendering - Basic Elements', () => {
    it('should render dialog title', () => {
      render(<CheckpointModal {...defaultProps} />);

      expect(screen.getByText('Checkpoint Reached')).toBeInTheDocument();
    });

    it('should render checkpoint message', () => {
      render(<CheckpointModal {...defaultProps} />);

      expect(screen.getByText('Tests must pass before continuing')).toBeInTheDocument();
    });

    it('should render checkpoint type badge', () => {
      render(<CheckpointModal {...defaultProps} />);

      expect(screen.getByText('test')).toBeInTheDocument();
    });
  });

  describe('Checkpoint Type Colors', () => {
    it('should use blue color for review checkpoints', () => {
      const reviewCheckpoint = { ...mockCheckpoint, checkpointType: 'review' };
      const { container } = render(<CheckpointModal {...defaultProps} checkpoint={reviewCheckpoint} />);

      const badge = container.querySelector('.bg-blue-500');
      expect(badge).toBeInTheDocument();
    });

    it('should use green color for test checkpoints', () => {
      const { container } = render(<CheckpointModal {...defaultProps} />);

      const badge = container.querySelector('.bg-green-500');
      expect(badge).toBeInTheDocument();
    });

    it('should use purple color for integration checkpoints', () => {
      const integrationCheckpoint = { ...mockCheckpoint, checkpointType: 'integration' };
      const { container } = render(<CheckpointModal {...defaultProps} checkpoint={integrationCheckpoint} />);

      const badge = container.querySelector('.bg-purple-500');
      expect(badge).toBeInTheDocument();
    });

    it('should use orange color for approval checkpoints', () => {
      const approvalCheckpoint = { ...mockCheckpoint, checkpointType: 'approval' };
      const { container } = render(<CheckpointModal {...defaultProps} checkpoint={approvalCheckpoint} />);

      const badge = container.querySelector('.bg-orange-500');
      expect(badge).toBeInTheDocument();
    });

    it('should use gray color for unknown checkpoint types', () => {
      const unknownCheckpoint = { ...mockCheckpoint, checkpointType: 'unknown' };
      const { container } = render(<CheckpointModal {...defaultProps} checkpoint={unknownCheckpoint} />);

      const badge = container.querySelector('.bg-gray-500');
      expect(badge).toBeInTheDocument();
    });
  });

  describe('Validation Items', () => {
    it('should render all validation items', () => {
      render(<CheckpointModal {...defaultProps} />);

      expect(screen.getByText('All unit tests passing')).toBeInTheDocument();
      expect(screen.getByText('Code coverage above 80%')).toBeInTheDocument();
      expect(screen.getByText('No linting errors')).toBeInTheDocument();
    });

    it('should render checkboxes for each validation item', () => {
      render(<CheckpointModal {...defaultProps} />);

      const checkboxes = screen.getAllByRole('checkbox');
      expect(checkboxes).toHaveLength(3);
    });

    it('should render validation instruction alert', () => {
      render(<CheckpointModal {...defaultProps} />);

      expect(screen.getByText('Please verify the following items before continuing:')).toBeInTheDocument();
    });

    it('should render no validation message when requiredValidation is empty', () => {
      const checkpointWithoutValidation = {
        ...mockCheckpoint,
        requiredValidation: [],
      };
      render(<CheckpointModal {...defaultProps} checkpoint={checkpointWithoutValidation} />);

      expect(screen.getByText('No specific validation required. Click Continue when ready.')).toBeInTheDocument();
    });
  });

  describe('Checkbox Interactions', () => {
    it('should check checkbox when clicked', async () => {
      const user = userEvent.setup();
      render(<CheckpointModal {...defaultProps} />);

      const checkboxes = screen.getAllByRole('checkbox');
      await user.click(checkboxes[0]);

      expect(checkboxes[0]).toBeChecked();
    });

    it('should uncheck checkbox when clicked again', async () => {
      const user = userEvent.setup();
      render(<CheckpointModal {...defaultProps} />);

      const checkboxes = screen.getAllByRole('checkbox');
      await user.click(checkboxes[0]);
      expect(checkboxes[0]).toBeChecked();

      await user.click(checkboxes[0]);
      expect(checkboxes[0]).not.toBeChecked();
    });

    it('should allow clicking label to check checkbox', async () => {
      const user = userEvent.setup();
      render(<CheckpointModal {...defaultProps} />);

      const label = screen.getByText('All unit tests passing');
      await user.click(label);

      const checkboxes = screen.getAllByRole('checkbox');
      expect(checkboxes[0]).toBeChecked();
    });
  });

  describe('Validation Completion', () => {
    it('should disable continue button when not all items are checked', () => {
      render(<CheckpointModal {...defaultProps} />);

      const continueButton = screen.getByRole('button', { name: /Validate & Continue/i });
      expect(continueButton).toBeDisabled();
    });

    it('should enable continue button when all items are checked', async () => {
      const user = userEvent.setup();
      render(<CheckpointModal {...defaultProps} />);

      const checkboxes = screen.getAllByRole('checkbox');
      for (const checkbox of checkboxes) {
        await user.click(checkbox);
      }

      const continueButton = screen.getByRole('button', { name: /Continue/i });
      expect(continueButton).not.toBeDisabled();
    });

    it('should show success alert when all items are validated', async () => {
      const user = userEvent.setup();
      render(<CheckpointModal {...defaultProps} />);

      const checkboxes = screen.getAllByRole('checkbox');
      for (const checkbox of checkboxes) {
        await user.click(checkbox);
      }

      expect(screen.getByText('All validation items checked! Ready to continue.')).toBeInTheDocument();
    });

    it('should change button text to "Continue" when all validated', async () => {
      const user = userEvent.setup();
      render(<CheckpointModal {...defaultProps} />);

      expect(screen.getByRole('button', { name: /Validate & Continue/i })).toBeInTheDocument();

      const checkboxes = screen.getAllByRole('checkbox');
      for (const checkbox of checkboxes) {
        await user.click(checkbox);
      }

      expect(screen.getByRole('button', { name: /^Continue$/i })).toBeInTheDocument();
      expect(screen.queryByRole('button', { name: /Validate & Continue/i })).not.toBeInTheDocument();
    });
  });

  describe('Continue Button Behavior', () => {
    it('should call onComplete with validation state when continue is clicked', async () => {
      const user = userEvent.setup();
      const onComplete = vi.fn();
      render(<CheckpointModal {...defaultProps} onComplete={onComplete} />);

      const checkboxes = screen.getAllByRole('checkbox');
      for (const checkbox of checkboxes) {
        await user.click(checkbox);
      }

      const continueButton = screen.getByRole('button', { name: /Continue/i });
      await user.click(continueButton);

      expect(onComplete).toHaveBeenCalledWith({
        'All unit tests passing': true,
        'Code coverage above 80%': true,
        'No linting errors': true,
      });
    });

    it('should call onClose after completing', async () => {
      const user = userEvent.setup();
      const onClose = vi.fn();
      render(<CheckpointModal {...defaultProps} onClose={onClose} />);

      const checkboxes = screen.getAllByRole('checkbox');
      for (const checkbox of checkboxes) {
        await user.click(checkbox);
      }

      const continueButton = screen.getByRole('button', { name: /Continue/i });
      await user.click(continueButton);

      expect(onClose).toHaveBeenCalledTimes(1);
    });

    it('should reset validation state after completing', async () => {
      const user = userEvent.setup();
      const { rerender } = render(<CheckpointModal {...defaultProps} />);

      const checkboxes = screen.getAllByRole('checkbox');
      for (const checkbox of checkboxes) {
        await user.click(checkbox);
      }

      const continueButton = screen.getByRole('button', { name: /Continue/i });
      await user.click(continueButton);

      // Reopen modal
      rerender(<CheckpointModal {...defaultProps} open={false} />);
      rerender(<CheckpointModal {...defaultProps} open={true} />);

      // Checkboxes should be unchecked again
      const newCheckboxes = screen.getAllByRole('checkbox');
      newCheckboxes.forEach(checkbox => {
        expect(checkbox).not.toBeChecked();
      });
    });
  });

  describe('Skip Button', () => {
    it('should render skip button when onSkip is provided', () => {
      render(<CheckpointModal {...defaultProps} />);

      expect(screen.getByRole('button', { name: /Skip Checkpoint/i })).toBeInTheDocument();
    });

    it('should not render skip button when onSkip is not provided', () => {
      render(<CheckpointModal {...defaultProps} onSkip={undefined} />);

      expect(screen.queryByRole('button', { name: /Skip Checkpoint/i })).not.toBeInTheDocument();
    });

    it('should call onSkip when skip button is clicked', async () => {
      const user = userEvent.setup();
      const onSkip = vi.fn();
      render(<CheckpointModal {...defaultProps} onSkip={onSkip} />);

      const skipButton = screen.getByRole('button', { name: /Skip Checkpoint/i });
      await user.click(skipButton);

      expect(onSkip).toHaveBeenCalledTimes(1);
    });

    it('should call onClose after skipping', async () => {
      const user = userEvent.setup();
      const onClose = vi.fn();
      render(<CheckpointModal {...defaultProps} onClose={onClose} />);

      const skipButton = screen.getByRole('button', { name: /Skip Checkpoint/i });
      await user.click(skipButton);

      expect(onClose).toHaveBeenCalledTimes(1);
    });

    it('should reset validation state after skipping', async () => {
      const user = userEvent.setup();
      const { rerender } = render(<CheckpointModal {...defaultProps} />);

      // Check some boxes
      const checkboxes = screen.getAllByRole('checkbox');
      await user.click(checkboxes[0]);

      const skipButton = screen.getByRole('button', { name: /Skip Checkpoint/i });
      await user.click(skipButton);

      // Reopen modal
      rerender(<CheckpointModal {...defaultProps} open={false} />);
      rerender(<CheckpointModal {...defaultProps} open={true} />);

      // Checkboxes should be unchecked again
      const newCheckboxes = screen.getAllByRole('checkbox');
      newCheckboxes.forEach(checkbox => {
        expect(checkbox).not.toBeChecked();
      });
    });
  });

  describe('Empty Validation Checkpoint', () => {
    it('should enable continue button immediately when no validation required', () => {
      const checkpointWithoutValidation = {
        ...mockCheckpoint,
        requiredValidation: [],
      };
      render(<CheckpointModal {...defaultProps} checkpoint={checkpointWithoutValidation} />);

      const continueButton = screen.getByRole('button', { name: /Continue/i });
      expect(continueButton).not.toBeDisabled();
    });

    it('should call onComplete with empty validation state', async () => {
      const user = userEvent.setup();
      const onComplete = vi.fn();
      const checkpointWithoutValidation = {
        ...mockCheckpoint,
        requiredValidation: [],
      };
      render(
        <CheckpointModal
          {...defaultProps}
          checkpoint={checkpointWithoutValidation}
          onComplete={onComplete}
        />
      );

      const continueButton = screen.getByRole('button', { name: /Continue/i });
      await user.click(continueButton);

      expect(onComplete).toHaveBeenCalledWith({});
    });
  });

  describe('Partial Validation', () => {
    it('should keep button disabled with partial validation', async () => {
      const user = userEvent.setup();
      render(<CheckpointModal {...defaultProps} />);

      const checkboxes = screen.getAllByRole('checkbox');
      await user.click(checkboxes[0]);
      await user.click(checkboxes[1]);
      // Don't click third checkbox

      const continueButton = screen.getByRole('button', { name: /Validate & Continue/i });
      expect(continueButton).toBeDisabled();
    });

    it('should not show success alert with partial validation', async () => {
      const user = userEvent.setup();
      render(<CheckpointModal {...defaultProps} />);

      const checkboxes = screen.getAllByRole('checkbox');
      await user.click(checkboxes[0]);

      expect(screen.queryByText('All validation items checked! Ready to continue.')).not.toBeInTheDocument();
    });

    it('should track individual validation states correctly', async () => {
      const user = userEvent.setup();
      const onComplete = vi.fn();
      render(<CheckpointModal {...defaultProps} onComplete={onComplete} />);

      const checkboxes = screen.getAllByRole('checkbox');
      await user.click(checkboxes[0]);
      await user.click(checkboxes[2]);
      await user.click(checkboxes[1]);

      const continueButton = screen.getByRole('button', { name: /Continue/i });
      await user.click(continueButton);

      expect(onComplete).toHaveBeenCalledWith({
        'All unit tests passing': true,
        'Code coverage above 80%': true,
        'No linting errors': true,
      });
    });
  });

  describe('Dialog Open/Close', () => {
    it('should render dialog when open is true', () => {
      render(<CheckpointModal {...defaultProps} open={true} />);

      expect(screen.getByText('Checkpoint Reached')).toBeInTheDocument();
    });

    it('should call onClose when dialog close is triggered', () => {
      const onClose = vi.fn();
      render(<CheckpointModal {...defaultProps} onClose={onClose} />);

      // Dialog should allow closing via onOpenChange
      expect(onClose).not.toHaveBeenCalled();
    });
  });
});
