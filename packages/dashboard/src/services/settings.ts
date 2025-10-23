import { apiRequest } from './api'

export interface SystemSetting {
  key: string
  value: string
  category: string
  description: string | null
  data_type: string
  is_secret: boolean
  requires_restart: boolean
  is_env_only: boolean
  updated_at: string
  updated_by: string
}

export interface SettingsResponse {
  settings: SystemSetting[]
  requires_restart: boolean
}

export interface SettingUpdate {
  value: string
}

export async function getAllSettings(): Promise<SettingsResponse> {
  const response = await apiRequest<SettingsResponse>('/api/settings')
  if (response.success && response.data) {
    return response.data
  }
  throw new Error(response.error || 'Failed to get settings')
}

export async function getSettingsByCategory(category: string): Promise<SettingsResponse> {
  const response = await apiRequest<SettingsResponse>(`/api/settings/category/${category}`)
  if (response.success && response.data) {
    return response.data
  }
  throw new Error(response.error || 'Failed to get settings')
}

export async function updateSetting(key: string, value: string): Promise<SystemSetting> {
  const response = await apiRequest<SystemSetting>(
    `/api/settings/key/${key}`,
    {
      method: 'PUT',
      body: JSON.stringify({ value }),
    }
  )
  if (response.success && response.data) {
    return response.data
  }
  throw new Error(response.error || 'Failed to update setting')
}

export async function bulkUpdateSettings(settings: Array<{ key: string; value: string }>): Promise<SettingsResponse> {
  const response = await apiRequest<SettingsResponse>(
    '/api/settings/bulk',
    {
      method: 'PUT',
      body: JSON.stringify({ settings }),
    }
  )
  if (response.success && response.data) {
    return response.data
  }
  throw new Error(response.error || 'Failed to bulk update settings')
}

export async function resetCategory(category: string): Promise<SettingsResponse> {
  const response = await apiRequest<SettingsResponse>(
    `/api/settings/reset/${category}`,
    {
      method: 'POST',
    }
  )
  if (response.success && response.data) {
    return response.data
  }
  throw new Error(response.error || 'Failed to reset category')
}
