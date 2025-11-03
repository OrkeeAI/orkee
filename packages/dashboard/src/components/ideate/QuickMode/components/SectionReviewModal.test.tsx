// ABOUTME: Tests for SectionReviewModal component
// ABOUTME: Validates section review display, quality score, and user actions (approve/regenerate/edit)

import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { SectionReviewModal } from './SectionReviewModal';
import type { SectionValidationResult } from '@/services/ideate';

describe('SectionReviewModal', () => {
  const mockValidationResult: SectionValidationResult = {
    is_valid: true,
    quality_score: 85,
    issues: ['Issue 1', 'Issue 2'],
    suggestions: ['Suggestion 1', 'Suggestion 2'],
    section_name: 'Overview',
  };

  const defaultProps = {
    open: true,
    onOpenChange: vi.fn(),
    sectionName: 'Overview',
    sectionContent: 'Test section content',
    validationResult: mockValidationResult,
    onApprove: vi.fn(),
    onRegenerate: vi.fn(),
    onEdit: vi.fn(),
    isRegenerating: false,
  };

  describe('Rendering', () => {
    it('should render section name in title', () => {
      render(<SectionReviewModal {...defaultProps} />);

      expect(screen.getByText('Review: Overview')).toBeInTheDocument();
    });

    it('should render Valid badge when validation passes', () => {
      render(<SectionReviewModal {...defaultProps} />);

      expect(screen.getByText('Valid')).toBeInTheDocument();
    });

    it('should render Invalid badge when validation fails', () => {
      render(
        <SectionReviewModal
          {...defaultProps}
          validationResult={{ ...mockValidationResult, is_valid: false }}
        />
      );

      expect(screen.getByText('Invalid')).toBeInTheDocument();
    });

    it('should render quality score', () => {
      render(<SectionReviewModal {...defaultProps} />);

      expect(screen.getByText(/85\/100/)).toBeInTheDocument();
      expect(screen.getByText(/Excellent/)).toBeInTheDocument();
    });

    it('should render section content', () => {
      render(<SectionReviewModal {...defaultProps} />);

      expect(screen.getByText('Test section content')).toBeInTheDocument();
    });

    it('should render issues list', () => {
      render(<SectionReviewModal {...defaultProps} />);

      expect(screen.getByText('Issues Found:')).toBeInTheDocument();
      expect(screen.getByText('Issue 1')).toBeInTheDocument();
      expect(screen.getByText('Issue 2')).toBeInTheDocument();
    });

    it('should render suggestions list', () => {
      render(<SectionReviewModal {...defaultProps} />);

      expect(screen.getByText('Suggestions:')).toBeInTheDocument();
      expect(screen.getByText('Suggestion 1')).toBeInTheDocument();
      expect(screen.getByText('Suggestion 2')).toBeInTheDocument();
    });
  });

  describe('Quality Score Display', () => {
    it('should show Excellent for score >= 80', () => {
      render(
        <SectionReviewModal
          {...defaultProps}
          validationResult={{ ...mockValidationResult, quality_score: 85 }}
        />
      );

      expect(screen.getByText(/Excellent/)).toBeInTheDocument();
    });

    it('should show Good for score between 60-79', () => {
      render(
        <SectionReviewModal
          {...defaultProps}
          validationResult={{ ...mockValidationResult, quality_score: 70 }}
        />
      );

      expect(screen.getByText(/Good/)).toBeInTheDocument();
    });

    it('should show Needs Improvement for score < 60', () => {
      render(
        <SectionReviewModal
          {...defaultProps}
          validationResult={{ ...mockValidationResult, quality_score: 50 }}
        />
      );

      expect(screen.getByText(/Needs Improvement/)).toBeInTheDocument();
    });
  });

  describe('User Actions', () => {
    it('should call onApprove when Approve button is clicked', async () => {
      const user = userEvent.setup();
      const onApprove = vi.fn();
      render(<SectionReviewModal {...defaultProps} onApprove={onApprove} />);

      const approveButton = screen.getByRole('button', { name: /Approve/i });
      await user.click(approveButton);

      expect(onApprove).toHaveBeenCalledTimes(1);
    });

    it('should call onRegenerate when Regenerate button is clicked', async () => {
      const user = userEvent.setup();
      const onRegenerate = vi.fn();
      render(<SectionReviewModal {...defaultProps} onRegenerate={onRegenerate} />);

      const regenerateButton = screen.getByRole('button', { name: /Regenerate/i });
      await user.click(regenerateButton);

      expect(onRegenerate).toHaveBeenCalledTimes(1);
    });

    it('should call onOpenChange when Skip button is clicked', async () => {
      const user = userEvent.setup();
      const onOpenChange = vi.fn();
      render(<SectionReviewModal {...defaultProps} onOpenChange={onOpenChange} />);

      const skipButton = screen.getByRole('button', { name: /Skip/i });
      await user.click(skipButton);

      expect(onOpenChange).toHaveBeenCalledWith(false);
    });
  });

  describe('Edit Mode', () => {
    it('should enter edit mode when Edit button is clicked', async () => {
      const user = userEvent.setup();
      render(<SectionReviewModal {...defaultProps} />);

      const editButton = screen.getByRole('button', { name: /^Edit$/i });
      await user.click(editButton);

      // Should show textarea in edit mode
      expect(screen.getByRole('textbox')).toBeInTheDocument();
      expect(screen.getByRole('button', { name: /Save Changes/i })).toBeInTheDocument();
      expect(screen.getByRole('button', { name: /Cancel/i })).toBeInTheDocument();
    });

    it('should allow editing content', async () => {
      const user = userEvent.setup();
      render(<SectionReviewModal {...defaultProps} />);

      const editButton = screen.getByRole('button', { name: /^Edit$/i });
      await user.click(editButton);

      const textarea = screen.getByRole('textbox');
      await user.clear(textarea);
      await user.type(textarea, 'Updated content');

      expect(textarea).toHaveValue('Updated content');
    });

    it('should call onEdit with updated content when Save Changes is clicked', async () => {
      const user = userEvent.setup();
      const onEdit = vi.fn();
      render(<SectionReviewModal {...defaultProps} onEdit={onEdit} />);

      const editButton = screen.getByRole('button', { name: /^Edit$/i });
      await user.click(editButton);

      const textarea = screen.getByRole('textbox');
      await user.clear(textarea);
      await user.type(textarea, 'Updated content');

      const saveButton = screen.getByRole('button', { name: /Save Changes/i });
      await user.click(saveButton);

      expect(onEdit).toHaveBeenCalledWith('Updated content');
    });

    it('should revert content when Cancel is clicked', async () => {
      const user = userEvent.setup();
      render(<SectionReviewModal {...defaultProps} />);

      const editButton = screen.getByRole('button', { name: /^Edit$/i });
      await user.click(editButton);

      const textarea = screen.getByRole('textbox');
      await user.clear(textarea);
      await user.type(textarea, 'Updated content');

      const cancelButton = screen.getByRole('button', { name: /Cancel/i });
      await user.click(cancelButton);

      // Should exit edit mode and show original content
      expect(screen.queryByRole('textbox')).not.toBeInTheDocument();
      expect(screen.getByText('Test section content')).toBeInTheDocument();
    });
  });

  describe('Regenerating State', () => {
    it('should disable Regenerate button when isRegenerating is true', () => {
      render(<SectionReviewModal {...defaultProps} isRegenerating={true} />);

      const regenerateButton = screen.getByRole('button', { name: /Regenerate/i });
      expect(regenerateButton).toBeDisabled();
    });

    it('should show regenerating state in button', () => {
      render(
        <SectionReviewModal {...defaultProps} isRegenerating={true} />
      );

      const regenerateButton = screen.getByRole('button', { name: /Regenerate/i });
      // Verify button is in regenerating state (disabled and has icon)
      expect(regenerateButton).toBeDisabled();
      expect(regenerateButton).toBeInTheDocument();
    });
  });

  describe('Edge Cases', () => {
    it('should handle null validation result', () => {
      render(<SectionReviewModal {...defaultProps} validationResult={null} />);

      // Should not show quality score section
      expect(screen.queryByText('Quality Score')).not.toBeInTheDocument();
    });

    it('should handle empty issues array', () => {
      render(
        <SectionReviewModal
          {...defaultProps}
          validationResult={{ ...mockValidationResult, issues: [] }}
        />
      );

      // Should not show issues section
      expect(screen.queryByText('Issues Found:')).not.toBeInTheDocument();
    });

    it('should handle empty suggestions array', () => {
      render(
        <SectionReviewModal
          {...defaultProps}
          validationResult={{ ...mockValidationResult, suggestions: [] }}
        />
      );

      // Should not show suggestions section
      expect(screen.queryByText('Suggestions:')).not.toBeInTheDocument();
    });

    it('should handle empty section content', () => {
      render(<SectionReviewModal {...defaultProps} sectionContent="" />);

      // Component should still render all sections even with empty content
      expect(screen.getByText('Content')).toBeInTheDocument();
      expect(screen.getByRole('button', { name: /Edit/i })).toBeInTheDocument();
    });
  });
});
