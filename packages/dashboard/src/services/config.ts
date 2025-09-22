import { apiRequest } from './api'

export interface Config {
  cloud_enabled: boolean
}

export interface ConfigResponse {
  success: boolean
  data: Config | null
  error: string | null
}

let configCache: Config | null = null

export async function fetchConfig(): Promise<Config> {
  if (configCache) {
    return configCache
  }

  try {
    const response = await apiRequest<ConfigResponse>('/api/config')
    if (response.success && response.data) {
      configCache = response.data
      return response.data
    } else {
      console.warn('Failed to fetch config:', response.error)
      return getDefaultConfig()
    }
  } catch (error) {
    console.error('Error fetching config:', error)
    return getDefaultConfig()
  }
}

function getDefaultConfig(): Config {
  return {
    cloud_enabled: false
  }
}

export function clearConfigCache() {
  configCache = null
}