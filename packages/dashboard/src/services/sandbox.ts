// ABOUTME: Service for managing sandbox configuration settings and provider credentials
// ABOUTME: Handles sandbox settings, provider configuration, and credential validation

import { apiRequest } from './api'

export interface SandboxSettings {
  // General Settings
  enabled: boolean
  default_provider: string
  default_image: string
  docker_username?: string | null

  // Resource Limits
  max_concurrent_local: number
  max_concurrent_cloud: number
  max_cpu_cores_per_sandbox: number
  max_memory_gb_per_sandbox: number
  max_disk_gb_per_sandbox: number
  max_gpu_per_sandbox: number

  // Lifecycle Settings
  auto_stop_idle_minutes: number
  max_runtime_hours: number
  cleanup_interval_minutes: number
  preserve_stopped_sandboxes: boolean
  auto_restart_failed: boolean
  max_restart_attempts: number

  // Cost Management
  cost_tracking_enabled: boolean
  cost_alert_threshold: number
  max_cost_per_sandbox: number
  max_total_cost: number
  auto_stop_at_cost_limit: boolean

  // Network Settings
  default_network_mode: string
  allow_public_endpoints: boolean
  require_auth_for_web: boolean

  // Security Settings
  allow_privileged_containers: boolean
  require_non_root_user: boolean
  enable_security_scanning: boolean
  allowed_base_images: string | null
  blocked_commands: string | null

  // Monitoring
  resource_monitoring_interval_seconds: number
  health_check_interval_seconds: number
  log_retention_days: number
  metrics_retention_days: number

  // Templates
  allow_custom_templates: boolean
  require_template_approval: boolean
  share_templates_globally: boolean

  updated_at: string
  updated_by: string | null
}

export interface ProviderSettings {
  provider: string

  // Status
  enabled: boolean
  configured: boolean
  validated_at: string | null
  validation_error: string | null

  // Credentials (encrypted)
  api_key: string | null
  api_secret: string | null
  api_endpoint: string | null

  // Provider-specific IDs
  workspace_id: string | null
  project_id: string | null
  account_id: string | null
  organization_id: string | null
  app_name: string | null
  namespace_id: string | null

  // Defaults
  default_region: string | null
  default_instance_type: string | null
  default_image: string | null

  // Resource defaults
  default_cpu_cores: number | null
  default_memory_mb: number | null
  default_disk_gb: number | null
  default_gpu_type: string | null

  // Cost overrides
  cost_per_hour: number | null
  cost_per_gb_memory: number | null
  cost_per_vcpu: number | null
  cost_per_gpu_hour: number | null

  // Limits
  max_sandboxes: number | null
  max_runtime_hours: number | null
  max_total_cost: number | null

  // Additional configuration (JSON)
  custom_config: string | null

  updated_at: string
  updated_by: string | null
}

export interface ProviderCredentials {
  api_key?: string
  api_secret?: string
  api_endpoint?: string
  workspace_id?: string
  project_id?: string
  account_id?: string
  organization_id?: string
  app_name?: string
  namespace_id?: string
}

export interface ValidationResult {
  valid: boolean
  message: string
  details?: Record<string, unknown>
}

// Get sandbox settings
export async function getSandboxSettings(): Promise<SandboxSettings> {
  const response = await apiRequest<SandboxSettings>('/api/sandbox/settings')
  if (response.success && response.data) {
    return response.data
  }
  throw new Error(response.error || 'Failed to get sandbox settings')
}

// Update sandbox settings
export async function updateSandboxSettings(settings: Partial<SandboxSettings>): Promise<SandboxSettings> {
  const response = await apiRequest<SandboxSettings>(
    '/api/sandbox/settings',
    {
      method: 'PUT',
      body: JSON.stringify(settings),
    }
  )
  if (response.success && response.data) {
    return response.data
  }
  throw new Error(response.error || 'Failed to update sandbox settings')
}

// Set the default Docker image for new sandboxes
export async function setDefaultImage(imageTag: string): Promise<SandboxSettings> {
  // Fetch current settings first, then update only the default_image field
  const currentSettings = await getSandboxSettings()
  return updateSandboxSettings({ ...currentSettings, default_image: imageTag })
}

// Get all provider settings
export async function getAllProviderSettings(): Promise<ProviderSettings[]> {
  const response = await apiRequest<ProviderSettings[]>('/api/sandbox/providers')
  if (response.success && response.data) {
    return response.data
  }
  throw new Error(response.error || 'Failed to get provider settings')
}

// Get specific provider settings
export async function getProviderSettings(provider: string): Promise<ProviderSettings> {
  const response = await apiRequest<ProviderSettings>(`/api/sandbox/providers/${provider}`)
  if (response.success && response.data) {
    return response.data
  }
  throw new Error(response.error || 'Failed to get provider settings')
}

// Update provider settings
export async function updateProviderSettings(
  provider: string,
  settings: Partial<ProviderSettings>
): Promise<ProviderSettings> {
  const response = await apiRequest<ProviderSettings>(
    `/api/sandbox/providers/${provider}`,
    {
      method: 'PUT',
      body: JSON.stringify(settings),
    }
  )
  if (response.success && response.data) {
    return response.data
  }
  throw new Error(response.error || 'Failed to update provider settings')
}

// Update provider credentials
export async function updateProviderCredentials(
  provider: string,
  credentials: ProviderCredentials
): Promise<ProviderSettings> {
  const response = await apiRequest<ProviderSettings>(
    `/api/sandbox/providers/${provider}/credentials`,
    {
      method: 'PUT',
      body: JSON.stringify(credentials),
    }
  )
  if (response.success && response.data) {
    return response.data
  }
  throw new Error(response.error || 'Failed to update provider credentials')
}

// Validate provider configuration
export async function validateProvider(provider: string): Promise<ValidationResult> {
  const response = await apiRequest<ValidationResult>(
    `/api/sandbox/providers/${provider}/validate`,
    {
      method: 'POST',
    }
  )
  if (response.success && response.data) {
    return response.data
  }
  throw new Error(response.error || 'Failed to validate provider')
}

// Toggle provider enabled status
export async function toggleProvider(provider: string, enabled: boolean): Promise<ProviderSettings> {
  return updateProviderSettings(provider, { enabled })
}

// ============================================================================
// Sandbox Instance Management (Phase 5)
// ============================================================================

export interface Sandbox {
  id: string
  name: string
  provider: string
  status: 'creating' | 'running' | 'stopped' | 'error' | 'terminating'
  created_at: string
  updated_at: string
  started_at: string | null
  stopped_at: string | null

  // Container/instance info
  container_id: string | null
  image: string
  provider_instance_id: string | null

  // Configuration
  cpu_cores: number
  memory_mb: number
  disk_gb: number
  gpu_type: string | null
  network_mode: string

  // Agent/Model info
  agent_id: string | null
  model: string | null

  // Cost tracking
  cost_per_hour: number
  total_cost: number

  // Template
  template_id: string | null

  // Project association
  project_id: string | null
  description: string | null

  error_message: string | null
}

export interface SandboxExecution {
  id: string
  sandbox_id: string
  command: string
  status: 'pending' | 'running' | 'completed' | 'failed'
  exit_code: number | null
  output: string | null
  error: string | null
  started_at: string
  completed_at: string | null
  duration_seconds: number | null

  // Agent tracking
  agent_id: string | null
  model: string | null

  // Cost tracking
  input_tokens: number | null
  output_tokens: number | null
  cost: number | null
}

export interface ResourceMetrics {
  cpu_usage_percent: number
  memory_usage_mb: number
  memory_limit_mb: number
  disk_usage_gb: number
  disk_limit_gb: number
  network_rx_bytes: number
  network_tx_bytes: number
  timestamp: string
}

export interface SandboxFile {
  path: string
  name: string
  is_directory: boolean
  size: number
  modified_at: string
}

export interface CreateSandboxRequest {
  name: string
  provider?: string
  image?: string
  cpu_cores?: number
  memory_mb?: number
  disk_gb?: number
  gpu_type?: string | null
  network_mode?: string
  agent_id?: string | null
  model?: string | null
  template_id?: string | null
  env_vars?: Record<string, string>
  volumes?: { host_path: string; container_path: string; read_only: boolean }[]
}

export interface ExecuteCommandRequest {
  command: string
  agent_id?: string | null
  model?: string | null
}

// List all sandboxes
export async function listSandboxes(): Promise<Sandbox[]> {
  const response = await apiRequest<Sandbox[]>('/api/sandboxes')
  if (response.success && response.data) {
    return response.data
  }
  throw new Error(response.error || 'Failed to list sandboxes')
}

// Get sandbox details
export async function getSandbox(id: string): Promise<Sandbox> {
  const response = await apiRequest<Sandbox>(`/api/sandboxes/${id}`)
  if (response.success && response.data) {
    return response.data
  }
  throw new Error(response.error || 'Failed to get sandbox')
}

// Create a new sandbox
export async function createSandbox(request: CreateSandboxRequest): Promise<Sandbox> {
  const response = await apiRequest<Sandbox>(
    '/api/sandboxes',
    {
      method: 'POST',
      body: JSON.stringify(request),
    }
  )
  if (response.success && response.data) {
    return response.data
  }
  throw new Error(response.error || 'Failed to create sandbox')
}

// Start a sandbox
export async function startSandbox(id: string): Promise<Sandbox> {
  const response = await apiRequest<Sandbox>(
    `/api/sandboxes/${id}/start`,
    {
      method: 'POST',
    }
  )
  if (response.success && response.data) {
    return response.data
  }
  throw new Error(response.error || 'Failed to start sandbox')
}

// Stop a sandbox
export async function stopSandbox(id: string): Promise<Sandbox> {
  const response = await apiRequest<Sandbox>(
    `/api/sandboxes/${id}/stop`,
    {
      method: 'POST',
    }
  )
  if (response.success && response.data) {
    return response.data
  }
  throw new Error(response.error || 'Failed to stop sandbox')
}

// Restart a sandbox
export async function restartSandbox(id: string): Promise<Sandbox> {
  const response = await apiRequest<Sandbox>(
    `/api/sandboxes/${id}/restart`,
    {
      method: 'POST',
    }
  )
  if (response.success && response.data) {
    return response.data
  }
  throw new Error(response.error || 'Failed to restart sandbox')
}

// Delete a sandbox
export async function deleteSandbox(id: string): Promise<void> {
  const response = await apiRequest<void>(
    `/api/sandboxes/${id}`,
    {
      method: 'DELETE',
    }
  )
  if (!response.success) {
    throw new Error(response.error || 'Failed to delete sandbox')
  }
}

// Execute a command in a sandbox
export async function executeCommand(id: string, request: ExecuteCommandRequest): Promise<SandboxExecution> {
  const response = await apiRequest<SandboxExecution>(
    `/api/sandboxes/${id}/execute`,
    {
      method: 'POST',
      body: JSON.stringify(request),
    }
  )
  if (response.success && response.data) {
    return response.data
  }
  throw new Error(response.error || 'Failed to execute command')
}

// Get sandbox executions
export async function getSandboxExecutions(id: string): Promise<SandboxExecution[]> {
  const response = await apiRequest<SandboxExecution[]>(`/api/sandboxes/${id}/executions`)
  if (response.success && response.data) {
    return response.data
  }
  throw new Error(response.error || 'Failed to get executions')
}

// Get resource metrics
export async function getSandboxMetrics(id: string): Promise<ResourceMetrics> {
  const response = await apiRequest<ResourceMetrics>(`/api/sandboxes/${id}/metrics`)
  if (response.success && response.data) {
    return response.data
  }
  throw new Error(response.error || 'Failed to get metrics')
}

// List files in sandbox
export async function listSandboxFiles(id: string, path: string = '/'): Promise<SandboxFile[]> {
  const response = await apiRequest<SandboxFile[]>(
    `/api/sandboxes/${id}/files?path=${encodeURIComponent(path)}`
  )
  if (response.success && response.data) {
    return response.data
  }
  throw new Error(response.error || 'Failed to list files')
}

// Read file from sandbox
export async function readSandboxFile(id: string, path: string): Promise<string> {
  const response = await apiRequest<{content: string}>(
    `/api/sandboxes/${id}/files/read?path=${encodeURIComponent(path)}`
  )
  if (response.success && response.data) {
    return response.data.content
  }
  throw new Error(response.error || 'Failed to read file')
}

// Write file to sandbox
export async function writeSandboxFile(id: string, path: string, content: string): Promise<void> {
  const response = await apiRequest<void>(
    `/api/sandboxes/${id}/files/write`,
    {
      method: 'POST',
      body: JSON.stringify({ path, content }),
    }
  )
  if (!response.success) {
    throw new Error(response.error || 'Failed to write file')
  }
}

// Delete file from sandbox
export async function deleteSandboxFile(id: string, path: string): Promise<void> {
  const response = await apiRequest<void>(
    `/api/sandboxes/${id}/files?path=${encodeURIComponent(path)}`,
    {
      method: 'DELETE',
    }
  )
  if (!response.success) {
    throw new Error(response.error || 'Failed to delete file')
  }
}
