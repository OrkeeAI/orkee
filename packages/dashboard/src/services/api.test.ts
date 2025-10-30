// ABOUTME: Tests for API client and error handling
// ABOUTME: Validates HTTP request handling, error responses, and auth token management

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { ApiClient, apiRequest, previewService } from './api';

// Mock platform utilities
vi.mock('@/lib/platform', () => ({
  isTauriApp: vi.fn(() => false),
  isWebApp: vi.fn(() => true),
  getPlatform: vi.fn(() => 'web'),
  platformFetch: vi.fn((url: string, options?: RequestInit) => {
    return global.fetch(url, options);
  }),
  getApiPort: vi.fn(async () => 4001),
  getApiToken: vi.fn(async () => null),
  onDesktop: vi.fn(),
  onWeb: vi.fn((callback: () => void) => callback()),
  platformFeatures: {
    hasFileSystemAccess: vi.fn(() => false),
    hasSystemTray: vi.fn(() => false),
    hasNativeNotifications: vi.fn(() => false),
    requiresProxy: vi.fn(() => true),
  },
}));

describe('ApiClient', () => {
  let mockFetch: any;
  let originalFetch: any;

  beforeEach(() => {
    originalFetch = global.fetch;
    mockFetch = vi.fn();
    global.fetch = mockFetch;
    vi.clearAllMocks();
  });

  afterEach(() => {
    global.fetch = originalFetch;
  });

  describe('get', () => {
    it('should successfully fetch data from an endpoint', async () => {
      const mockData = { id: 1, name: 'Test Project' };
      mockFetch.mockResolvedValue({
        ok: true,
        json: async () => mockData,
      });

      const client = new ApiClient('http://localhost:4001');
      const result = await client.get('/api/projects');

      expect(result.data).toEqual(mockData);
      expect(result.error).toBeUndefined();
      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:4001/api/projects',
        expect.objectContaining({
          method: 'GET',
          headers: { 'Content-Type': 'application/json' },
        })
      );
    });

    it('should handle HTTP error responses', async () => {
      mockFetch.mockResolvedValue({
        ok: false,
        status: 404,
      });

      const client = new ApiClient('http://localhost:4001');
      const result = await client.get('/api/projects/nonexistent');

      expect(result.data).toBeNull();
      expect(result.error).toBe('HTTP error! status: 404');
    });

    it('should handle network errors', async () => {
      mockFetch.mockRejectedValue(new Error('Network failure'));

      const client = new ApiClient('http://localhost:4001');
      const result = await client.get('/api/projects');

      expect(result.data).toBeNull();
      expect(result.error).toBe('Network failure');
    });

    it('should handle non-Error exceptions', async () => {
      mockFetch.mockRejectedValue('String error');

      const client = new ApiClient('http://localhost:4001');
      const result = await client.get('/api/projects');

      expect(result.data).toBeNull();
      expect(result.error).toBe('Unknown error');
    });
  });

  describe('post', () => {
    it('should successfully post data to an endpoint', async () => {
      const requestBody = { name: 'New Project', path: '/path/to/project' };
      const mockResponse = { id: 1, ...requestBody };
      mockFetch.mockResolvedValue({
        ok: true,
        json: async () => mockResponse,
      });

      const client = new ApiClient('http://localhost:4001');
      const result = await client.post('/api/projects', requestBody);

      expect(result.data).toEqual(mockResponse);
      expect(result.error).toBeUndefined();
      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:4001/api/projects',
        expect.objectContaining({
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(requestBody),
        })
      );
    });

    it('should handle HTTP error responses', async () => {
      mockFetch.mockResolvedValue({
        ok: false,
        status: 400,
      });

      const client = new ApiClient('http://localhost:4001');
      const result = await client.post('/api/projects', { invalid: 'data' });

      expect(result.data).toBeNull();
      expect(result.error).toBe('HTTP error! status: 400');
    });

    it('should handle network errors', async () => {
      mockFetch.mockRejectedValue(new Error('Connection refused'));

      const client = new ApiClient('http://localhost:4001');
      const result = await client.post('/api/projects', { name: 'Test' });

      expect(result.data).toBeNull();
      expect(result.error).toBe('Connection refused');
    });
  });
});

describe('apiRequest', () => {
  let mockFetch: any;
  let originalFetch: any;
  let originalLocalStorage: any;

  beforeEach(() => {
    originalFetch = global.fetch;
    originalLocalStorage = global.localStorage;
    mockFetch = vi.fn();
    global.fetch = mockFetch;
    global.localStorage = {
      getItem: vi.fn(),
      setItem: vi.fn(),
      removeItem: vi.fn(),
      clear: vi.fn(),
      key: vi.fn(),
      length: 0,
    };
    vi.clearAllMocks();
  });

  afterEach(() => {
    global.fetch = originalFetch;
    global.localStorage = originalLocalStorage;
  });

  it('should make successful API request without auth token', async () => {
    const mockResponse = { data: { id: 1, name: 'Test' } };
    mockFetch.mockResolvedValue({
      ok: true,
      json: async () => mockResponse,
    });
    (global.localStorage.getItem as any).mockReturnValue(null);

    const result = await apiRequest('/api/projects');

    expect(result.success).toBe(true);
    expect(result.data).toEqual(mockResponse);
    expect(result.error).toBeUndefined();
  });

  it('should include auth token when available', async () => {
    const mockResponse = { data: { user: 'test' } };
    mockFetch.mockResolvedValue({
      ok: true,
      json: async () => mockResponse,
    });
    (global.localStorage.getItem as any).mockReturnValue('test-token-123');

    await apiRequest('/api/user');

    expect(mockFetch).toHaveBeenCalledWith(
      expect.stringContaining('/api/user'),
      expect.objectContaining({
        headers: expect.objectContaining({
          Authorization: 'Bearer test-token-123',
        }),
      })
    );
  });

  it('should handle HTTP error responses', async () => {
    const errorResponse = { error: 'Not found' };
    mockFetch.mockResolvedValue({
      ok: false,
      status: 404,
      json: async () => errorResponse,
    });

    const result = await apiRequest('/api/projects/missing');

    expect(result.success).toBe(false);
    expect(result.error).toBe('Not found');
  });

  it('should handle HTTP errors without error message in response', async () => {
    mockFetch.mockResolvedValue({
      ok: false,
      status: 500,
      json: async () => ({}),
    });

    const result = await apiRequest('/api/projects');

    expect(result.success).toBe(false);
    expect(result.error).toBe('HTTP error! status: 500');
  });

  it('should handle network errors', async () => {
    mockFetch.mockRejectedValue(new Error('Network error'));

    const result = await apiRequest('/api/projects');

    expect(result.success).toBe(false);
    expect(result.error).toBe('Network error');
  });

  it('should handle non-Error exceptions', async () => {
    mockFetch.mockRejectedValue('Unexpected error');

    const result = await apiRequest('/api/projects');

    expect(result.success).toBe(false);
    expect(result.error).toBe('Unknown error');
  });

  it('should merge custom headers with default headers', async () => {
    const mockResponse = { data: {} };
    mockFetch.mockResolvedValue({
      ok: true,
      json: async () => mockResponse,
    });

    await apiRequest('/api/test', {
      headers: {
        'X-Custom-Header': 'custom-value',
      },
    });

    expect(mockFetch).toHaveBeenCalledWith(
      expect.any(String),
      expect.objectContaining({
        headers: expect.objectContaining({
          'Content-Type': 'application/json',
          'X-Custom-Header': 'custom-value',
        }),
      })
    );
  });
});

describe('previewService', () => {
  let mockFetch: any;
  let originalFetch: any;

  beforeEach(() => {
    originalFetch = global.fetch;
    mockFetch = vi.fn();
    global.fetch = mockFetch;
    vi.clearAllMocks();
  });

  afterEach(() => {
    global.fetch = originalFetch;
  });

  describe('getActiveServers', () => {
    it('should return list of active servers on success', async () => {
      const mockProjectIds = ['project1', 'project2', 'project3'];
      mockFetch.mockResolvedValue({
        ok: true,
        json: async () => ({
          success: true,
          data: {
            servers: mockProjectIds.map(id => ({ project_id: id })),
          },
        }),
      });

      const result = await previewService.getActiveServers();

      expect(result).toEqual(mockProjectIds);
    });

    it('should return empty array on HTTP error', async () => {
      mockFetch.mockResolvedValue({
        ok: false,
        status: 500,
      });

      const result = await previewService.getActiveServers();

      expect(result).toEqual([]);
    });

    it('should return empty array when success is false', async () => {
      mockFetch.mockResolvedValue({
        ok: true,
        json: async () => ({
          success: false,
          data: [],
        }),
      });

      const result = await previewService.getActiveServers();

      expect(result).toEqual([]);
    });

    it('should return empty array on network error', async () => {
      mockFetch.mockRejectedValue(new Error('Network failure'));

      const result = await previewService.getActiveServers();

      expect(result).toEqual([]);
    });

    it('should handle malformed JSON response', async () => {
      mockFetch.mockResolvedValue({
        ok: true,
        json: async () => {
          throw new Error('Invalid JSON');
        },
      });

      const result = await previewService.getActiveServers();

      expect(result).toEqual([]);
    });
  });
});
