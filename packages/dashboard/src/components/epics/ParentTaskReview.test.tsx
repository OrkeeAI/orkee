// ABOUTME: Tests for ParentTaskReview component
// ABOUTME: Validates parent task review, drag-and-drop, editing, and task management

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { ParentTaskReview } from './ParentTaskReview';
import type { ParentTask } from '@/services/epics';

describe('ParentTaskReview', () => {
  const mockParentTasks: ParentTask[] = [
    {
      id: '1',
      title: 'Task 1',
      description: 'Description 1',
      order: 1,
      estimatedSubtasks: 3,
    },
    {
      id: '2',
      title: 'Task 2',
      description: 'Description 2',
      order: 2,
      estimatedSubtasks: 2,
    },
    {
      id: '3',
      title: 'Task 3',
      description: 'Description 3',
      order: 3,
      estimatedSubtasks: 4,
    },
  ];

  const defaultProps = {
    parentTasks: mockParentTasks,
    estimatedTotalTasks: 9,
    complexity: 5,
    onTasksChange: vi.fn(),
    onGenerateDetailedTasks: vi.fn(),
    isGenerating: false,
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('Rendering - Summary Card', () => {
    it('should render summary card title', () => {
      render(<ParentTaskReview {...defaultProps} />);

      expect(screen.getByText('Parent Task Review')).toBeInTheDocument();
      expect(screen.getByText(/Review and edit high-level tasks/)).toBeInTheDocument();
    });

    it('should display parent task count', () => {
      render(<ParentTaskReview {...defaultProps} />);

      expect(screen.getByText('Parent Tasks')).toBeInTheDocument();
      expect(screen.getByText('3')).toBeInTheDocument();
    });

    it('should display estimated total tasks', () => {
      render(<ParentTaskReview {...defaultProps} />);

      expect(screen.getByText('Estimated Total Tasks')).toBeInTheDocument();
      expect(screen.getByText('9')).toBeInTheDocument();
    });

    it('should display complexity score with color coding', () => {
      render(<ParentTaskReview {...defaultProps} />);

      expect(screen.getByText('Complexity Score')).toBeInTheDocument();
      const scoreElement = screen.getByText('5/10');
      expect(scoreElement).toBeInTheDocument();
      expect(scoreElement).toHaveClass('text-yellow-600');
    });
  });

  describe('Complexity Color Coding', () => {
    it('should use green color for low complexity (1-3)', () => {
      render(<ParentTaskReview {...defaultProps} complexity={3} />);

      const scoreElement = screen.getByText('3/10');
      expect(scoreElement).toHaveClass('text-green-600');
    });

    it('should use yellow color for medium complexity (4-6)', () => {
      render(<ParentTaskReview {...defaultProps} complexity={5} />);

      const scoreElement = screen.getByText('5/10');
      expect(scoreElement).toHaveClass('text-yellow-600');
    });

    it('should use orange color for high complexity (7-8)', () => {
      render(<ParentTaskReview {...defaultProps} complexity={7} />);

      const scoreElement = screen.getByText('7/10');
      expect(scoreElement).toHaveClass('text-orange-600');
    });

    it('should use red color for very high complexity (9-10)', () => {
      render(<ParentTaskReview {...defaultProps} complexity={10} />);

      const scoreElement = screen.getByText('10/10');
      expect(scoreElement).toHaveClass('text-red-600');
    });
  });

  describe('Rendering - Task List', () => {
    it('should render all parent tasks', () => {
      render(<ParentTaskReview {...defaultProps} />);

      expect(screen.getByText('Task 1')).toBeInTheDocument();
      expect(screen.getByText('Task 2')).toBeInTheDocument();
      expect(screen.getByText('Task 3')).toBeInTheDocument();
    });

    it('should display task order badges', () => {
      render(<ParentTaskReview {...defaultProps} />);

      expect(screen.getByText('1')).toBeInTheDocument();
      expect(screen.getByText('2')).toBeInTheDocument();
      expect(screen.getByText('3')).toBeInTheDocument();
    });

    it('should display estimated subtasks for each task', () => {
      render(<ParentTaskReview {...defaultProps} />);

      expect(screen.getByText('~3 subtasks')).toBeInTheDocument();
      expect(screen.getByText('~2 subtasks')).toBeInTheDocument();
      expect(screen.getByText('~4 subtasks')).toBeInTheDocument();
    });

    it('should render Add Task button', () => {
      render(<ParentTaskReview {...defaultProps} />);

      expect(screen.getByRole('button', { name: /Add Task/i })).toBeInTheDocument();
    });

    it('should render edit and delete buttons for each task', () => {
      render(<ParentTaskReview {...defaultProps} />);

      const editButtons = screen.getAllByRole('button', { name: '' }).filter(btn => {
        const svg = btn.querySelector('svg');
        return svg?.classList.contains('lucide-edit-2');
      });
      expect(editButtons).toHaveLength(3);

      const deleteButtons = screen.getAllByRole('button', { name: '' }).filter(btn => {
        const svg = btn.querySelector('svg');
        return svg?.classList.contains('lucide-trash-2');
      });
      expect(deleteButtons).toHaveLength(3);
    });
  });

  describe('Task Expansion', () => {
    it('should not show task description initially', () => {
      render(<ParentTaskReview {...defaultProps} />);

      expect(screen.queryByText('Description 1')).not.toBeInTheDocument();
      expect(screen.queryByText('Description 2')).not.toBeInTheDocument();
    });

    it('should expand task when chevron button is clicked', async () => {
      const user = userEvent.setup();
      render(<ParentTaskReview {...defaultProps} />);

      const expandButtons = screen.getAllByRole('button', { name: '' }).filter(btn => {
        const svg = btn.querySelector('svg');
        return svg?.classList.contains('lucide-chevron-right') || svg?.classList.contains('lucide-chevron-down');
      });

      await user.click(expandButtons[0]);

      expect(screen.getByText('Description 1')).toBeInTheDocument();
    });

    it('should toggle expansion state', async () => {
      const user = userEvent.setup();
      render(<ParentTaskReview {...defaultProps} />);

      const expandButtons = screen.getAllByRole('button', { name: '' }).filter(btn => {
        const svg = btn.querySelector('svg');
        return svg?.classList.contains('lucide-chevron-right') || svg?.classList.contains('lucide-chevron-down');
      });

      // Expand
      await user.click(expandButtons[0]);
      expect(screen.getByText('Description 1')).toBeInTheDocument();

      // Collapse
      await user.click(expandButtons[0]);
      expect(screen.queryByText('Description 1')).not.toBeInTheDocument();
    });

    it('should show "No description" when task has no description', async () => {
      const user = userEvent.setup();
      const tasksWithoutDescription = [
        { ...mockParentTasks[0], description: '' },
      ];
      render(<ParentTaskReview {...defaultProps} parentTasks={tasksWithoutDescription} />);

      const expandButtons = screen.getAllByRole('button', { name: '' }).filter(btn => {
        const svg = btn.querySelector('svg');
        return svg?.classList.contains('lucide-chevron-right') || svg?.classList.contains('lucide-chevron-down');
      });

      await user.click(expandButtons[0]);

      expect(screen.getByText('No description')).toBeInTheDocument();
    });
  });

  describe('Task Editing', () => {
    it('should enter edit mode when edit button is clicked', async () => {
      const user = userEvent.setup();
      render(<ParentTaskReview {...defaultProps} />);

      // Click first expand button to show description
      const expandButtons = screen.getAllByRole('button', { name: '' }).filter(btn => {
        const svg = btn.querySelector('svg');
        return svg?.classList.contains('lucide-chevron-right') || svg?.classList.contains('lucide-chevron-down');
      });
      await user.click(expandButtons[0]);

      // Click edit button
      const editButtons = screen.getAllByRole('button', { name: '' }).filter(btn => {
        const svg = btn.querySelector('svg');
        return svg?.classList.contains('lucide-edit-2');
      });
      await user.click(editButtons[0]);

      // Should show input fields
      expect(screen.getByDisplayValue('Task 1')).toBeInTheDocument();
      expect(screen.getByDisplayValue('Description 1')).toBeInTheDocument();
      expect(screen.getByDisplayValue('3')).toBeInTheDocument(); // estimatedSubtasks
    });

    it('should update task title', async () => {
      const user = userEvent.setup();
      const onTasksChange = vi.fn();
      render(<ParentTaskReview {...defaultProps} onTasksChange={onTasksChange} />);

      // Click edit button
      const editButtons = screen.getAllByRole('button', { name: '' }).filter(btn => {
        const svg = btn.querySelector('svg');
        return svg?.classList.contains('lucide-edit-2');
      });
      await user.click(editButtons[0]);

      // Update title
      const titleInput = screen.getByDisplayValue('Task 1');
      await user.clear(titleInput);
      await user.type(titleInput, 'Updated Task 1');

      expect(onTasksChange).toHaveBeenCalled();
      const updatedTasks = onTasksChange.mock.calls[onTasksChange.mock.calls.length - 1][0];
      expect(updatedTasks[0].title).toBe('Updated Task 1');
    });

    it('should update task description', async () => {
      const user = userEvent.setup();
      const onTasksChange = vi.fn();
      render(<ParentTaskReview {...defaultProps} onTasksChange={onTasksChange} />);

      // Expand first
      const expandButtons = screen.getAllByRole('button', { name: '' }).filter(btn => {
        const svg = btn.querySelector('svg');
        return svg?.classList.contains('lucide-chevron-right') || svg?.classList.contains('lucide-chevron-down');
      });
      await user.click(expandButtons[0]);

      // Click edit button
      const editButtons = screen.getAllByRole('button', { name: '' }).filter(btn => {
        const svg = btn.querySelector('svg');
        return svg?.classList.contains('lucide-edit-2');
      });
      await user.click(editButtons[0]);

      // Update description
      const descriptionTextarea = screen.getByDisplayValue('Description 1');
      await user.clear(descriptionTextarea);
      await user.type(descriptionTextarea, 'Updated Description');

      expect(onTasksChange).toHaveBeenCalled();
      const updatedTasks = onTasksChange.mock.calls[onTasksChange.mock.calls.length - 1][0];
      expect(updatedTasks[0].description).toBe('Updated Description');
    });

    it('should update estimated subtasks', async () => {
      const user = userEvent.setup();
      const onTasksChange = vi.fn();
      render(<ParentTaskReview {...defaultProps} onTasksChange={onTasksChange} />);

      // Expand first
      const expandButtons = screen.getAllByRole('button', { name: '' }).filter(btn => {
        const svg = btn.querySelector('svg');
        return svg?.classList.contains('lucide-chevron-right') || svg?.classList.contains('lucide-chevron-down');
      });
      await user.click(expandButtons[0]);

      // Click edit button
      const editButtons = screen.getAllByRole('button', { name: '' }).filter(btn => {
        const svg = btn.querySelector('svg');
        return svg?.classList.contains('lucide-edit-2');
      });
      await user.click(editButtons[0]);

      // Update estimated subtasks
      const subtasksInput = screen.getByDisplayValue('3');
      await user.clear(subtasksInput);
      await user.type(subtasksInput, '5');

      expect(onTasksChange).toHaveBeenCalled();
      const updatedTasks = onTasksChange.mock.calls[onTasksChange.mock.calls.length - 1][0];
      expect(updatedTasks[0].estimatedSubtasks).toBe(5);
    });

    it('should toggle edit mode when edit button is clicked again', async () => {
      const user = userEvent.setup();
      render(<ParentTaskReview {...defaultProps} />);

      const editButtons = screen.getAllByRole('button', { name: '' }).filter(btn => {
        const svg = btn.querySelector('svg');
        return svg?.classList.contains('lucide-edit-2');
      });

      // Enter edit mode
      await user.click(editButtons[0]);
      expect(screen.getByDisplayValue('Task 1')).toBeInTheDocument();

      // Exit edit mode
      await user.click(editButtons[0]);
      expect(screen.queryByDisplayValue('Task 1')).not.toBeInTheDocument();
    });
  });

  describe('Task Deletion', () => {
    it('should delete task when delete button is clicked', async () => {
      const user = userEvent.setup();
      const onTasksChange = vi.fn();
      render(<ParentTaskReview {...defaultProps} onTasksChange={onTasksChange} />);

      const deleteButtons = screen.getAllByRole('button', { name: '' }).filter(btn => {
        const svg = btn.querySelector('svg');
        return svg?.classList.contains('lucide-trash-2');
      });

      await user.click(deleteButtons[0]);

      expect(onTasksChange).toHaveBeenCalled();
      const updatedTasks = onTasksChange.mock.calls[0][0];
      expect(updatedTasks).toHaveLength(2);
      expect(updatedTasks.find(t => t.id === '1')).toBeUndefined();
    });

    it('should reorder remaining tasks after deletion', async () => {
      const user = userEvent.setup();
      const onTasksChange = vi.fn();
      render(<ParentTaskReview {...defaultProps} onTasksChange={onTasksChange} />);

      const deleteButtons = screen.getAllByRole('button', { name: '' }).filter(btn => {
        const svg = btn.querySelector('svg');
        return svg?.classList.contains('lucide-trash-2');
      });

      await user.click(deleteButtons[0]); // Delete first task

      expect(onTasksChange).toHaveBeenCalled();
      const updatedTasks = onTasksChange.mock.calls[0][0];
      expect(updatedTasks[0].order).toBe(1);
      expect(updatedTasks[1].order).toBe(2);
    });
  });

  describe('Adding New Task', () => {
    it('should add new task when Add Task button is clicked', async () => {
      const user = userEvent.setup();
      const onTasksChange = vi.fn();
      render(<ParentTaskReview {...defaultProps} onTasksChange={onTasksChange} />);

      const addButton = screen.getByRole('button', { name: /Add Task/i });
      await user.click(addButton);

      expect(onTasksChange).toHaveBeenCalled();
      const updatedTasks = onTasksChange.mock.calls[0][0];
      expect(updatedTasks).toHaveLength(4);
      expect(updatedTasks[3].title).toBe('New Parent Task');
      expect(updatedTasks[3].estimatedSubtasks).toBe(2);
    });

    it('should auto-enter edit mode for new task', async () => {
      const user = userEvent.setup();
      render(<ParentTaskReview {...defaultProps} />);

      const addButton = screen.getByRole('button', { name: /Add Task/i });
      await user.click(addButton);

      // Should show input field for new task
      expect(screen.getByDisplayValue('New Parent Task')).toBeInTheDocument();
    });

    it('should assign correct order to new task', async () => {
      const user = userEvent.setup();
      const onTasksChange = vi.fn();
      render(<ParentTaskReview {...defaultProps} onTasksChange={onTasksChange} />);

      const addButton = screen.getByRole('button', { name: /Add Task/i });
      await user.click(addButton);

      expect(onTasksChange).toHaveBeenCalled();
      const updatedTasks = onTasksChange.mock.calls[0][0];
      expect(updatedTasks[3].order).toBe(4);
    });
  });

  describe('Generate Button', () => {
    it('should render generate button', () => {
      render(<ParentTaskReview {...defaultProps} />);

      expect(screen.getByRole('button', { name: /Generate Detailed Tasks/i })).toBeInTheDocument();
    });

    it('should call onGenerateDetailedTasks when clicked', async () => {
      const user = userEvent.setup();
      const onGenerateDetailedTasks = vi.fn();
      render(<ParentTaskReview {...defaultProps} onGenerateDetailedTasks={onGenerateDetailedTasks} />);

      const generateButton = screen.getByRole('button', { name: /Generate Detailed Tasks/i });
      await user.click(generateButton);

      expect(onGenerateDetailedTasks).toHaveBeenCalledTimes(1);
    });

    it('should disable button when generating', () => {
      render(<ParentTaskReview {...defaultProps} isGenerating={true} />);

      const generateButton = screen.getByRole('button', { name: /Generating.../i });
      expect(generateButton).toBeDisabled();
    });

    it('should disable button when no tasks', () => {
      render(<ParentTaskReview {...defaultProps} parentTasks={[]} />);

      const generateButton = screen.getByRole('button', { name: /Generate Detailed Tasks/i });
      expect(generateButton).toBeDisabled();
    });

    it('should show "Generating..." text when generating', () => {
      render(<ParentTaskReview {...defaultProps} isGenerating={true} />);

      expect(screen.getByText('Generating...')).toBeInTheDocument();
    });
  });

  describe('Edge Cases', () => {
    it('should handle empty task list', () => {
      render(<ParentTaskReview {...defaultProps} parentTasks={[]} />);

      expect(screen.getByText('Parent Tasks')).toBeInTheDocument();
      expect(screen.getByText('0')).toBeInTheDocument();
    });

    it('should handle task with missing estimatedSubtasks', () => {
      const tasksWithMissingEstimate = [
        { ...mockParentTasks[0], estimatedSubtasks: undefined },
      ];
      render(<ParentTaskReview {...defaultProps} parentTasks={tasksWithMissingEstimate as ParentTask[]} />);

      expect(screen.getByText('~2 subtasks')).toBeInTheDocument(); // Default value
    });

    it('should handle complexity of 0', () => {
      render(<ParentTaskReview {...defaultProps} complexity={0} />);

      const scoreElement = screen.getByText('0/10');
      expect(scoreElement).toBeInTheDocument();
      expect(scoreElement).toHaveClass('text-green-600');
    });

    it('should handle complexity greater than 10', () => {
      render(<ParentTaskReview {...defaultProps} complexity={15} />);

      const scoreElement = screen.getByText('15/10');
      expect(scoreElement).toBeInTheDocument();
      expect(scoreElement).toHaveClass('text-red-600');
    });
  });

  describe('Initial State', () => {
    it('should initialize with provided parent tasks', () => {
      render(<ParentTaskReview {...defaultProps} />);

      expect(screen.getByText('Task 1')).toBeInTheDocument();
      expect(screen.getByText('Task 2')).toBeInTheDocument();
      expect(screen.getByText('Task 3')).toBeInTheDocument();
    });

    it('should not be in edit mode initially', () => {
      render(<ParentTaskReview {...defaultProps} />);

      expect(screen.queryByDisplayValue('Task 1')).not.toBeInTheDocument();
    });

    it('should have all tasks collapsed initially', () => {
      render(<ParentTaskReview {...defaultProps} />);

      expect(screen.queryByText('Description 1')).not.toBeInTheDocument();
      expect(screen.queryByText('Description 2')).not.toBeInTheDocument();
      expect(screen.queryByText('Description 3')).not.toBeInTheDocument();
    });
  });
});
