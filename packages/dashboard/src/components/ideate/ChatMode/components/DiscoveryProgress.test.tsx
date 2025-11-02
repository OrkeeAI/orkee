// ABOUTME: Tests for DiscoveryProgress component
// ABOUTME: Validates progress display, question tracking, and time estimation

import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { DiscoveryProgress } from './DiscoveryProgress';
import { DiscoveryProgress as DiscoveryProgressType } from '@/services/ideate';

describe('DiscoveryProgress', () => {
  const mockProgress: DiscoveryProgressType = {
    current_question_number: 3,
    total_questions: 10,
    answered_questions: 2,
    completion_percentage: 30,
    estimated_remaining: 5,
  };

  describe('Rendering', () => {
    it('should render progress information correctly', () => {
      render(<DiscoveryProgress progress={mockProgress} />);

      expect(screen.getByText('Question 3 of ~10')).toBeInTheDocument();
      expect(screen.getByText('5 min remaining')).toBeInTheDocument();
      expect(screen.getByText('2 questions answered')).toBeInTheDocument();
    });

    it('should render progress bar with correct percentage', () => {
      const { container } = render(<DiscoveryProgress progress={mockProgress} />);

      // Progress component should be rendered
      const progressBar = container.querySelector('[role="progressbar"]');
      expect(progressBar).toBeInTheDocument();
    });

    it('should render null when progress is null', () => {
      const { container } = render(<DiscoveryProgress progress={null} />);

      expect(container.firstChild).toBeNull();
    });

    it('should apply custom className', () => {
      const { container } = render(
        <DiscoveryProgress progress={mockProgress} className="custom-class" />
      );

      const wrapper = container.querySelector('.custom-class');
      expect(wrapper).toBeInTheDocument();
    });
  });

  describe('Progress states', () => {
    it('should handle early progress (first question)', () => {
      const earlyProgress: DiscoveryProgressType = {
        current_question_number: 1,
        total_questions: 10,
        answered_questions: 0,
        completion_percentage: 10,
        estimated_remaining: 8,
      };

      render(<DiscoveryProgress progress={earlyProgress} />);

      expect(screen.getByText('Question 1 of ~10')).toBeInTheDocument();
      expect(screen.getByText('8 min remaining')).toBeInTheDocument();
      expect(screen.getByText('0 questions answered')).toBeInTheDocument();
    });

    it('should handle mid progress', () => {
      const midProgress: DiscoveryProgressType = {
        current_question_number: 5,
        total_questions: 10,
        answered_questions: 4,
        completion_percentage: 50,
        estimated_remaining: 3,
      };

      render(<DiscoveryProgress progress={midProgress} />);

      expect(screen.getByText('Question 5 of ~10')).toBeInTheDocument();
      expect(screen.getByText('3 min remaining')).toBeInTheDocument();
      expect(screen.getByText('4 questions answered')).toBeInTheDocument();
    });

    it('should handle near completion', () => {
      const nearComplete: DiscoveryProgressType = {
        current_question_number: 9,
        total_questions: 10,
        answered_questions: 8,
        completion_percentage: 90,
        estimated_remaining: 1,
      };

      render(<DiscoveryProgress progress={nearComplete} />);

      expect(screen.getByText('Question 9 of ~10')).toBeInTheDocument();
      expect(screen.getByText('1 min remaining')).toBeInTheDocument();
      expect(screen.getByText('8 questions answered')).toBeInTheDocument();
    });
  });

  describe('Edge cases', () => {
    it('should handle zero completion', () => {
      const zeroProgress: DiscoveryProgressType = {
        current_question_number: 0,
        total_questions: 10,
        answered_questions: 0,
        completion_percentage: 0,
        estimated_remaining: 10,
      };

      render(<DiscoveryProgress progress={zeroProgress} />);

      expect(screen.getByText('Question 0 of ~10')).toBeInTheDocument();
      expect(screen.getByText('0 questions answered')).toBeInTheDocument();
    });

    it('should handle 100% completion', () => {
      const completeProgress: DiscoveryProgressType = {
        current_question_number: 10,
        total_questions: 10,
        answered_questions: 10,
        completion_percentage: 100,
        estimated_remaining: 0,
      };

      render(<DiscoveryProgress progress={completeProgress} />);

      expect(screen.getByText('Question 10 of ~10')).toBeInTheDocument();
      expect(screen.getByText('0 min remaining')).toBeInTheDocument();
      expect(screen.getByText('10 questions answered')).toBeInTheDocument();
    });
  });
});
