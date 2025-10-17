// ABOUTME: Test setup file for Vitest
// ABOUTME: Configures jest-dom matchers and mocks Tauri API for testing

import '@testing-library/jest-dom';
import { vi } from 'vitest';

// Mock @tauri-apps/api/core
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockResolvedValue(4001),
}));

// Mock @tauri-apps/plugin-http
vi.mock('@tauri-apps/plugin-http', () => ({
  fetch: vi.fn(),
}));

// Set up global window object for testing
if (typeof window === 'undefined') {
  (global as any).window = {} as any;
}

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
