// ABOUTME: Sandbox configuration settings component with provider management
// ABOUTME: Handles sandbox settings, provider credentials, resource limits, lifecycle, costs, and security

import { useState, useEffect } from 'react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Switch } from '@/components/ui/switch'
import { Button } from '@/components/ui/button'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { Badge } from '@/components/ui/badge'
import { useToast } from '@/hooks/use-toast'
import {
  AlertTriangle,
  Check,
  RefreshCw,
  X,
  Server,
  Cloud,
  Cpu,
  HardDrive,
  Clock,
  DollarSign,
  Shield,
  Settings,
} from 'lucide-react'
import {
  getSandboxSettings,
  updateSandboxSettings,
  getAllProviderSettings,
  toggleProvider,
  updateProviderCredentials,
  validateProvider,
  type SandboxSettings as SandboxSettingsType,
  type ProviderSettings,
  type ProviderCredentials,
} from '@/services/sandbox'

export function SandboxSettings() {
  const { toast } = useToast()
  const [settings, setSettings] = useState<SandboxSettingsType | null>(null)
  const [providers, setProviders] = useState<ProviderSettings[]>([])
  const [loading, setLoading] = useState(true)
  const [saving, setSaving] = useState(false)
  const [selectedProvider, setSelectedProvider] = useState<string | null>(null)
  const [showProviderConfig, setShowProviderConfig] = useState(false)

  useEffect(() => {
    fetchSettings()
  }, [])

  const fetchSettings = async () => {
    try {
      const [settingsData, providersData] = await Promise.all([
        getSandboxSettings(),
        getAllProviderSettings(),
      ])
      setSettings(settingsData)
      setProviders(providersData)
    } catch (error) {
      toast({
        title: 'Error',
        description: 'Failed to load sandbox settings',
        variant: 'destructive',
      })
      console.error('Failed to load sandbox settings:', error)
    } finally {
      setLoading(false)
    }
  }

  const saveSettings = async () => {
    if (!settings) return

    setSaving(true)
    try {
      const updated = await updateSandboxSettings(settings)
      setSettings(updated)
      toast({
        title: 'Success',
        description: 'Settings saved successfully',
      })
    } catch (error) {
      toast({
        title: 'Error',
        description: 'Failed to save settings',
        variant: 'destructive',
      })
      console.error('Failed to save settings:', error)
    } finally {
      setSaving(false)
    }
  }

  const configureProvider = (provider: string) => {
    setSelectedProvider(provider)
    setShowProviderConfig(true)
  }

  const handleToggleProvider = async (provider: string, enabled: boolean) => {
    try {
      const updated = await toggleProvider(provider, enabled)
      setProviders(providers.map(p => p.provider === provider ? updated : p))
      toast({
        title: 'Success',
        description: `${provider} ${enabled ? 'enabled' : 'disabled'}`,
      })
    } catch (error) {
      toast({
        title: 'Error',
        description: `Failed to toggle ${provider}`,
        variant: 'destructive',
      })
      console.error('Failed to toggle provider:', error)
    }
  }

  if (loading) {
    return <div className="flex items-center justify-center p-8">
      <RefreshCw className="h-6 w-6 animate-spin text-muted-foreground" />
      <span className="ml-2 text-muted-foreground">Loading sandbox settings...</span>
    </div>
  }

  if (!settings) {
    return <div className="text-center p-8 text-muted-foreground">Failed to load settings</div>
  }

  return (
    <div className="space-y-6">
      <div>
        <h3 className="text-lg font-medium">Sandbox Configuration</h3>
        <p className="text-sm text-muted-foreground">
          Configure sandbox providers, resource limits, and lifecycle settings
        </p>
      </div>

      <Tabs defaultValue="general" className="space-y-4">
        <TabsList className="grid w-full grid-cols-6">
          <TabsTrigger value="general">General</TabsTrigger>
          <TabsTrigger value="providers">Providers</TabsTrigger>
          <TabsTrigger value="resources">Resources</TabsTrigger>
          <TabsTrigger value="lifecycle">Lifecycle</TabsTrigger>
          <TabsTrigger value="costs">Costs</TabsTrigger>
          <TabsTrigger value="security">Security</TabsTrigger>
        </TabsList>

        <TabsContent value="general" className="space-y-4">
          <GeneralTab settings={settings} setSettings={setSettings} providers={providers} />
        </TabsContent>

        <TabsContent value="providers" className="space-y-4">
          <ProvidersTab
            providers={providers}
            onToggle={handleToggleProvider}
            onConfigure={configureProvider}
          />
        </TabsContent>

        <TabsContent value="resources" className="space-y-4">
          <ResourcesTab settings={settings} setSettings={setSettings} />
        </TabsContent>

        <TabsContent value="lifecycle" className="space-y-4">
          <LifecycleTab settings={settings} setSettings={setSettings} />
        </TabsContent>

        <TabsContent value="costs" className="space-y-4">
          <CostsTab settings={settings} setSettings={setSettings} />
        </TabsContent>

        <TabsContent value="security" className="space-y-4">
          <SecurityTab settings={settings} setSettings={setSettings} />
        </TabsContent>
      </Tabs>

      <div className="flex justify-end">
        <Button onClick={saveSettings} disabled={saving}>
          {saving ? (
            <>
              <RefreshCw className="mr-2 h-4 w-4 animate-spin" />
              Saving...
            </>
          ) : (
            'Save Settings'
          )}
        </Button>
      </div>

      {selectedProvider && (
        <ProviderConfigDialog
          provider={selectedProvider}
          open={showProviderConfig}
          onOpenChange={setShowProviderConfig}
          onSave={fetchSettings}
        />
      )}
    </div>
  )
}

function GeneralTab({
  settings,
  setSettings,
  providers,
}: {
  settings: SandboxSettingsType
  setSettings: (settings: SandboxSettingsType) => void
  providers: ProviderSettings[]
}) {
  const enabledProviders = providers.filter(p => p.enabled)

  return (
    <Card>
      <CardHeader>
        <CardTitle>General Settings</CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="flex items-center justify-between">
          <div className="space-y-0.5">
            <Label>Enable Sandboxes</Label>
            <p className="text-sm text-muted-foreground">
              Enable or disable the entire sandbox system
            </p>
          </div>
          <Switch
            checked={settings.enabled}
            onCheckedChange={(checked) =>
              setSettings({ ...settings, enabled: checked })
            }
          />
        </div>

        <div className="space-y-2">
          <Label>Default Provider</Label>
          <select
            className="w-full p-2 border rounded"
            value={settings.default_provider}
            onChange={(e) =>
              setSettings({ ...settings, default_provider: e.target.value })
            }
          >
            <option value="local">Local Docker</option>
            {enabledProviders.map(p => (
              <option key={p.provider} value={p.provider}>
                {p.provider.charAt(0).toUpperCase() + p.provider.slice(1)}
              </option>
            ))}
          </select>
        </div>

        <div className="space-y-2">
          <Label>Default Image</Label>
          <Input
            value={settings.default_image}
            onChange={(e) =>
              setSettings({ ...settings, default_image: e.target.value })
            }
            placeholder="orkee/sandbox:latest"
          />
        </div>
      </CardContent>
    </Card>
  )
}

function ProvidersTab({
  providers,
  onToggle,
  onConfigure,
}: {
  providers: ProviderSettings[]
  onToggle: (provider: string, enabled: boolean) => void
  onConfigure: (provider: string) => void
}) {
  return (
    <Card>
      <CardHeader>
        <CardTitle>Provider Configuration</CardTitle>
        <CardDescription>
          Configure and authenticate with sandbox providers
        </CardDescription>
      </CardHeader>
      <CardContent>
        <div className="space-y-4">
          {['local', 'beam', 'cloudflare', 'daytona', 'e2b', 'flyio', 'modal', 'northflank'].map(providerId => {
            const provider = providers.find(p => p.provider === providerId)
            return (
              <div key={providerId} className="flex items-center justify-between p-4 border rounded">
                <div className="flex items-center space-x-4">
                  <Switch
                    checked={provider?.enabled || false}
                    onCheckedChange={(checked) => onToggle(providerId, checked)}
                  />
                  <div>
                    <p className="font-medium capitalize">{providerId}</p>
                    <div className="flex items-center gap-2">
                      {provider?.configured ? (
                        <Badge variant="secondary" className="text-xs">
                          <Check className="h-3 w-3 mr-1" />
                          Configured
                        </Badge>
                      ) : (
                        <span className="text-sm text-muted-foreground">Not configured</span>
                      )}
                      {provider?.validated_at && (
                        <Badge variant="outline" className="text-xs text-green-600 border-green-600">
                          Validated
                        </Badge>
                      )}
                      {provider?.validation_error && (
                        <Badge variant="outline" className="text-xs text-red-600 border-red-600">
                          <X className="h-3 w-3 mr-1" />
                          Error
                        </Badge>
                      )}
                    </div>
                  </div>
                </div>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => onConfigure(providerId)}
                >
                  Configure
                </Button>
              </div>
            )
          })}
        </div>
      </CardContent>
    </Card>
  )
}

function ResourcesTab({
  settings,
  setSettings,
}: {
  settings: SandboxSettingsType
  setSettings: (settings: SandboxSettingsType) => void
}) {
  return (
    <Card>
      <CardHeader>
        <CardTitle>Resource Limits</CardTitle>
        <CardDescription>
          Set maximum resources for sandboxes
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="grid grid-cols-2 gap-4">
          <div className="space-y-2">
            <Label>Max Local Sandboxes</Label>
            <Input
              type="number"
              value={settings.max_concurrent_local}
              onChange={(e) =>
                setSettings({ ...settings, max_concurrent_local: parseInt(e.target.value) })
              }
            />
          </div>

          <div className="space-y-2">
            <Label>Max Cloud Sandboxes</Label>
            <Input
              type="number"
              value={settings.max_concurrent_cloud}
              onChange={(e) =>
                setSettings({ ...settings, max_concurrent_cloud: parseInt(e.target.value) })
              }
            />
          </div>

          <div className="space-y-2">
            <Label>Max CPU Cores</Label>
            <Input
              type="number"
              value={settings.max_cpu_cores_per_sandbox}
              onChange={(e) =>
                setSettings({ ...settings, max_cpu_cores_per_sandbox: parseInt(e.target.value) })
              }
            />
          </div>

          <div className="space-y-2">
            <Label>Max Memory (GB)</Label>
            <Input
              type="number"
              value={settings.max_memory_gb_per_sandbox}
              onChange={(e) =>
                setSettings({ ...settings, max_memory_gb_per_sandbox: parseInt(e.target.value) })
              }
            />
          </div>

          <div className="space-y-2">
            <Label>Max Disk (GB)</Label>
            <Input
              type="number"
              value={settings.max_disk_gb_per_sandbox}
              onChange={(e) =>
                setSettings({ ...settings, max_disk_gb_per_sandbox: parseInt(e.target.value) })
              }
            />
          </div>

          <div className="space-y-2">
            <Label>Max GPU per Sandbox</Label>
            <Input
              type="number"
              value={settings.max_gpu_per_sandbox}
              onChange={(e) =>
                setSettings({ ...settings, max_gpu_per_sandbox: parseInt(e.target.value) })
              }
            />
          </div>
        </div>
      </CardContent>
    </Card>
  )
}

function LifecycleTab({
  settings,
  setSettings,
}: {
  settings: SandboxSettingsType
  setSettings: (settings: SandboxSettingsType) => void
}) {
  return (
    <Card>
      <CardHeader>
        <CardTitle>Lifecycle Settings</CardTitle>
        <CardDescription>
          Configure sandbox lifecycle and cleanup
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="space-y-2">
          <Label>Auto-stop Idle Time (minutes)</Label>
          <Input
            type="number"
            value={settings.auto_stop_idle_minutes}
            onChange={(e) =>
              setSettings({ ...settings, auto_stop_idle_minutes: parseInt(e.target.value) })
            }
          />
          <p className="text-sm text-muted-foreground">
            Automatically stop sandboxes after this many minutes of inactivity
          </p>
        </div>

        <div className="space-y-2">
          <Label>Max Runtime (hours)</Label>
          <Input
            type="number"
            value={settings.max_runtime_hours}
            onChange={(e) =>
              setSettings({ ...settings, max_runtime_hours: parseInt(e.target.value) })
            }
          />
          <p className="text-sm text-muted-foreground">
            Maximum time a sandbox can run before forced stop
          </p>
        </div>

        <div className="space-y-2">
          <Label>Cleanup Interval (minutes)</Label>
          <Input
            type="number"
            value={settings.cleanup_interval_minutes}
            onChange={(e) =>
              setSettings({ ...settings, cleanup_interval_minutes: parseInt(e.target.value) })
            }
          />
          <p className="text-sm text-muted-foreground">
            How often to check for and clean up terminated sandboxes
          </p>
        </div>

        <div className="flex items-center justify-between">
          <div className="space-y-0.5">
            <Label>Preserve Stopped Sandboxes</Label>
            <p className="text-sm text-muted-foreground">
              Keep sandbox data after stopping (uses more storage)
            </p>
          </div>
          <Switch
            checked={settings.preserve_stopped_sandboxes}
            onCheckedChange={(checked) =>
              setSettings({ ...settings, preserve_stopped_sandboxes: checked })
            }
          />
        </div>

        <div className="flex items-center justify-between">
          <div className="space-y-0.5">
            <Label>Auto-restart Failed Sandboxes</Label>
            <p className="text-sm text-muted-foreground">
              Automatically restart sandboxes that fail
            </p>
          </div>
          <Switch
            checked={settings.auto_restart_failed}
            onCheckedChange={(checked) =>
              setSettings({ ...settings, auto_restart_failed: checked })
            }
          />
        </div>

        <div className="space-y-2">
          <Label>Max Restart Attempts</Label>
          <Input
            type="number"
            value={settings.max_restart_attempts}
            onChange={(e) =>
              setSettings({ ...settings, max_restart_attempts: parseInt(e.target.value) })
            }
          />
        </div>
      </CardContent>
    </Card>
  )
}

function CostsTab({
  settings,
  setSettings,
}: {
  settings: SandboxSettingsType
  setSettings: (settings: SandboxSettingsType) => void
}) {
  return (
    <Card>
      <CardHeader>
        <CardTitle>Cost Management</CardTitle>
        <CardDescription>
          Configure cost tracking and limits
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="flex items-center justify-between">
          <div className="space-y-0.5">
            <Label>Enable Cost Tracking</Label>
            <p className="text-sm text-muted-foreground">
              Track and display costs for cloud sandboxes
            </p>
          </div>
          <Switch
            checked={settings.cost_tracking_enabled}
            onCheckedChange={(checked) =>
              setSettings({ ...settings, cost_tracking_enabled: checked })
            }
          />
        </div>

        <div className="space-y-2">
          <Label>Alert Threshold ($)</Label>
          <Input
            type="number"
            step="0.01"
            value={settings.cost_alert_threshold}
            onChange={(e) =>
              setSettings({ ...settings, cost_alert_threshold: parseFloat(e.target.value) })
            }
          />
        </div>

        <div className="space-y-2">
          <Label>Max Cost per Sandbox ($)</Label>
          <Input
            type="number"
            step="0.01"
            value={settings.max_cost_per_sandbox}
            onChange={(e) =>
              setSettings({ ...settings, max_cost_per_sandbox: parseFloat(e.target.value) })
            }
          />
        </div>

        <div className="space-y-2">
          <Label>Max Total Cost ($)</Label>
          <Input
            type="number"
            step="0.01"
            value={settings.max_total_cost}
            onChange={(e) =>
              setSettings({ ...settings, max_total_cost: parseFloat(e.target.value) })
            }
          />
        </div>

        <div className="flex items-center justify-between">
          <div className="space-y-0.5">
            <Label>Auto-stop at Cost Limit</Label>
            <p className="text-sm text-muted-foreground">
              Automatically stop sandboxes when cost limit is reached
            </p>
          </div>
          <Switch
            checked={settings.auto_stop_at_cost_limit}
            onCheckedChange={(checked) =>
              setSettings({ ...settings, auto_stop_at_cost_limit: checked })
            }
          />
        </div>
      </CardContent>
    </Card>
  )
}

function SecurityTab({
  settings,
  setSettings,
}: {
  settings: SandboxSettingsType
  setSettings: (settings: SandboxSettingsType) => void
}) {
  return (
    <Card>
      <CardHeader>
        <CardTitle>Security Settings</CardTitle>
        <CardDescription>
          Configure sandbox security policies
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="space-y-2">
          <Label>Network Mode</Label>
          <select
            className="w-full p-2 border rounded"
            value={settings.default_network_mode}
            onChange={(e) =>
              setSettings({ ...settings, default_network_mode: e.target.value })
            }
          >
            <option value="none">None</option>
            <option value="isolated">Isolated</option>
            <option value="host">Host</option>
            <option value="custom">Custom</option>
          </select>
        </div>

        <div className="flex items-center justify-between">
          <div className="space-y-0.5">
            <Label>Allow Public Endpoints</Label>
            <p className="text-sm text-muted-foreground">
              Allow sandboxes to expose public URLs
            </p>
          </div>
          <Switch
            checked={settings.allow_public_endpoints}
            onCheckedChange={(checked) =>
              setSettings({ ...settings, allow_public_endpoints: checked })
            }
          />
        </div>

        <div className="flex items-center justify-between">
          <div className="space-y-0.5">
            <Label>Require Authentication for Web</Label>
            <p className="text-sm text-muted-foreground">
              Require auth for web-based sandbox access
            </p>
          </div>
          <Switch
            checked={settings.require_auth_for_web}
            onCheckedChange={(checked) =>
              setSettings({ ...settings, require_auth_for_web: checked })
            }
          />
        </div>

        <div className="flex items-center justify-between">
          <div className="space-y-0.5">
            <Label>Allow Privileged Containers</Label>
            <p className="text-sm text-muted-foreground">
              Allow sandboxes to run in privileged mode (security risk)
            </p>
          </div>
          <Switch
            checked={settings.allow_privileged_containers}
            onCheckedChange={(checked) =>
              setSettings({ ...settings, allow_privileged_containers: checked })
            }
          />
        </div>

        <div className="flex items-center justify-between">
          <div className="space-y-0.5">
            <Label>Require Non-root User</Label>
            <p className="text-sm text-muted-foreground">
              Enforce non-root user in containers
            </p>
          </div>
          <Switch
            checked={settings.require_non_root_user}
            onCheckedChange={(checked) =>
              setSettings({ ...settings, require_non_root_user: checked })
            }
          />
        </div>

        <div className="flex items-center justify-between">
          <div className="space-y-0.5">
            <Label>Enable Security Scanning</Label>
            <p className="text-sm text-muted-foreground">
              Scan container images for vulnerabilities
            </p>
          </div>
          <Switch
            checked={settings.enable_security_scanning}
            onCheckedChange={(checked) =>
              setSettings({ ...settings, enable_security_scanning: checked })
            }
          />
        </div>
      </CardContent>
    </Card>
  )
}

function ProviderConfigDialog({
  provider,
  open,
  onOpenChange,
  onSave,
}: {
  provider: string
  open: boolean
  onOpenChange: (open: boolean) => void
  onSave: () => void
}) {
  const { toast } = useToast()
  const [credentials, setCredentials] = useState<ProviderCredentials>({})
  const [isSaving, setIsSaving] = useState(false)
  const [isValidating, setIsValidating] = useState(false)
  const [validationResult, setValidationResult] = useState<{ valid: boolean; message: string } | null>(null)

  const handleSave = async () => {
    setIsSaving(true)
    try {
      await updateProviderCredentials(provider, credentials)
      toast({
        title: 'Success',
        description: `${provider} credentials saved successfully`,
      })
      onSave()
      onOpenChange(false)
    } catch (error) {
      toast({
        title: 'Error',
        description: `Failed to save ${provider} credentials`,
        variant: 'destructive',
      })
      console.error('Failed to save credentials:', error)
    } finally {
      setIsSaving(false)
    }
  }

  const handleValidate = async () => {
    setIsValidating(true)
    setValidationResult(null)
    try {
      const result = await validateProvider(provider)
      setValidationResult(result)
      if (result.valid) {
        toast({
          title: 'Success',
          description: 'Provider validation successful',
        })
      }
    } catch (error) {
      toast({
        title: 'Error',
        description: 'Provider validation failed',
        variant: 'destructive',
      })
      console.error('Validation failed:', error)
    } finally {
      setIsValidating(false)
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-2xl">
        <DialogHeader>
          <DialogTitle>Configure {provider.charAt(0).toUpperCase() + provider.slice(1)}</DialogTitle>
          <DialogDescription>
            Enter your provider credentials and configuration
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4 py-4">
          <div className="space-y-2">
            <Label>API Key</Label>
            <Input
              type="password"
              placeholder="Enter API key"
              value={credentials.api_key || ''}
              onChange={(e) => setCredentials({ ...credentials, api_key: e.target.value })}
            />
          </div>

          {provider !== 'local' && provider !== 'e2b' && (
            <div className="space-y-2">
              <Label>API Secret (if required)</Label>
              <Input
                type="password"
                placeholder="Enter API secret"
                value={credentials.api_secret || ''}
                onChange={(e) => setCredentials({ ...credentials, api_secret: e.target.value })}
              />
            </div>
          )}

          <div className="space-y-2">
            <Label>API Endpoint (optional)</Label>
            <Input
              type="url"
              placeholder="https://api.provider.com"
              value={credentials.api_endpoint || ''}
              onChange={(e) => setCredentials({ ...credentials, api_endpoint: e.target.value })}
            />
          </div>

          {(provider === 'beam' || provider === 'daytona') && (
            <div className="space-y-2">
              <Label>Workspace ID</Label>
              <Input
                placeholder="Enter workspace ID"
                value={credentials.workspace_id || ''}
                onChange={(e) => setCredentials({ ...credentials, workspace_id: e.target.value })}
              />
            </div>
          )}

          {(provider === 'modal' || provider === 'northflank') && (
            <div className="space-y-2">
              <Label>Project ID</Label>
              <Input
                placeholder="Enter project ID"
                value={credentials.project_id || ''}
                onChange={(e) => setCredentials({ ...credentials, project_id: e.target.value })}
              />
            </div>
          )}

          {provider === 'cloudflare' && (
            <div className="space-y-2">
              <Label>Account ID</Label>
              <Input
                placeholder="Enter account ID"
                value={credentials.account_id || ''}
                onChange={(e) => setCredentials({ ...credentials, account_id: e.target.value })}
              />
            </div>
          )}

          {provider === 'flyio' && (
            <div className="space-y-2">
              <Label>App Name</Label>
              <Input
                placeholder="Enter app name"
                value={credentials.app_name || ''}
                onChange={(e) => setCredentials({ ...credentials, app_name: e.target.value })}
              />
            </div>
          )}

          {validationResult && (
            <Alert variant={validationResult.valid ? 'default' : 'destructive'}>
              {validationResult.valid ? (
                <Check className="h-4 w-4" />
              ) : (
                <AlertTriangle className="h-4 w-4" />
              )}
              <AlertDescription>{validationResult.message}</AlertDescription>
            </Alert>
          )}
        </div>

        <DialogFooter>
          <Button
            variant="outline"
            onClick={handleValidate}
            disabled={isValidating}
          >
            {isValidating ? (
              <>
                <RefreshCw className="mr-2 h-4 w-4 animate-spin" />
                Validating...
              </>
            ) : (
              'Validate'
            )}
          </Button>
          <Button
            onClick={handleSave}
            disabled={isSaving}
          >
            {isSaving ? (
              <>
                <RefreshCw className="mr-2 h-4 w-4 animate-spin" />
                Saving...
              </>
            ) : (
              'Save'
            )}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
