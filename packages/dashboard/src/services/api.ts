// Construct API URL from port (passed from CLI via VITE_ORKEE_API_PORT)
const API_PORT = import.meta.env.VITE_ORKEE_API_PORT || '4001';
const API_BASE_URL = import.meta.env.VITE_API_URL || `http://localhost:${API_PORT}`;

export interface ApiResponse<T> {
  data: T;
  error?: string;
}

export class ApiClient {
  private baseURL: string;

  constructor(baseURL: string = API_BASE_URL) {
    this.baseURL = baseURL;
  }

  async get<T>(endpoint: string): Promise<ApiResponse<T>> {
    try {
      const response = await fetch(`${this.baseURL}${endpoint}`, {
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
      const response = await fetch(`${this.baseURL}${endpoint}`, {
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
    
    const response = await fetch(`${API_BASE_URL}${url}`, {
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

export const previewService = {
  async getActiveServers(): Promise<string[]> {
    try {
      const response = await fetch(`${API_BASE_URL}/api/preview/servers`);
      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }
      const result: PreviewApiResponse<string[]> = await response.json();
      return result.success ? result.data : [];
    } catch (error) {
      console.error('Failed to fetch active servers:', error);
      return [];
    }
  }
};