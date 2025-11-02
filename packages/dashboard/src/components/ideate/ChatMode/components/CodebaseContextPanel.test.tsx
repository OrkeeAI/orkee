// ABOUTME: Tests for CodebaseContextPanel component
// ABOUTME: Validates codebase analysis display, loading states, and user interactions

import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { CodebaseContextPanel } from './CodebaseContextPanel';
import { CodebaseContext } from '@/services/ideate';

describe('CodebaseContextPanel', () => {
  const mockContext: CodebaseContext = {
    session_id: 'test-session',
    patterns: [
      {
        pattern_type: 'database',
        name: 'SQLx runtime queries',
        description: 'Uses SQLx with runtime queries',
        file_references: ['src/db.rs'],
      },
      {
        pattern_type: 'api',
        name: 'Axum handlers',
        description: 'REST API endpoints',
        file_references: ['src/api.rs'],
      },
    ],
    similar_features: [
      {
        name: 'User authentication',
        description: 'JWT-based authentication',
        file_path: 'src/auth.rs',
        similarity_score: 0.85,
      },
    ],
    reusable_components: ['ApiResponse struct', 'ErrorResponse'],
    architecture_style: 'Rust Axum REST API',
    analyzed_at: new Date().toISOString(),
  };

  const defaultProps = {
    sessionId: 'test-session',
    projectPath: '/test/project',
    context: null,
    isAnalyzing: false,
    onAnalyze: vi.fn(),
  };

  describe('Initial states', () => {
    it('should show no project message when projectPath is null', () => {
      render(<CodebaseContextPanel {...defaultProps} projectPath={null} />);

      expect(screen.getByText('Codebase Context')).toBeInTheDocument();
      expect(screen.getByText('No project path specified')).toBeInTheDocument();
    });

    it('should show analyze button when no context and not analyzing', () => {
      render(<CodebaseContextPanel {...defaultProps} />);

      expect(screen.getByText('Analyze Codebase')).toBeInTheDocument();
      expect(
        screen.getByText(/Analyze your codebase to find patterns/)
      ).toBeInTheDocument();
    });

    it('should show loading state when analyzing', () => {
      render(<CodebaseContextPanel {...defaultProps} isAnalyzing={true} />);

      expect(screen.getByText(/Analyzing Codebase/)).toBeInTheDocument();
      expect(
        screen.getByText(/Scanning for patterns, features, and components/)
      ).toBeInTheDocument();
    });
  });

  describe('User interactions', () => {
    it('should call onAnalyze when analyze button is clicked', () => {
      const onAnalyze = vi.fn();
      render(<CodebaseContextPanel {...defaultProps} onAnalyze={onAnalyze} />);

      const analyzeButton = screen.getByText('Analyze Codebase');
      fireEvent.click(analyzeButton);

      expect(onAnalyze).toHaveBeenCalledTimes(1);
    });

    it('should not call onAnalyze when button is disabled during analysis', () => {
      const onAnalyze = vi.fn();
      render(
        <CodebaseContextPanel
          {...defaultProps}
          isAnalyzing={true}
          onAnalyze={onAnalyze}
        />
      );

      // Button should not be clickable when analyzing
      const button = screen.queryByRole('button', { name: /Analyze/ });
      expect(button).not.toBeInTheDocument();
    });
  });

  describe('Context display', () => {
    it('should render context sections when context is available', () => {
      render(<CodebaseContextPanel {...defaultProps} context={mockContext} />);

      expect(screen.getByText(/Patterns \(2\)/)).toBeInTheDocument();
      expect(screen.getByText(/Similar Features \(1\)/)).toBeInTheDocument();
      expect(screen.getByText(/Reusable Components \(2\)/)).toBeInTheDocument();
    });

    it('should display pattern information correctly', () => {
      render(<CodebaseContextPanel {...defaultProps} context={mockContext} />);

      expect(screen.getByText('SQLx runtime queries')).toBeInTheDocument();
      expect(screen.getByText('Uses SQLx with runtime queries')).toBeInTheDocument();
      expect(screen.getByText('Axum handlers')).toBeInTheDocument();
      expect(screen.getByText('REST API endpoints')).toBeInTheDocument();
    });

    it('should display similar features correctly', () => {
      render(<CodebaseContextPanel {...defaultProps} context={mockContext} />);

      expect(screen.getByText('User authentication')).toBeInTheDocument();
      expect(screen.getByText('85% similar')).toBeInTheDocument();
      expect(screen.getByText('src/auth.rs')).toBeInTheDocument();
    });

    it('should display reusable components correctly', () => {
      render(<CodebaseContextPanel {...defaultProps} context={mockContext} />);

      expect(screen.getByText(/ApiResponse struct/)).toBeInTheDocument();
      expect(screen.getByText(/ErrorResponse/)).toBeInTheDocument();
    });

    it('should display architecture style', () => {
      render(<CodebaseContextPanel {...defaultProps} context={mockContext} />);

      expect(screen.getByText('Architecture:')).toBeInTheDocument();
      expect(screen.getByText('Rust Axum REST API')).toBeInTheDocument();
    });
  });

  describe('Section expansion', () => {
    it('should toggle pattern section when clicking header', () => {
      render(<CodebaseContextPanel {...defaultProps} context={mockContext} />);

      const patternsHeader = screen.getByText(/Patterns \(2\)/).closest('button');
      expect(patternsHeader).toBeInTheDocument();

      // Initially expanded, content should be visible
      expect(screen.getByText('SQLx runtime queries')).toBeInTheDocument();

      // Click to collapse
      fireEvent.click(patternsHeader!);

      // Content should not be visible after collapse
      expect(screen.queryByText('Uses SQLx with runtime queries')).not.toBeInTheDocument();
    });

    it('should toggle features section when clicking header', () => {
      render(<CodebaseContextPanel {...defaultProps} context={mockContext} />);

      const featuresHeader = screen.getByText(/Similar Features \(1\)/).closest('button');
      expect(featuresHeader).toBeInTheDocument();

      // Initially expanded
      expect(screen.getByText('User authentication')).toBeInTheDocument();

      // Click to collapse
      fireEvent.click(featuresHeader!);

      // Check if collapsed
      expect(screen.queryByText('JWT-based authentication')).not.toBeInTheDocument();
    });

    it('should toggle components section when clicking header', () => {
      render(<CodebaseContextPanel {...defaultProps} context={mockContext} />);

      const componentsHeader = screen
        .getByText(/Reusable Components \(2\)/)
        .closest('button');
      expect(componentsHeader).toBeInTheDocument();

      // Initially expanded
      expect(screen.getByText(/ApiResponse struct/)).toBeInTheDocument();

      // Click to collapse
      fireEvent.click(componentsHeader!);

      // Check if collapsed
      expect(screen.queryByText(/ErrorResponse/)).not.toBeInTheDocument();
    });
  });

  describe('Empty states', () => {
    it('should handle context with no patterns', () => {
      const emptyContext: CodebaseContext = {
        session_id: 'test',
        patterns: [],
        similar_features: [],
        reusable_components: [],
        architecture_style: 'Unknown',
        analyzed_at: new Date().toISOString(),
      };

      render(<CodebaseContextPanel {...defaultProps} context={emptyContext} />);

      // No patterns section should not render
      expect(screen.queryByText(/Patterns/)).not.toBeInTheDocument();
    });

    it('should handle context with no similar features', () => {
      const emptyContext: CodebaseContext = {
        session_id: 'test',
        patterns: [],
        similar_features: [],
        reusable_components: [],
        architecture_style: 'Unknown',
        analyzed_at: new Date().toISOString(),
      };

      render(<CodebaseContextPanel {...defaultProps} context={emptyContext} />);

      // No features section should not render
      expect(screen.queryByText(/Similar Features/)).not.toBeInTheDocument();
    });

    it('should handle context with no reusable components', () => {
      const emptyContext: CodebaseContext = {
        session_id: 'test',
        patterns: [],
        similar_features: [],
        reusable_components: [],
        architecture_style: 'Unknown',
        analyzed_at: new Date().toISOString(),
      };

      render(<CodebaseContextPanel {...defaultProps} context={emptyContext} />);

      // No components section should not render
      expect(screen.queryByText(/Reusable Components/)).not.toBeInTheDocument();
    });
  });

  describe('Custom styling', () => {
    it('should apply custom className', () => {
      const { container } = render(
        <CodebaseContextPanel {...defaultProps} className="custom-test-class" />
      );

      const card = container.querySelector('.custom-test-class');
      expect(card).toBeInTheDocument();
    });
  });
});
