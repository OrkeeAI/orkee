// ABOUTME: Tests for PRDUploadDialog component
// ABOUTME: Validates PRD upload, file handling, markdown preview, AI analysis, and save functionality

import React from 'react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { PRDUploadDialog } from './PRDUploadDialog';

// Mock hooks
const mockCreatePRD = vi.fn();
const mockTriggerAnalysis = vi.fn();
const mockDeletePRD = vi.fn();

vi.mock('@/hooks/usePRDs', () => ({
  useCreatePRD: () => ({
    mutateAsync: mockCreatePRD,
    isPending: false,
    error: null,
    reset: vi.fn(),
  }),
  useTriggerPRDAnalysis: () => ({
    mutateAsync: mockTriggerAnalysis,
  }),
  useDeletePRD: () => ({
    mutateAsync: mockDeletePRD,
  }),
}));

// Mock Dialog components
vi.mock('@/components/ui/dialog', () => ({
  Dialog: ({ children, open }: { children: React.ReactNode; open: boolean }) =>
    open ? <div data-testid="dialog">{children}</div> : null,
  DialogContent: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
  DialogDescription: ({ children }: { children: React.ReactNode }) => <p>{children}</p>,
  DialogHeader: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
  DialogTitle: ({ children }: { children: React.ReactNode }) => <h2>{children}</h2>,
  DialogFooter: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
}));

// Mock Input component
vi.mock('@/components/ui/input', () => ({
  Input: React.forwardRef((props: any, ref: any) => <input {...props} ref={ref} />),
}));

// Mock Label component
vi.mock('@/components/ui/label', () => ({
  Label: ({ children, htmlFor }: { children: React.ReactNode; htmlFor?: string }) => (
    <label htmlFor={htmlFor}>{children}</label>
  ),
}));

// Mock Textarea component
vi.mock('@/components/ui/textarea', () => ({
  Textarea: (props: any) => <textarea {...props} />,
}));

// Mock Button component
vi.mock('@/components/ui/button', () => ({
  Button: ({ children, onClick, disabled, variant, type }: any) => (
    <button onClick={onClick} disabled={disabled} data-variant={variant} type={type}>
      {children}
    </button>
  ),
}));

// Mock Tabs components
vi.mock('@/components/ui/tabs', () => ({
  Tabs: ({ children, value, onValueChange }: any) => (
    <div data-tab-value={value} onClick={() => onValueChange?.('preview')}>
      {children}
    </div>
  ),
  TabsContent: ({ children, value }: { children: React.ReactNode; value: string }) => (
    <div data-tab-content={value}>{children}</div>
  ),
  TabsList: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
  TabsTrigger: ({ children, value, disabled }: any) => (
    <button data-tab={value} disabled={disabled}>
      {children}
    </button>
  ),
}));

// Mock Progress component
vi.mock('@/components/ui/progress', () => ({
  Progress: ({ value }: { value?: number }) => (
    <div role="progressbar" aria-valuenow={value} />
  ),
}));

// Mock ReactMarkdown
vi.mock('react-markdown', () => ({
  default: ({ children }: { children: string }) => <div data-testid="markdown">{children}</div>,
}));

// Mock remark/rehype plugins
vi.mock('remark-gfm', () => ({ default: {} }));
vi.mock('rehype-highlight', () => ({ default: {} }));
vi.mock('rehype-sanitize', () => ({ default: {} }));

// Mock lucide-react icons
vi.mock('lucide-react', () => ({
  FileText: () => <span>FileText</span>,
  Upload: () => <span>Upload</span>,
  Eye: () => <span>Eye</span>,
  Sparkles: () => <span>Sparkles</span>,
  CheckCircle: () => <span>CheckCircle</span>,
  Loader2: ({ className }: { className?: string }) => <span className={className}>Loader2</span>,
}));

describe('PRDUploadDialog', () => {
  const defaultProps = {
    projectId: 'project-123',
    open: true,
    onOpenChange: vi.fn(),
    onComplete: vi.fn(),
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('Initial render', () => {
    it('should render dialog when open', () => {
      render(<PRDUploadDialog {...defaultProps} />);

      expect(screen.getByTestId('dialog')).toBeInTheDocument();
      expect(screen.getByText('Upload Product Requirements Document')).toBeInTheDocument();
    });

    it('should not render dialog when closed', () => {
      render(<PRDUploadDialog {...defaultProps} open={false} />);

      expect(screen.queryByTestId('dialog')).not.toBeInTheDocument();
    });

    it('should render upload tab by default', () => {
      render(<PRDUploadDialog {...defaultProps} />);

      expect(screen.getByLabelText('PRD Title')).toBeInTheDocument();
      expect(screen.getByLabelText('PRD Content (Markdown)')).toBeInTheDocument();
    });

    it('should render save and cancel buttons', () => {
      render(<PRDUploadDialog {...defaultProps} />);

      expect(screen.getByText('Save PRD')).toBeInTheDocument();
      expect(screen.getByText('Cancel')).toBeInTheDocument();
    });
  });

  describe('Title input', () => {
    it('should update title when typing', () => {
      render(<PRDUploadDialog {...defaultProps} />);

      const titleInput = screen.getByLabelText('PRD Title') as HTMLInputElement;
      fireEvent.change(titleInput, { target: { value: 'My PRD' } });

      expect(titleInput.value).toBe('My PRD');
    });

    it('should have placeholder text', () => {
      render(<PRDUploadDialog {...defaultProps} />);

      const titleInput = screen.getByPlaceholderText('e.g., User Authentication System');
      expect(titleInput).toBeInTheDocument();
    });
  });

  describe('Content textarea', () => {
    it('should update content when typing', () => {
      render(<PRDUploadDialog {...defaultProps} />);

      const contentInput = screen.getByLabelText('PRD Content (Markdown)') as HTMLTextAreaElement;
      fireEvent.change(contentInput, { target: { value: '# My PRD Content' } });

      expect(contentInput.value).toBe('# My PRD Content');
    });

    it('should display character count', () => {
      render(<PRDUploadDialog {...defaultProps} />);

      const contentInput = screen.getByLabelText('PRD Content (Markdown)');
      fireEvent.change(contentInput, { target: { value: 'Hello World' } });

      expect(screen.getByText('11 characters')).toBeInTheDocument();
    });

    it('should start with 0 characters', () => {
      render(<PRDUploadDialog {...defaultProps} />);

      expect(screen.getByText('0 characters')).toBeInTheDocument();
    });
  });

  describe('File upload', () => {
    it('should render file input', () => {
      render(<PRDUploadDialog {...defaultProps} />);

      const fileInput = screen.getByLabelText('Upload Markdown File');
      expect(fileInput).toBeInTheDocument();
      expect(fileInput).toHaveAttribute('type', 'file');
      expect(fileInput).toHaveAttribute('accept', '.md,.markdown,.txt');
    });

    it('should render Browse button', () => {
      render(<PRDUploadDialog {...defaultProps} />);

      expect(screen.getByText('Browse')).toBeInTheDocument();
    });

    it('should handle file upload', async () => {
      render(<PRDUploadDialog {...defaultProps} />);

      const fileInput = screen.getByLabelText('Upload Markdown File') as HTMLInputElement;
      const file = new File(['# Test Content'], 'test.md', { type: 'text/markdown' });

      Object.defineProperty(fileInput, 'files', {
        value: [file],
        writable: false,
      });

      fireEvent.change(fileInput);

      await waitFor(() => {
        const contentInput = screen.getByLabelText('PRD Content (Markdown)') as HTMLTextAreaElement;
        expect(contentInput.value).toBe('# Test Content');
      });
    });

    it('should extract title from filename', async () => {
      render(<PRDUploadDialog {...defaultProps} />);

      const fileInput = screen.getByLabelText('Upload Markdown File') as HTMLInputElement;
      const file = new File(['content'], 'authentication-system.md', { type: 'text/markdown' });

      Object.defineProperty(fileInput, 'files', {
        value: [file],
        writable: false,
      });

      fireEvent.change(fileInput);

      await waitFor(() => {
        const titleInput = screen.getByLabelText('PRD Title') as HTMLInputElement;
        expect(titleInput.value).toBe('authentication-system');
      });
    });

    it('should not overwrite existing title', async () => {
      render(<PRDUploadDialog {...defaultProps} />);

      const titleInput = screen.getByLabelText('PRD Title') as HTMLInputElement;
      fireEvent.change(titleInput, { target: { value: 'Existing Title' } });

      const fileInput = screen.getByLabelText('Upload Markdown File') as HTMLInputElement;
      const file = new File(['content'], 'new-file.md', { type: 'text/markdown' });

      Object.defineProperty(fileInput, 'files', {
        value: [file],
        writable: false,
      });

      fireEvent.change(fileInput);

      await waitFor(() => {
        expect(titleInput.value).toBe('Existing Title');
      });
    });
  });

  describe('Tab navigation', () => {
    it('should show preview and analyze buttons when content exists', () => {
      render(<PRDUploadDialog {...defaultProps} />);

      const contentInput = screen.getByLabelText('PRD Content (Markdown)');
      fireEvent.change(contentInput, { target: { value: '# Content' } });

      // Preview tab trigger and button are both shown
      const previewElements = screen.getAllByText('Preview');
      expect(previewElements.length).toBeGreaterThan(0);
      expect(screen.getByText('Analyze with AI')).toBeInTheDocument();
    });

    it('should show preview tab trigger even when content is empty', () => {
      render(<PRDUploadDialog {...defaultProps} />);

      // Preview tab trigger is always shown, just disabled when empty
      const previewElements = screen.getAllByText('Preview');
      expect(previewElements.length).toBeGreaterThan(0);
    });

    it('should disable preview tab when no content', () => {
      render(<PRDUploadDialog {...defaultProps} />);

      const previewTab = screen.getAllByText('Preview')[0].closest('button');
      expect(previewTab).toHaveAttribute('disabled');
    });
  });

  describe('Preview tab', () => {
    it('should render markdown in preview', () => {
      render(<PRDUploadDialog {...defaultProps} />);

      const contentInput = screen.getByLabelText('PRD Content (Markdown)');
      fireEvent.change(contentInput, { target: { value: '# Test Header' } });

      const markdown = screen.getByTestId('markdown');
      expect(markdown).toHaveTextContent('# Test Header');
    });
  });

  describe('AI Analysis', () => {
    it('should trigger analysis on button click', async () => {
      mockCreatePRD.mockResolvedValue({ id: 'prd-123' });
      mockTriggerAnalysis.mockResolvedValue({
        summary: 'Test summary',
        capabilities: [],
        suggestedTasks: [],
      });

      render(<PRDUploadDialog {...defaultProps} />);

      const titleInput = screen.getByLabelText('PRD Title');
      const contentInput = screen.getByLabelText('PRD Content (Markdown)');

      fireEvent.change(titleInput, { target: { value: 'Test PRD' } });
      fireEvent.change(contentInput, { target: { value: '# Content' } });

      const analyzeButton = screen.getByText('Analyze with AI');
      fireEvent.click(analyzeButton);

      await waitFor(() => {
        expect(mockCreatePRD).toHaveBeenCalledWith({
          title: 'Test PRD',
          contentMarkdown: '# Content',
          status: 'draft',
          source: 'manual',
        });
      });
    });

    it('should display analysis results', async () => {
      mockCreatePRD.mockResolvedValue({ id: 'prd-123' });
      mockTriggerAnalysis.mockResolvedValue({
        summary: 'Authentication system PRD',
        capabilities: [
          {
            name: 'User Login',
            purpose: 'Allow users to authenticate',
            requirements: [
              {
                name: 'Email Login',
                content: 'Users can login with email',
                scenarios: ['Valid credentials', 'Invalid credentials'],
              },
            ],
          },
        ],
        suggestedTasks: [
          {
            title: 'Implement login endpoint',
            description: 'Create POST /login endpoint',
            capabilityId: 'cap-1',
            complexity: 5,
          },
        ],
      });

      render(<PRDUploadDialog {...defaultProps} />);

      const titleInput = screen.getByLabelText('PRD Title');
      const contentInput = screen.getByLabelText('PRD Content (Markdown)');

      fireEvent.change(titleInput, { target: { value: 'Auth PRD' } });
      fireEvent.change(contentInput, { target: { value: '# Auth' } });

      const analyzeButton = screen.getByText('Analyze with AI');
      fireEvent.click(analyzeButton);

      await waitFor(() => {
        expect(screen.getByText('Authentication system PRD')).toBeInTheDocument();
        expect(screen.getByText('User Login')).toBeInTheDocument();
        expect(screen.getByText('Capabilities (1)')).toBeInTheDocument();
        expect(screen.getByText('Suggested Tasks (1)')).toBeInTheDocument();
      });
    });

    it('should show analyzing state', async () => {
      mockCreatePRD.mockImplementation(
        () => new Promise((resolve) => setTimeout(() => resolve({ id: 'prd-123' }), 100))
      );

      render(<PRDUploadDialog {...defaultProps} />);

      const titleInput = screen.getByLabelText('PRD Title');
      const contentInput = screen.getByLabelText('PRD Content (Markdown)');

      fireEvent.change(titleInput, { target: { value: 'Test' } });
      fireEvent.change(contentInput, { target: { value: 'Content' } });

      const analyzeButton = screen.getByText('Analyze with AI');
      fireEvent.click(analyzeButton);

      expect(screen.getByText('Analyzing PRD with AI...')).toBeInTheDocument();
      expect(screen.getByRole('progressbar')).toBeInTheDocument();
    });

    it('should clean up temp PRD on analysis failure', async () => {
      mockCreatePRD.mockResolvedValue({ id: 'temp-prd' });
      mockTriggerAnalysis.mockRejectedValue(new Error('Analysis failed'));

      render(<PRDUploadDialog {...defaultProps} />);

      const titleInput = screen.getByLabelText('PRD Title');
      const contentInput = screen.getByLabelText('PRD Content (Markdown)');

      fireEvent.change(titleInput, { target: { value: 'Test' } });
      fireEvent.change(contentInput, { target: { value: 'Content' } });

      const analyzeButton = screen.getByText('Analyze with AI');
      fireEvent.click(analyzeButton);

      await waitFor(() => {
        expect(mockDeletePRD).toHaveBeenCalledWith('temp-prd');
      });
    });
  });

  describe('Save functionality', () => {
    it('should disable save button when title is empty', () => {
      render(<PRDUploadDialog {...defaultProps} />);

      const contentInput = screen.getByLabelText('PRD Content (Markdown)');
      fireEvent.change(contentInput, { target: { value: 'Content' } });

      const saveButton = screen.getByText('Save PRD');
      expect(saveButton).toBeDisabled();
    });

    it('should disable save button when content is empty', () => {
      render(<PRDUploadDialog {...defaultProps} />);

      const titleInput = screen.getByLabelText('PRD Title');
      fireEvent.change(titleInput, { target: { value: 'Title' } });

      const saveButton = screen.getByText('Save PRD');
      expect(saveButton).toBeDisabled();
    });

    it('should enable save button when both title and content exist', () => {
      render(<PRDUploadDialog {...defaultProps} />);

      const titleInput = screen.getByLabelText('PRD Title');
      const contentInput = screen.getByLabelText('PRD Content (Markdown)');

      fireEvent.change(titleInput, { target: { value: 'Title' } });
      fireEvent.change(contentInput, { target: { value: 'Content' } });

      const saveButton = screen.getByText('Save PRD');
      expect(saveButton).not.toBeDisabled();
    });

    it('should save PRD on save button click', async () => {
      mockCreatePRD.mockResolvedValue({ id: 'saved-prd' });

      render(<PRDUploadDialog {...defaultProps} />);

      const titleInput = screen.getByLabelText('PRD Title');
      const contentInput = screen.getByLabelText('PRD Content (Markdown)');

      fireEvent.change(titleInput, { target: { value: 'My PRD' } });
      fireEvent.change(contentInput, { target: { value: '# Content' } });

      const saveButton = screen.getByText('Save PRD');
      fireEvent.click(saveButton);

      await waitFor(() => {
        expect(mockCreatePRD).toHaveBeenCalledWith({
          title: 'My PRD',
          contentMarkdown: '# Content',
          status: 'draft',
          source: 'manual',
        });
      });
    });

    it('should call onComplete with PRD ID after save', async () => {
      mockCreatePRD.mockResolvedValue({ id: 'new-prd' });

      render(<PRDUploadDialog {...defaultProps} />);

      const titleInput = screen.getByLabelText('PRD Title');
      const contentInput = screen.getByLabelText('PRD Content (Markdown)');

      fireEvent.change(titleInput, { target: { value: 'Title' } });
      fireEvent.change(contentInput, { target: { value: 'Content' } });

      const saveButton = screen.getByText('Save PRD');
      fireEvent.click(saveButton);

      await waitFor(() => {
        expect(defaultProps.onComplete).toHaveBeenCalledWith('new-prd');
      });
    });

    it('should close dialog after successful save', async () => {
      mockCreatePRD.mockResolvedValue({ id: 'prd-id' });

      render(<PRDUploadDialog {...defaultProps} />);

      const titleInput = screen.getByLabelText('PRD Title');
      const contentInput = screen.getByLabelText('PRD Content (Markdown)');

      fireEvent.change(titleInput, { target: { value: 'Title' } });
      fireEvent.change(contentInput, { target: { value: 'Content' } });

      const saveButton = screen.getByText('Save PRD');
      fireEvent.click(saveButton);

      await waitFor(() => {
        expect(defaultProps.onOpenChange).toHaveBeenCalledWith(false);
      });
    });
  });

  describe('Cancel functionality', () => {
    it('should call onOpenChange when cancel button clicked', () => {
      render(<PRDUploadDialog {...defaultProps} />);

      const cancelButton = screen.getByText('Cancel');
      fireEvent.click(cancelButton);

      expect(defaultProps.onOpenChange).toHaveBeenCalledWith(false);
    });
  });

  describe('Capability card rendering', () => {
    it('should display capability with requirements count', async () => {
      mockCreatePRD.mockResolvedValue({ id: 'prd-123' });
      mockTriggerAnalysis.mockResolvedValue({
        summary: 'Summary',
        capabilities: [
          {
            name: 'Authentication',
            purpose: 'User login',
            requirements: [
              { name: 'Req 1', content: 'Content 1', scenarios: [] },
              { name: 'Req 2', content: 'Content 2', scenarios: [] },
              { name: 'Req 3', content: 'Content 3', scenarios: [] },
            ],
          },
        ],
        suggestedTasks: [],
      });

      render(<PRDUploadDialog {...defaultProps} />);

      const titleInput = screen.getByLabelText('PRD Title');
      const contentInput = screen.getByLabelText('PRD Content (Markdown)');

      fireEvent.change(titleInput, { target: { value: 'Test' } });
      fireEvent.change(contentInput, { target: { value: 'Content' } });

      const analyzeButton = screen.getByText('Analyze with AI');
      fireEvent.click(analyzeButton);

      await waitFor(() => {
        expect(screen.getByText('3 req')).toBeInTheDocument();
      });
    });

    it('should show scenario count for requirements', async () => {
      mockCreatePRD.mockResolvedValue({ id: 'prd-123' });
      mockTriggerAnalysis.mockResolvedValue({
        summary: 'Summary',
        capabilities: [
          {
            name: 'Feature',
            purpose: 'Purpose',
            requirements: [
              {
                name: 'Requirement',
                content: 'Content',
                scenarios: ['Scenario 1', 'Scenario 2', 'Scenario 3'],
              },
            ],
          },
        ],
        suggestedTasks: [],
      });

      render(<PRDUploadDialog {...defaultProps} />);

      const titleInput = screen.getByLabelText('PRD Title');
      const contentInput = screen.getByLabelText('PRD Content (Markdown)');

      fireEvent.change(titleInput, { target: { value: 'Test' } });
      fireEvent.change(contentInput, { target: { value: 'Content' } });

      const analyzeButton = screen.getByText('Analyze with AI');
      fireEvent.click(analyzeButton);

      await waitFor(() => {
        expect(screen.getByText('3 scenarios')).toBeInTheDocument();
      });
    });

    it('should show singular form for single scenario', async () => {
      mockCreatePRD.mockResolvedValue({ id: 'prd-123' });
      mockTriggerAnalysis.mockResolvedValue({
        summary: 'Summary',
        capabilities: [
          {
            name: 'Feature',
            purpose: 'Purpose',
            requirements: [{ name: 'Req', content: 'Content', scenarios: ['Scenario 1'] }],
          },
        ],
        suggestedTasks: [],
      });

      render(<PRDUploadDialog {...defaultProps} />);

      const titleInput = screen.getByLabelText('PRD Title');
      const contentInput = screen.getByLabelText('PRD Content (Markdown)');

      fireEvent.change(titleInput, { target: { value: 'Test' } });
      fireEvent.change(contentInput, { target: { value: 'Content' } });

      const analyzeButton = screen.getByText('Analyze with AI');
      fireEvent.click(analyzeButton);

      await waitFor(() => {
        expect(screen.getByText('1 scenario')).toBeInTheDocument();
      });
    });
  });
});
