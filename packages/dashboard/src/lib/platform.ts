/**
 * Platform detection utilities for Orkee
 * Detects whether the app is running in a web browser or Tauri desktop environment
 */

import { fetch as tauriFetch } from '@tauri-apps/plugin-http';
import { invoke } from '@tauri-apps/api/core';

/**
 * Check if the app is running in Tauri desktop mode
 * @returns true if running in Tauri, false if running in web browser
 */
export function isTauriApp(): boolean {
  // @ts-expect-error - __TAURI__ is injected by Tauri at runtime
  return typeof window !== 'undefined' && window.__TAURI__ !== undefined
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
  if (isTauriApp()) {
    try {
      // Query Tauri for the actual port the CLI server is using
      const port = await invoke<number>('get_api_port');
      return port;
    } catch (error) {
      console.error('Failed to get API port from Tauri:', error);
      return 4001; // Fallback
    }
  }

  // Web mode: use env var or default
  const envPort = import.meta.env.VITE_ORKEE_API_PORT;
  return envPort ? parseInt(envPort) : 4001;
}

/**
 * Platform-aware fetch that bypasses CORS in Tauri
 * In desktop mode, uses Tauri's HTTP client which bypasses browser CORS
 * In web mode, uses standard fetch (which respects CORS)
 */
export async function platformFetch(url: string, options?: RequestInit): Promise<Response> {
  if (isTauriApp()) {
    // Use Tauri's fetch which bypasses CORS restrictions
    return tauriFetch(url, options);
  }

  // Use standard browser fetch in web mode
  return fetch(url, options);
}
