// ABOUTME: Tests for SpecDetailsView component
// ABOUTME: Validates spec display with metadata cards, tabs, requirements, and scenarios

import React from 'react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { SpecDetailsView } from './SpecDetailsView';

// Mock hooks
const mockUseSpec = vi.fn();
const mockUseSpecRequirements = vi.fn();

vi.mock('@/hooks/useSpecs', () => ({
  useSpec: (projectId: string, specId: string) => mockUseSpec(projectId, specId),
  useSpecRequirements: (projectId: string, specId: string) => mockUseSpecRequirements(projectId, specId),
}));

// Mock Card components
vi.mock('@/components/ui/card', () => ({
  Card: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
  CardContent: ({ children, className }: { children: React.ReactNode; className?: string }) => (
    <div className={className}>{children}</div>
  ),
  CardDescription: ({ children }: { children: React.ReactNode }) => <p>{children}</p>,
  CardHeader: ({ children, className }: { children: React.ReactNode; className?: string }) => (
    <div className={className}>{children}</div>
  ),
  CardTitle: ({ children, className }: { children: React.ReactNode; className?: string }) => (
    <h3 className={className}>{children}</h3>
  ),
}));

// Mock Badge component
vi.mock('@/components/ui/badge', () => ({
  Badge: ({ children, variant }: { children: React.ReactNode; variant?: string }) => (
    <span data-variant={variant}>{children}</span>
  ),
}));

// Mock Button component
vi.mock('@/components/ui/button', () => ({
  Button: ({ children, onClick, variant }: any) => (
    <button onClick={onClick} data-variant={variant}>
      {children}
    </button>
  ),
}));

// Mock Separator component
vi.mock('@/components/ui/separator', () => ({
  Separator: () => <hr />,
}));

// Mock Tabs components
vi.mock('@/components/ui/tabs', () => ({
  Tabs: ({ children, defaultValue }: any) => <div data-default-tab={defaultValue}>{children}</div>,
  TabsContent: ({ children, value, className }: any) => (
    <div data-tab-content={value} className={className}>{children}</div>
  ),
  TabsList: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
  TabsTrigger: ({ children, value }: any) => <button data-tab={value}>{children}</button>,
}));

// Mock Alert components
vi.mock('@/components/ui/alert', () => ({
  Alert: ({ children, variant }: { children: React.ReactNode; variant?: string }) => (
    <div role="alert" data-variant={variant}>{children}</div>
  ),
  AlertDescription: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
}));

// Mock lucide-react icons
vi.mock('lucide-react', () => ({
  FileText: () => <span>FileText</span>,
  GitBranch: () => <span>GitBranch</span>,
  CheckCircle2: ({ className }: { className?: string }) => <span className={className}>CheckCircle2</span>,
  Calendar: () => <span>Calendar</span>,
  AlertCircle: ({ className }: { className?: string }) => <span className={className}>AlertCircle</span>,
}));

// Mock ReactMarkdown
vi.mock('react-markdown', () => ({
  default: ({ children }: { children: string }) => <div className="markdown">{children}</div>,
}));

// Mock remark and rehype plugins
vi.mock('remark-gfm', () => ({ default: {} }));
vi.mock('rehype-sanitize', () => ({ default: {} }));

describe('SpecDetailsView', () => {
  const defaultProps = {
    projectId: 'project-123',
    specId: 'spec-456',
  };

  const mockSpec = {
    id: 'spec-456',
    name: 'Authentication Spec',
    purpose: 'Define authentication requirements for the application',
    specMarkdown: '# Authentication\n\nUsers must be able to authenticate',
    designMarkdown: '# Design\n\nOAuth 2.0 flow',
    version: 2,
    status: 'active',
    requirementCount: 3,
    prdId: 'prd-789',
    updatedAt: '2024-01-15T12:00:00Z',
  };

  const mockRequirements = [
    {
      id: 'req-1',
      name: 'Login Requirement',
      content: 'Users must log in with email and password',
      scenarios: [
        {
          id: 'scen-1',
          name: 'Valid credentials',
          whenClause: 'user enters valid email and password',
          thenClause: 'system authenticates and redirects to dashboard',
          andClauses: ['session is created', 'user preferences are loaded'],
        },
        {
          id: 'scen-2',
          name: 'Invalid credentials',
          whenClause: 'user enters invalid credentials',
          thenClause: 'system shows error message',
        },
      ],
    },
    {
      id: 'req-2',
      name: 'Logout Requirement',
      content: 'Users can log out',
      scenarios: [],
    },
  ];

  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('Loading state', () => {
    it('should display loading message when spec is loading', () => {
      mockUseSpec.mockReturnValue({ data: undefined, isLoading: true });
      mockUseSpecRequirements.mockReturnValue({ data: [], isLoading: false });

      render(<SpecDetailsView {...defaultProps} />);

      expect(screen.getByText('Loading spec details...')).toBeInTheDocument();
    });

    it('should display loading message when requirements are loading', () => {
      mockUseSpec.mockReturnValue({ data: mockSpec, isLoading: false });
      mockUseSpecRequirements.mockReturnValue({ data: undefined, isLoading: true });

      render(<SpecDetailsView {...defaultProps} />);

      expect(screen.getByText('Loading spec details...')).toBeInTheDocument();
    });
  });

  describe('Error state', () => {
    it('should display error when spec not found', () => {
      mockUseSpec.mockReturnValue({ data: null, isLoading: false });
      mockUseSpecRequirements.mockReturnValue({ data: [], isLoading: false });

      render(<SpecDetailsView {...defaultProps} />);

      expect(screen.getByText('Spec not found or failed to load.')).toBeInTheDocument();
      expect(screen.getByText('AlertCircle')).toBeInTheDocument();
    });
  });

  describe('Header', () => {
    beforeEach(() => {
      mockUseSpec.mockReturnValue({ data: mockSpec, isLoading: false });
      mockUseSpecRequirements.mockReturnValue({ data: mockRequirements, isLoading: false });
    });

    it('should display spec name', () => {
      render(<SpecDetailsView {...defaultProps} />);

      expect(screen.getByText('Authentication Spec')).toBeInTheDocument();
    });

    it('should display version badge', () => {
      render(<SpecDetailsView {...defaultProps} />);

      expect(screen.getByText('v2')).toBeInTheDocument();
    });

    it('should display status badge', () => {
      render(<SpecDetailsView {...defaultProps} />);

      const statusBadges = screen.getAllByText('active');
      expect(statusBadges.length).toBeGreaterThan(0);
    });

    it('should display PRD link indicator when prdId present', () => {
      render(<SpecDetailsView {...defaultProps} />);

      expect(screen.getByText('Linked to PRD')).toBeInTheDocument();
      expect(screen.getByText('GitBranch')).toBeInTheDocument();
    });

    it('should not display PRD link when prdId is null', () => {
      const specWithoutPRD = { ...mockSpec, prdId: null };
      mockUseSpec.mockReturnValue({ data: specWithoutPRD, isLoading: false });

      render(<SpecDetailsView {...defaultProps} />);

      expect(screen.queryByText('Linked to PRD')).not.toBeInTheDocument();
    });

    it('should display Edit button when onEdit provided', () => {
      const onEdit = vi.fn();

      render(<SpecDetailsView {...defaultProps} onEdit={onEdit} />);

      const editButton = screen.getByText('Edit Spec');
      expect(editButton).toBeInTheDocument();
    });

    it('should call onEdit when Edit button clicked', () => {
      const onEdit = vi.fn();

      render(<SpecDetailsView {...defaultProps} onEdit={onEdit} />);

      const editButton = screen.getByText('Edit Spec');
      fireEvent.click(editButton);

      expect(onEdit).toHaveBeenCalledOnce();
    });

    it('should not display Edit button when onEdit not provided', () => {
      render(<SpecDetailsView {...defaultProps} />);

      expect(screen.queryByText('Edit Spec')).not.toBeInTheDocument();
    });
  });

  describe('Metadata cards', () => {
    beforeEach(() => {
      mockUseSpec.mockReturnValue({ data: mockSpec, isLoading: false });
      mockUseSpecRequirements.mockReturnValue({ data: mockRequirements, isLoading: false });
    });

    it('should display requirements count', () => {
      render(<SpecDetailsView {...defaultProps} />);

      expect(screen.getByText('Requirements')).toBeInTheDocument();
      expect(screen.getByText('3')).toBeInTheDocument();
      expect(screen.getByText('Total requirements defined')).toBeInTheDocument();
      expect(screen.getByText('FileText')).toBeInTheDocument();
    });

    it('should display total scenarios count', () => {
      render(<SpecDetailsView {...defaultProps} />);

      expect(screen.getByText('Scenarios')).toBeInTheDocument();
      expect(screen.getByText('2')).toBeInTheDocument(); // 2 scenarios from req-1, 0 from req-2
      expect(screen.getByText('Test scenarios across all requirements')).toBeInTheDocument();
    });

    it('should display last updated date', () => {
      render(<SpecDetailsView {...defaultProps} />);

      expect(screen.getByText('Last Updated')).toBeInTheDocument();
      expect(screen.getByText('Jan 15')).toBeInTheDocument();
      expect(screen.getByText('1/15/2024')).toBeInTheDocument();
      expect(screen.getByText('Calendar')).toBeInTheDocument();
    });
  });

  describe('Tabs', () => {
    beforeEach(() => {
      mockUseSpec.mockReturnValue({ data: mockSpec, isLoading: false });
      mockUseSpecRequirements.mockReturnValue({ data: mockRequirements, isLoading: false });
    });

    it('should display Overview tab', () => {
      render(<SpecDetailsView {...defaultProps} />);

      expect(screen.getByText('Overview')).toBeInTheDocument();
    });

    it('should display Requirements tab with count', () => {
      render(<SpecDetailsView {...defaultProps} />);

      expect(screen.getByText(/Requirements \(2\)/)).toBeInTheDocument();
    });

    it('should display Design tab when designMarkdown present', () => {
      render(<SpecDetailsView {...defaultProps} />);

      expect(screen.getByText('Design')).toBeInTheDocument();
    });

    it('should not display Design tab when designMarkdown is null', () => {
      const specWithoutDesign = { ...mockSpec, designMarkdown: null };
      mockUseSpec.mockReturnValue({ data: specWithoutDesign, isLoading: false });

      render(<SpecDetailsView {...defaultProps} />);

      const designButtons = screen.queryAllByText('Design');
      expect(designButtons.length).toBe(0);
    });

    it('should default to Overview tab', () => {
      render(<SpecDetailsView {...defaultProps} />);

      const tabsContainer = screen.getByText('Overview').closest('[data-default-tab]');
      expect(tabsContainer).toHaveAttribute('data-default-tab', 'overview');
    });
  });

  describe('Overview tab', () => {
    beforeEach(() => {
      mockUseSpec.mockReturnValue({ data: mockSpec, isLoading: false });
      mockUseSpecRequirements.mockReturnValue({ data: mockRequirements, isLoading: false });
    });

    it('should display Purpose section', () => {
      render(<SpecDetailsView {...defaultProps} />);

      expect(screen.getByText('Purpose')).toBeInTheDocument();
      expect(screen.getByText('Define authentication requirements for the application')).toBeInTheDocument();
    });

    it('should display default text when purpose is null', () => {
      const specWithoutPurpose = { ...mockSpec, purpose: null };
      mockUseSpec.mockReturnValue({ data: specWithoutPurpose, isLoading: false });

      render(<SpecDetailsView {...defaultProps} />);

      expect(screen.getByText('No purpose specified')).toBeInTheDocument();
    });

    it('should display Specification section when specMarkdown present', () => {
      render(<SpecDetailsView {...defaultProps} />);

      const specificationTitles = screen.getAllByText('Specification');
      expect(specificationTitles.length).toBeGreaterThan(0);
      expect(screen.getByText(/Users must be able to authenticate/)).toBeInTheDocument();
    });

    it('should not display Specification section when specMarkdown is null', () => {
      const specWithoutMarkdown = { ...mockSpec, specMarkdown: null };
      mockUseSpec.mockReturnValue({ data: specWithoutMarkdown, isLoading: false });

      render(<SpecDetailsView {...defaultProps} />);

      const specificationTitles = screen.queryAllByText('Specification');
      expect(specificationTitles.length).toBe(0);
    });
  });

  describe('Requirements tab', () => {
    beforeEach(() => {
      mockUseSpec.mockReturnValue({ data: mockSpec, isLoading: false });
      mockUseSpecRequirements.mockReturnValue({ data: mockRequirements, isLoading: false });
    });

    it('should display all requirements', () => {
      render(<SpecDetailsView {...defaultProps} />);

      expect(screen.getByText('Login Requirement')).toBeInTheDocument();
      expect(screen.getByText('Logout Requirement')).toBeInTheDocument();
    });

    it('should display requirement content', () => {
      render(<SpecDetailsView {...defaultProps} />);

      expect(screen.getByText('Users must log in with email and password')).toBeInTheDocument();
      expect(screen.getByText('Users can log out')).toBeInTheDocument();
    });

    it('should display scenario count for each requirement', () => {
      render(<SpecDetailsView {...defaultProps} />);

      expect(screen.getByText('2 scenarios')).toBeInTheDocument();
      expect(screen.getByText('0 scenarios')).toBeInTheDocument();
    });

    it('should display singular scenario text', () => {
      const singleScenarioReqs = [
        {
          id: 'req-1',
          name: 'Test Requirement',
          content: 'Test content',
          scenarios: [
            {
              id: 'scen-1',
              name: 'Test Scenario',
              whenClause: 'condition',
              thenClause: 'outcome',
            },
          ],
        },
      ];
      mockUseSpecRequirements.mockReturnValue({ data: singleScenarioReqs, isLoading: false });

      render(<SpecDetailsView {...defaultProps} />);

      expect(screen.getByText('1 scenario')).toBeInTheDocument();
    });

    it('should display requirement index badges', () => {
      render(<SpecDetailsView {...defaultProps} />);

      expect(screen.getByText('#1')).toBeInTheDocument();
      expect(screen.getByText('#2')).toBeInTheDocument();
    });

    it('should display scenario details', () => {
      render(<SpecDetailsView {...defaultProps} />);

      expect(screen.getByText('Valid credentials')).toBeInTheDocument();
      expect(screen.getByText('Invalid credentials')).toBeInTheDocument();
    });

    it('should display WHEN clauses', () => {
      render(<SpecDetailsView {...defaultProps} />);

      expect(screen.getByText('user enters valid email and password')).toBeInTheDocument();
      expect(screen.getByText('user enters invalid credentials')).toBeInTheDocument();
    });

    it('should display THEN clauses', () => {
      render(<SpecDetailsView {...defaultProps} />);

      expect(screen.getByText('system authenticates and redirects to dashboard')).toBeInTheDocument();
      expect(screen.getByText('system shows error message')).toBeInTheDocument();
    });

    it('should display AND clauses when present', () => {
      render(<SpecDetailsView {...defaultProps} />);

      expect(screen.getByText('session is created')).toBeInTheDocument();
      expect(screen.getByText('user preferences are loaded')).toBeInTheDocument();
    });

    it('should not display AND section when andClauses is empty', () => {
      const reqsWithoutAnd = [
        {
          id: 'req-1',
          name: 'Test',
          content: 'Content',
          scenarios: [
            {
              id: 'scen-1',
              name: 'Scenario',
              whenClause: 'when',
              thenClause: 'then',
              andClauses: [],
            },
          ],
        },
      ];
      mockUseSpecRequirements.mockReturnValue({ data: reqsWithoutAnd, isLoading: false });

      render(<SpecDetailsView {...defaultProps} />);

      const andLabels = screen.queryAllByText(/AND:/);
      expect(andLabels.length).toBe(0);
    });

    it('should display "No requirements" message when requirements array is empty', () => {
      mockUseSpecRequirements.mockReturnValue({ data: [], isLoading: false });

      render(<SpecDetailsView {...defaultProps} />);

      expect(screen.getByText('No requirements defined for this spec.')).toBeInTheDocument();
    });

    it('should display Test Scenarios section when scenarios present', () => {
      render(<SpecDetailsView {...defaultProps} />);

      const testScenariosHeaders = screen.getAllByText('Test Scenarios');
      expect(testScenariosHeaders.length).toBeGreaterThan(0);
    });
  });

  describe('Design tab', () => {
    beforeEach(() => {
      mockUseSpec.mockReturnValue({ data: mockSpec, isLoading: false });
      mockUseSpecRequirements.mockReturnValue({ data: mockRequirements, isLoading: false });
    });

    it('should display Design Documentation section', () => {
      render(<SpecDetailsView {...defaultProps} />);

      expect(screen.getByText('Design Documentation')).toBeInTheDocument();
    });

    it('should display design markdown content', () => {
      render(<SpecDetailsView {...defaultProps} />);

      expect(screen.getByText(/OAuth 2.0 flow/)).toBeInTheDocument();
    });
  });

  describe('Hook integration', () => {
    it('should call useSpec with correct parameters', () => {
      mockUseSpec.mockReturnValue({ data: mockSpec, isLoading: false });
      mockUseSpecRequirements.mockReturnValue({ data: mockRequirements, isLoading: false });

      render(<SpecDetailsView {...defaultProps} />);

      expect(mockUseSpec).toHaveBeenCalledWith('project-123', 'spec-456');
    });

    it('should call useSpecRequirements with correct parameters', () => {
      mockUseSpec.mockReturnValue({ data: mockSpec, isLoading: false });
      mockUseSpecRequirements.mockReturnValue({ data: mockRequirements, isLoading: false });

      render(<SpecDetailsView {...defaultProps} />);

      expect(mockUseSpecRequirements).toHaveBeenCalledWith('project-123', 'spec-456');
    });
  });
});
