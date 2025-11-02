// ABOUTME: Tests for ProgressHistory component
// ABOUTME: Validates progress timeline display, entry types, and timestamp formatting

import { describe, it, expect, beforeEach, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import { ProgressHistory } from './ProgressHistory';
import type { ValidationEntry } from '@/services/tasks';

describe('ProgressHistory', () => {
  const now = new Date('2025-01-15T12:00:00Z');

  const mockEntries: ValidationEntry[] = [
    {
      entryType: 'progress',
      content: 'Completed user authentication implementation',
      timestamp: new Date('2025-01-15T11:50:00Z').toISOString(),
      author: 'Alice',
    },
    {
      entryType: 'issue',
      content: 'Found bug in password validation logic',
      timestamp: new Date('2025-01-15T11:45:00Z').toISOString(),
      author: 'Bob',
    },
    {
      entryType: 'decision',
      content: 'Decided to use bcrypt for password hashing',
      timestamp: new Date('2025-01-15T11:30:00Z').toISOString(),
      author: 'Charlie',
    },
    {
      entryType: 'checkpoint',
      content: 'Reached testing checkpoint - all tests passing',
      timestamp: new Date('2025-01-15T11:15:00Z').toISOString(),
    },
  ];

  beforeEach(() => {
    vi.useFakeTimers();
    vi.setSystemTime(now);
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  describe('Empty State', () => {
    it('should display empty state message when no entries', () => {
      render(<ProgressHistory entries={[]} />);

      expect(screen.getByText('No progress history yet')).toBeInTheDocument();
      expect(screen.getByText('Updates will appear here as work progresses')).toBeInTheDocument();
    });

    it('should display clock icon in empty state', () => {
      const { container } = render(<ProgressHistory entries={[]} />);

      const icon = container.querySelector('svg.lucide-clock');
      expect(icon).toBeInTheDocument();
    });

    it('should not render entry list when empty', () => {
      render(<ProgressHistory entries={[]} />);

      expect(screen.queryByText('Progress History')).not.toBeInTheDocument();
    });
  });

  describe('Rendering - Basic Elements', () => {
    it('should render card title', () => {
      render(<ProgressHistory entries={mockEntries} />);

      expect(screen.getByText('Progress History')).toBeInTheDocument();
    });

    it('should display entry count', () => {
      render(<ProgressHistory entries={mockEntries} />);

      expect(screen.getByText('4 entries')).toBeInTheDocument();
    });

    it('should display singular form for single entry', () => {
      render(<ProgressHistory entries={[mockEntries[0]]} />);

      expect(screen.getByText('1 entry')).toBeInTheDocument();
    });

    it('should render all entry contents', () => {
      render(<ProgressHistory entries={mockEntries} />);

      expect(screen.getByText('Completed user authentication implementation')).toBeInTheDocument();
      expect(screen.getByText('Found bug in password validation logic')).toBeInTheDocument();
      expect(screen.getByText('Decided to use bcrypt for password hashing')).toBeInTheDocument();
      expect(screen.getByText('Reached testing checkpoint - all tests passing')).toBeInTheDocument();
    });
  });

  describe('Entry Types and Icons', () => {
    it('should render progress entry with green check icon', () => {
      const { container } = render(<ProgressHistory entries={[mockEntries[0]]} />);

      const icon = container.querySelector('svg.lucide-check-circle-2.text-green-600');
      expect(icon).toBeInTheDocument();
    });

    it('should render issue entry with red alert icon', () => {
      const { container } = render(<ProgressHistory entries={[mockEntries[1]]} />);

      const icon = container.querySelector('svg.lucide-alert-circle.text-red-600');
      expect(icon).toBeInTheDocument();
    });

    it('should render decision entry with yellow lightbulb icon', () => {
      const { container } = render(<ProgressHistory entries={[mockEntries[2]]} />);

      const icon = container.querySelector('svg.lucide-lightbulb.text-yellow-600');
      expect(icon).toBeInTheDocument();
    });

    it('should render checkpoint entry with blue flag icon', () => {
      const { container } = render(<ProgressHistory entries={[mockEntries[3]]} />);

      const icon = container.querySelector('svg.lucide-flag.text-blue-600');
      expect(icon).toBeInTheDocument();
    });

    it('should render unknown entry type with gray clock icon', () => {
      const unknownEntry: ValidationEntry = {
        entryType: 'unknown' as any,
        content: 'Unknown entry',
        timestamp: now.toISOString(),
      };
      const { container } = render(<ProgressHistory entries={[unknownEntry]} />);

      const icon = container.querySelector('svg.lucide-clock.text-gray-600');
      expect(icon).toBeInTheDocument();
    });
  });

  describe('Entry Type Badges', () => {
    it('should render progress badge with green styling', () => {
      render(<ProgressHistory entries={[mockEntries[0]]} />);

      const badge = screen.getByText('progress');
      expect(badge).toHaveClass('bg-green-100');
      expect(badge).toHaveClass('text-green-800');
      expect(badge).toHaveClass('border-green-300');
    });

    it('should render issue badge with red styling', () => {
      render(<ProgressHistory entries={[mockEntries[1]]} />);

      const badge = screen.getByText('issue');
      expect(badge).toHaveClass('bg-red-100');
      expect(badge).toHaveClass('text-red-800');
      expect(badge).toHaveClass('border-red-300');
    });

    it('should render decision badge with yellow styling', () => {
      render(<ProgressHistory entries={[mockEntries[2]]} />);

      const badge = screen.getByText('decision');
      expect(badge).toHaveClass('bg-yellow-100');
      expect(badge).toHaveClass('text-yellow-800');
      expect(badge).toHaveClass('border-yellow-300');
    });

    it('should render checkpoint badge with blue styling', () => {
      render(<ProgressHistory entries={[mockEntries[3]]} />);

      const badge = screen.getByText('checkpoint');
      expect(badge).toHaveClass('bg-blue-100');
      expect(badge).toHaveClass('text-blue-800');
      expect(badge).toHaveClass('border-blue-300');
    });
  });

  describe('Author Display', () => {
    it('should display author name when present', () => {
      render(<ProgressHistory entries={[mockEntries[0]]} />);

      expect(screen.getByText('by Alice')).toBeInTheDocument();
    });

    it('should not display author line when author is missing', () => {
      render(<ProgressHistory entries={[mockEntries[3]]} />);

      expect(screen.queryByText(/^by /)).not.toBeInTheDocument();
    });

    it('should display all authors correctly', () => {
      render(<ProgressHistory entries={mockEntries} />);

      expect(screen.getByText('by Alice')).toBeInTheDocument();
      expect(screen.getByText('by Bob')).toBeInTheDocument();
      expect(screen.getByText('by Charlie')).toBeInTheDocument();
    });
  });

  describe('Timestamp Formatting', () => {
    it('should display "Just now" for timestamps less than 1 minute ago', () => {
      const recentEntry: ValidationEntry = {
        ...mockEntries[0],
        timestamp: new Date('2025-01-15T11:59:30Z').toISOString(),
      };
      render(<ProgressHistory entries={[recentEntry]} />);

      expect(screen.getByText('Just now')).toBeInTheDocument();
    });

    it('should display minutes ago for timestamps less than 1 hour ago', () => {
      const entry10MinutesAgo: ValidationEntry = {
        ...mockEntries[0],
        timestamp: new Date('2025-01-15T11:50:00Z').toISOString(),
      };
      render(<ProgressHistory entries={[entry10MinutesAgo]} />);

      expect(screen.getByText('10m ago')).toBeInTheDocument();
    });

    it('should display hours ago for timestamps less than 24 hours ago', () => {
      const entry2HoursAgo: ValidationEntry = {
        ...mockEntries[0],
        timestamp: new Date('2025-01-15T10:00:00Z').toISOString(),
      };
      render(<ProgressHistory entries={[entry2HoursAgo]} />);

      expect(screen.getByText('2h ago')).toBeInTheDocument();
    });

    it('should display days ago for timestamps less than 7 days ago', () => {
      const entry3DaysAgo: ValidationEntry = {
        ...mockEntries[0],
        timestamp: new Date('2025-01-12T12:00:00Z').toISOString(),
      };
      render(<ProgressHistory entries={[entry3DaysAgo]} />);

      expect(screen.getByText('3d ago')).toBeInTheDocument();
    });

    it('should display formatted date for timestamps older than 7 days', () => {
      const entry10DaysAgo: ValidationEntry = {
        ...mockEntries[0],
        timestamp: new Date('2025-01-05T12:00:00Z').toISOString(),
      };
      render(<ProgressHistory entries={[entry10DaysAgo]} />);

      expect(screen.getByText('Jan 5')).toBeInTheDocument();
    });

    it('should include year for timestamps from different year', () => {
      const entryLastYear: ValidationEntry = {
        ...mockEntries[0],
        timestamp: new Date('2024-12-25T12:00:00Z').toISOString(),
      };
      render(<ProgressHistory entries={[entryLastYear]} />);

      expect(screen.getByText(/Dec 25, 2024/)).toBeInTheDocument();
    });
  });

  describe('Entry Sorting', () => {
    it('should sort entries by timestamp with newest first', () => {
      render(<ProgressHistory entries={mockEntries} />);

      const entryContents = screen.getAllByText(/^(Completed|Found|Decided|Reached)/);

      // Newest should be first
      expect(entryContents[0]).toHaveTextContent('Completed user authentication implementation');
      expect(entryContents[1]).toHaveTextContent('Found bug in password validation logic');
      expect(entryContents[2]).toHaveTextContent('Decided to use bcrypt for password hashing');
      expect(entryContents[3]).toHaveTextContent('Reached testing checkpoint - all tests passing');
    });

    it('should maintain sort order even with unsorted input', () => {
      const unsortedEntries = [mockEntries[2], mockEntries[0], mockEntries[3], mockEntries[1]];
      render(<ProgressHistory entries={unsortedEntries} />);

      const entryContents = screen.getAllByText(/^(Completed|Found|Decided|Reached)/);

      // Should still be sorted newest first
      expect(entryContents[0]).toHaveTextContent('Completed user authentication implementation');
      expect(entryContents[1]).toHaveTextContent('Found bug in password validation logic');
    });
  });

  describe('Custom Max Height', () => {
    it('should use default max height of 500px', () => {
      const { container } = render(<ProgressHistory entries={mockEntries} />);

      const scrollArea = container.querySelector('[style*="max-height"]');
      expect(scrollArea).toHaveStyle({ maxHeight: '500px' });
    });

    it('should use custom max height when provided', () => {
      const { container } = render(
        <ProgressHistory entries={mockEntries} maxHeight="300px" />
      );

      const scrollArea = container.querySelector('[style*="max-height"]');
      expect(scrollArea).toHaveStyle({ maxHeight: '300px' });
    });
  });

  describe('Content Whitespace', () => {
    it('should preserve whitespace in entry content', () => {
      const multilineEntry: ValidationEntry = {
        entryType: 'progress',
        content: 'Line 1\n\nLine 2\nLine 3',
        timestamp: now.toISOString(),
      };
      render(<ProgressHistory entries={[multilineEntry]} />);

      const contentElement = screen.getByText(/Line 1/);
      expect(contentElement).toHaveClass('whitespace-pre-wrap');
    });
  });

  describe('Edge Cases', () => {
    it('should handle entry with empty content', () => {
      const emptyContentEntry: ValidationEntry = {
        entryType: 'progress',
        content: '',
        timestamp: now.toISOString(),
      };
      render(<ProgressHistory entries={[emptyContentEntry]} />);

      expect(screen.getByText('Progress History')).toBeInTheDocument();
    });

    it('should handle entry with very long content', () => {
      const longContentEntry: ValidationEntry = {
        entryType: 'progress',
        content: 'A'.repeat(1000),
        timestamp: now.toISOString(),
      };
      render(<ProgressHistory entries={[longContentEntry]} />);

      expect(screen.getByText('A'.repeat(1000))).toBeInTheDocument();
    });

    it('should handle single entry', () => {
      render(<ProgressHistory entries={[mockEntries[0]]} />);

      expect(screen.getByText('1 entry')).toBeInTheDocument();
      expect(screen.getByText('Completed user authentication implementation')).toBeInTheDocument();
    });

    it('should handle many entries', () => {
      const manyEntries = Array.from({ length: 50 }, (_, i) => ({
        ...mockEntries[0],
        content: `Entry ${i + 1}`,
        timestamp: new Date(now.getTime() - i * 60000).toISOString(),
      }));
      render(<ProgressHistory entries={manyEntries} />);

      expect(screen.getByText('50 entries')).toBeInTheDocument();
    });
  });

  describe('Timeline Visual Elements', () => {
    it('should render timeline with proper structure', () => {
      const { container } = render(<ProgressHistory entries={mockEntries} />);

      const timelineItems = container.querySelectorAll('.border-l-2');
      expect(timelineItems.length).toBe(4);
    });

    it('should position icons on the timeline', () => {
      const { container } = render(<ProgressHistory entries={mockEntries} />);

      const iconContainers = container.querySelectorAll('.absolute.-left-\\[13px\\]');
      expect(iconContainers.length).toBe(4);
    });
  });

  describe('Scroll Area', () => {
    it('should render scroll area for entries', () => {
      const { container } = render(<ProgressHistory entries={mockEntries} />);

      const scrollArea = container.querySelector('[style*="max-height"]');
      expect(scrollArea).toBeInTheDocument();
    });

    it('should apply custom max height to scroll area', () => {
      const { container } = render(
        <ProgressHistory entries={mockEntries} maxHeight="200px" />
      );

      const scrollArea = container.querySelector('[style*="max-height"]');
      expect(scrollArea).toHaveStyle({ maxHeight: '200px' });
    });
  });
});
