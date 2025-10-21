// ABOUTME: Tests for TaskSpecIndicator component
// ABOUTME: Validates rendering of task-requirement link status with loading, empty, and populated states

import React from 'react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';

// Mock useTaskSpecLinks hook
const mockUseTaskSpecLinks = vi.fn();
vi.mock('@/hooks/useTaskSpecLinks', () => ({
  useTaskSpecLinks: (taskId: string) => mockUseTaskSpecLinks(taskId),
}));

// Mock Badge component
vi.mock('@/components/ui/badge', () => ({
  Badge: ({ children, variant, className }: { children: React.ReactNode; variant?: string; className?: string }) => (
    <span data-variant={variant} className={className}>{children}</span>
  ),
}));

// Mock Button component
vi.mock('@/components/ui/button', () => ({
  Button: ({ children, onClick, variant, size, className, asChild }: any) => (
    <button onClick={onClick} data-variant={variant} data-size={size} className={className}>
      {children}
    </button>
  ),
}));

// Mock Tooltip components
vi.mock('@/components/ui/tooltip', () => ({
  TooltipProvider: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
  Tooltip: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
  TooltipTrigger: ({ children, asChild }: { children: React.ReactNode; asChild?: boolean }) => (
    <div>{children}</div>
  ),
  TooltipContent: ({ children }: { children: React.ReactNode }) => <div role="tooltip">{children}</div>,
}));

// Mock Popover components
vi.mock('@/components/ui/popover', () => ({
  Popover: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
  PopoverTrigger: ({ children, asChild }: { children: React.ReactNode; asChild?: boolean }) => (
    <div>{children}</div>
  ),
  PopoverContent: ({ children, className, align }: any) => (
    <div className={className} data-align={align}>{children}</div>
  ),
}));

// Mock TaskSpecLinker component
vi.mock('./TaskSpecLinker', () => ({
  TaskSpecLinker: ({ projectId, taskId, taskTitle, open, onOpenChange }: any) => (
    <div data-testid="task-spec-linker" data-open={open}>
      TaskSpecLinker: {taskTitle}
    </div>
  ),
}));

// Mock lucide-react icons
vi.mock('lucide-react', () => ({
  CheckCircle2: () => <span>CheckCircle2</span>,
  AlertTriangle: () => <span>AlertTriangle</span>,
  FileText: () => <span>FileText</span>,
  Loader2: ({ className }: { className?: string }) => <span className={className}>Loader2</span>,
}));

import { TaskSpecIndicator } from './TaskSpecIndicator';

describe('TaskSpecIndicator', () => {
  const defaultProps = {
    projectId: 'project-123',
    taskId: 'task-456',
    taskTitle: 'Test Task',
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('Loading state', () => {
    it('should display loading badge when data is loading', () => {
      mockUseTaskSpecLinks.mockReturnValue({
        data: undefined,
        isLoading: true,
      });

      render(<TaskSpecIndicator {...defaultProps} />);

      expect(screen.getByText('Loader2')).toBeInTheDocument();
      expect(screen.getByText('Loading...')).toBeInTheDocument();
    });

    it('should display compact loading badge without text', () => {
      mockUseTaskSpecLinks.mockReturnValue({
        data: undefined,
        isLoading: true,
      });

      render(<TaskSpecIndicator {...defaultProps} compact={true} />);

      expect(screen.getByText('Loader2')).toBeInTheDocument();
      expect(screen.queryByText('Loading...')).not.toBeInTheDocument();
    });
  });

  describe('No links state', () => {
    beforeEach(() => {
      mockUseTaskSpecLinks.mockReturnValue({
        data: [],
        isLoading: false,
      });
    });

    it('should display "No Spec" button when there are no links', () => {
      render(<TaskSpecIndicator {...defaultProps} />);

      expect(screen.getByText('AlertTriangle')).toBeInTheDocument();
      expect(screen.getByText('No Spec')).toBeInTheDocument();
    });

    it('should display tooltip content explaining no spec links', () => {
      render(<TaskSpecIndicator {...defaultProps} />);

      expect(screen.getByText('This task is not linked to any spec requirements')).toBeInTheDocument();
      expect(screen.getByText('Click to link')).toBeInTheDocument();
    });

    it('should display compact version without "No Spec" text', () => {
      render(<TaskSpecIndicator {...defaultProps} compact={true} />);

      expect(screen.getByText('AlertTriangle')).toBeInTheDocument();
      expect(screen.queryByText('No Spec')).not.toBeInTheDocument();
    });

    it('should render TaskSpecLinker component', () => {
      render(<TaskSpecIndicator {...defaultProps} />);

      expect(screen.getByTestId('task-spec-linker')).toBeInTheDocument();
    });
  });

  describe('Has links state', () => {
    const mockLinkedRequirements = [
      {
        id: 'req-1',
        name: 'User Authentication',
        content: 'Users should be able to log in with email and password',
        scenarios: [
          { id: 'scen-1', description: 'Valid credentials' },
          { id: 'scen-2', description: 'Invalid credentials' },
        ],
      },
      {
        id: 'req-2',
        name: 'Password Reset',
        content: 'Users should be able to reset their password',
        scenarios: [],
      },
    ];

    beforeEach(() => {
      mockUseTaskSpecLinks.mockReturnValue({
        data: mockLinkedRequirements,
        isLoading: false,
      });
    });

    it('should display CheckCircle2 icon when there are links', () => {
      render(<TaskSpecIndicator {...defaultProps} />);

      expect(screen.getByText('CheckCircle2')).toBeInTheDocument();
    });

    it('should display correct link count in singular form', () => {
      mockUseTaskSpecLinks.mockReturnValue({
        data: [mockLinkedRequirements[0]],
        isLoading: false,
      });

      render(<TaskSpecIndicator {...defaultProps} />);

      expect(screen.getByText('1 Spec')).toBeInTheDocument();
    });

    it('should display correct link count in plural form', () => {
      render(<TaskSpecIndicator {...defaultProps} />);

      expect(screen.getByText('2 Specs')).toBeInTheDocument();
    });

    it('should display compact version without count text', () => {
      render(<TaskSpecIndicator {...defaultProps} compact={true} />);

      expect(screen.getByText('CheckCircle2')).toBeInTheDocument();
      expect(screen.queryByText('2 Specs')).not.toBeInTheDocument();
    });

    it('should display popover header with link count', () => {
      render(<TaskSpecIndicator {...defaultProps} />);

      expect(screen.getByText('Linked Requirements')).toBeInTheDocument();
      const linkCountBadges = screen.getAllByText('2');
      expect(linkCountBadges.length).toBeGreaterThan(0);
    });

    it('should display all linked requirements', () => {
      render(<TaskSpecIndicator {...defaultProps} />);

      expect(screen.getByText('User Authentication')).toBeInTheDocument();
      expect(screen.getByText('Users should be able to log in with email and password')).toBeInTheDocument();
      expect(screen.getByText('Password Reset')).toBeInTheDocument();
      expect(screen.getByText('Users should be able to reset their password')).toBeInTheDocument();
    });

    it('should display scenario count for requirements with scenarios', () => {
      render(<TaskSpecIndicator {...defaultProps} />);

      expect(screen.getByText('2 scenarios')).toBeInTheDocument();
    });

    it('should display "Manage Links" button', () => {
      render(<TaskSpecIndicator {...defaultProps} />);

      expect(screen.getByText('Manage Links')).toBeInTheDocument();
    });

    it('should render TaskSpecLinker component', () => {
      render(<TaskSpecIndicator {...defaultProps} />);

      expect(screen.getByTestId('task-spec-linker')).toBeInTheDocument();
    });
  });

  describe('Hook integration', () => {
    it('should call useTaskSpecLinks with correct taskId', () => {
      mockUseTaskSpecLinks.mockReturnValue({
        data: [],
        isLoading: false,
      });

      render(<TaskSpecIndicator {...defaultProps} />);

      expect(mockUseTaskSpecLinks).toHaveBeenCalledWith('task-456');
    });
  });
});
