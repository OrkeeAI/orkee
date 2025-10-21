// ABOUTME: Tests for TaskSpecLinker component
// ABOUTME: Validates task-to-spec linking functionality with search, filtering, and validation

import React from 'react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { TaskSpecLinker } from './TaskSpecLinker';

// Mock hooks with configurable state
const mockUseSpecs = vi.fn();
const mockUseTaskSpecLinks = vi.fn();
const mockLinkMutation = {
  mutateAsync: vi.fn(),
  isPending: false,
};
const mockValidateMutation = {
  mutateAsync: vi.fn(),
  isPending: false,
  data: null as any,
};

vi.mock('@/hooks/useSpecs', () => ({
  useSpecs: (projectId: string) => mockUseSpecs(projectId),
}));

vi.mock('@/hooks/useTaskSpecLinks', () => ({
  useTaskSpecLinks: (taskId: string) => mockUseTaskSpecLinks(taskId),
  useLinkTaskToRequirement: () => mockLinkMutation,
  useValidateTask: () => mockValidateMutation,
}));

// Mock Dialog components
vi.mock('@/components/ui/dialog', () => ({
  Dialog: ({ children, open }: { children: React.ReactNode; open: boolean }) =>
    open ? <div data-testid="dialog">{children}</div> : null,
  DialogContent: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
  DialogDescription: ({ children }: { children: React.ReactNode }) => <p>{children}</p>,
  DialogHeader: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
  DialogTitle: ({ children }: { children: React.ReactNode }) => <h2>{children}</h2>,
}));

// Mock Card components
vi.mock('@/components/ui/card', () => ({
  Card: ({ children, className }: { children: React.ReactNode; className?: string }) => (
    <div className={className}>{children}</div>
  ),
  CardContent: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
  CardDescription: ({ children }: { children: React.ReactNode }) => <p>{children}</p>,
  CardHeader: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
  CardTitle: ({ children }: { children: React.ReactNode }) => <h3>{children}</h3>,
}));

// Mock Input component
vi.mock('@/components/ui/input', () => ({
  Input: (props: any) => <input {...props} />,
}));

// Mock Button component
vi.mock('@/components/ui/button', () => ({
  Button: ({ children, onClick, disabled, variant, size }: any) => (
    <button onClick={onClick} disabled={disabled} data-variant={variant} data-size={size}>
      {children}
    </button>
  ),
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
    <div role="alert" data-variant={variant}>{children}</div>
  ),
  AlertDescription: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
}));

// Mock Separator component
vi.mock('@/components/ui/separator', () => ({
  Separator: () => <hr />,
}));

// Mock lucide-react icons
vi.mock('lucide-react', () => ({
  Search: () => <span>Search</span>,
  Link: () => <span>Link</span>,
  CheckCircle2: () => <span>CheckCircle2</span>,
  XCircle: () => <span>XCircle</span>,
  AlertCircle: () => <span>AlertCircle</span>,
  Loader2: ({ className }: { className?: string }) => <span className={className}>Loader2</span>,
  ChevronDown: () => <span>ChevronDown</span>,
  ChevronUp: () => <span>ChevronUp</span>,
}));

describe('TaskSpecLinker', () => {
  const defaultProps = {
    projectId: 'project-123',
    taskId: 'task-456',
    taskTitle: 'Implement User Authentication',
    open: true,
    onOpenChange: vi.fn(),
  };

  const mockSpec = {
    id: 'spec-1',
    name: 'User Management',
    requirements: [
      {
        id: 'req-1',
        name: 'Login Requirement',
        content: 'Users should be able to log in with email and password',
        scenarios: [
          {
            name: 'Valid login',
            whenClause: 'user enters valid credentials',
            thenClause: 'system authenticates and redirects to dashboard',
            andClauses: ['session is created', 'user preferences are loaded'],
          },
          {
            name: 'Invalid login',
            whenClause: 'user enters invalid credentials',
            thenClause: 'system shows error message',
          },
        ],
      },
      {
        id: 'req-2',
        name: 'Password Reset',
        content: 'Users should be able to reset forgotten passwords',
        scenarios: [],
      },
    ],
  };

  beforeEach(() => {
    vi.clearAllMocks();
    mockUseSpecs.mockReturnValue({ data: [], isLoading: false });
    mockUseTaskSpecLinks.mockReturnValue({ data: [], isLoading: false });
    mockLinkMutation.mutateAsync.mockResolvedValue({});
    mockLinkMutation.isPending = false;
    mockValidateMutation.mutateAsync.mockResolvedValue({});
    mockValidateMutation.isPending = false;
    mockValidateMutation.data = null;
  });

  describe('Initial render', () => {
    it('should render dialog when open', () => {
      render(<TaskSpecLinker {...defaultProps} />);

      expect(screen.getByTestId('dialog')).toBeInTheDocument();
      expect(screen.getByText('Link Task to Spec Requirements')).toBeInTheDocument();
    });

    it('should not render dialog when closed', () => {
      render(<TaskSpecLinker {...defaultProps} open={false} />);

      expect(screen.queryByTestId('dialog')).not.toBeInTheDocument();
    });

    it('should display task title in description', () => {
      render(<TaskSpecLinker {...defaultProps} />);

      expect(screen.getByText(/Implement User Authentication/)).toBeInTheDocument();
    });

    it('should render search input', () => {
      render(<TaskSpecLinker {...defaultProps} />);

      expect(
        screen.getByPlaceholderText('Search requirements by name, content, or capability...')
      ).toBeInTheDocument();
    });
  });

  describe('Loading state', () => {
    it('should display loading indicator when specs are loading', () => {
      mockUseSpecs.mockReturnValue({ data: undefined, isLoading: true });
      mockUseTaskSpecLinks.mockReturnValue({ data: [], isLoading: false });

      render(<TaskSpecLinker {...defaultProps} />);

      expect(screen.getByText('Loader2')).toBeInTheDocument();
    });

    it('should display loading indicator when links are loading', () => {
      mockUseSpecs.mockReturnValue({ data: [], isLoading: false });
      mockUseTaskSpecLinks.mockReturnValue({ data: undefined, isLoading: true });

      render(<TaskSpecLinker {...defaultProps} />);

      expect(screen.getByText('Loader2')).toBeInTheDocument();
    });
  });

  describe('Empty state', () => {
    it('should display message when no requirements available', () => {
      mockUseSpecs.mockReturnValue({ data: [], isLoading: false });
      mockUseTaskSpecLinks.mockReturnValue({ data: [], isLoading: false });

      render(<TaskSpecLinker {...defaultProps} />);

      expect(screen.getByText('No spec requirements available. Create a spec capability first.')).toBeInTheDocument();
    });

    it('should display message when search has no results', () => {
      mockUseSpecs.mockReturnValue({ data: [mockSpec], isLoading: false });
      mockUseTaskSpecLinks.mockReturnValue({ data: [], isLoading: false });

      render(<TaskSpecLinker {...defaultProps} />);

      const searchInput = screen.getByPlaceholderText('Search requirements by name, content, or capability...');
      fireEvent.change(searchInput, { target: { value: 'nonexistent requirement' } });

      expect(screen.getByText('No requirements match your search.')).toBeInTheDocument();
    });
  });

  describe('Linked requirements section', () => {
    const linkedRequirement = {
      id: 'req-1',
      name: 'Login Requirement',
      scenarios: [{ name: 'Test' }],
    };

    beforeEach(() => {
      mockUseSpecs.mockReturnValue({ data: [mockSpec], isLoading: false });
      mockUseTaskSpecLinks.mockReturnValue({ data: [linkedRequirement], isLoading: false });
    });

    it('should display linked requirements section', () => {
      render(<TaskSpecLinker {...defaultProps} />);

      expect(screen.getByText(/Linked Requirements \(1\)/)).toBeInTheDocument();
    });

    it('should display linked requirement details', () => {
      render(<TaskSpecLinker {...defaultProps} />);

      const loginRequirements = screen.getAllByText('Login Requirement');
      expect(loginRequirements.length).toBeGreaterThan(0);
      expect(screen.getByText('1 scenario')).toBeInTheDocument();
    });

    it('should display plural scenarios correctly', () => {
      mockUseTaskSpecLinks.mockReturnValue({
        data: [{
          id: 'req-1',
          name: 'Login Requirement',
          scenarios: [{ name: 'Test1' }, { name: 'Test2' }],
        }],
        isLoading: false,
      });

      render(<TaskSpecLinker {...defaultProps} />);

      const scenarioTexts = screen.getAllByText('2 scenarios');
      expect(scenarioTexts.length).toBeGreaterThan(0);
    });

    it('should not display linked section when no links', () => {
      mockUseTaskSpecLinks.mockReturnValue({ data: [], isLoading: false });

      render(<TaskSpecLinker {...defaultProps} />);

      expect(screen.queryByText(/Linked Requirements/)).not.toBeInTheDocument();
    });
  });

  describe('Available requirements display', () => {
    beforeEach(() => {
      mockUseSpecs.mockReturnValue({ data: [mockSpec], isLoading: false });
      mockUseTaskSpecLinks.mockReturnValue({ data: [], isLoading: false });
    });

    it('should display available requirements count', () => {
      render(<TaskSpecLinker {...defaultProps} />);

      expect(screen.getByText('Available Requirements (2)')).toBeInTheDocument();
    });

    it('should display all requirement names', () => {
      render(<TaskSpecLinker {...defaultProps} />);

      const loginRequirements = screen.getAllByText('Login Requirement');
      expect(loginRequirements.length).toBeGreaterThan(0);
      expect(screen.getByText('Password Reset')).toBeInTheDocument();
    });

    it('should display requirement content', () => {
      render(<TaskSpecLinker {...defaultProps} />);

      expect(screen.getByText('Users should be able to log in with email and password')).toBeInTheDocument();
      expect(screen.getByText('Users should be able to reset forgotten passwords')).toBeInTheDocument();
    });

    it('should display capability name badge', () => {
      render(<TaskSpecLinker {...defaultProps} />);

      const badges = screen.getAllByText('User Management');
      expect(badges.length).toBeGreaterThan(0);
    });

    it('should display scenario count', () => {
      render(<TaskSpecLinker {...defaultProps} />);

      expect(screen.getByText('2 scenarios')).toBeInTheDocument();
      expect(screen.getByText('0 scenarios')).toBeInTheDocument();
    });
  });

  describe('Search functionality', () => {
    beforeEach(() => {
      mockUseSpecs.mockReturnValue({ data: [mockSpec], isLoading: false });
      mockUseTaskSpecLinks.mockReturnValue({ data: [], isLoading: false });
    });

    it('should filter by requirement name', () => {
      render(<TaskSpecLinker {...defaultProps} />);

      const searchInput = screen.getByPlaceholderText('Search requirements by name, content, or capability...');
      fireEvent.change(searchInput, { target: { value: 'Login' } });

      expect(screen.getByText('Login Requirement')).toBeInTheDocument();
      expect(screen.queryByText('Password Reset')).not.toBeInTheDocument();
    });

    it('should filter by requirement content', () => {
      render(<TaskSpecLinker {...defaultProps} />);

      const searchInput = screen.getByPlaceholderText('Search requirements by name, content, or capability...');
      fireEvent.change(searchInput, { target: { value: 'forgotten passwords' } });

      expect(screen.getByText('Password Reset')).toBeInTheDocument();
      expect(screen.queryByText('Login Requirement')).not.toBeInTheDocument();
    });

    it('should filter by capability name', () => {
      render(<TaskSpecLinker {...defaultProps} />);

      const searchInput = screen.getByPlaceholderText('Search requirements by name, content, or capability...');
      fireEvent.change(searchInput, { target: { value: 'User Management' } });

      expect(screen.getByText('Login Requirement')).toBeInTheDocument();
      expect(screen.getByText('Password Reset')).toBeInTheDocument();
    });

    it('should be case insensitive', () => {
      render(<TaskSpecLinker {...defaultProps} />);

      const searchInput = screen.getByPlaceholderText('Search requirements by name, content, or capability...');
      fireEvent.change(searchInput, { target: { value: 'LOGIN' } });

      expect(screen.getByText('Login Requirement')).toBeInTheDocument();
    });

    it('should update count when filtering', () => {
      render(<TaskSpecLinker {...defaultProps} />);

      const searchInput = screen.getByPlaceholderText('Search requirements by name, content, or capability...');
      fireEvent.change(searchInput, { target: { value: 'Login' } });

      expect(screen.getByText('Available Requirements (1)')).toBeInTheDocument();
    });
  });

  describe('Requirement expansion', () => {
    beforeEach(() => {
      mockUseSpecs.mockReturnValue({ data: [mockSpec], isLoading: false });
      mockUseTaskSpecLinks.mockReturnValue({ data: [], isLoading: false });
    });

    it('should show chevron down when collapsed', () => {
      render(<TaskSpecLinker {...defaultProps} />);

      const chevronDowns = screen.getAllByText('ChevronDown');
      expect(chevronDowns.length).toBeGreaterThan(0);
    });

    it('should expand requirement on chevron click', () => {
      render(<TaskSpecLinker {...defaultProps} />);

      // Find the first ChevronDown button
      const chevronButtons = screen.getAllByText('ChevronDown');
      fireEvent.click(chevronButtons[0].closest('button')!);

      // After expansion, should show ChevronUp
      expect(screen.getByText('ChevronUp')).toBeInTheDocument();
    });

    it('should display scenarios when expanded', () => {
      render(<TaskSpecLinker {...defaultProps} />);

      const chevronButtons = screen.getAllByText('ChevronDown');
      fireEvent.click(chevronButtons[0].closest('button')!);

      expect(screen.getByText('Scenarios:')).toBeInTheDocument();
      expect(screen.getByText('Valid login')).toBeInTheDocument();
      expect(screen.getByText('Invalid login')).toBeInTheDocument();
    });

    it('should display scenario clauses', () => {
      render(<TaskSpecLinker {...defaultProps} />);

      const chevronButtons = screen.getAllByText('ChevronDown');
      fireEvent.click(chevronButtons[0].closest('button')!);

      expect(screen.getByText('user enters valid credentials')).toBeInTheDocument();
      expect(screen.getByText('system authenticates and redirects to dashboard')).toBeInTheDocument();
    });

    it('should display AND clauses when present', () => {
      render(<TaskSpecLinker {...defaultProps} />);

      const chevronButtons = screen.getAllByText('ChevronDown');
      fireEvent.click(chevronButtons[0].closest('button')!);

      expect(screen.getByText('session is created')).toBeInTheDocument();
      expect(screen.getByText('user preferences are loaded')).toBeInTheDocument();
    });

    it('should collapse when clicking chevron again', () => {
      render(<TaskSpecLinker {...defaultProps} />);

      const chevronButtons = screen.getAllByText('ChevronDown');
      const button = chevronButtons[0].closest('button')!;

      // Expand
      fireEvent.click(button);
      expect(screen.getByText('ChevronUp')).toBeInTheDocument();

      // Collapse
      fireEvent.click(button);
      expect(screen.queryByText('Scenarios:')).not.toBeInTheDocument();
    });
  });

  describe('Link requirement functionality', () => {
    beforeEach(() => {
      mockUseSpecs.mockReturnValue({ data: [mockSpec], isLoading: false });
      mockUseTaskSpecLinks.mockReturnValue({ data: [], isLoading: false });
    });

    it('should display Link button for unlinked requirements', () => {
      render(<TaskSpecLinker {...defaultProps} />);

      const linkButtons = screen.getAllByText('Link');
      expect(linkButtons.length).toBeGreaterThan(0);
    });

    it('should call link mutation on Link button click', async () => {
      render(<TaskSpecLinker {...defaultProps} />);

      const linkButtons = screen.getAllByText('Link');
      fireEvent.click(linkButtons[0].closest('button')!);

      expect(mockLinkMutation.mutateAsync).toHaveBeenCalledWith({ requirementId: 'req-1' });
    });

    it('should display Linked badge for already linked requirements', () => {
      mockUseTaskSpecLinks.mockReturnValue({
        data: [{ id: 'req-1', name: 'Login Requirement', scenarios: [] }],
        isLoading: false,
      });

      render(<TaskSpecLinker {...defaultProps} />);

      const linkedBadges = screen.getAllByText('Linked');
      expect(linkedBadges.length).toBeGreaterThan(0);
    });

    it('should disable Link button when mutation is pending', () => {
      mockLinkMutation.isPending = true;

      render(<TaskSpecLinker {...defaultProps} />);

      const linkButtons = screen.getAllByText('Link');
      const button = linkButtons[0].closest('button')!;
      expect(button).toBeDisabled();
    });
  });

  describe('Task validation', () => {
    beforeEach(() => {
      mockUseSpecs.mockReturnValue({ data: [mockSpec], isLoading: false });
      mockUseTaskSpecLinks.mockReturnValue({
        data: [{ id: 'req-1', name: 'Login Requirement', scenarios: [{ name: 'Test' }] }],
        isLoading: false,
      });
    });

    it('should display Validate Task button when requirements are linked', () => {
      render(<TaskSpecLinker {...defaultProps} />);

      expect(screen.getByText('Validate Task')).toBeInTheDocument();
    });

    it('should call validate mutation on button click', () => {
      render(<TaskSpecLinker {...defaultProps} />);

      const validateButton = screen.getByText('Validate Task').closest('button')!;
      fireEvent.click(validateButton);

      expect(mockValidateMutation.mutateAsync).toHaveBeenCalled();
    });

    it('should display loading state when validating', () => {
      mockValidateMutation.isPending = true;

      render(<TaskSpecLinker {...defaultProps} />);

      expect(screen.getByText('Validating...')).toBeInTheDocument();
    });

    it('should display success validation result', () => {
      mockValidateMutation.data = {
        valid: true,
        passedScenarios: 5,
        totalScenarios: 5,
        errors: [],
      };

      render(<TaskSpecLinker {...defaultProps} />);

      expect(screen.getByText(/All scenarios passed \(5\/5\)/)).toBeInTheDocument();
    });

    it('should display failed validation result', () => {
      mockValidateMutation.data = {
        valid: false,
        passedScenarios: 3,
        totalScenarios: 5,
        errors: ['Scenario "Login timeout" failed', 'Scenario "Invalid token" failed'],
      };

      render(<TaskSpecLinker {...defaultProps} />);

      expect(screen.getByText(/Validation failed \(3\/5 scenarios passed\)/)).toBeInTheDocument();
      expect(screen.getByText('Scenario "Login timeout" failed')).toBeInTheDocument();
      expect(screen.getByText('Scenario "Invalid token" failed')).toBeInTheDocument();
    });

    it('should not display Validate button when no requirements linked', () => {
      mockUseTaskSpecLinks.mockReturnValue({ data: [], isLoading: false });

      render(<TaskSpecLinker {...defaultProps} />);

      expect(screen.queryByText('Validate Task')).not.toBeInTheDocument();
    });
  });

  describe('Hook integration', () => {
    it('should call useSpecs with correct projectId', () => {
      render(<TaskSpecLinker {...defaultProps} />);

      expect(mockUseSpecs).toHaveBeenCalledWith('project-123');
    });

    it('should call useTaskSpecLinks with correct taskId', () => {
      render(<TaskSpecLinker {...defaultProps} />);

      expect(mockUseTaskSpecLinks).toHaveBeenCalledWith('task-456');
    });
  });

  describe('Edge cases', () => {
    it('should handle requirements without scenarios', () => {
      mockUseSpecs.mockReturnValue({
        data: [{
          id: 'spec-1',
          name: 'Capability',
          requirements: [{
            id: 'req-1',
            name: 'Requirement',
            content: 'Content',
            scenarios: [],
          }],
        }],
        isLoading: false,
      });
      mockUseTaskSpecLinks.mockReturnValue({ data: [], isLoading: false });

      render(<TaskSpecLinker {...defaultProps} />);

      expect(screen.getByText('0 scenarios')).toBeInTheDocument();
    });

    it('should handle multiple specs with requirements', () => {
      mockUseSpecs.mockReturnValue({
        data: [
          mockSpec,
          {
            id: 'spec-2',
            name: 'Data Management',
            requirements: [{
              id: 'req-3',
              name: 'Data Export',
              content: 'Export data to CSV',
              scenarios: [],
            }],
          },
        ],
        isLoading: false,
      });
      mockUseTaskSpecLinks.mockReturnValue({ data: [], isLoading: false });

      render(<TaskSpecLinker {...defaultProps} />);

      expect(screen.getByText('Available Requirements (3)')).toBeInTheDocument();
      expect(screen.getByText('Data Export')).toBeInTheDocument();
    });
  });
});
