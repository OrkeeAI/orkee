// ABOUTME: Tests for platform detection and Tauri API interactions
// ABOUTME: Validates platform utilities work correctly in both Tauri and web environments

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import {
  isTauriApp,
  isWebApp,
  getPlatform,
  onDesktop,
  onWeb,
  platformFeatures,
  getApiPort,
  platformFetch,
} from './platform';

describe('Platform Detection', () => {
  let originalWindow: any;

  beforeEach(() => {
    originalWindow = global.window;
  });

  afterEach(() => {
    global.window = originalWindow;
  });

  describe('isTauriApp', () => {
    it('should return true when __TAURI__ is defined', () => {
      global.window = { __TAURI__: {} } as any;
      expect(isTauriApp()).toBe(true);
    });

    it('should return false when __TAURI__ is undefined', () => {
      global.window = {} as any;
      expect(isTauriApp()).toBe(false);
    });

    it('should return false when window is undefined', () => {
      (global as any).window = undefined;
      expect(isTauriApp()).toBe(false);
    });
  });

  describe('isWebApp', () => {
    it('should return false when in Tauri environment', () => {
      global.window = { __TAURI__: {} } as any;
      expect(isWebApp()).toBe(false);
    });

    it('should return true when not in Tauri environment', () => {
      global.window = {} as any;
      expect(isWebApp()).toBe(true);
    });
  });

  describe('getPlatform', () => {
    it('should return "desktop" when in Tauri environment', () => {
      global.window = { __TAURI__: {} } as any;
      expect(getPlatform()).toBe('desktop');
    });

    it('should return "web" when not in Tauri environment', () => {
      global.window = {} as any;
      expect(getPlatform()).toBe('web');
    });
  });

  describe('onDesktop', () => {
    it('should execute callback when in Tauri environment', () => {
      global.window = { __TAURI__: {} } as any;
      const callback = vi.fn();
      onDesktop(callback);
      expect(callback).toHaveBeenCalled();
    });

    it('should not execute callback when in web environment', () => {
      global.window = {} as any;
      const callback = vi.fn();
      onDesktop(callback);
      expect(callback).not.toHaveBeenCalled();
    });
  });

  describe('onWeb', () => {
    it('should execute callback when in web environment', () => {
      global.window = {} as any;
      const callback = vi.fn();
      onWeb(callback);
      expect(callback).toHaveBeenCalled();
    });

    it('should not execute callback when in Tauri environment', () => {
      global.window = { __TAURI__: {} } as any;
      const callback = vi.fn();
      onWeb(callback);
      expect(callback).not.toHaveBeenCalled();
    });
  });
});

describe('Platform Features', () => {
  let originalWindow: any;

  beforeEach(() => {
    originalWindow = global.window;
  });

  afterEach(() => {
    global.window = originalWindow;
  });

  describe('hasFileSystemAccess', () => {
    it('should return true in Tauri environment', () => {
      global.window = { __TAURI__: {} } as any;
      expect(platformFeatures.hasFileSystemAccess()).toBe(true);
    });

    it('should return false in web environment', () => {
      global.window = {} as any;
      expect(platformFeatures.hasFileSystemAccess()).toBe(false);
    });
  });

  describe('hasSystemTray', () => {
    it('should return true in Tauri environment', () => {
      global.window = { __TAURI__: {} } as any;
      expect(platformFeatures.hasSystemTray()).toBe(true);
    });

    it('should return false in web environment', () => {
      global.window = {} as any;
      expect(platformFeatures.hasSystemTray()).toBe(false);
    });
  });

  describe('hasNativeNotifications', () => {
    it('should return true in Tauri environment', () => {
      global.window = { __TAURI__: {} } as any;
      expect(platformFeatures.hasNativeNotifications()).toBe(true);
    });

    it('should return false in web environment', () => {
      global.window = {} as any;
      expect(platformFeatures.hasNativeNotifications()).toBe(false);
    });
  });

  describe('requiresProxy', () => {
    it('should return false in Tauri environment', () => {
      global.window = { __TAURI__: {} } as any;
      expect(platformFeatures.requiresProxy()).toBe(false);
    });

    it('should return true in web environment', () => {
      global.window = {} as any;
      expect(platformFeatures.requiresProxy()).toBe(true);
    });
  });
});

describe('getApiPort', () => {
  let originalWindow: any;

  beforeEach(() => {
    originalWindow = global.window;
    vi.clearAllMocks();
  });

  afterEach(() => {
    global.window = originalWindow;
  });

  it('should use env var in web mode when available', async () => {
    global.window = {} as any;
    import.meta.env.VITE_ORKEE_API_PORT = '9000';

    const port = await getApiPort();
    expect(port).toBe(9000);
  });

  it('should use default 4001 in web mode when env var not set', async () => {
    global.window = {} as any;
    delete import.meta.env.VITE_ORKEE_API_PORT;

    const port = await getApiPort();
    expect(port).toBe(4001);
  });
});

describe('platformFetch', () => {
  let originalWindow: any;
  let mockTauriFetch: any;
  let mockBrowserFetch: any;

  beforeEach(() => {
    originalWindow = global.window;
    mockTauriFetch = vi.fn();
    mockBrowserFetch = vi.fn();
    global.fetch = mockBrowserFetch;
    vi.clearAllMocks();
  });

  afterEach(() => {
    global.window = originalWindow;
  });

  it('should use Tauri fetch in desktop mode', async () => {
    global.window = { __TAURI__: {} } as any;
    vi.doMock('@tauri-apps/plugin-http', () => ({
      fetch: mockTauriFetch,
    }));

    await platformFetch('http://localhost:4001/api/health');
    expect(mockBrowserFetch).not.toHaveBeenCalled();
  });

  it('should use browser fetch in web mode', async () => {
    global.window = {} as any;
    mockBrowserFetch.mockResolvedValue({
      ok: true,
      json: async () => ({ status: 'ok' }),
    });

    await platformFetch('http://localhost:4001/api/health');
    expect(mockBrowserFetch).toHaveBeenCalledWith('http://localhost:4001/api/health', undefined);
  });

  it('should pass options to fetch in web mode', async () => {
    global.window = {} as any;
    const options = {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ test: 'data' }),
    };
    mockBrowserFetch.mockResolvedValue({
      ok: true,
      json: async () => ({ success: true }),
    });

    await platformFetch('http://localhost:4001/api/test', options);
    expect(mockBrowserFetch).toHaveBeenCalledWith('http://localhost:4001/api/test', options);
  });
});
