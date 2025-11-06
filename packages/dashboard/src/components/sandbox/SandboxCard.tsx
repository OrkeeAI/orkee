// ABOUTME: Sandbox card component displaying sandbox status and actions
// ABOUTME: Shows sandbox metrics, provider info, and control buttons

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { useToast } from '@/hooks/use-toast'
import {
  startSandbox,
  stopSandbox,
  restartSandbox,
  deleteSandbox,
  type Sandbox,
} from '@/services/sandbox'
import {
  Play,
  Square,
  RotateCcw,
  Trash2,
  Server,
  Cpu,
  HardDrive,
  DollarSign,
  AlertCircle,
  CheckCircle2,
  Clock,
  XCircle,
} from 'lucide-react'
import { useState } from 'react'
import { formatDistanceToNow } from 'date-fns'

interface SandboxCardProps {
  sandbox: Sandbox
  onUpdate: () => void
  onClick?: () => void
}

export function SandboxCard({ sandbox, onUpdate, onClick }: SandboxCardProps) {
  const { toast } = useToast()
  const [loading, setLoading] = useState(false)

  const getStatusIcon = () => {
    switch (sandbox.status) {
      case 'running':
        return <CheckCircle2 className="h-4 w-4 text-green-500" />
      case 'stopped':
        return <Square className="h-4 w-4 text-gray-500" />
      case 'creating':
        return <Clock className="h-4 w-4 text-blue-500 animate-pulse" />
      case 'terminating':
        return <Clock className="h-4 w-4 text-orange-500 animate-pulse" />
      case 'error':
        return <XCircle className="h-4 w-4 text-red-500" />
    }
  }

  const getStatusBadge = () => {
    const variants: Record<string, 'default' | 'secondary' | 'destructive' | 'outline'> = {
      running: 'default',
      stopped: 'secondary',
      creating: 'outline',
      terminating: 'outline',
      error: 'destructive',
    }

    return (
      <Badge variant={variants[sandbox.status] || 'outline'}>
        {sandbox.status.charAt(0).toUpperCase() + sandbox.status.slice(1)}
      </Badge>
    )
  }

  const handleStart = async (e: React.MouseEvent) => {
    e.stopPropagation()
    setLoading(true)
    try {
      await startSandbox(sandbox.id)
      toast({
        title: 'Sandbox started',
        description: `${sandbox.name} is now running`,
      })
      onUpdate()
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Failed to start sandbox'
      toast({
        title: 'Failed to start sandbox',
        description: errorMessage,
        variant: 'destructive',
      })
    } finally {
      setLoading(false)
    }
  }

  const handleStop = async (e: React.MouseEvent) => {
    e.stopPropagation()
    setLoading(true)
    try {
      await stopSandbox(sandbox.id)
      toast({
        title: 'Sandbox stopped',
        description: `${sandbox.name} has been stopped`,
      })
      onUpdate()
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Failed to stop sandbox'
      toast({
        title: 'Failed to stop sandbox',
        description: errorMessage,
        variant: 'destructive',
      })
    } finally {
      setLoading(false)
    }
  }

  const handleRestart = async (e: React.MouseEvent) => {
    e.stopPropagation()
    setLoading(true)
    try {
      await restartSandbox(sandbox.id)
      toast({
        title: 'Sandbox restarted',
        description: `${sandbox.name} is restarting`,
      })
      onUpdate()
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Failed to restart sandbox'
      toast({
        title: 'Failed to restart sandbox',
        description: errorMessage,
        variant: 'destructive',
      })
    } finally {
      setLoading(false)
    }
  }

  const handleDelete = async (e: React.MouseEvent) => {
    e.stopPropagation()
    if (!confirm(`Are you sure you want to delete ${sandbox.name}? This action cannot be undone.`)) {
      return
    }

    setLoading(true)
    try {
      await deleteSandbox(sandbox.id)
      toast({
        title: 'Sandbox deleted',
        description: `${sandbox.name} has been deleted`,
      })
      onUpdate()
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Failed to delete sandbox'
      toast({
        title: 'Failed to delete sandbox',
        description: errorMessage,
        variant: 'destructive',
      })
    } finally {
      setLoading(false)
    }
  }

  const formatCost = (cost: number | undefined | null): string => {
    if (cost === undefined || cost === null) return '$0.00'
    return `$${cost.toFixed(2)}`
  }

  return (
    <Card
      className="hover:shadow-lg transition-shadow cursor-pointer"
      onClick={onClick}
    >
      <CardHeader className="pb-3">
        <div className="flex items-start justify-between">
          <div className="flex-1">
            <CardTitle className="flex items-center gap-2">
              {getStatusIcon()}
              {sandbox.name}
            </CardTitle>
            <CardDescription className="flex items-center gap-2 mt-1">
              <Server className="h-3 w-3" />
              {sandbox.provider.charAt(0).toUpperCase() + sandbox.provider.slice(1)}
              {sandbox.agent_id && (
                <>
                  <span className="text-muted-foreground">•</span>
                  <span>{sandbox.agent_id}</span>
                </>
              )}
            </CardDescription>
          </div>
          <div className="flex gap-1">
            {getStatusBadge()}
          </div>
        </div>
      </CardHeader>

      <CardContent className="space-y-4">
        {/* Error Message */}
        {sandbox.error_message && (
          <div className="flex items-start gap-2 p-3 bg-destructive/10 border border-destructive/20 rounded-md">
            <AlertCircle className="h-4 w-4 text-destructive flex-shrink-0 mt-0.5" />
            <p className="text-sm text-destructive">{sandbox.error_message}</p>
          </div>
        )}

        {/* Resource Info */}
        <div className="grid grid-cols-2 gap-3 text-sm">
          <div className="flex items-center gap-2">
            <Cpu className="h-4 w-4 text-muted-foreground" />
            <span>{sandbox.cpu_cores} cores</span>
          </div>
          <div className="flex items-center gap-2">
            <HardDrive className="h-4 w-4 text-muted-foreground" />
            <span>{sandbox.memory_mb} MB</span>
          </div>
          <div className="flex items-center gap-2">
            <DollarSign className="h-4 w-4 text-muted-foreground" />
            <span>{formatCost(sandbox.total_cost)}</span>
          </div>
          <div className="flex items-center gap-2">
            <Clock className="h-4 w-4 text-muted-foreground" />
            <span>
              {sandbox.started_at
                ? formatDistanceToNow(new Date(sandbox.started_at), { addSuffix: true })
                : formatDistanceToNow(new Date(sandbox.created_at), { addSuffix: true })}
            </span>
          </div>
        </div>

        {/* Actions */}
        <div className="flex gap-2 pt-2 border-t">
          {sandbox.status === 'stopped' && (
            <Button
              variant="outline"
              size="sm"
              onClick={handleStart}
              disabled={loading}
            >
              <Play className="h-4 w-4 mr-1" />
              Start
            </Button>
          )}
          {sandbox.status === 'running' && (
            <>
              <Button
                variant="outline"
                size="sm"
                onClick={handleStop}
                disabled={loading}
              >
                <Square className="h-4 w-4 mr-1" />
                Stop
              </Button>
              <Button
                variant="outline"
                size="sm"
                onClick={handleRestart}
                disabled={loading}
              >
                <RotateCcw className="h-4 w-4 mr-1" />
                Restart
              </Button>
            </>
          )}
          <Button
            variant="outline"
            size="sm"
            onClick={handleDelete}
            disabled={loading || sandbox.status === 'terminating'}
            className="ml-auto text-destructive hover:text-destructive"
          >
            <Trash2 className="h-4 w-4 mr-1" />
            Delete
          </Button>
        </div>
      </CardContent>
    </Card>
  )
}
