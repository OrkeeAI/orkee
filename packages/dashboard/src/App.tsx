import { BrowserRouter, Routes, Route } from 'react-router-dom'
import { QueryClientProvider } from '@tanstack/react-query'
import { ReactQueryDevtools } from '@tanstack/react-query-devtools'
import { Layout } from '@/components/layout/Layout'
import { ConnectionProvider } from '@/contexts/ConnectionContext'
import { queryClient } from '@/lib/queryClient'
import { Usage } from '@/pages/Usage'
import { Projects } from '@/pages/Projects'
import { ProjectDetail } from '@/pages/ProjectDetail'
import { AIChat } from '@/pages/AIChat'
import { MCPServers } from '@/pages/MCPServers'
import { Monitoring } from '@/pages/Monitoring'
import { Settings } from '@/pages/Settings'

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <ConnectionProvider>
        <BrowserRouter>
          <Layout>
            <Routes>
              <Route path="/" element={<Usage />} />
              <Route path="/projects" element={<Projects />} />
              <Route path="/projects/:id" element={<ProjectDetail />} />
              <Route path="/ai-chat" element={<AIChat />} />
              <Route path="/mcp-servers" element={<MCPServers />} />
              <Route path="/monitoring" element={<Monitoring />} />
              <Route path="/settings" element={<Settings />} />
            </Routes>
          </Layout>
        </BrowserRouter>
      </ConnectionProvider>
      {/* Only show devtools in development */}
      {import.meta.env.DEV && <ReactQueryDevtools initialIsOpen={false} />}
    </QueryClientProvider>
  )
}

export default App