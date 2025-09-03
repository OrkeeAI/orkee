import { Button } from '@/components/ui/button'
import { Server, Plus, Power, Settings as SettingsIcon } from 'lucide-react'

interface MCPServer {
  id: number
  name: string
  url: string
  status: 'online' | 'offline' | 'error'
  version: string
  description: string
  lastSeen: Date
}

export function MCPServers() {
  const servers: MCPServer[] = [
    {
      id: 1,
      name: "OpenAI API Server",
      url: "https://api.openai.com/v1",
      status: "online",
      version: "v1.2.3",
      description: "Primary OpenAI API endpoint for GPT models",
      lastSeen: new Date()
    },
    {
      id: 2,
      name: "Local Llama Server",
      url: "http://localhost:8000",
      status: "offline", 
      version: "v2.0.1",
      description: "Local Llama model server for privacy-focused tasks",
      lastSeen: new Date(Date.now() - 300000) // 5 minutes ago
    },
    {
      id: 3,
      name: "Claude API Server", 
      url: "https://api.anthropic.com",
      status: "online",
      version: "v1.0.0",
      description: "Anthropic Claude API for conversational AI",
      lastSeen: new Date()
    },
    {
      id: 4,
      name: "Custom Model Server",
      url: "https://custom-models.company.com", 
      status: "error",
      version: "v0.9.2",
      description: "Company internal model server",
      lastSeen: new Date(Date.now() - 900000) // 15 minutes ago
    }
  ]

  const getStatusColor = (status: MCPServer['status']) => {
    switch (status) {
      case 'online': return 'bg-green-500'
      case 'offline': return 'bg-gray-500'
      case 'error': return 'bg-red-500'
      default: return 'bg-gray-500'
    }
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">MCP Servers</h1>
          <p className="text-muted-foreground">
            Manage Model Context Protocol servers and their connections.
          </p>
        </div>
        <Button>
          <Plus className="mr-2 h-4 w-4" />
          Add Server
        </Button>
      </div>

      <div className="grid gap-4">
        {servers.map((server) => (
          <div key={server.id} className="rounded-lg border p-6">
            <div className="flex items-center justify-between mb-4">
              <div className="flex items-center gap-3">
                <div className="flex items-center gap-2">
                  <Server className="h-5 w-5 text-primary" />
                  <h3 className="font-semibold">{server.name}</h3>
                </div>
                <div className="flex items-center gap-2">
                  <div className={`w-2 h-2 rounded-full ${getStatusColor(server.status)}`} />
                  <span className="text-sm capitalize font-medium">{server.status}</span>
                </div>
              </div>
              <div className="flex items-center gap-2">
                <Button variant="ghost" size="icon">
                  <Power className="h-4 w-4" />
                </Button>
                <Button variant="ghost" size="icon">
                  <SettingsIcon className="h-4 w-4" />
                </Button>
              </div>
            </div>
            
            <p className="text-sm text-muted-foreground mb-3">{server.description}</p>
            
            <div className="flex items-center justify-between text-sm">
              <div className="flex items-center gap-4">
                <span className="font-mono bg-muted px-2 py-1 rounded">{server.url}</span>
                <span className="text-muted-foreground">v{server.version}</span>
              </div>
              <span className="text-muted-foreground">
                Last seen: {server.lastSeen.toLocaleTimeString()}
              </span>
            </div>
          </div>
        ))}
      </div>
    </div>
  )
}