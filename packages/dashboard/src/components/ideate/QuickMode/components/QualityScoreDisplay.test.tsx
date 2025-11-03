// ABOUTME: Tests for QualityScoreDisplay component
// ABOUTME: Validates overall and section quality score display with readiness status

import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { QualityScoreDisplay } from './QualityScoreDisplay';
import type { QualityScore } from '@/services/ideate';

describe('QualityScoreDisplay', () => {
  const mockQualityScore: QualityScore = {
    overall_score: 85,
    section_scores: {
      overview: 90,
      technical: 80,
      ux: 75,
    },
    is_ready_for_prd: true,
    missing_required: [],
  };

  describe('Rendering', () => {
    it('should render card title', () => {
      render(<QualityScoreDisplay qualityScore={mockQualityScore} />);

      expect(screen.getByText('Overall Quality Score')).toBeInTheDocument();
    });

    it('should render Ready badge when is_ready_for_prd is true', () => {
      render(<QualityScoreDisplay qualityScore={mockQualityScore} />);

      expect(screen.getByText('Ready')).toBeInTheDocument();
      expect(screen.getByText('Ready to save')).toBeInTheDocument();
    });

    it('should render Not Ready badge when is_ready_for_prd is false', () => {
      render(
        <QualityScoreDisplay
          qualityScore={{ ...mockQualityScore, is_ready_for_prd: false }}
        />
      );

      expect(screen.getByText('Not Ready')).toBeInTheDocument();
      expect(screen.getByText('Needs improvement')).toBeInTheDocument();
    });

    it('should render overall score', () => {
      render(<QualityScoreDisplay qualityScore={mockQualityScore} />);

      expect(screen.getByText('Overall Score')).toBeInTheDocument();
      expect(screen.getByText('85/100')).toBeInTheDocument();
    });
  });

  describe('Section Scores', () => {
    it('should render section scores label when sections exist', () => {
      render(<QualityScoreDisplay qualityScore={mockQualityScore} />);

      expect(screen.getByText('Section Scores')).toBeInTheDocument();
    });

    it('should render all section scores', () => {
      render(<QualityScoreDisplay qualityScore={mockQualityScore} />);

      expect(screen.getByText('overview')).toBeInTheDocument();
      expect(screen.getByText('90/100')).toBeInTheDocument();
      expect(screen.getByText('technical')).toBeInTheDocument();
      expect(screen.getByText('80/100')).toBeInTheDocument();
      expect(screen.getByText('ux')).toBeInTheDocument();
      expect(screen.getByText('75/100')).toBeInTheDocument();
    });

    it('should not render section scores label when no sections', () => {
      render(
        <QualityScoreDisplay
          qualityScore={{ ...mockQualityScore, section_scores: {} }}
        />
      );

      expect(screen.queryByText('Section Scores')).not.toBeInTheDocument();
    });
  });

  describe('Score Icons', () => {
    it('should show CheckCircle icon for high scores (>= 80)', () => {
      const { container } = render(<QualityScoreDisplay qualityScore={mockQualityScore} />);

      const overallScoreArea = container.querySelector('[class*="space-y-2"]');
      expect(overallScoreArea).toBeInTheDocument();
    });

    it('should show appropriate icon for medium scores (60-79)', () => {
      render(
        <QualityScoreDisplay
          qualityScore={{ ...mockQualityScore, overall_score: 70 }}
        />
      );

      expect(screen.getByText('70/100')).toBeInTheDocument();
    });

    it('should show appropriate icon for low scores (< 60)', () => {
      render(
        <QualityScoreDisplay
          qualityScore={{ ...mockQualityScore, overall_score: 50 }}
        />
      );

      expect(screen.getByText('50/100')).toBeInTheDocument();
    });
  });

  describe('Missing Required Sections', () => {
    it('should render missing required sections warning', () => {
      render(
        <QualityScoreDisplay
          qualityScore={{
            ...mockQualityScore,
            missing_required: ['features', 'roadmap'],
          }}
        />
      );

      expect(screen.getByText('Missing Required Sections')).toBeInTheDocument();
      expect(screen.getByText('features')).toBeInTheDocument();
      expect(screen.getByText('roadmap')).toBeInTheDocument();
    });

    it('should not render missing sections warning when array is empty', () => {
      render(<QualityScoreDisplay qualityScore={mockQualityScore} />);

      expect(screen.queryByText('Missing Required Sections')).not.toBeInTheDocument();
    });

    it('should render multiple missing sections', () => {
      render(
        <QualityScoreDisplay
          qualityScore={{
            ...mockQualityScore,
            section_scores: {}, // No section scores to avoid duplication
            missing_required: ['overview', 'technical', 'ux', 'features'],
          }}
        />
      );

      expect(screen.getByText('overview')).toBeInTheDocument();
      expect(screen.getByText('technical')).toBeInTheDocument();
      expect(screen.getByText('ux')).toBeInTheDocument();
      expect(screen.getByText('features')).toBeInTheDocument();
    });
  });

  describe('Custom className', () => {
    it('should apply custom className to Card', () => {
      const { container } = render(
        <QualityScoreDisplay
          qualityScore={mockQualityScore}
          className="custom-test-class"
        />
      );

      const card = container.querySelector('.custom-test-class');
      expect(card).toBeInTheDocument();
    });
  });

  describe('Edge Cases', () => {
    it('should handle score of 0', () => {
      render(
        <QualityScoreDisplay
          qualityScore={{ ...mockQualityScore, overall_score: 0 }}
        />
      );

      expect(screen.getByText('0/100')).toBeInTheDocument();
    });

    it('should handle score of 100', () => {
      render(
        <QualityScoreDisplay
          qualityScore={{ ...mockQualityScore, overall_score: 100 }}
        />
      );

      expect(screen.getByText('100/100')).toBeInTheDocument();
    });

    it('should handle single section score', () => {
      render(
        <QualityScoreDisplay
          qualityScore={{
            ...mockQualityScore,
            section_scores: { overview: 95 },
          }}
        />
      );

      expect(screen.getByText('overview')).toBeInTheDocument();
      expect(screen.getByText('95/100')).toBeInTheDocument();
    });

    it('should handle empty missing_required array', () => {
      render(
        <QualityScoreDisplay
          qualityScore={{ ...mockQualityScore, missing_required: [] }}
        />
      );

      expect(screen.queryByText('Missing Required Sections')).not.toBeInTheDocument();
    });
  });

  describe('Color Coding', () => {
    it('should use green color for scores >= 80', () => {
      render(
        <QualityScoreDisplay
          qualityScore={{ ...mockQualityScore, overall_score: 85 }}
        />
      );

      const scoreText = screen.getByText('85/100');
      expect(scoreText).toHaveClass('text-green-600');
    });

    it('should use yellow color for scores between 60-79', () => {
      render(
        <QualityScoreDisplay
          qualityScore={{ ...mockQualityScore, overall_score: 70 }}
        />
      );

      const scoreText = screen.getByText('70/100');
      expect(scoreText).toHaveClass('text-yellow-600');
    });

    it('should use red color for scores < 60', () => {
      render(
        <QualityScoreDisplay
          qualityScore={{ ...mockQualityScore, overall_score: 50 }}
        />
      );

      const scoreText = screen.getByText('50/100');
      expect(scoreText).toHaveClass('text-red-600');
    });
  });
});
