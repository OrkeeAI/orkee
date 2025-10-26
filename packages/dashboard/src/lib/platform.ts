/**
 * Platform detection utilities for Orkee
 * Detects whether the app is running in a web browser or Tauri desktop environment
 */

/**
 * Check if the app is running in Tauri desktop mode
 * @returns true if running in Tauri, false if running in web browser
 */
export function isTauriApp(): boolean {
  // @ts-expect-error - __TAURI__ is injected by Tauri at runtime
  const hasTauri = typeof window !== 'undefined' && window.__TAURI__ !== undefined;
  console.log('[Platform] Detection:', {
    hasWindow: typeof window !== 'undefined',
    hasTauriGlobal: typeof window !== 'undefined' ? window.__TAURI__ !== undefined : false,
    windowKeys: typeof window !== 'undefined' ? Object.keys(window).filter(k => k.includes('TAURI') || k.includes('tauri')) : []
  });
  return hasTauri;
}

/**
 * Check if the app is running in a web browser
 * @returns true if running in browser, false if running in Tauri
 */
export function isWebApp(): boolean {
  return !isTauriApp()
}

/**
 * Get the current platform type
 * @returns 'desktop' or 'web'
 */
export function getPlatform(): 'desktop' | 'web' {
  return isTauriApp() ? 'desktop' : 'web'
}

/**
 * Execute code only in desktop environment
 * @param callback - Function to execute if in desktop mode
 */
export function onDesktop(callback: () => void): void {
  if (isTauriApp()) {
    callback()
  }
}

/**
 * Execute code only in web environment
 * @param callback - Function to execute if in web mode
 */
export function onWeb(callback: () => void): void {
  if (isWebApp()) {
    callback()
  }
}

/**
 * Feature flags for platform-specific capabilities
 */
export const platformFeatures = {
  /**
   * Whether the platform supports native file system access
   */
  hasFileSystemAccess(): boolean {
    return isTauriApp()
  },

  /**
   * Whether the platform supports system tray
   */
  hasSystemTray(): boolean {
    return isTauriApp()
  },

  /**
   * Whether the platform supports native notifications
   */
  hasNativeNotifications(): boolean {
    return isTauriApp()
  },

  /**
   * Whether the platform requires proxy for API calls
   */
  requiresProxy(): boolean {
    return isWebApp()
  },
}

/**
 * Get the API port that the CLI server is running on
 * In desktop mode, queries Tauri for the dynamically assigned port
 * In web mode, returns the default port from env or 4001
 */
export async function getApiPort(): Promise<number> {
  const isTauri = isTauriApp();
  console.log('[Platform] isTauriApp:', isTauri);

  if (isTauri) {
    try {
      console.log('[Platform] Invoking get_api_port command...');
      // Dynamically import Tauri modules only when needed
      const { invoke } = await import('@tauri-apps/api/core');
      // Query Tauri for the actual port the CLI server is using
      const port = await invoke<number>('get_api_port');
      console.log('[Platform] Got dynamic API port from Tauri:', port);
      return port;
    } catch (error) {
      console.error('[Platform] Failed to get API port from Tauri:', error);
      return 4001; // Fallback
    }
  }

  // Web mode: use env var or default
  const envPort = import.meta.env.VITE_ORKEE_API_PORT;
  const port = envPort ? parseInt(envPort) : 4001;
  console.log('[Platform] Web mode - using port:', port);
  return port;
}

/**
 * Get the API token for authenticating with the CLI server
 * In desktop mode, reads token from ~/.orkee/api-token via Tauri command
 * In web mode, returns null (web mode doesn't need token authentication)
 */
export async function getApiToken(): Promise<string | null> {
  const isTauri = isTauriApp();
  console.log('[Platform] getApiToken - isTauriApp:', isTauri);

  if (isTauri) {
    try {
      console.log('[Platform] Invoking get_api_token command...');
      // Dynamically import Tauri modules only when needed
      const { invoke } = await import('@tauri-apps/api/core');
      // Query Tauri for the API token
      const token = await invoke<string>('get_api_token');
      console.log('[Platform] Got API token from Tauri (length:', token.length, ')');
      return token;
    } catch (error) {
      console.error('[Platform] Failed to get API token from Tauri:', error);
      // Don't return null here - propagate error so API calls fail with clear message
      throw new Error('Failed to read API token. Please restart the Orkee server.');
    }
  }

  // Web mode: Check for token in environment variable first
  const envToken = import.meta.env.VITE_ORKEE_API_TOKEN;
  if (envToken) {
    console.log('[Platform] Web mode - using token from VITE_ORKEE_API_TOKEN');
    return envToken;
  }

  // Fall back to no token (development mode with ORKEE_DEV_MODE=true on server)
  console.log('[Platform] Web mode - no API token (dev mode assumed)');
  return null;
}

/**
 * Platform-aware fetch that bypasses CORS in Tauri
 * In desktop mode, uses Tauri's HTTP client which bypasses browser CORS
 * In web mode, uses standard fetch (which respects CORS)
 */
export async function platformFetch(url: string, options?: RequestInit): Promise<Response> {
  if (isTauriApp()) {
    // Dynamically import Tauri HTTP plugin only when needed
    const { fetch: tauriFetch } = await import('@tauri-apps/plugin-http');
    // Use Tauri's fetch which bypasses CORS restrictions
    return tauriFetch(url, options);
  }

  // Use standard browser fetch in web mode
  return fetch(url, options);
}
