import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom'
import { QueryClientProvider } from '@tanstack/react-query'
import { ReactQueryDevtools } from '@tanstack/react-query-devtools'
import { Layout } from '@/components/layout/Layout'
import { ConnectionProvider } from '@/contexts/ConnectionContext'
import { CloudProvider } from '@/contexts/CloudContext'
import { queryClient } from '@/lib/queryClient'
import { Projects } from '@/pages/Projects'
import { ProjectDetail } from '@/pages/ProjectDetail'
import { Settings } from '@/pages/Settings'
import OAuthCallback from '@/pages/OAuthCallback'
import { PopupCloseHandler } from '@/components/PopupCloseHandler'

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <ConnectionProvider>
        <CloudProvider>
          <BrowserRouter>
            <PopupCloseHandler />
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
  )
}

export default App