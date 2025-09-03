import { BrowserRouter, Routes, Route } from 'react-router-dom'
import { Layout } from '@/components/layout/Layout'
import { Usage } from '@/pages/Usage'
import { Projects } from '@/pages/Projects'
import { AIChat } from '@/pages/AIChat'
import { MCPServers } from '@/pages/MCPServers'
import { Monitoring } from '@/pages/Monitoring'
import { Settings } from '@/pages/Settings'

function App() {
  return (
    <BrowserRouter>
      <Layout>
        <Routes>
          <Route path="/" element={<Usage />} />
          <Route path="/projects" element={<Projects />} />
          <Route path="/ai-chat" element={<AIChat />} />
          <Route path="/mcp-servers" element={<MCPServers />} />
          <Route path="/monitoring" element={<Monitoring />} />
          <Route path="/settings" element={<Settings />} />
        </Routes>
      </Layout>
    </BrowserRouter>
  )
}

export default App