// ABOUTME: Constants for conversational mode
// ABOUTME: Centralized configuration values and magic strings

/**
 * Streaming configuration
 */
export const STREAMING_CONFIG = {
  TIMEOUT_MS: 60000, // 1 minute timeout for AI responses
} as const;

/**
 * UI text constants
 */
export const UI_TEXT = {
  EMPTY_STATE_TITLE: "Let's discover your PRD together!",
  EMPTY_STATE_DESCRIPTION: 'Start by describing your idea, or choose a question below.',
  INPUT_PLACEHOLDER: 'Type your message... (Shift+Enter for new line)',
  GENERATING_PRD: 'Generating PRD...',
  GENERATE_PRD: 'Generate PRD',
  READY_FOR_PRD: '✓ Ready to generate your PRD',
  KEEP_EXPLORING: 'Keep exploring to improve PRD quality',
} as const;

/**
 * Session defaults
 */
export const SESSION_DEFAULTS = {
  NEW_CONVERSATION_TITLE: 'New conversation',
  STARTING_CHAT_TITLE: 'Starting Chat...',
} as const;

/**
 * Error messages
 */
export const ERROR_MESSAGES = {
  STREAMING_TIMEOUT: 'Response timed out. Please try again.',
  STREAMING_ABORTED: 'Response was cancelled.',
  NETWORK_ERROR: 'Network error. Please check your connection.',
  GENERIC_ERROR: 'An error occurred. Please try again.',
} as const;
