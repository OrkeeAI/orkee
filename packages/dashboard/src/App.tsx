import { useEffect, useState } from 'react'
import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom'
import { QueryClientProvider } from '@tanstack/react-query'
import { ReactQueryDevtools } from '@tanstack/react-query-devtools'
import { invoke } from '@tauri-apps/api/core'
import { Layout } from '@/components/layout/Layout'
import { ConnectionProvider } from '@/contexts/ConnectionContext'
import { CloudProvider } from '@/contexts/CloudContext'
import { ThemeProvider } from '@/contexts/ThemeContext'
import { queryClient } from '@/lib/queryClient'
import { Projects } from '@/pages/Projects'
import { ProjectDetail } from '@/pages/ProjectDetail'
import { Settings } from '@/pages/Settings'
import OAuthCallback from '@/pages/OAuthCallback'
import { PopupCloseHandler } from '@/components/PopupCloseHandler'
import { CliSetupDialog } from '@/components/CliSetupDialog'

function App() {
  const [showCliDialog, setShowCliDialog] = useState(false)

  useEffect(() => {
    // Check if we should show the CLI setup dialog (macOS only)
    const checkCliSetup = async () => {
      try {
        // Check if we're on macOS - using user agent as a simple check
        // In a real Tauri app, platform() from @tauri-apps/api would be more reliable
        const isMac = navigator.platform.toLowerCase().includes('mac')

        if (!isMac) {
          return // Only show on macOS
        }

        // Check user preference
        const preference = await invoke<string>('get_cli_prompt_preference')

        // Check if CLI is already installed
        const isInstalled = await invoke<boolean>('check_cli_installed')

        // Show dialog if: preference is not 'never' AND CLI is not installed
        if (preference !== 'never' && !isInstalled) {
          setShowCliDialog(true)
        }
      } catch (error) {
        console.error('Failed to check CLI setup:', error)
      }
    }

    checkCliSetup()
  }, [])

  return (
    <ThemeProvider>
      <QueryClientProvider client={queryClient}>
        <ConnectionProvider>
          <CloudProvider>
            <BrowserRouter>
              <PopupCloseHandler />
              <CliSetupDialog
                open={showCliDialog}
                onOpenChange={setShowCliDialog}
              />
              <Routes>
                {/* OAuth callback route - outside Layout */}
                <Route path="/oauth/callback" element={<OAuthCallback />} />

                {/* Main app routes - inside Layout */}
                <Route path="/*" element={
                  <Layout>
                    <Routes>
                      <Route path="/" element={<Navigate to="/projects" replace />} />
                      <Route path="/projects" element={<Projects />} />
                      <Route path="/projects/:id" element={<ProjectDetail />} />
                      <Route path="/settings" element={<Settings />} />
                    </Routes>
                  </Layout>
                } />
              </Routes>
            </BrowserRouter>
          </CloudProvider>
        </ConnectionProvider>
        {/* Only show devtools in development */}
        {import.meta.env.DEV && <ReactQueryDevtools initialIsOpen={false} />}
      </QueryClientProvider>
    </ThemeProvider>
  )
}

export default App