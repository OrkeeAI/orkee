import { platformFetch } from '@/lib/platform'

export interface ImportResult {
  projectsImported: number
  projectsSkipped: number
  conflictsCount: number
  conflicts: Array<{
    projectId: string
    projectName: string
    conflictType: string
  }>
}

/**
 * Export the database as a compressed backup file
 * Downloads the file automatically in the browser
 */
export async function exportDatabase(): Promise<{ success: boolean; error?: string }> {
  try {
    const baseUrl = await getApiBaseUrl()
    const rawResponse = await platformFetch(`${baseUrl}/api/projects/export`)

    if (!rawResponse.ok) {
      return {
        success: false,
        error: `HTTP error! status: ${rawResponse.status}`,
      }
    }

    // Get filename from Content-Disposition header
    const contentDisposition = rawResponse.headers.get('Content-Disposition')
    const filenameMatch = contentDisposition?.match(/filename="([^"]+)"/)
    const filename = filenameMatch ? filenameMatch[1] : `orkee-backup-${new Date().toISOString().slice(0, 10)}.gz`

    // Download the file
    const blob = await rawResponse.blob()
    const url = window.URL.createObjectURL(blob)
    const link = document.createElement('a')
    link.href = url
    link.download = filename
    document.body.appendChild(link)
    link.click()
    document.body.removeChild(link)
    window.URL.revokeObjectURL(url)

    return { success: true }
  } catch (error) {
    console.error('Database export error:', error)
    return {
      success: false,
      error: error instanceof Error ? error.message : 'Unknown error',
    }
  }
}

/**
 * Import a database backup file
 */
export async function importDatabase(file: File): Promise<{
  success: boolean
  data?: ImportResult
  error?: string
}> {
  try {
    // Read file as binary
    const arrayBuffer = await file.arrayBuffer()
    const bytes = new Uint8Array(arrayBuffer)

    // Send to API
    const baseUrl = await getApiBaseUrl()
    const response = await platformFetch(`${baseUrl}/api/projects/import`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/octet-stream',
      },
      body: bytes,
    })

    if (!response.ok) {
      const errorText = await response.text()
      return {
        success: false,
        error: `HTTP error! status: ${response.status}: ${errorText}`,
      }
    }

    const result = await response.json()

    if (result.success && result.data) {
      return {
        success: true,
        data: result.data,
      }
    } else {
      return {
        success: false,
        error: result.error || 'Import failed',
      }
    }
  } catch (error) {
    console.error('Database import error:', error)
    return {
      success: false,
      error: error instanceof Error ? error.message : 'Unknown error',
    }
  }
}

// Helper function to get API base URL (duplicated from api.ts to avoid circular dependency)
async function getApiBaseUrl(): Promise<string> {
  const API_PORT = import.meta.env.VITE_ORKEE_API_PORT || '4001'
  return import.meta.env.VITE_API_URL || `http://localhost:${API_PORT}`
}
