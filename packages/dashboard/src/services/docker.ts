// Docker image management service
// Provides API functions for Docker authentication, image listing, building, and pushing

import { apiRequest } from './api';

export interface DockerStatus {
  logged_in: boolean;
  username: string | null;
  email: string | null;
  server_address: string | null;
}

export interface DockerDaemonStatus {
  running: boolean;
  version?: string;
  error?: string;
}

export interface DockerConfig {
  username: string | null;
  auth_servers: string[];
}

export interface DockerImage {
  repository: string;
  tag: string;
  image_id: string;
  size: string;
  created: string;
}

export interface DockerHubImage {
  name: string;
  description: string;
  star_count: number;
  pull_count: number;
  is_official: boolean;
  is_automated: boolean;
}

export interface BuildImageRequest {
  dockerfile_path: string;
  build_context: string;
  image_tag: string;
  labels?: Record<string, string>;
  push_to_registry?: boolean;
}

export interface BuildImageResponse {
  message: string;
  image_tag: string;
  output: string;
}

export interface PushImageRequest {
  image_tag: string;
}

export interface PushImageResponse {
  message: string;
  image_tag: string;
  output: string;
}

export interface DeleteImageRequest {
  image: string;
  force?: boolean;
}

export interface DockerLoginRequest {
  username: string;
  password: string;
}

// ============================================================================
// Docker Authentication
// ============================================================================

/**
 * Get Docker login status
 */
export async function getDockerStatus(): Promise<DockerStatus> {
  const response = await apiRequest<DockerStatus>('/api/sandbox/docker/status');
  if (!response.success || !response.data) {
    throw new Error(response.error || 'Failed to get Docker status');
  }
  return response.data;
}

/**
 * Check if Docker daemon is running
 */
export async function getDockerDaemonStatus(): Promise<DockerDaemonStatus> {
  const response = await apiRequest<DockerDaemonStatus>('/api/sandbox/docker/daemon-status');
  if (!response.success || !response.data) {
    throw new Error(response.error || 'Failed to get Docker daemon status');
  }
  return response.data;
}

/**
 * Get Docker configuration
 */
export async function getDockerConfig(): Promise<DockerConfig> {
  const response = await apiRequest<DockerConfig>('/api/sandbox/docker/config');
  if (!response.success || !response.data) {
    throw new Error(response.error || 'Failed to get Docker config');
  }
  return response.data;
}

/**
 * Login to Docker Hub (username/password - legacy)
 */
export async function dockerLogin(
  request: DockerLoginRequest
): Promise<{ message: string }> {
  const response = await apiRequest<{ message: string }>(
    '/api/sandbox/docker/login',
    {
      method: 'POST',
      body: JSON.stringify(request),
    }
  );
  if (!response.success || !response.data) {
    throw new Error(response.error || 'Failed to login to Docker');
  }
  return response.data;
}

/**
 * Login to Docker Hub using OAuth (opens terminal)
 */
export async function dockerLoginOAuth(): Promise<{ message: string }> {
  const response = await apiRequest<{ message: string }>(
    '/api/sandbox/docker/login-oauth',
    {
      method: 'POST',
    }
  );
  if (!response.success || !response.data) {
    throw new Error(response.error || 'Failed to launch Docker OAuth login');
  }
  return response.data;
}

/**
 * Logout from Docker Hub
 */
export async function dockerLogout(): Promise<{ message: string }> {
  const response = await apiRequest<{ message: string }>(
    '/api/sandbox/docker/logout',
    {
      method: 'POST',
    }
  );
  if (!response.success || !response.data) {
    throw new Error(response.error || 'Failed to logout from Docker');
  }
  return response.data;
}

// ============================================================================
// Local Images
// ============================================================================

/**
 * List local Docker images with orkee.sandbox label
 */
export async function listLocalImages(): Promise<DockerImage[]> {
  const response = await apiRequest<DockerImage[]>('/api/sandbox/docker/images/local');
  if (!response.success || !response.data) {
    throw new Error(response.error || 'Failed to list local images');
  }
  return response.data;
}

/**
 * Delete a local Docker image
 */
export async function deleteDockerImage(
  request: DeleteImageRequest
): Promise<{ message: string }> {
  const response = await apiRequest<{ message: string }>(
    '/api/sandbox/docker/images/delete',
    {
      method: 'POST',
      body: JSON.stringify(request),
    }
  );
  if (!response.success || !response.data) {
    throw new Error(response.error || 'Failed to delete Docker image');
  }
  return response.data;
}

// ============================================================================
// Docker Hub Images
// ============================================================================

/**
 * List Docker Hub images for a specific user
 */
export async function listUserDockerHubImages(
  username: string
): Promise<DockerHubImage[]> {
  const params = new URLSearchParams({ username });
  const response = await apiRequest<DockerHubImage[]>(
    `/api/sandbox/docker/images/user?${params.toString()}`
  );
  if (!response.success || !response.data) {
    throw new Error(response.error || 'Failed to list user Docker Hub images');
  }
  return response.data;
}

// ============================================================================
// Build & Push Operations
// ============================================================================

/**
 * Build a Docker image
 */
export async function buildDockerImage(
  request: BuildImageRequest
): Promise<BuildImageResponse> {
  const response = await apiRequest<BuildImageResponse>(
    '/api/sandbox/docker/images/build',
    {
      method: 'POST',
      body: JSON.stringify(request),
    }
  );
  if (!response.success || !response.data) {
    throw new Error(response.error || 'Failed to build Docker image');
  }
  return response.data;
}

/**
 * Pull a Docker image from Docker Hub
 */
export async function pullDockerImage(
  request: PushImageRequest
): Promise<PushImageResponse> {
  const response = await apiRequest<PushImageResponse>(
    '/api/sandbox/docker/images/pull',
    {
      method: 'POST',
      body: JSON.stringify(request),
    }
  );
  if (!response.success || !response.data) {
    throw new Error(response.error || 'Failed to pull Docker image');
  }
  return response.data;
}

/**
 * Push a Docker image to Docker Hub
 */
export async function pushDockerImage(
  request: PushImageRequest
): Promise<PushImageResponse> {
  const response = await apiRequest<PushImageResponse>(
    '/api/sandbox/docker/images/push',
    {
      method: 'POST',
      body: JSON.stringify(request),
    }
  );
  if (!response.success || !response.data) {
    throw new Error(response.error || 'Failed to push Docker image');
  }
  return response.data;
}
