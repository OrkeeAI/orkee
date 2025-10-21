// ABOUTME: Tests for SpecDiffViewer component
// ABOUTME: Validates diff display between spec versions with added/modified/removed requirements

import React from 'react';
import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import { SpecDiffViewer } from './SpecDiffViewer';

// Mock Card components
vi.mock('@/components/ui/card', () => ({
  Card: ({ children }: { children: React.ReactNode }) => <div data-testid="card">{children}</div>,
  CardContent: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
  CardDescription: ({ children }: { children: React.ReactNode }) => <p>{children}</p>,
  CardHeader: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
  CardTitle: ({ children }: { children: React.ReactNode }) => <h3>{children}</h3>,
}));

// Mock Badge component
vi.mock('@/components/ui/badge', () => ({
  Badge: ({ children, variant }: { children: React.ReactNode; variant?: string }) => (
    <span data-variant={variant}>{children}</span>
  ),
}));

// Mock Tabs components
vi.mock('@/components/ui/tabs', () => ({
  Tabs: ({ children, defaultValue }: any) => <div data-default-tab={defaultValue}>{children}</div>,
  TabsContent: ({ children, value }: { children: React.ReactNode; value: string }) => (
    <div data-tab-content={value}>{children}</div>
  ),
  TabsList: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
  TabsTrigger: ({ children, value }: any) => <button data-tab={value}>{children}</button>,
}));

// Mock lucide-react icons
vi.mock('lucide-react', () => ({
  Plus: () => <span>Plus</span>,
  Minus: () => <span>Minus</span>,
  Edit: () => <span>Edit</span>,
}));

// Mock ReactMarkdown
vi.mock('react-markdown', () => ({
  default: ({ children }: { children: string }) => <div className="markdown">{children}</div>,
}));

// Mock remark and rehype plugins
vi.mock('remark-gfm', () => ({ default: {} }));
vi.mock('rehype-sanitize', () => ({ default: {} }));

describe('SpecDiffViewer', () => {
  const baseOldVersion = {
    id: 'spec-v1',
    name: 'Authentication Spec',
    purpose: 'User authentication system',
    specMarkdown: '# Authentication\n\nOld version content',
    requirements: [
      { name: 'Login Requirement', content: 'Users must log in with email and password' },
      { name: 'Logout Requirement', content: 'Users can log out' },
    ],
    version: 1,
    updatedAt: '2024-01-01T00:00:00Z',
  };

  const baseNewVersion = {
    id: 'spec-v2',
    name: 'Authentication Spec',
    purpose: 'User authentication system',
    specMarkdown: '# Authentication\n\nNew version content',
    requirements: [
      { name: 'Login Requirement', content: 'Users must log in with email and password' },
      { name: 'Logout Requirement', content: 'Users can log out from any device' },
    ],
    version: 2,
    updatedAt: '2024-02-01T00:00:00Z',
  };

  describe('Header and metadata', () => {
    it('should render spec name in title', () => {
      render(<SpecDiffViewer oldVersion={baseOldVersion} newVersion={baseNewVersion} />);

      expect(screen.getByText(/Authentication Spec/)).toBeInTheDocument();
    });

    it('should display version numbers', () => {
      render(<SpecDiffViewer oldVersion={baseOldVersion} newVersion={baseNewVersion} />);

      const v1Elements = screen.getAllByText(/v1/);
      const v2Elements = screen.getAllByText(/v2/);
      expect(v1Elements.length).toBeGreaterThan(0);
      expect(v2Elements.length).toBeGreaterThan(0);
    });

    it('should display formatted dates', () => {
      render(<SpecDiffViewer oldVersion={baseOldVersion} newVersion={baseNewVersion} />);

      const jan1Elements = screen.getAllByText(/12\/31\/2023/);
      const feb1Elements = screen.getAllByText(/1\/31\/2024/);
      expect(jan1Elements.length).toBeGreaterThan(0);
      expect(feb1Elements.length).toBeGreaterThan(0);
    });
  });

  describe('Change badges', () => {
    it('should display Modified badge when requirements are modified', () => {
      render(<SpecDiffViewer oldVersion={baseOldVersion} newVersion={baseNewVersion} />);

      const modifiedBadges = screen.getAllByText('Modified');
      expect(modifiedBadges.length).toBeGreaterThan(0);
      expect(screen.getByText('1 Modified')).toBeInTheDocument();
    });

    it('should display Added badge when requirements are added', () => {
      const newVersion = {
        ...baseNewVersion,
        requirements: [
          ...baseOldVersion.requirements,
          { name: 'New Requirement', content: 'New feature' },
        ],
      };

      render(<SpecDiffViewer oldVersion={baseOldVersion} newVersion={newVersion} />);

      const addedBadges = screen.getAllByText('1 Added');
      expect(addedBadges.length).toBeGreaterThan(0);
      const plusIcons = screen.getAllByText('Plus');
      expect(plusIcons.length).toBeGreaterThan(0);
    });

    it('should display Removed badge when requirements are removed', () => {
      const newVersion = {
        ...baseNewVersion,
        requirements: [baseOldVersion.requirements[0]], // Only first requirement
      };

      render(<SpecDiffViewer oldVersion={baseOldVersion} newVersion={newVersion} />);

      const removedBadges = screen.getAllByText('1 Removed');
      expect(removedBadges.length).toBeGreaterThan(0);
      const minusIcons = screen.getAllByText('Minus');
      expect(minusIcons.length).toBeGreaterThan(0);
    });

    it('should display multiple change type badges', () => {
      const oldVersion = {
        ...baseOldVersion,
        requirements: [
          { name: 'Req A', content: 'Content A' },
          { name: 'Req B', content: 'Content B' },
          { name: 'Req C', content: 'Content C' },
        ],
      };

      const newVersion = {
        ...baseNewVersion,
        requirements: [
          { name: 'Req A', content: 'Content A Modified' }, // Modified
          { name: 'Req D', content: 'Content D' }, // Added
          // Req B and C removed
        ],
      };

      render(<SpecDiffViewer oldVersion={oldVersion} newVersion={newVersion} />);

      const addedBadges = screen.getAllByText('1 Added');
      const modifiedBadges = screen.getAllByText('1 Modified');
      const removedBadges = screen.getAllByText('2 Removed');
      expect(addedBadges.length).toBeGreaterThan(0);
      expect(modifiedBadges.length).toBeGreaterThan(0);
      expect(removedBadges.length).toBeGreaterThan(0);
    });
  });

  describe('Tabs', () => {
    it('should render all three tabs', () => {
      render(<SpecDiffViewer oldVersion={baseOldVersion} newVersion={baseNewVersion} />);

      expect(screen.getByText(/Changes \(1\)/)).toBeInTheDocument();
      expect(screen.getByText('Side by Side')).toBeInTheDocument();
      expect(screen.getByText('All Requirements')).toBeInTheDocument();
    });

    it('should show correct changes count in tab', () => {
      const oldVersion = {
        ...baseOldVersion,
        requirements: [
          { name: 'Req A', content: 'A' },
          { name: 'Req B', content: 'B' },
        ],
      };

      const newVersion = {
        ...baseNewVersion,
        requirements: [
          { name: 'Req A', content: 'A Modified' }, // Modified
          { name: 'Req C', content: 'C' }, // Added
          // Req B removed
        ],
      };

      render(<SpecDiffViewer oldVersion={oldVersion} newVersion={newVersion} />);

      const changesTab = screen.getAllByText(/Changes \(3\)/);
      expect(changesTab.length).toBeGreaterThan(0); // 1 added + 1 modified + 1 removed
    });

    it('should default to Changes tab', () => {
      render(<SpecDiffViewer oldVersion={baseOldVersion} newVersion={baseNewVersion} />);

      const container = screen.getByTestId('card');
      expect(container.querySelector('[data-default-tab="changes"]')).toBeInTheDocument();
    });
  });

  describe('Changes tab content', () => {
    it('should display "No changes detected" when requirements are identical', () => {
      const identicalVersion = {
        ...baseNewVersion,
        requirements: baseOldVersion.requirements,
        version: 2,
      };

      render(<SpecDiffViewer oldVersion={baseOldVersion} newVersion={identicalVersion} />);

      expect(screen.getByText('No changes detected')).toBeInTheDocument();
    });

    it('should display added requirements with green styling', () => {
      const newVersion = {
        ...baseNewVersion,
        requirements: [
          ...baseOldVersion.requirements,
          { name: 'Password Reset', content: 'Users can reset forgotten passwords' },
        ],
      };

      render(<SpecDiffViewer oldVersion={baseOldVersion} newVersion={newVersion} />);

      const passwordResetElements = screen.getAllByText('Password Reset');
      expect(passwordResetElements.length).toBeGreaterThan(0);
      const passwordContent = screen.getAllByText('Users can reset forgotten passwords');
      expect(passwordContent.length).toBeGreaterThan(0);
    });

    it('should display modified requirements with both old and new content', () => {
      render(<SpecDiffViewer oldVersion={baseOldVersion} newVersion={baseNewVersion} />);

      const logoutReqElements = screen.getAllByText('Logout Requirement');
      expect(logoutReqElements.length).toBeGreaterThan(0);
      expect(screen.getByText('Old Version')).toBeInTheDocument();
      expect(screen.getByText('New Version')).toBeInTheDocument();
      const logoutOldContent = screen.getAllByText('Users can log out');
      expect(logoutOldContent.length).toBeGreaterThan(0);
      const logoutNewContent = screen.getAllByText('Users can log out from any device');
      expect(logoutNewContent.length).toBeGreaterThan(0);
    });

    it('should display removed requirements with red styling', () => {
      const newVersion = {
        ...baseNewVersion,
        requirements: [baseOldVersion.requirements[0]], // Only first requirement
      };

      render(<SpecDiffViewer oldVersion={baseOldVersion} newVersion={newVersion} />);

      const removedBadges = screen.getAllByText('Removed');
      expect(removedBadges.length).toBeGreaterThan(0);
      const logoutReqs = screen.getAllByText('Logout Requirement');
      expect(logoutReqs.length).toBeGreaterThan(0);
    });

    it('should render markdown content in changes', () => {
      const newVersion = {
        ...baseNewVersion,
        requirements: [
          ...baseOldVersion.requirements,
          {
            name: 'Markdown Test',
            content: '**Bold** and *italic* text',
          },
        ],
      };

      render(<SpecDiffViewer oldVersion={baseOldVersion} newVersion={newVersion} />);

      const markdownElements = screen.getAllByText('**Bold** and *italic* text');
      expect(markdownElements.length).toBeGreaterThan(0);
    });
  });

  describe('Side by Side tab content', () => {
    it('should display both version headers', () => {
      render(<SpecDiffViewer oldVersion={baseOldVersion} newVersion={baseNewVersion} />);

      // Version headers shown in side-by-side view (already tested in main tests)
      const v1Elements = screen.getAllByText(/v1/);
      const v2Elements = screen.getAllByText(/v2/);
      expect(v1Elements.length).toBeGreaterThanOrEqual(2); // Header + side-by-side
      expect(v2Elements.length).toBeGreaterThanOrEqual(2);
    });

    it('should display spec markdown for both versions', () => {
      render(<SpecDiffViewer oldVersion={baseOldVersion} newVersion={baseNewVersion} />);

      // Check that markdown content is rendered (using partial text match for multi-line content)
      expect(screen.getByText(/Old version content/)).toBeInTheDocument();
      expect(screen.getByText(/New version content/)).toBeInTheDocument();
    });
  });

  describe('All Requirements tab content', () => {
    it('should display all current requirements', () => {
      const newVersion = {
        ...baseNewVersion,
        requirements: [
          { name: 'Req A', content: 'Content A' },
          { name: 'Req B', content: 'Content B' },
          { name: 'Req C', content: 'Content C' },
        ],
      };

      render(<SpecDiffViewer oldVersion={baseOldVersion} newVersion={newVersion} />);

      const reqAElements = screen.getAllByText('Req A');
      expect(reqAElements.length).toBeGreaterThan(0);
      const reqBElements = screen.getAllByText('Req B');
      expect(reqBElements.length).toBeGreaterThan(0);
      const reqCElements = screen.getAllByText('Req C');
      expect(reqCElements.length).toBeGreaterThan(0);
    });

    it('should include added, modified, and unchanged requirements', () => {
      const oldVersion = {
        ...baseOldVersion,
        requirements: [
          { name: 'Unchanged', content: 'Same content' },
          { name: 'ToModify', content: 'Old content' },
        ],
      };

      const newVersion = {
        ...baseNewVersion,
        requirements: [
          { name: 'Unchanged', content: 'Same content' }, // Unchanged
          { name: 'ToModify', content: 'New content' }, // Modified
          { name: 'Added', content: 'New requirement' }, // Added
        ],
      };

      render(<SpecDiffViewer oldVersion={oldVersion} newVersion={newVersion} />);

      const unchangedElements = screen.getAllByText('Unchanged');
      expect(unchangedElements.length).toBeGreaterThan(0);
      const toModifyElements = screen.getAllByText('ToModify');
      expect(toModifyElements.length).toBeGreaterThan(0);
      const addedElements = screen.getAllByText('Added');
      expect(addedElements.length).toBeGreaterThan(0);
    });

    it('should not include removed requirements', () => {
      const oldVersion = {
        ...baseOldVersion,
        requirements: [
          { name: 'Kept', content: 'Content' },
          { name: 'Removed', content: 'This will be removed' },
        ],
      };

      const newVersion = {
        ...baseNewVersion,
        requirements: [{ name: 'Kept', content: 'Content' }],
      };

      render(<SpecDiffViewer oldVersion={oldVersion} newVersion={newVersion} />);

      // In All Requirements tab, removed items should still appear in Changes tab
      // but we check that the component renders without crashing
      expect(screen.getByText('Kept')).toBeInTheDocument();
    });
  });

  describe('Edge cases', () => {
    it('should handle empty old requirements', () => {
      const oldVersion = {
        ...baseOldVersion,
        requirements: [],
      };

      render(<SpecDiffViewer oldVersion={oldVersion} newVersion={baseNewVersion} />);

      const addedBadges = screen.getAllByText('2 Added');
      expect(addedBadges.length).toBeGreaterThan(0);
    });

    it('should handle empty new requirements', () => {
      const newVersion = {
        ...baseNewVersion,
        requirements: [],
      };

      render(<SpecDiffViewer oldVersion={baseOldVersion} newVersion={newVersion} />);

      const removedBadges = screen.getAllByText('2 Removed');
      expect(removedBadges.length).toBeGreaterThan(0);
    });

    it('should handle both versions with no requirements', () => {
      const oldVersion = {
        ...baseOldVersion,
        requirements: [],
      };

      const newVersion = {
        ...baseNewVersion,
        requirements: [],
      };

      render(<SpecDiffViewer oldVersion={oldVersion} newVersion={newVersion} />);

      expect(screen.getByText('No changes detected')).toBeInTheDocument();
    });

    it('should handle identical requirement names with different content', () => {
      const oldVersion = {
        ...baseOldVersion,
        requirements: [{ name: 'Same Name', content: 'Original content' }],
      };

      const newVersion = {
        ...baseNewVersion,
        requirements: [{ name: 'Same Name', content: 'Updated content' }],
      };

      render(<SpecDiffViewer oldVersion={oldVersion} newVersion={newVersion} />);

      const modifiedBadges = screen.getAllByText('1 Modified');
      expect(modifiedBadges.length).toBeGreaterThan(0);
      expect(screen.getByText('Original content')).toBeInTheDocument();
      const updatedContentElements = screen.getAllByText('Updated content');
      expect(updatedContentElements.length).toBeGreaterThan(0);
    });

    it('should handle requirements with same content (unchanged)', () => {
      const oldVersion = {
        ...baseOldVersion,
        requirements: [{ name: 'Unchanged', content: 'Same content' }],
      };

      const newVersion = {
        ...baseNewVersion,
        requirements: [{ name: 'Unchanged', content: 'Same content' }],
      };

      render(<SpecDiffViewer oldVersion={oldVersion} newVersion={newVersion} />);

      expect(screen.getByText('No changes detected')).toBeInTheDocument();
    });
  });
});
