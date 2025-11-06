// ABOUTME: Project-specific sandbox monitoring component
// ABOUTME: Shows sandboxes filtered by project ID with real-time status

import { useEffect, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { useToast } from '@/hooks/use-toast'
import { listSandboxes, type Sandbox } from '@/services/sandbox'
import { Plus, Server, Activity } from 'lucide-react'

interface ProjectSandboxesProps {
  projectId: string
  projectName: string
}

export function ProjectSandboxes({ projectId, projectName }: ProjectSandboxesProps) {
  const { toast } = useToast()
  const navigate = useNavigate()
  const [sandboxes, setSandboxes] = useState<Sandbox[]>([])
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    loadSandboxes()
  }, [projectId])

  const loadSandboxes = async () => {
    try {
      setLoading(true)
      const allSandboxes = await listSandboxes()

      // Filter sandboxes by project_id
      const projectSandboxes = allSandboxes.filter(
        (sandbox) => sandbox.project_id === projectId
      )

      setSandboxes(projectSandboxes)
    } catch (error) {
      console.error('Failed to load sandboxes:', error)
      toast({
        title: 'Failed to load sandboxes',
        description: error instanceof Error ? error.message : 'Unknown error',
        variant: 'destructive',
      })
    } finally {
      setLoading(false)
    }
  }

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'running':
        return 'bg-green-500'
      case 'stopped':
        return 'bg-gray-500'
      case 'creating':
        return 'bg-blue-500'
      case 'error':
        return 'bg-red-500'
      default:
        return 'bg-gray-400'
    }
  }

  const getStatusBadgeVariant = (status: string) => {
    switch (status) {
      case 'running':
        return 'default'
      case 'stopped':
        return 'secondary'
      case 'creating':
        return 'outline'
      case 'error':
        return 'destructive'
      default:
        return 'secondary'
    }
  }

  if (loading) {
    return (
      <div className="flex items-center justify-center py-8">
        <div className="text-center">
          <div className="h-8 w-8 animate-spin rounded-full border-2 border-primary border-t-transparent mx-auto mb-4" />
          <p className="text-muted-foreground">Loading sandboxes...</p>
        </div>
      </div>
    )
  }

  return (
    <div className="space-y-4">
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle>Sandboxes</CardTitle>
              <CardDescription>
                Development environments for {projectName}
              </CardDescription>
            </div>
            <Button onClick={() => navigate('/sandboxes')}>
              <Plus className="h-4 w-4 mr-2" />
              New Sandbox
            </Button>
          </div>
        </CardHeader>
        <CardContent>
          {sandboxes.length === 0 ? (
            <div className="text-center py-8 text-sm text-muted-foreground">
              <Server className="h-8 w-8 mx-auto mb-2 opacity-50" />
              <p>No sandboxes found for this project</p>
              <p className="text-xs mt-1">Create a sandbox to get started</p>
              <Button
                variant="outline"
                className="mt-4"
                onClick={() => navigate('/sandboxes')}
              >
                <Plus className="h-4 w-4 mr-2" />
                Create Sandbox
              </Button>
            </div>
          ) : (
            <div className="space-y-4">
              {sandboxes.map((sandbox) => (
                <Card key={sandbox.id} className="cursor-pointer hover:bg-accent/50 transition-colors">
                  <CardContent className="p-4">
                    <div className="flex items-start justify-between">
                      <div className="flex items-start gap-3 flex-1">
                        <div className={`h-2 w-2 rounded-full mt-2 ${getStatusColor(sandbox.status)}`} />
                        <div className="flex-1 min-w-0">
                          <div className="flex items-center gap-2 mb-1">
                            <h3 className="font-semibold truncate">{sandbox.name}</h3>
                            <Badge variant={getStatusBadgeVariant(sandbox.status)}>
                              {sandbox.status}
                            </Badge>
                          </div>
                          {sandbox.description && (
                            <p className="text-sm text-muted-foreground mb-2">
                              {sandbox.description}
                            </p>
                          )}
                          <div className="flex items-center gap-4 text-xs text-muted-foreground">
                            <div className="flex items-center gap-1">
                              <Server className="h-3 w-3" />
                              <span>{sandbox.provider}</span>
                            </div>
                            {sandbox.cpu_cores && (
                              <div className="flex items-center gap-1">
                                <Activity className="h-3 w-3" />
                                <span>{sandbox.cpu_cores} CPU</span>
                              </div>
                            )}
                            {sandbox.memory_mb && (
                              <span>{(sandbox.memory_mb / 1024).toFixed(1)}GB RAM</span>
                            )}
                          </div>
                        </div>
                      </div>
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => navigate(`/sandboxes?sandbox=${sandbox.id}`)}
                      >
                        View Details
                      </Button>
                    </div>
                  </CardContent>
                </Card>
              ))}
            </div>
          )}
        </CardContent>
      </Card>

      {sandboxes.length > 0 && (
        <Card>
          <CardHeader>
            <CardTitle className="text-sm font-medium">Quick Stats</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
              <div className="space-y-1">
                <p className="text-xs text-muted-foreground">Total</p>
                <p className="text-2xl font-bold">{sandboxes.length}</p>
              </div>
              <div className="space-y-1">
                <p className="text-xs text-muted-foreground">Running</p>
                <p className="text-2xl font-bold text-green-600">
                  {sandboxes.filter((s) => s.status === 'running').length}
                </p>
              </div>
              <div className="space-y-1">
                <p className="text-xs text-muted-foreground">Stopped</p>
                <p className="text-2xl font-bold text-gray-600">
                  {sandboxes.filter((s) => s.status === 'stopped').length}
                </p>
              </div>
              <div className="space-y-1">
                <p className="text-xs text-muted-foreground">Errors</p>
                <p className="text-2xl font-bold text-red-600">
                  {sandboxes.filter((s) => s.status === 'error').length}
                </p>
              </div>
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  )
}
