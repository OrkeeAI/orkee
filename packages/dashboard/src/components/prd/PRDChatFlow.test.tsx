// ABOUTME: Tests for PRDChatFlow component
// ABOUTME: Validates chat-first PRD creation flow including readiness sidebar, generation, and save

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { PRDChatFlow } from './PRDChatFlow';

// Mock react-router-dom
vi.mock('react-router-dom', () => ({
  useNavigate: () => vi.fn(),
}));

// Mock useChat hook
const mockSendMessage = vi.fn();
const mockRefresh = vi.fn();
vi.mock('@/components/ideate/ChatMode/hooks/useChat', () => ({
  useChat: () => ({
    messages: [],
    qualityMetrics: null,
    isLoading: false,
    isSending: false,
    error: null,
    sendMessage: mockSendMessage,
    refresh: mockRefresh,
  }),
}));

// Mock useStreamingResponse hook
const mockStartStreaming = vi.fn();
const mockStopStreaming = vi.fn();
vi.mock('@/components/ideate/ChatMode/hooks/useStreamingResponse', () => ({
  useStreamingResponse: () => ({
    streamingMessage: null,
    isStreaming: false,
    startStreaming: mockStartStreaming,
    stopStreaming: mockStopStreaming,
  }),
}));

// Mock ChatView component
vi.mock('@/components/ideate/ChatMode/components/ChatView', () => ({
  ChatView: ({ onSendMessage }: { onSendMessage: (msg: string) => void }) => (
    <div data-testid="chat-view">
      <button onClick={() => onSendMessage('test message')}>Send Test</button>
    </div>
  ),
}));

// Mock chatService
vi.mock('@/services/chat', () => ({
  chatService: {
    validateForPRD: vi.fn().mockResolvedValue({
      is_valid: false,
      missing_required: ['problem', 'users'],
      warnings: [],
    }),
    sendMessage: vi.fn(),
    getHistory: vi.fn().mockResolvedValue([]),
    getQualityMetrics: vi.fn().mockResolvedValue(null),
    getInsights: vi.fn().mockResolvedValue([]),
  },
}));

// Mock chat-ai
vi.mock('@/services/chat-ai', () => ({
  extractInsights: vi.fn(),
  generatePRDFromChat: vi.fn().mockResolvedValue({
    prd_markdown: '# Test PRD',
    prd_data: {},
  }),
}));

// Mock hooks
vi.mock('@/hooks/useUsers', () => ({
  useCurrentUser: () => ({ data: { id: 'user-1' }, isLoading: false }),
}));

vi.mock('@/services/model-preferences', () => ({
  useModelPreferences: () => ({ data: null }),
  getModelForTask: () => ({ provider: 'anthropic', model: 'claude-sonnet-4-5-20250929' }),
}));

vi.mock('@/hooks/useIdeate', () => ({
  useSaveAsPRD: () => ({
    mutateAsync: vi.fn().mockResolvedValue({ prd_id: 'prd-123', success: true }),
    isPending: false,
  }),
}));

// Mock SavePreview
vi.mock('./SavePreview', () => ({
  SavePreview: ({ open, onConfirmSave }: { open: boolean; onConfirmSave: (name: string) => void }) =>
    open ? (
      <div data-testid="save-preview">
        <button onClick={() => onConfirmSave('Test PRD')}>Confirm Save</button>
      </div>
    ) : null,
}));

// Mock UI components
vi.mock('@/components/ui/button', () => ({
  Button: ({ children, onClick, disabled, variant, size, className }: any) => (
    <button
      onClick={onClick}
      disabled={disabled}
      data-variant={variant}
      data-size={size}
      className={className}
    >
      {children}
    </button>
  ),
}));

vi.mock('@/components/ui/alert', () => ({
  Alert: ({ children, variant }: any) => <div data-variant={variant} role="alert">{children}</div>,
  AlertDescription: ({ children }: any) => <span>{children}</span>,
}));

vi.mock('@/components/ui/card', () => ({
  Card: ({ children, className }: any) => <div className={className}>{children}</div>,
  CardContent: ({ children }: any) => <div>{children}</div>,
  CardHeader: ({ children }: any) => <div>{children}</div>,
  CardTitle: ({ children }: any) => <h3>{children}</h3>,
}));

describe('PRDChatFlow', () => {
  const defaultProps = {
    projectId: 'project-123',
    sessionId: 'session-456',
    onClose: vi.fn(),
    onPRDSaved: vi.fn(),
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('Layout', () => {
    it('should render the back button', () => {
      render(<PRDChatFlow {...defaultProps} />);
      expect(screen.getByText(/Back to PRDs/)).toBeInTheDocument();
    });

    it('should render the Generate PRD button', () => {
      render(<PRDChatFlow {...defaultProps} />);
      expect(screen.getByText('Generate PRD')).toBeInTheDocument();
    });

    it('should render the chat view', () => {
      render(<PRDChatFlow {...defaultProps} />);
      expect(screen.getByTestId('chat-view')).toBeInTheDocument();
    });

    it('should render the readiness sidebar', () => {
      render(<PRDChatFlow {...defaultProps} />);
      expect(screen.getByText('PRD Readiness')).toBeInTheDocument();
    });
  });

  describe('Back navigation', () => {
    it('should call onClose when back button clicked', () => {
      render(<PRDChatFlow {...defaultProps} />);
      fireEvent.click(screen.getByText(/Back to PRDs/));
      expect(defaultProps.onClose).toHaveBeenCalled();
    });
  });

  describe('Generate PRD button', () => {
    it('should be enabled even without full readiness', () => {
      render(<PRDChatFlow {...defaultProps} />);
      const generateButton = screen.getByText('Generate PRD');
      expect(generateButton).not.toBeDisabled();
    });
  });

  describe('Readiness checklist', () => {
    it('should show checklist items', () => {
      render(<PRDChatFlow {...defaultProps} />);
      expect(screen.getByText('Problem defined')).toBeInTheDocument();
      expect(screen.getByText('Target users identified')).toBeInTheDocument();
      expect(screen.getByText('Core features discussed')).toBeInTheDocument();
      expect(screen.getByText('Technical approach')).toBeInTheDocument();
    });
  });
});
