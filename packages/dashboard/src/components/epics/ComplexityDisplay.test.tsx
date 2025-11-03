// ABOUTME: Tests for ComplexityDisplay component
// ABOUTME: Validates complexity score rendering, labels, and recommendations

import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { ComplexityDisplay } from './ComplexityDisplay';
import type { ComplexityReport } from '@/services/epics';

describe('ComplexityDisplay', () => {
  const mockComplexityReport: ComplexityReport = {
    complexityScore: 5,
    recommendedTaskCount: 8,
    reasoning: 'This epic involves multiple components and data flows that need careful coordination.',
    expansionStrategy: 'Break down into smaller tasks focusing on individual components first, then integration.',
  };

  describe('Null State', () => {
    it('should display message when report is null', () => {
      render(<ComplexityDisplay report={null} />);

      expect(screen.getByText('No complexity analysis available')).toBeInTheDocument();
    });

    it('should display alert icon when report is null', () => {
      const { container } = render(<ComplexityDisplay report={null} />);

      const icon = container.querySelector('svg.lucide-alert-circle');
      expect(icon).toBeInTheDocument();
    });

    it('should not display any complexity cards when report is null', () => {
      render(<ComplexityDisplay report={null} />);

      expect(screen.queryByText('Complexity Analysis')).not.toBeInTheDocument();
      expect(screen.queryByText('Reasoning')).not.toBeInTheDocument();
      expect(screen.queryByText('Expansion Strategy')).not.toBeInTheDocument();
    });
  });

  describe('Complexity Analysis Card', () => {
    it('should render complexity analysis title', () => {
      render(<ComplexityDisplay report={mockComplexityReport} />);

      expect(screen.getByText('Complexity Analysis')).toBeInTheDocument();
    });

    it('should display complexity score', () => {
      render(<ComplexityDisplay report={mockComplexityReport} />);

      expect(screen.getByText('Complexity Score')).toBeInTheDocument();
      expect(screen.getByText('5/10')).toBeInTheDocument();
    });

    it('should display recommended task count', () => {
      render(<ComplexityDisplay report={mockComplexityReport} />);

      expect(screen.getByText('Recommended Tasks')).toBeInTheDocument();
      expect(screen.getByText('8')).toBeInTheDocument();
    });

    it('should render progress bar', () => {
      const { container } = render(<ComplexityDisplay report={mockComplexityReport} />);

      const progressBars = container.querySelectorAll('[role="progressbar"]');
      expect(progressBars.length).toBeGreaterThan(0);
    });
  });

  describe('Complexity Labels and Colors', () => {
    it('should show Low label with green color for score 1-3', () => {
      const lowComplexityReport = { ...mockComplexityReport, complexityScore: 3 };
      render(<ComplexityDisplay report={lowComplexityReport} />);

      const badge = screen.getByText('Low');
      expect(badge).toBeInTheDocument();
      expect(badge).toHaveClass('bg-green-500');
    });

    it('should show Medium label with yellow color for score 4-6', () => {
      const mediumComplexityReport = { ...mockComplexityReport, complexityScore: 5 };
      render(<ComplexityDisplay report={mediumComplexityReport} />);

      const badge = screen.getByText('Medium');
      expect(badge).toBeInTheDocument();
      expect(badge).toHaveClass('bg-yellow-500');
    });

    it('should show High label with orange color for score 7-8', () => {
      const highComplexityReport = { ...mockComplexityReport, complexityScore: 8 };
      render(<ComplexityDisplay report={highComplexityReport} />);

      const badge = screen.getByText('High');
      expect(badge).toBeInTheDocument();
      expect(badge).toHaveClass('bg-orange-500');
    });

    it('should show Very High label with red color for score 9-10', () => {
      const veryHighComplexityReport = { ...mockComplexityReport, complexityScore: 10 };
      render(<ComplexityDisplay report={veryHighComplexityReport} />);

      const badge = screen.getByText('Very High');
      expect(badge).toBeInTheDocument();
      expect(badge).toHaveClass('bg-red-500');
    });
  });

  describe('Complexity Score Boundaries', () => {
    it('should handle score of 0 as Low complexity', () => {
      const report = { ...mockComplexityReport, complexityScore: 0 };
      render(<ComplexityDisplay report={report} />);

      expect(screen.getByText('Low')).toBeInTheDocument();
      expect(screen.getByText('0/10')).toBeInTheDocument();
    });

    it('should handle score at boundary (3) as Low complexity', () => {
      const report = { ...mockComplexityReport, complexityScore: 3 };
      render(<ComplexityDisplay report={report} />);

      expect(screen.getByText('Low')).toBeInTheDocument();
      expect(screen.getByText('3/10')).toBeInTheDocument();
    });

    it('should handle score at boundary (4) as Medium complexity', () => {
      const report = { ...mockComplexityReport, complexityScore: 4 };
      render(<ComplexityDisplay report={report} />);

      expect(screen.getByText('Medium')).toBeInTheDocument();
      expect(screen.getByText('4/10')).toBeInTheDocument();
    });

    it('should handle score at boundary (6) as Medium complexity', () => {
      const report = { ...mockComplexityReport, complexityScore: 6 };
      render(<ComplexityDisplay report={report} />);

      expect(screen.getByText('Medium')).toBeInTheDocument();
      expect(screen.getByText('6/10')).toBeInTheDocument();
    });

    it('should handle score at boundary (7) as High complexity', () => {
      const report = { ...mockComplexityReport, complexityScore: 7 };
      render(<ComplexityDisplay report={report} />);

      expect(screen.getByText('High')).toBeInTheDocument();
      expect(screen.getByText('7/10')).toBeInTheDocument();
    });

    it('should handle score at boundary (8) as High complexity', () => {
      const report = { ...mockComplexityReport, complexityScore: 8 };
      render(<ComplexityDisplay report={report} />);

      expect(screen.getByText('High')).toBeInTheDocument();
      expect(screen.getByText('8/10')).toBeInTheDocument();
    });

    it('should handle score at boundary (9) as Very High complexity', () => {
      const report = { ...mockComplexityReport, complexityScore: 9 };
      render(<ComplexityDisplay report={report} />);

      expect(screen.getByText('Very High')).toBeInTheDocument();
      expect(screen.getByText('9/10')).toBeInTheDocument();
    });
  });

  describe('Reasoning Card', () => {
    it('should render reasoning title with icon', () => {
      const { container } = render(<ComplexityDisplay report={mockComplexityReport} />);

      expect(screen.getByText('Reasoning')).toBeInTheDocument();
      const icon = container.querySelector('svg.lucide-check-circle-2');
      expect(icon).toBeInTheDocument();
    });

    it('should display reasoning text', () => {
      render(<ComplexityDisplay report={mockComplexityReport} />);

      expect(screen.getByText(mockComplexityReport.reasoning)).toBeInTheDocument();
    });

    it('should preserve whitespace in reasoning text', () => {
      const reportWithMultilineReasoning = {
        ...mockComplexityReport,
        reasoning: 'Line 1\n\nLine 2\nLine 3',
      };
      render(<ComplexityDisplay report={reportWithMultilineReasoning} />);

      const reasoningElement = screen.getByText(/Line 1/);
      expect(reasoningElement).toHaveClass('whitespace-pre-wrap');
    });
  });

  describe('Expansion Strategy Card', () => {
    it('should render expansion strategy title with icon', () => {
      const { container } = render(<ComplexityDisplay report={mockComplexityReport} />);

      expect(screen.getByText('Expansion Strategy')).toBeInTheDocument();
      const icon = container.querySelector('svg.lucide-trending-up');
      expect(icon).toBeInTheDocument();
    });

    it('should display expansion strategy text', () => {
      render(<ComplexityDisplay report={mockComplexityReport} />);

      expect(screen.getByText(mockComplexityReport.expansionStrategy)).toBeInTheDocument();
    });

    it('should preserve whitespace in expansion strategy text', () => {
      const reportWithMultilineStrategy = {
        ...mockComplexityReport,
        expansionStrategy: 'Step 1\n\nStep 2\nStep 3',
      };
      render(<ComplexityDisplay report={reportWithMultilineStrategy} />);

      const strategyElement = screen.getByText(/Step 1/);
      expect(strategyElement).toHaveClass('whitespace-pre-wrap');
    });
  });

  describe('Task Count vs. Limit Card', () => {
    it('should render task count vs limit title', () => {
      render(<ComplexityDisplay report={mockComplexityReport} />);

      expect(screen.getByText('Task Count vs. Limit')).toBeInTheDocument();
    });

    it('should display recommended task count', () => {
      render(<ComplexityDisplay report={mockComplexityReport} />);

      expect(screen.getByText('Recommended: 8')).toBeInTheDocument();
    });

    it('should display task limit of 20', () => {
      render(<ComplexityDisplay report={mockComplexityReport} />);

      expect(screen.getByText('Limit: 20')).toBeInTheDocument();
    });

    it('should calculate correct progress percentage', () => {
      const { container } = render(<ComplexityDisplay report={mockComplexityReport} />);

      // 8/20 = 40% progress
      const progressBars = container.querySelectorAll('[role="progressbar"]');
      // Second progress bar is for task count vs limit
      expect(progressBars.length).toBeGreaterThan(1);
    });
  });

  describe('Edge Cases', () => {
    it('should handle empty reasoning text', () => {
      const reportWithEmptyReasoning = { ...mockComplexityReport, reasoning: '' };
      render(<ComplexityDisplay report={reportWithEmptyReasoning} />);

      expect(screen.getByText('Reasoning')).toBeInTheDocument();
    });

    it('should handle empty expansion strategy text', () => {
      const reportWithEmptyStrategy = { ...mockComplexityReport, expansionStrategy: '' };
      render(<ComplexityDisplay report={reportWithEmptyStrategy} />);

      expect(screen.getByText('Expansion Strategy')).toBeInTheDocument();
    });

    it('should handle recommended task count of 0', () => {
      const reportWithZeroTasks = { ...mockComplexityReport, recommendedTaskCount: 0 };
      render(<ComplexityDisplay report={reportWithZeroTasks} />);

      expect(screen.getByText('0')).toBeInTheDocument();
      expect(screen.getByText('Recommended: 0')).toBeInTheDocument();
    });

    it('should handle recommended task count exceeding limit', () => {
      const reportWithManyTasks = { ...mockComplexityReport, recommendedTaskCount: 25 };
      render(<ComplexityDisplay report={reportWithManyTasks} />);

      expect(screen.getByText('25')).toBeInTheDocument();
      expect(screen.getByText('Recommended: 25')).toBeInTheDocument();
    });

    it('should handle maximum complexity score', () => {
      const reportWithMaxScore = { ...mockComplexityReport, complexityScore: 10 };
      render(<ComplexityDisplay report={reportWithMaxScore} />);

      expect(screen.getByText('10/10')).toBeInTheDocument();
      expect(screen.getByText('Very High')).toBeInTheDocument();
    });

    it('should handle score greater than 10', () => {
      const reportWithOverMaxScore = { ...mockComplexityReport, complexityScore: 15 };
      render(<ComplexityDisplay report={reportWithOverMaxScore} />);

      expect(screen.getByText('15/10')).toBeInTheDocument();
      expect(screen.getByText('Very High')).toBeInTheDocument();
    });
  });

  describe('Card Layout', () => {
    it('should render all four cards when report is provided', () => {
      render(<ComplexityDisplay report={mockComplexityReport} />);

      expect(screen.getByText('Complexity Analysis')).toBeInTheDocument();
      expect(screen.getByText('Reasoning')).toBeInTheDocument();
      expect(screen.getByText('Expansion Strategy')).toBeInTheDocument();
      expect(screen.getByText('Task Count vs. Limit')).toBeInTheDocument();
    });

    it('should render cards in correct order', () => {
      const { container } = render(<ComplexityDisplay report={mockComplexityReport} />);

      const cards = container.querySelectorAll('[class*="space-y-4"] > div');
      expect(cards.length).toBe(4);
    });
  });

  describe('Progress Bar Rendering', () => {
    it('should render two progress bars', () => {
      const { container } = render(<ComplexityDisplay report={mockComplexityReport} />);

      const progressBars = container.querySelectorAll('[role="progressbar"]');
      expect(progressBars).toHaveLength(2);
    });

    it('should render complexity progress bar with correct value', () => {
      const { container } = render(<ComplexityDisplay report={mockComplexityReport} />);

      // Complexity score of 5 should be 50% (5 * 10)
      const progressBars = container.querySelectorAll('[role="progressbar"]');
      expect(progressBars.length).toBeGreaterThan(0);
    });

    it('should render task count progress bar with correct value', () => {
      const { container } = render(<ComplexityDisplay report={mockComplexityReport} />);

      // 8/20 tasks should be 40%
      const progressBars = container.querySelectorAll('[role="progressbar"]');
      expect(progressBars.length).toBe(2);
    });
  });
});
