// ABOUTME: Service for managing sandbox configuration settings and provider credentials
// ABOUTME: Handles sandbox settings, provider configuration, and credential validation

import { apiRequest } from './api'

export interface SandboxSettings {
  // General Settings
  enabled: boolean
  default_provider: string
  default_image: string

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
  const response = await apiRequest<SandboxSettings>('/api/sandbox-settings')
  if (response.success && response.data) {
    return response.data
  }
  throw new Error(response.error || 'Failed to get sandbox settings')
}

// Update sandbox settings
export async function updateSandboxSettings(settings: Partial<SandboxSettings>): Promise<SandboxSettings> {
  const response = await apiRequest<SandboxSettings>(
    '/api/sandbox-settings',
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

// Get all provider settings
export async function getAllProviderSettings(): Promise<ProviderSettings[]> {
  const response = await apiRequest<ProviderSettings[]>('/api/sandbox-providers/settings')
  if (response.success && response.data) {
    return response.data
  }
  throw new Error(response.error || 'Failed to get provider settings')
}

// Get specific provider settings
export async function getProviderSettings(provider: string): Promise<ProviderSettings> {
  const response = await apiRequest<ProviderSettings>(`/api/sandbox-providers/${provider}/settings`)
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
    `/api/sandbox-providers/${provider}/settings`,
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
    `/api/sandbox-providers/${provider}/credentials`,
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
    `/api/sandbox-providers/${provider}/validate`,
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
