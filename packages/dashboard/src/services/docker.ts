// Docker image management service
// Provides API functions for Docker authentication, image listing, building, and pushing

import { apiCall } from './api';

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

// ============================================================================
// Docker Authentication
// ============================================================================

/**
 * Get Docker login status
 */
export async function getDockerStatus(): Promise<DockerStatus> {
  const response = await apiCall<DockerStatus>('/sandbox/docker/status');
  return response.data;
}

/**
 * Get Docker configuration
 */
export async function getDockerConfig(): Promise<DockerConfig> {
  const response = await apiCall<DockerConfig>('/sandbox/docker/config');
  return response.data;
}

// ============================================================================
// Local Images
// ============================================================================

/**
 * List local Docker images with orkee.sandbox label
 */
export async function listLocalImages(): Promise<DockerImage[]> {
  const response = await apiCall<DockerImage[]>('/sandbox/docker/images/local');
  return response.data;
}

/**
 * Delete a local Docker image
 */
export async function deleteDockerImage(
  request: DeleteImageRequest
): Promise<{ message: string }> {
  const response = await apiCall<{ message: string }>(
    '/sandbox/docker/images/delete',
    {
      method: 'POST',
      body: JSON.stringify(request),
    }
  );
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

  const response = await apiCall<DockerHubImage[]>(
    `/sandbox/docker/images/search?${params.toString()}`
  );
  return response.data;
}

/**
 * List Docker Hub images for a specific user
 */
export async function listUserDockerHubImages(
  username: string
): Promise<DockerHubImage[]> {
  const params = new URLSearchParams({ username });
  const response = await apiCall<DockerHubImage[]>(
    `/sandbox/docker/images/user?${params.toString()}`
  );
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
  const response = await apiCall<BuildImageResponse>(
    '/sandbox/docker/images/build',
    {
      method: 'POST',
      body: JSON.stringify(request),
    }
  );
  return response.data;
}

/**
 * Push a Docker image to Docker Hub
 */
export async function pushDockerImage(
  request: PushImageRequest
): Promise<PushImageResponse> {
  const response = await apiCall<PushImageResponse>(
    '/sandbox/docker/images/push',
    {
      method: 'POST',
      body: JSON.stringify(request),
    }
  );
  return response.data;
}
