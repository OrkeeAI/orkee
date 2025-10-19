import { isTauriApp, platformFetch, getApiPort } from '@/lib/platform';

console.log('[API] Module loaded - checking platform...');
console.log('[API] isTauriApp():', isTauriApp());

// Default API configuration
const API_PORT = parseInt(import.meta.env.VITE_ORKEE_API_PORT || '4001');
const DEFAULT_API_BASE_URL = import.meta.env.VITE_API_URL || `http://localhost:${API_PORT}`;

// Cache for the dynamically determined API base URL
let cachedApiBaseUrl: string | null = null;

/**
 * Get the appropriate API base URL based on the platform
 * In web mode: uses generated port config or env var (direct connection)
 * In desktop mode: queries Tauri for the dynamic port and connects directly
 */
async function getApiBaseUrl(): Promise<string> {
  // Return cached value if available
  if (cachedApiBaseUrl) {
    console.log(`[API] Using cached API URL: ${cachedApiBaseUrl}`);
    return cachedApiBaseUrl;
  }

  console.log('[API] Determining API base URL...');
  console.log('[API] isTauriApp():', isTauriApp());

  if (isTauriApp()) {
    try {
      // Get the dynamically assigned port from Tauri
      console.log('[API] Fetching dynamic port from Tauri...');
      const port = await getApiPort();
      cachedApiBaseUrl = `http://localhost:${port}`;
      console.log(`[API] ✓ Using dynamic Tauri API port: ${port} (URL: ${cachedApiBaseUrl})`);
      return cachedApiBaseUrl;
    } catch (error) {
      console.error('[API] ✗ Failed to get API port from Tauri:', error);
      // Fallback to default
      cachedApiBaseUrl = DEFAULT_API_BASE_URL;
      console.warn(`[API] Falling back to default: ${cachedApiBaseUrl}`);
      return cachedApiBaseUrl;
    }
  }

  // Web mode: use env var or default
  cachedApiBaseUrl = DEFAULT_API_BASE_URL;
  console.log(`[API] Web mode - using port: ${API_PORT} (URL: ${cachedApiBaseUrl})`);
  return cachedApiBaseUrl;
}

export interface ApiResponse<T> {
  data: T;
  error?: string;
}

export class ApiClient {
  private baseURL: Promise<string>;

  constructor(baseURL?: string) {
    this.baseURL = baseURL ? Promise.resolve(baseURL) : getApiBaseUrl();
  }

  async get<T>(endpoint: string): Promise<ApiResponse<T>> {
    try {
      const baseUrl = await this.baseURL;
      const response = await platformFetch(`${baseUrl}${endpoint}`, {
        method: 'GET',
        headers: {
          'Content-Type': 'application/json',
        },
      });

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      const data = await response.json();
      return { data };
    } catch (error) {
      console.error(`API GET error for ${endpoint}:`, error);
      return {
        data: null as unknown as T,
        error: error instanceof Error ? error.message : 'Unknown error',
      };
    }
  }

  async post<T>(endpoint: string, body: unknown): Promise<ApiResponse<T>> {
    try {
      const baseUrl = await this.baseURL;
      const response = await platformFetch(`${baseUrl}${endpoint}`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(body),
      });

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      const data = await response.json();
      return { data };
    } catch (error) {
      console.error(`API POST error for ${endpoint}:`, error);
      return {
        data: null as unknown as T,
        error: error instanceof Error ? error.message : 'Unknown error',
      };
    }
  }
}

export const apiClient = new ApiClient();

// Generic API request function for consistent response handling
export async function apiRequest<T>(
  url: string, 
  options: RequestInit = {}
): Promise<{ success: boolean; data?: T; error?: string }> {
  try {
    // Include auth token if available
    const accessToken = localStorage.getItem('orkee_access_token');
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
      ...(options.headers as Record<string, string> || {}),
    };
    
    if (accessToken) {
      headers['Authorization'] = `Bearer ${accessToken}`;
    }
    
    const baseUrl = await getApiBaseUrl();
    const response = await platformFetch(`${baseUrl}${url}`, {
      ...options,
      headers,
    });

    const result = await response.json();
    
    if (!response.ok) {
      return {
        success: false,
        error: result.error || `HTTP error! status: ${response.status}`,
      };
    }

    return {
      success: true,
      data: result,
    };
  } catch (error) {
    console.error(`API request error for ${url}:`, error);
    return {
      success: false,
      error: error instanceof Error ? error.message : 'Unknown error',
    };
  }
}

export interface PreviewApiResponse<T> {
  success: boolean;
  data: T;
  error?: string;
}

interface ServersListResponse {
  servers: Array<{
    project_id: string;
    [key: string]: unknown;
  }>;
}

export const previewService = {
  async getActiveServers(): Promise<string[]> {
    try {
      const baseUrl = await getApiBaseUrl();
      const response = await platformFetch(`${baseUrl}/api/preview/servers`);
      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }
      const result: PreviewApiResponse<ServersListResponse> = await response.json();
      if (result.success && result.data && result.data.servers) {
        return result.data.servers.map(s => s.project_id);
      }
      return [];
    } catch (error) {
      console.error('Failed to fetch active servers:', error);
      return [];
    }
  }
};