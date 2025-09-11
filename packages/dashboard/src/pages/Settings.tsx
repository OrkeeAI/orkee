import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { useCloudAuth, useCloudSync } from '@/contexts/CloudContext'
import { cloudService, formatLastSync } from '@/services/cloud'
import { Settings as SettingsIcon, Key, Bell, Palette, Download, Upload, Cloud, User, RefreshCw } from 'lucide-react'

export function Settings() {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold tracking-tight">Settings</h1>
        <p className="text-muted-foreground">
          Configure your Orkee dashboard preferences and integrations.
        </p>
      </div>

      <div className="grid gap-6">
        {/* API Configuration */}
        <div className="rounded-lg border p-6">
          <div className="flex items-center gap-2 mb-4">
            <Key className="h-5 w-5 text-primary" />
            <h2 className="text-xl font-semibold">API Configuration</h2>
          </div>
          <div className="space-y-4">
            <div>
              <label className="text-sm font-medium mb-2 block">OpenAI API Key</label>
              <div className="flex gap-2">
                <input 
                  type="password" 
                  placeholder="sk-..." 
                  className="flex-1 px-3 py-2 border rounded-md"
                />
                <Button variant="outline">Update</Button>
              </div>
            </div>
            <div>
              <label className="text-sm font-medium mb-2 block">Anthropic API Key</label>
              <div className="flex gap-2">
                <input 
                  type="password" 
                  placeholder="sk-ant-..." 
                  className="flex-1 px-3 py-2 border rounded-md"
                />
                <Button variant="outline">Update</Button>
              </div>
            </div>
            <div>
              <label className="text-sm font-medium mb-2 block">Custom Endpoint URL</label>
              <div className="flex gap-2">
                <input 
                  type="url" 
                  placeholder="https://your-api-endpoint.com" 
                  className="flex-1 px-3 py-2 border rounded-md"
                />
                <Button variant="outline">Save</Button>
              </div>
            </div>
          </div>
        </div>

        {/* Cloud Settings */}
        <CloudSettings />

        {/* Notifications */}
        <div className="rounded-lg border p-6">
          <div className="flex items-center gap-2 mb-4">
            <Bell className="h-5 w-5 text-primary" />
            <h2 className="text-xl font-semibold">Notifications</h2>
          </div>
          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <div>
                <p className="font-medium">Agent Status Changes</p>
                <p className="text-sm text-muted-foreground">Get notified when agents go online/offline</p>
              </div>
              <label className="relative inline-flex items-center cursor-pointer">
                <input type="checkbox" className="sr-only peer" defaultChecked />
                <div className="w-11 h-6 bg-gray-200 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-primary"></div>
              </label>
            </div>
            <div className="flex items-center justify-between">
              <div>
                <p className="font-medium">Error Alerts</p>
                <p className="text-sm text-muted-foreground">Receive alerts for system errors</p>
              </div>
              <label className="relative inline-flex items-center cursor-pointer">
                <input type="checkbox" className="sr-only peer" defaultChecked />
                <div className="w-11 h-6 bg-gray-200 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-primary"></div>
              </label>
            </div>
            <div className="flex items-center justify-between">
              <div>
                <p className="font-medium">Usage Alerts</p>
                <p className="text-sm text-muted-foreground">Alerts for high usage or approaching limits</p>
              </div>
              <label className="relative inline-flex items-center cursor-pointer">
                <input type="checkbox" className="sr-only peer" />
                <div className="w-11 h-6 bg-gray-200 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-primary"></div>
              </label>
            </div>
          </div>
        </div>

        {/* Appearance */}
        <div className="rounded-lg border p-6">
          <div className="flex items-center gap-2 mb-4">
            <Palette className="h-5 w-5 text-primary" />
            <h2 className="text-xl font-semibold">Appearance</h2>
          </div>
          <div className="space-y-4">
            <div>
              <p className="font-medium mb-3">Theme</p>
              <div className="flex gap-4">
                <label className="flex items-center gap-2 cursor-pointer">
                  <input type="radio" name="theme" value="light" className="w-4 h-4" defaultChecked />
                  <span>Light</span>
                </label>
                <label className="flex items-center gap-2 cursor-pointer">
                  <input type="radio" name="theme" value="dark" className="w-4 h-4" />
                  <span>Dark</span>
                </label>
                <label className="flex items-center gap-2 cursor-pointer">
                  <input type="radio" name="theme" value="system" className="w-4 h-4" />
                  <span>System</span>
                </label>
              </div>
            </div>
            <div>
              <p className="font-medium mb-3">Dashboard Density</p>
              <div className="flex gap-4">
                <label className="flex items-center gap-2 cursor-pointer">
                  <input type="radio" name="density" value="compact" className="w-4 h-4" />
                  <span>Compact</span>
                </label>
                <label className="flex items-center gap-2 cursor-pointer">
                  <input type="radio" name="density" value="normal" className="w-4 h-4" defaultChecked />
                  <span>Normal</span>
                </label>
                <label className="flex items-center gap-2 cursor-pointer">
                  <input type="radio" name="density" value="comfortable" className="w-4 h-4" />
                  <span>Comfortable</span>
                </label>
              </div>
            </div>
          </div>
        </div>

        {/* Data Management */}
        <div className="rounded-lg border p-6">
          <div className="flex items-center gap-2 mb-4">
            <SettingsIcon className="h-5 w-5 text-primary" />
            <h2 className="text-xl font-semibold">Data Management</h2>
          </div>
          <div className="flex gap-4">
            <Button variant="outline">
              <Download className="mr-2 h-4 w-4" />
              Export Configuration
            </Button>
            <Button variant="outline">
              <Upload className="mr-2 h-4 w-4" />
              Import Configuration
            </Button>
          </div>
          <p className="text-sm text-muted-foreground mt-2">
            Export your settings and configurations, or import from a backup.
          </p>
        </div>
      </div>
    </div>
  )
}

// Cloud Settings Component
function CloudSettings() {
  const { isAuthenticating, login, logout, isAuthenticated, user } = useCloudAuth();
  const { syncStatus, refreshSyncStatus } = useCloudSync();

  const handleRefreshSync = async () => {
    await refreshSyncStatus();
  };

  const handleTestConnection = async () => {
    try {
      if (isAuthenticated) {
        await cloudService.getUsageStats();
        alert('Cloud connection test successful!');
      } else {
        alert('Please authenticate first');
      }
    } catch (error) {
      alert('Cloud connection test failed: ' + (error instanceof Error ? error.message : 'Unknown error'));
    }
  };

  return (
    <div className="rounded-lg border p-6">
      <div className="flex items-center gap-2 mb-4">
        <Cloud className="h-5 w-5 text-primary" />
        <h2 className="text-xl font-semibold">Cloud Sync</h2>
        {isAuthenticated && (
          <Badge variant="secondary" className="ml-2">
            Connected
          </Badge>
        )}
      </div>
      
      <div className="space-y-6">
        {/* Authentication Status */}
        <div className="space-y-3">
          <h3 className="text-sm font-medium">Account</h3>
          {isAuthenticated && user ? (
            <div className="flex items-center justify-between p-3 bg-green-50 border border-green-200 rounded-md">
              <div className="flex items-center gap-3">
                <div className="w-8 h-8 bg-green-500 rounded-full flex items-center justify-center text-white text-sm font-medium">
                  {user.name?.charAt(0)?.toUpperCase() || user.email?.charAt(0)?.toUpperCase() || 'U'}
                </div>
                <div>
                  <p className="font-medium text-green-800">{user.name || 'Cloud User'}</p>
                  <p className="text-sm text-green-600">{user.email}</p>
                  <div className="flex items-center gap-2 mt-1">
                    <Badge variant="outline" className="text-xs">
                      {user.tier || 'Free'} Plan
                    </Badge>
                  </div>
                </div>
              </div>
              <Button 
                variant="outline" 
                size="sm" 
                onClick={logout}
                disabled={isAuthenticating}
              >
                Sign Out
              </Button>
            </div>
          ) : (
            <div className="flex items-center justify-between p-3 bg-gray-50 border border-gray-200 rounded-md">
              <div className="flex items-center gap-3">
                <div className="w-8 h-8 bg-gray-400 rounded-full flex items-center justify-center text-white text-sm">
                  <User className="h-4 w-4" />
                </div>
                <div>
                  <p className="font-medium text-gray-700">Not connected</p>
                  <p className="text-sm text-gray-500">Connect to sync your projects to the cloud</p>
                </div>
              </div>
              <Button 
                onClick={login}
                disabled={isAuthenticating}
                className="min-w-20"
              >
                {isAuthenticating ? (
                  <>
                    <RefreshCw className="mr-2 h-4 w-4 animate-spin" />
                    Connecting...
                  </>
                ) : (
                  <>
                    <Cloud className="mr-2 h-4 w-4" />
                    Connect
                  </>
                )}
              </Button>
            </div>
          )}
        </div>

        {/* Sync Status */}
        {isAuthenticated && (
          <div className="space-y-3">
            <h3 className="text-sm font-medium">Sync Status</h3>
            <div className="p-3 bg-blue-50 border border-blue-200 rounded-md">
              <div className="flex items-center justify-between mb-2">
                <span className="text-sm font-medium text-blue-800">Project Synchronization</span>
                <Button
                  variant="ghost" 
                  size="sm"
                  onClick={handleRefreshSync}
                >
                  <RefreshCw className="h-4 w-4" />
                </Button>
              </div>
              <div className="text-sm text-blue-600 space-y-1">
                <p>{syncStatus.synced_projects} of {syncStatus.total_projects} projects synced</p>
                {syncStatus.pending_projects > 0 && (
                  <p>{syncStatus.pending_projects} projects pending sync</p>
                )}
                {syncStatus.conflict_projects > 0 && (
                  <p className="text-red-600">{syncStatus.conflict_projects} projects have conflicts</p>
                )}
                {syncStatus.last_sync && (
                  <p>Last sync: {formatLastSync(syncStatus.last_sync)}</p>
                )}
              </div>
            </div>
          </div>
        )}

        {/* Cloud Configuration */}
        <div className="space-y-3">
          <h3 className="text-sm font-medium">Configuration</h3>
          <div className="space-y-3">
            <div>
              <label className="text-sm font-medium mb-2 block">Cloud API URL</label>
              <div className="flex gap-2">
                <input 
                  type="url" 
                  value="https://api.orkee.ai"
                  readOnly
                  className="flex-1 px-3 py-2 border rounded-md bg-gray-50 text-gray-600"
                />
                <Button 
                  variant="outline" 
                  onClick={handleTestConnection}
                  disabled={!isAuthenticated}
                >
                  Test
                </Button>
              </div>
              <p className="text-xs text-muted-foreground mt-1">
                Official Orkee Cloud API endpoint
              </p>
            </div>
          </div>
        </div>

        {/* Usage Information */}
        {isAuthenticated && (
          <div className="space-y-3">
            <h3 className="text-sm font-medium">Usage & Limits</h3>
            <div className="p-3 bg-gray-50 border border-gray-200 rounded-md">
              <div className="text-sm text-gray-600 space-y-1">
                <div className="flex justify-between">
                  <span>Plan:</span>
                  <span className="font-medium">{user?.tier || 'Free'}</span>
                </div>
                <div className="flex justify-between">
                  <span>Projects:</span>
                  <span>{syncStatus.total_projects} / {user?.tier === 'Pro' ? '∞' : user?.tier === 'Starter' ? '10' : '2'}</span>
                </div>
                <div className="flex justify-between">
                  <span>Storage:</span>
                  <span>-- / {user?.tier === 'Pro' ? '50GB' : user?.tier === 'Starter' ? '5GB' : '100MB'}</span>
                </div>
              </div>
            </div>
          </div>
        )}

        {/* Actions */}
        <div className="space-y-3">
          <h3 className="text-sm font-medium">Actions</h3>
          <div className="flex gap-2">
            <Button variant="outline" size="sm" disabled={!isAuthenticated}>
              <Download className="mr-2 h-4 w-4" />
              Export Data
            </Button>
            <Button variant="outline" size="sm" disabled={!isAuthenticated}>
              <Upload className="mr-2 h-4 w-4" />
              Import Data
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
}