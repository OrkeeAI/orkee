// ABOUTME: Tests for ValidationResultsPanel component
// ABOUTME: Validates rendering of task validation results with scenarios and recommendations

import React from 'react';
import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';

// Mock Card components to bypass Radix UI React duplicate instance issue
vi.mock('@/components/ui/card', () => ({
  Card: ({ children, className }: { children: React.ReactNode; className?: string }) => (
    <div className={className}>{children}</div>
  ),
  CardContent: ({ children, className }: { children: React.ReactNode; className?: string }) => (
    <div className={className}>{children}</div>
  ),
  CardDescription: ({ children }: { children: React.ReactNode }) => <p>{children}</p>,
  CardHeader: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
  CardTitle: ({ children }: { children: React.ReactNode }) => <h3>{children}</h3>,
}));

// Mock Badge component
vi.mock('@/components/ui/badge', () => ({
  Badge: ({ children, variant, className }: { children: React.ReactNode; variant?: string; className?: string }) => (
    <span data-variant={variant} className={className}>{children}</span>
  ),
}));

// Mock Alert components
vi.mock('@/components/ui/alert', () => ({
  Alert: ({ children, variant }: { children: React.ReactNode; variant?: string }) => (
    <div data-variant={variant}>{children}</div>
  ),
  AlertDescription: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
  AlertTitle: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
}));

// Mock Progress component
vi.mock('@/components/ui/progress', () => ({
  Progress: ({ value, className }: { value: number; className?: string }) => (
    <div data-value={value} className={className} role="progressbar" aria-valuenow={value} />
  ),
}));

// Mock Separator component
vi.mock('@/components/ui/separator', () => ({
  Separator: () => <hr />,
}));

// Mock lucide-react icons
vi.mock('lucide-react', () => ({
  CheckCircle: () => null,
  XCircle: () => null,
  AlertCircle: () => null,
  Info: () => null,
}));

import { ValidationResultsPanel } from './ValidationResultsPanel';
import type { TaskValidation } from '@/lib/ai/schemas';

describe('ValidationResultsPanel', () => {
  const mockPassedValidation: TaskValidation = {
    overallAssessment: {
      passed: true,
      notes: 'All scenarios passed successfully',
    },
    scenarioResults: [
      {
        scenarioName: 'Valid login credentials',
        passed: true,
        confidence: 0.95,
        notes: 'Authentication flow works correctly',
      },
      {
        scenarioName: 'Invalid password',
        passed: true,
        confidence: 0.88,
        notes: 'Error handling implemented properly',
      },
    ],
    recommendations: ['Consider adding rate limiting', 'Add logging for failed attempts'],
  };

  const mockFailedValidation: TaskValidation = {
    overallAssessment: {
      passed: false,
      notes: 'Critical scenarios failed - requires immediate attention',
    },
    scenarioResults: [
      {
        scenarioName: 'Session timeout',
        passed: false,
        confidence: 0.45,
        notes: 'Session timeout not implemented',
      },
      {
        scenarioName: 'Token refresh',
        passed: true,
        confidence: 0.92,
        notes: 'Token refresh works as expected',
      },
    ],
    recommendations: [
      'Implement session timeout mechanism',
      'Add comprehensive timeout tests',
    ],
  };

  describe('Overall assessment display', () => {
    it('should render passed validation with correct badge', () => {
      const { container } = render(<ValidationResultsPanel validation={mockPassedValidation} />);

      expect(screen.getByText('Validation Results')).toBeInTheDocument();
      // Check for "Passed" badge in the header (not in scenario results)
      const badges = screen.getAllByText('Passed');
      expect(badges.length).toBeGreaterThan(0);
      expect(screen.getByText('All scenarios passed successfully')).toBeInTheDocument();
    });

    it('should render failed validation with correct badge', () => {
      const { container } = render(<ValidationResultsPanel validation={mockFailedValidation} />);

      // Check for "Failed" badge
      const failedBadges = screen.getAllByText('Failed');
      expect(failedBadges.length).toBeGreaterThan(0);
      expect(
        screen.getByText('Critical scenarios failed - requires immediate attention')
      ).toBeInTheDocument();
    });
  });

  describe('Pass rate calculation', () => {
    it('should calculate and display correct pass rate', () => {
      render(<ValidationResultsPanel validation={mockPassedValidation} />);

      // 2 passed out of 2 scenarios = 100%
      expect(screen.getByText('2 / 2 scenarios (100%)')).toBeInTheDocument();
    });

    it('should calculate partial pass rate correctly', () => {
      render(<ValidationResultsPanel validation={mockFailedValidation} />);

      // 1 passed out of 2 scenarios = 50%
      expect(screen.getByText('1 / 2 scenarios (50%)')).toBeInTheDocument();
    });

    it('should handle zero scenarios gracefully', () => {
      const emptyValidation: TaskValidation = {
        overallAssessment: { passed: true, notes: 'No scenarios to validate' },
        scenarioResults: [],
        recommendations: [],
      };

      render(<ValidationResultsPanel validation={emptyValidation} />);

      expect(screen.getByText('0 / 0 scenarios (0%)')).toBeInTheDocument();
    });
  });

  describe('Scenario results display', () => {
    it('should render all scenario results', () => {
      render(<ValidationResultsPanel validation={mockPassedValidation} />);

      expect(screen.getByText('Valid login credentials')).toBeInTheDocument();
      expect(screen.getByText('Invalid password')).toBeInTheDocument();
    });

    it('should display confidence levels as percentages', () => {
      render(<ValidationResultsPanel validation={mockPassedValidation} />);

      // 0.95 * 100 = 95%
      expect(screen.getByText('95%')).toBeInTheDocument();
      // 0.88 * 100 = 88%
      expect(screen.getByText('88%')).toBeInTheDocument();
    });

    it('should show scenario notes', () => {
      render(<ValidationResultsPanel validation={mockPassedValidation} />);

      expect(screen.getByText('Authentication flow works correctly')).toBeInTheDocument();
      expect(screen.getByText('Error handling implemented properly')).toBeInTheDocument();
    });

    it('should apply different styling for passed and failed scenarios', () => {
      const { container } = render(
        <ValidationResultsPanel validation={mockFailedValidation} />
      );

      // Check for passed scenario styling (green)
      const passedScenarios = container.querySelectorAll('.border-green-200');
      expect(passedScenarios.length).toBeGreaterThan(0);

      // Check for failed scenario styling (red)
      const failedScenarios = container.querySelectorAll('.border-red-200');
      expect(failedScenarios.length).toBeGreaterThan(0);

      // Verify both "Passed" and "Failed" badges are present
      const passedBadges = screen.getAllByText('Passed');
      const failedBadges = screen.getAllByText('Failed');
      expect(passedBadges.length).toBeGreaterThan(0);
      expect(failedBadges.length).toBeGreaterThan(0);
    });

    it('should display confidence level descriptions', () => {
      render(<ValidationResultsPanel validation={mockPassedValidation} />);

      // There are two scenarios with high confidence, so use getAllByText
      const confidenceLabels = screen.getAllByText('High confidence');
      expect(confidenceLabels.length).toBe(2); // Both scenarios have confidence > 0.8
    });
  });

  describe('Recommendations display', () => {
    it('should render all recommendations', () => {
      render(<ValidationResultsPanel validation={mockPassedValidation} />);

      expect(screen.getByText('Recommendations')).toBeInTheDocument();
      expect(screen.getByText('Consider adding rate limiting')).toBeInTheDocument();
      expect(screen.getByText('Add logging for failed attempts')).toBeInTheDocument();
    });

    it('should not render recommendations section when empty', () => {
      const noRecommendations: TaskValidation = {
        ...mockPassedValidation,
        recommendations: [],
      };

      render(<ValidationResultsPanel validation={noRecommendations} />);

      expect(screen.queryByText('Recommendations')).not.toBeInTheDocument();
    });

    it('should not render recommendations section when undefined', () => {
      const noRecommendations: TaskValidation = {
        ...mockPassedValidation,
        recommendations: undefined,
      };

      render(<ValidationResultsPanel validation={noRecommendations} />);

      expect(screen.queryByText('Recommendations')).not.toBeInTheDocument();
    });
  });

  describe('Confidence level indicators', () => {
    it('should show high confidence for values > 0.8', () => {
      const highConfidence: TaskValidation = {
        overallAssessment: { passed: true, notes: 'Test' },
        scenarioResults: [
          {
            scenarioName: 'High confidence test',
            passed: true,
            confidence: 0.92,
            notes: '',
          },
        ],
        recommendations: [],
      };

      render(<ValidationResultsPanel validation={highConfidence} />);
      expect(screen.getByText('High confidence')).toBeInTheDocument();
    });

    it('should show medium confidence for values between 0.5 and 0.8', () => {
      const mediumConfidence: TaskValidation = {
        overallAssessment: { passed: true, notes: 'Test' },
        scenarioResults: [
          {
            scenarioName: 'Medium confidence test',
            passed: true,
            confidence: 0.65,
            notes: '',
          },
        ],
        recommendations: [],
      };

      render(<ValidationResultsPanel validation={mediumConfidence} />);
      expect(screen.getByText('Medium confidence')).toBeInTheDocument();
    });

    it('should show low confidence for values <= 0.5', () => {
      const lowConfidence: TaskValidation = {
        overallAssessment: { passed: false, notes: 'Test' },
        scenarioResults: [
          {
            scenarioName: 'Low confidence test',
            passed: false,
            confidence: 0.35,
            notes: '',
          },
        ],
        recommendations: [],
      };

      render(<ValidationResultsPanel validation={lowConfidence} />);
      expect(screen.getByText('Low confidence')).toBeInTheDocument();
    });
  });

  describe('Custom className', () => {
    it('should apply custom className to root element', () => {
      const { container } = render(
        <ValidationResultsPanel
          validation={mockPassedValidation}
          className="custom-class"
        />
      );

      const card = container.querySelector('.custom-class');
      expect(card).toBeInTheDocument();
    });
  });

  describe('Edge cases', () => {
    it('should handle scenario without notes', () => {
      const noNotes: TaskValidation = {
        overallAssessment: { passed: true, notes: 'Test' },
        scenarioResults: [
          {
            scenarioName: 'Test scenario',
            passed: true,
            confidence: 0.9,
          },
        ],
        recommendations: [],
      };

      render(<ValidationResultsPanel validation={noNotes} />);
      expect(screen.getByText('Test scenario')).toBeInTheDocument();
    });

    it('should round confidence percentages correctly', () => {
      const preciseConfidence: TaskValidation = {
        overallAssessment: { passed: true, notes: 'Test' },
        scenarioResults: [
          {
            scenarioName: 'Precise confidence',
            passed: true,
            confidence: 0.876, // Should round to 88%
            notes: '',
          },
        ],
        recommendations: [],
      };

      render(<ValidationResultsPanel validation={preciseConfidence} />);
      expect(screen.getByText('88%')).toBeInTheDocument();
    });
  });
});
