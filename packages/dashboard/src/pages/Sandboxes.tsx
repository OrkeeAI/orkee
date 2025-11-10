// ABOUTME: Main sandboxes page for managing sandbox instances
// ABOUTME: Provides sandbox list, creation, detail view with terminal/files/monitoring

import { useState, useEffect, useCallback } from 'react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { ScrollArea } from '@/components/ui/scroll-area'
import { useToast } from '@/hooks/use-toast'
import {
  listSandboxes,
  createSandbox,
  getSandbox,
  getSandboxSettings,
  getAllProviderSettings,
  type Sandbox,
  type CreateSandboxRequest,
  type SandboxSettings as SandboxSettingsType,
  type ProviderSettings,
} from '@/services/sandbox'
import { SandboxCard } from '@/components/sandbox/SandboxCard'
import { Terminal } from '@/components/sandbox/Terminal'
import { FileBrowser } from '@/components/sandbox/FileBrowser'
import { ResourceMonitor } from '@/components/sandbox/ResourceMonitor'
import { CostTracking } from '@/components/sandbox/CostTracking'
import { AgentModelSelector } from '@/components/sandbox/AgentModelSelector'
import { TemplateManagement } from '@/components/sandbox/TemplateManagement'
import { SandboxImageManager } from '@/components/sandbox/SandboxImageManager'
import {
  Plus,
  RefreshCw,
  ChevronLeft,
  Server,
  Terminal as TerminalIcon,
  FolderOpen,
  BarChart3,
  Package,
} from 'lucide-react'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Badge } from '@/components/ui/badge'

export default function Sandboxes() {
  const { toast } = useToast()
  const [sandboxes, setSandboxes] = useState<Sandbox[]>([])
  const [selectedSandbox, setSelectedSandbox] = useState<Sandbox | null>(null)
  const [loading, setLoading] = useState(true)
  const [createDialogOpen, setCreateDialogOpen] = useState(false)
  const [settings, setSettings] = useState<SandboxSettingsType | null>(null)
  const [providers, setProviders] = useState<ProviderSettings[]>([])

  // Create sandbox form state
  const [newSandbox, setNewSandbox] = useState<CreateSandboxRequest>({
    name: '',
    provider: undefined,
    image: undefined,
    cpu_cores: undefined,
    memory_mb: undefined,
    agent_id: null,
    model: null,
  })

  const loadData = useCallback(async () => {
    setLoading(true)
    try {
      const results = await Promise.allSettled([
        listSandboxes(),
        getSandboxSettings(),
        getAllProviderSettings(),
      ])

      // Handle sandboxes result
      if (results[0].status === 'fulfilled') {
        setSandboxes(results[0].value)
      } else {
        console.error('Failed to load sandboxes:', results[0].reason)
      }

      // Handle settings result
      if (results[1].status === 'fulfilled') {
        setSettings(results[1].value)
      } else {
        console.error('Failed to load settings:', results[1].reason)
      }

      // Handle providers result
      if (results[2].status === 'fulfilled') {
        setProviders(results[2].value.filter((p) => p.enabled))
      } else {
        console.error('Failed to load providers:', results[2].reason)
        toast({
          title: 'Failed to load providers',
          description: results[2].reason instanceof Error ? results[2].reason.message : 'Unknown error',
          variant: 'destructive',
        })
      }
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Failed to load data'
      toast({
        title: 'Failed to load data',
        description: errorMessage,
        variant: 'destructive',
      })
    } finally {
      setLoading(false)
    }
  }, [toast])

  useEffect(() => {
    loadData()
  }, [loadData])

  const handleCreateSandbox = async () => {
    if (!newSandbox.name.trim()) {
      toast({
        title: 'Name required',
        description: 'Please enter a name for the sandbox',
        variant: 'destructive',
      })
      return
    }

    try {
      // Use default image from settings if not specified
      const sandboxToCreate = {
        ...newSandbox,
        image: newSandbox.image || settings?.default_image,
      }
      await createSandbox(sandboxToCreate)
      toast({
        title: 'Sandbox created',
        description: `${newSandbox.name} has been created successfully`,
      })
      setCreateDialogOpen(false)
      setNewSandbox({
        name: '',
        provider: undefined,
        image: undefined,
        cpu_cores: undefined,
        memory_mb: undefined,
        agent_id: null,
        model: null,
      })
      await loadData()
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Failed to create sandbox'
      toast({
        title: 'Failed to create sandbox',
        description: errorMessage,
        variant: 'destructive',
      })
    }
  }

  const handleSandboxClick = async (sandbox: Sandbox) => {
    try {
      const fullSandbox = await getSandbox(sandbox.id)
      setSelectedSandbox(fullSandbox)
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Failed to load sandbox details'
      toast({
        title: 'Failed to load sandbox',
        description: errorMessage,
        variant: 'destructive',
      })
    }
  }

  const handleCloseSandboxDetail = () => {
    setSelectedSandbox(null)
    loadData() // Refresh list
  }

  const runningSandboxes = sandboxes.filter((s) => s.status === 'running')
  const stoppedSandboxes = sandboxes.filter((s) => s.status === 'stopped')
  const errorSandboxes = sandboxes.filter((s) => s.status === 'error')

  if (selectedSandbox) {
    return (
      <div className="h-full flex flex-col">
        {/* Header */}
        <div className="border-b p-4 bg-card">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <Button variant="ghost" size="sm" onClick={handleCloseSandboxDetail}>
                <ChevronLeft className="h-4 w-4" />
              </Button>
              <div>
                <h1 className="text-2xl font-bold">{selectedSandbox.name}</h1>
                <p className="text-sm text-muted-foreground flex items-center gap-2 mt-1">
                  <Server className="h-3 w-3" />
                  {selectedSandbox.provider.charAt(0).toUpperCase() + selectedSandbox.provider.slice(1)}
                  <Badge variant="outline">{selectedSandbox.status}</Badge>
                </p>
              </div>
            </div>
          </div>
        </div>

        {/* Tabs */}
        <div className="flex-1 overflow-hidden">
          <Tabs defaultValue="terminal" className="h-full flex flex-col">
            <TabsList className="mx-4 mt-4">
              <TabsTrigger value="terminal" className="flex items-center gap-2">
                <TerminalIcon className="h-4 w-4" />
                Terminal
              </TabsTrigger>
              <TabsTrigger value="files" className="flex items-center gap-2">
                <FolderOpen className="h-4 w-4" />
                Files
              </TabsTrigger>
              <TabsTrigger value="monitoring" className="flex items-center gap-2">
                <BarChart3 className="h-4 w-4" />
                Monitoring
              </TabsTrigger>
            </TabsList>

            <div className="flex-1 overflow-hidden p-4">
              <TabsContent value="terminal" className="h-full mt-0">
                <Terminal
                  sandboxId={selectedSandbox.id}
                  agentId={selectedSandbox.agent_id}
                  model={selectedSandbox.model}
                />
              </TabsContent>

              <TabsContent value="files" className="h-full mt-0">
                <FileBrowser sandboxId={selectedSandbox.id} />
              </TabsContent>

              <TabsContent value="monitoring" className="h-full mt-0 overflow-auto">
                <ResourceMonitor sandboxId={selectedSandbox.id} />
              </TabsContent>
            </div>
          </Tabs>
        </div>
      </div>
    )
  }

  return (
    <div className="h-full flex flex-col p-4">
      {/* Header */}
      <div className="mb-4">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-3xl font-bold tracking-tight">Sandboxes</h1>
            <p className="text-muted-foreground">
              Manage isolated execution environments for AI agents
            </p>
          </div>
          <div className="flex gap-2">
            <Button variant="outline" onClick={loadData} disabled={loading}>
              <RefreshCw className={`h-4 w-4 mr-2 ${loading ? 'animate-spin' : ''}`} />
              Refresh
            </Button>
            <Button onClick={() => setCreateDialogOpen(true)}>
              <Plus className="h-4 w-4 mr-2" />
              Create Sandbox
            </Button>
          </div>
        </div>
      </div>

      {/* Tabs */}
      <Tabs defaultValue="sandboxes" className="flex-1 flex flex-col min-h-0">
        <TabsList>
          <TabsTrigger value="sandboxes" className="flex items-center gap-2">
            <Server className="h-4 w-4" />
            Sandboxes
          </TabsTrigger>
          <TabsTrigger value="images" className="flex items-center gap-2">
            <Package className="h-4 w-4" />
            Images
          </TabsTrigger>
        </TabsList>

        <TabsContent value="sandboxes" className="flex-1 mt-4 min-h-0">
          <div className="h-full flex flex-col md:flex-row gap-4">
            {/* Main Content */}
            <div className="flex-1 flex flex-col min-w-0">
              {/* Stats */}
              <div className="grid grid-cols-1 md:grid-cols-4 gap-4 mb-4">
                <div className="p-4 border rounded-lg">
                  <div className="text-2xl font-bold">{sandboxes.length}</div>
                  <div className="text-sm text-muted-foreground">Total Sandboxes</div>
                </div>
                <div className="p-4 border rounded-lg">
                  <div className="text-2xl font-bold text-green-500">{runningSandboxes.length}</div>
                  <div className="text-sm text-muted-foreground">Running</div>
                </div>
                <div className="p-4 border rounded-lg">
                  <div className="text-2xl font-bold text-gray-500">{stoppedSandboxes.length}</div>
                  <div className="text-sm text-muted-foreground">Stopped</div>
                </div>
                <div className="p-4 border rounded-lg">
                  <div className="text-2xl font-bold text-red-500">{errorSandboxes.length}</div>
                  <div className="text-sm text-muted-foreground">Errors</div>
                </div>
              </div>

              {/* Sandbox List */}
              <ScrollArea className="flex-1">
                {loading ? (
                  <div className="flex items-center justify-center py-12 text-muted-foreground">
                    <RefreshCw className="h-5 w-5 animate-spin mr-2" />
                    Loading sandboxes...
                  </div>
                ) : sandboxes.length === 0 ? (
                  <div className="flex flex-col items-center justify-center py-12 text-center">
                    <Server className="h-12 w-12 text-muted-foreground mb-4" />
                    <h3 className="text-lg font-semibold mb-2">No sandboxes yet</h3>
                    <p className="text-sm text-muted-foreground mb-4">
                      Create your first sandbox to get started
                    </p>
                    <Button onClick={() => setCreateDialogOpen(true)}>
                      <Plus className="h-4 w-4 mr-2" />
                      Create Sandbox
                    </Button>
                  </div>
                ) : (
                  <div className="grid grid-cols-1 lg:grid-cols-2 gap-4 pb-4">
                    {sandboxes.map((sandbox) => (
                      <SandboxCard
                        key={sandbox.id}
                        sandbox={sandbox}
                        onUpdate={loadData}
                        onClick={() => handleSandboxClick(sandbox)}
                      />
                    ))}
                  </div>
                )}
              </ScrollArea>
            </div>

            {/* Sidebar */}
            <div className="w-full md:w-80 flex-shrink-0 space-y-4">
              {settings && (
                <CostTracking
                  sandboxes={sandboxes}
                  maxTotalCost={settings.max_total_cost}
                  costAlertThreshold={settings.cost_alert_threshold}
                />
              )}
              <TemplateManagement />
            </div>
          </div>
        </TabsContent>

        <TabsContent value="images" className="flex-1 mt-4 min-h-0">
          <SandboxImageManager />
        </TabsContent>
      </Tabs>

      {/* Create Sandbox Dialog */}
      <Dialog open={createDialogOpen} onOpenChange={setCreateDialogOpen}>
        <DialogContent className="max-w-2xl">
          <DialogHeader>
            <DialogTitle>Create New Sandbox</DialogTitle>
            <DialogDescription>
              Configure a new isolated execution environment
            </DialogDescription>
          </DialogHeader>

          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label htmlFor="name">Name *</Label>
              <Input
                id="name"
                value={newSandbox.name}
                onChange={(e) => setNewSandbox({ ...newSandbox, name: e.target.value })}
                placeholder="my-sandbox"
              />
            </div>

            <div className="space-y-2">
              <Label htmlFor="provider">Provider</Label>
              <Select
                value={newSandbox.provider || settings?.default_provider || 'local'}
                onValueChange={(value) => setNewSandbox({ ...newSandbox, provider: value })}
              >
                <SelectTrigger id="provider">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {providers.map((p) => (
                    <SelectItem key={p.provider} value={p.provider}>
                      {p.provider.charAt(0).toUpperCase() + p.provider.slice(1)}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <Label htmlFor="cpu">CPU Cores</Label>
                <Input
                  id="cpu"
                  type="number"
                  value={newSandbox.cpu_cores || ''}
                  onChange={(e) =>
                    setNewSandbox({ ...newSandbox, cpu_cores: parseInt(e.target.value) || undefined })
                  }
                  placeholder="2"
                />
              </div>

              <div className="space-y-2">
                <Label htmlFor="memory">Memory (MB)</Label>
                <Input
                  id="memory"
                  type="number"
                  value={newSandbox.memory_mb || ''}
                  onChange={(e) =>
                    setNewSandbox({ ...newSandbox, memory_mb: parseInt(e.target.value) || undefined })
                  }
                  placeholder="4096"
                />
              </div>
            </div>

            <AgentModelSelector
              agentId={newSandbox.agent_id}
              model={newSandbox.model}
              onAgentChange={(agentId) => setNewSandbox({ ...newSandbox, agent_id: agentId })}
              onModelChange={(model) => setNewSandbox({ ...newSandbox, model })}
            />
          </div>

          <DialogFooter>
            <Button variant="outline" onClick={() => setCreateDialogOpen(false)}>
              Cancel
            </Button>
            <Button onClick={handleCreateSandbox}>
              <Plus className="h-4 w-4 mr-2" />
              Create Sandbox
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
