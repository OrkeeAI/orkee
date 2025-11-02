// ABOUTME: Tests for SectionValidationPanel component in Guided Mode
// ABOUTME: Validates section quality display, validation flow, and user actions

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { SectionValidationPanel } from './SectionValidationPanel';
import { ideateService } from '@/services/ideate';
import type { SectionValidationResult } from '@/services/ideate';

// Mock the ideate service
vi.mock('@/services/ideate', () => ({
  ideateService: {
    validateSection: vi.fn(),
    storeValidationFeedback: vi.fn(),
  },
}));

// Mock sonner toast
vi.mock('sonner', () => ({
  toast: {
    error: vi.fn(),
  },
}));

describe('SectionValidationPanel', () => {
  const mockValidationResult: SectionValidationResult = {
    is_valid: true,
    quality_score: 85,
    issues: ['Issue 1', 'Issue 2'],
    suggestions: ['Suggestion 1', 'Suggestion 2'],
    section_name: 'Overview',
  };

  const defaultProps = {
    sessionId: 'session-123',
    sectionName: 'Overview',
    sectionContent: 'Test section content',
    onContinue: vi.fn(),
    onRegenerate: vi.fn(),
  };

  beforeEach(() => {
    vi.clearAllMocks();
    (ideateService.validateSection as any).mockResolvedValue(mockValidationResult);
    (ideateService.storeValidationFeedback as any).mockResolvedValue({});
  });

  afterEach(() => {
    vi.clearAllTimers();
  });

  describe('Validation State', () => {
    it('should show validating state initially', () => {
      render(<SectionValidationPanel {...defaultProps} />);

      expect(screen.getByText('Validating section quality...')).toBeInTheDocument();
    });

    it('should call validateSection API on mount', async () => {
      render(<SectionValidationPanel {...defaultProps} />);

      await waitFor(() => {
        expect(ideateService.validateSection).toHaveBeenCalledWith(
          'session-123',
          'Overview',
          'Test section content'
        );
      });
    });

    it('should call storeValidationFeedback after validation', async () => {
      render(<SectionValidationPanel {...defaultProps} />);

      await waitFor(() => {
        expect(ideateService.storeValidationFeedback).toHaveBeenCalledWith('session-123', {
          section_name: 'Overview',
          validation_status: 'approved',
          quality_score: 85,
        });
      });
    });

    it('should not validate when sectionContent is empty', async () => {
      render(<SectionValidationPanel {...defaultProps} sectionContent="" />);

      await waitFor(() => {
        expect(ideateService.validateSection).not.toHaveBeenCalled();
      });
    });

    it('should revalidate when sectionContent changes', async () => {
      const { rerender } = render(<SectionValidationPanel {...defaultProps} />);

      await waitFor(() => {
        expect(ideateService.validateSection).toHaveBeenCalledTimes(1);
      });

      rerender(<SectionValidationPanel {...defaultProps} sectionContent="Updated content" />);

      await waitFor(() => {
        expect(ideateService.validateSection).toHaveBeenCalledTimes(2);
        expect(ideateService.validateSection).toHaveBeenLastCalledWith(
          'session-123',
          'Overview',
          'Updated content'
        );
      });
    });
  });

  describe('Rendering - Valid Section', () => {
    beforeEach(async () => {
      render(<SectionValidationPanel {...defaultProps} />);
      await waitFor(() => {
        expect(screen.queryByText('Validating section quality...')).not.toBeInTheDocument();
      });
    });

    it('should render section title', () => {
      expect(screen.getByText('Section Quality')).toBeInTheDocument();
    });

    it('should render Valid badge when is_valid is true', () => {
      expect(screen.getByText('Valid')).toBeInTheDocument();
    });

    it('should render quality score', () => {
      expect(screen.getByText('Quality Score')).toBeInTheDocument();
      expect(screen.getByText('85/100 - Excellent')).toBeInTheDocument();
    });

    it('should render progress bar', () => {
      const progressBar = document.querySelector('[role="progressbar"]');
      expect(progressBar).toBeInTheDocument();
    });

    it('should render issues list', () => {
      expect(screen.getByText('Issues Found:')).toBeInTheDocument();
      expect(screen.getByText('Issue 1')).toBeInTheDocument();
      expect(screen.getByText('Issue 2')).toBeInTheDocument();
    });

    it('should render suggestions list', () => {
      expect(screen.getByText('Suggestions:')).toBeInTheDocument();
      expect(screen.getByText('Suggestion 1')).toBeInTheDocument();
      expect(screen.getByText('Suggestion 2')).toBeInTheDocument();
    });

    it('should render Continue button for valid sections', () => {
      const continueButton = screen.getByRole('button', { name: /Continue/i });
      expect(continueButton).toBeInTheDocument();
      expect(continueButton).toHaveTextContent('Continue');
    });

    it('should render Regenerate button', () => {
      expect(screen.getByRole('button', { name: /Regenerate/i })).toBeInTheDocument();
    });
  });

  describe('Rendering - Invalid/Low Quality Section', () => {
    beforeEach(async () => {
      (ideateService.validateSection as any).mockResolvedValue({
        ...mockValidationResult,
        is_valid: false,
        quality_score: 50,
      });

      render(<SectionValidationPanel {...defaultProps} />);
      await waitFor(() => {
        expect(screen.queryByText('Validating section quality...')).not.toBeInTheDocument();
      });
    });

    it('should render Needs Attention badge when is_valid is false', () => {
      expect(screen.getByText('Needs Attention')).toBeInTheDocument();
    });

    it('should render Continue Anyway button for low quality sections', () => {
      expect(screen.getByRole('button', { name: /Continue Anyway/i })).toBeInTheDocument();
    });

    it('should show Needs Improvement label for score < 60', () => {
      expect(screen.getByText('50/100 - Needs Improvement')).toBeInTheDocument();
    });
  });

  describe('Quality Score Labels', () => {
    it('should show Excellent for score >= 80', async () => {
      (ideateService.validateSection as any).mockResolvedValue({
        ...mockValidationResult,
        quality_score: 85,
      });

      render(<SectionValidationPanel {...defaultProps} />);
      await waitFor(() => {
        expect(screen.getByText(/Excellent/)).toBeInTheDocument();
      });
    });

    it('should show Good for score between 60-79', async () => {
      (ideateService.validateSection as any).mockResolvedValue({
        ...mockValidationResult,
        quality_score: 70,
      });

      render(<SectionValidationPanel {...defaultProps} />);
      await waitFor(() => {
        expect(screen.getByText(/Good/)).toBeInTheDocument();
      });
    });

    it('should show Needs Improvement for score < 60', async () => {
      (ideateService.validateSection as any).mockResolvedValue({
        ...mockValidationResult,
        quality_score: 50,
      });

      render(<SectionValidationPanel {...defaultProps} />);
      await waitFor(() => {
        expect(screen.getByText(/Needs Improvement/)).toBeInTheDocument();
      });
    });
  });

  describe('User Actions', () => {
    it('should call onContinue when Continue button is clicked', async () => {
      const user = userEvent.setup();
      const onContinue = vi.fn();

      render(<SectionValidationPanel {...defaultProps} onContinue={onContinue} />);

      await waitFor(() => {
        expect(screen.queryByText('Validating section quality...')).not.toBeInTheDocument();
      });

      const continueButton = screen.getByRole('button', { name: /Continue/i });
      await user.click(continueButton);

      expect(onContinue).toHaveBeenCalledTimes(1);
    });

    it('should call onRegenerate when Regenerate button is clicked', async () => {
      const user = userEvent.setup();
      const onRegenerate = vi.fn();

      render(<SectionValidationPanel {...defaultProps} onRegenerate={onRegenerate} />);

      await waitFor(() => {
        expect(screen.queryByText('Validating section quality...')).not.toBeInTheDocument();
      });

      const regenerateButton = screen.getByRole('button', { name: /Regenerate/i });
      await user.click(regenerateButton);

      expect(onRegenerate).toHaveBeenCalledTimes(1);
    });

    it('should call onContinue when Continue Anyway button is clicked', async () => {
      const user = userEvent.setup();
      const onContinue = vi.fn();

      (ideateService.validateSection as any).mockResolvedValue({
        ...mockValidationResult,
        quality_score: 50,
      });

      render(<SectionValidationPanel {...defaultProps} onContinue={onContinue} />);

      await waitFor(() => {
        expect(screen.queryByText('Validating section quality...')).not.toBeInTheDocument();
      });

      const continueButton = screen.getByRole('button', { name: /Continue Anyway/i });
      await user.click(continueButton);

      expect(onContinue).toHaveBeenCalledTimes(1);
    });
  });

  describe('Error Handling', () => {
    it('should handle validation API error gracefully', async () => {
      const consoleError = vi.spyOn(console, 'error').mockImplementation(() => {});
      (ideateService.validateSection as any).mockRejectedValue(new Error('API Error'));

      render(<SectionValidationPanel {...defaultProps} />);

      await waitFor(() => {
        expect(consoleError).toHaveBeenCalledWith('Failed to validate section:', expect.any(Error));
      });

      consoleError.mockRestore();
    });

    it('should handle storeValidationFeedback error silently', async () => {
      (ideateService.storeValidationFeedback as any).mockRejectedValue(new Error('Storage Error'));

      render(<SectionValidationPanel {...defaultProps} />);

      // Should still render validation result
      await waitFor(() => {
        expect(screen.getByText('Section Quality')).toBeInTheDocument();
      });
    });
  });

  describe('Edge Cases', () => {
    it('should handle empty issues array', async () => {
      (ideateService.validateSection as any).mockResolvedValue({
        ...mockValidationResult,
        issues: [],
      });

      render(<SectionValidationPanel {...defaultProps} />);

      await waitFor(() => {
        expect(screen.queryByText('Issues Found:')).not.toBeInTheDocument();
      });
    });

    it('should handle empty suggestions array', async () => {
      (ideateService.validateSection as any).mockResolvedValue({
        ...mockValidationResult,
        suggestions: [],
      });

      render(<SectionValidationPanel {...defaultProps} />);

      await waitFor(() => {
        expect(screen.queryByText('Suggestions:')).not.toBeInTheDocument();
      });
    });

    it('should handle score of 0', async () => {
      (ideateService.validateSection as any).mockResolvedValue({
        ...mockValidationResult,
        quality_score: 0,
      });

      render(<SectionValidationPanel {...defaultProps} />);

      await waitFor(() => {
        expect(screen.getByText('0/100 - Needs Improvement')).toBeInTheDocument();
      });
    });

    it('should handle score of 100', async () => {
      (ideateService.validateSection as any).mockResolvedValue({
        ...mockValidationResult,
        quality_score: 100,
      });

      render(<SectionValidationPanel {...defaultProps} />);

      await waitFor(() => {
        expect(screen.getByText('100/100 - Excellent')).toBeInTheDocument();
      });
    });
  });

  describe('Conditional Rendering', () => {
    it('should render nothing when validation is null and not validating', async () => {
      (ideateService.validateSection as any).mockResolvedValue(null);

      const { container } = render(<SectionValidationPanel {...defaultProps} sectionContent="" />);

      await waitFor(() => {
        expect(container.firstChild).toBeNull();
      });
    });

    it('should show different border color for valid vs invalid sections', async () => {
      const { container, rerender } = render(<SectionValidationPanel {...defaultProps} />);

      await waitFor(() => {
        const card = container.querySelector('[class*="border-green"]');
        expect(card).toBeInTheDocument();
      });

      (ideateService.validateSection as any).mockResolvedValue({
        ...mockValidationResult,
        is_valid: false,
      });

      rerender(<SectionValidationPanel {...defaultProps} sectionContent="Updated" />);

      await waitFor(() => {
        const card = container.querySelector('[class*="border-yellow"]');
        expect(card).toBeInTheDocument();
      });
    });
  });
});
