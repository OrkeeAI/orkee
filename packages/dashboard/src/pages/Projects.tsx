import { Button } from '@/components/ui/button'
import { FolderOpen, Plus } from 'lucide-react'

export function Projects() {
  const projects = [
    {
      id: 1,
      name: "Customer Support Bot",
      description: "Automated customer service with sentiment analysis",
      status: "active",
      agents: 3
    },
    {
      id: 2, 
      name: "Data Processing Pipeline",
      description: "Extract, transform, and load data workflows", 
      status: "paused",
      agents: 5
    },
    {
      id: 3,
      name: "Content Moderation",
      description: "AI-powered content filtering and safety checks",
      status: "active", 
      agents: 2
    }
  ]

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Projects</h1>
          <p className="text-muted-foreground">
            Manage your AI agent orchestration projects.
          </p>
        </div>
        <Button>
          <Plus className="mr-2 h-4 w-4" />
          New Project
        </Button>
      </div>

      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
        {projects.map((project) => (
          <div key={project.id} className="rounded-lg border p-6 hover:shadow-md transition-shadow">
            <div className="flex items-center gap-2 mb-3">
              <FolderOpen className="h-5 w-5 text-primary" />
              <h3 className="font-semibold">{project.name}</h3>
            </div>
            <p className="text-sm text-muted-foreground mb-4">{project.description}</p>
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <div className={`w-2 h-2 rounded-full ${
                  project.status === 'active' ? 'bg-green-500' : 'bg-yellow-500'
                }`} />
                <span className="text-sm capitalize">{project.status}</span>
              </div>
              <span className="text-sm text-muted-foreground">
                {project.agents} agents
              </span>
            </div>
          </div>
        ))}
      </div>
    </div>
  )
}