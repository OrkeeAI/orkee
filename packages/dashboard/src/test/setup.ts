// ABOUTME: Test setup file for Vitest
// ABOUTME: Configures jest-dom matchers and mocks Tauri API for testing

import '@testing-library/jest-dom';
import { vi, beforeAll, afterEach, afterAll } from 'vitest';
import { cleanup } from '@testing-library/react';

// Mock @tauri-apps/api/core
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockResolvedValue(4001),
}));

// Mock @tauri-apps/plugin-http
vi.mock('@tauri-apps/plugin-http', () => ({
  fetch: vi.fn(),
}));

// Mock lucide-react icons - create a default mock component for all icons
vi.mock('lucide-react', () => {
  const mockIcon = () => null;
  const iconNames = [
    'Activity', 'AlertCircle', 'AlertTriangle', 'Archive', 'ArrowLeft', 'BarChart', 'Bot', 'Box',
    'Calendar', 'Check', 'CheckCircle', 'CheckCircle2', 'CheckSquare', 'ChevronDown', 'ChevronRight',
    'ChevronsUpDown', 'ChevronUp', 'Circle', 'Clock', 'Cloud', 'Code', 'Code2', 'Copy', 'Database',
    'DollarSign', 'Download', 'Edit', 'Edit2', 'ExternalLink', 'Eye', 'File', 'FileCode', 'FileEdit',
    'FilePlus', 'FileText', 'Folder', 'FolderOpen', 'FolderTree', 'FolderUp', 'Function', 'GitBranch',
    'GitCommit', 'GripVertical', 'Hash', 'History', 'Home', 'Info', 'Key', 'Layers', 'LayoutGrid',
    'Lightbulb', 'List', 'ListTodo', 'Loader2', 'Lock', 'LogOut', 'MapPin', 'Maximize', 'Maximize2',
    'MessageCircle', 'MessageSquare', 'Minus', 'Monitor', 'Moon', 'MoreHorizontal', 'Network', 'Package',
    'PanelLeft', 'Play', 'Plus', 'RefreshCw', 'RotateCcw', 'Save', 'Search', 'Send', 'Server', 'Settings',
    'Shield', 'SkipForward', 'Sliders', 'Smartphone', 'Sparkles', 'Square', 'Star', 'Sun', 'Tablet',
    'Terminal', 'Trash2', 'TrendingUp', 'Upload', 'User', 'Users', 'Variable', 'Wifi', 'WifiOff', 'X',
    'XCircle', 'Zap', 'ZoomIn', 'ZoomOut'
  ];

  const mocks: Record<string, any> = {};
  iconNames.forEach(name => {
    mocks[name] = mockIcon;
  });
  return mocks;
});

// Mock @/lib/platform
vi.mock('@/lib/platform', () => ({
  isTauriApp: vi.fn(() => false),
  isWebApp: vi.fn(() => true),
  getPlatform: vi.fn(() => 'web'),
  getApiPort: vi.fn(async () => 4001),
  getApiToken: vi.fn(async () => null),
  platformFetch: vi.fn(async (url: string, options?: RequestInit) => {
    return fetch(url, options);
  }),
  onDesktop: vi.fn(),
  onWeb: vi.fn((callback: () => void) => callback()),
  platformFeatures: {
    hasFileSystemAccess: vi.fn(() => false),
    hasSystemTray: vi.fn(() => false),
    hasNativeNotifications: vi.fn(() => false),
    requiresProxy: vi.fn(() => true),
  },
}));

// Clean up after each test
afterEach(() => {
  cleanup();
});

// Mock window.__TAURI__ object that Tauri apps use
(global as any).window.__TAURI__ = {
  invoke: vi.fn().mockResolvedValue(4001),
  event: {
    listen: vi.fn(),
    emit: vi.fn(),
  },
};

// Mock window.setInterval and window.clearInterval for telemetry service
(global as any).window.setInterval = vi.fn(() => 1);
(global as any).window.clearInterval = vi.fn();
(global as any).window.addEventListener = vi.fn();

// Mock window.matchMedia for responsive UI tests
Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: vi.fn().mockImplementation((query: string) => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: vi.fn(),
    removeListener: vi.fn(),
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    dispatchEvent: vi.fn(),
  })),
});

// Suppress console errors in tests unless explicitly testing error handling
const originalError = console.error;
beforeAll(() => {
  console.error = (...args: any[]) => {
    if (
      typeof args[0] === 'string' &&
      args[0].includes('Not implemented: HTMLFormElement.prototype.requestSubmit')
    ) {
      return;
    }
    originalError.call(console, ...args);
  };
});

afterAll(() => {
  console.error = originalError;
});
