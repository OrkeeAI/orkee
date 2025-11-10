// Docker image management service
// Provides API functions for Docker authentication, image listing, building, and pushing

import { apiRequest } from './api';

export interface DockerStatus {
  logged_in: boolean;
  username: string | null;
  email: string | null;
  server_address: string | null;
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
  const response = await apiRequest<DockerStatus>('/sandbox/docker/status');
  if (!response.success || !response.data) {
    throw new Error(response.error || 'Failed to get Docker status');
  }
  return response.data;
}

/**
 * Get Docker configuration
 */
export async function getDockerConfig(): Promise<DockerConfig> {
  const response = await apiRequest<DockerConfig>('/sandbox/docker/config');
  if (!response.success || !response.data) {
    throw new Error(response.error || 'Failed to get Docker config');
  }
  return response.data;
}

/**
 * Login to Docker Hub
 */
export async function dockerLogin(
  request: DockerLoginRequest
): Promise<{ message: string }> {
  const response = await apiRequest<{ message: string }>(
    '/sandbox/docker/login',
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
 * Logout from Docker Hub
 */
export async function dockerLogout(): Promise<{ message: string }> {
  const response = await apiRequest<{ message: string }>(
    '/sandbox/docker/logout',
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
  const response = await apiRequest<DockerImage[]>('/sandbox/docker/images/local');
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
    '/sandbox/docker/images/delete',
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
 * Search Docker Hub for images
 */
export async function searchDockerHubImages(
  query: string,
  limit?: number
): Promise<DockerHubImage[]> {
  const params = new URLSearchParams({ query });
  if (limit) {
    params.append('limit', limit.toString());
  }

  const response = await apiRequest<DockerHubImage[]>(
    `/sandbox/docker/images/search?${params.toString()}`
  );
  if (!response.success || !response.data) {
    throw new Error(response.error || 'Failed to search Docker Hub images');
  }
  return response.data;
}

/**
 * List Docker Hub images for a specific user
 */
export async function listUserDockerHubImages(
  username: string
): Promise<DockerHubImage[]> {
  const params = new URLSearchParams({ username });
  const response = await apiRequest<DockerHubImage[]>(
    `/sandbox/docker/images/user?${params.toString()}`
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
    '/sandbox/docker/images/build',
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
 * Push a Docker image to Docker Hub
 */
export async function pushDockerImage(
  request: PushImageRequest
): Promise<PushImageResponse> {
  const response = await apiRequest<PushImageResponse>(
    '/sandbox/docker/images/push',
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
