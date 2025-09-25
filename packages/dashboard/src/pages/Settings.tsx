import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { useCloudAuth, useCloudSync } from '@/hooks/useCloud'
import { cloudService, formatLastSync } from '@/services/cloud'
import { fetchConfig } from '@/services/config'
import { Cloud, User, RefreshCw, Download, Upload, Code2, ExternalLink } from 'lucide-react'
import { useState, useEffect } from 'react'
import { SUPPORTED_EDITORS, getDefaultEditorSettings, findEditorById } from '@/lib/editor-utils'
import type { EditorSettings } from '@/lib/editor-utils'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Switch } from '@/components/ui/switch'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'

export function Settings() {
  const [isCloudEnabled, setIsCloudEnabled] = useState(false)

  useEffect(() => {
    fetchConfig().then(config => {
      setIsCloudEnabled(config.cloud_enabled)
    })
  }, [])

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold tracking-tight">Settings</h1>
        <p className="text-muted-foreground">
          Configure your Orkee dashboard preferences and integrations.
        </p>
      </div>

      <div className="grid gap-6">
        {/* Editor Settings - Always show */}
        <EditorSettings />
        
        {/* Cloud Settings */}
        {isCloudEnabled && <CloudSettings />}
        
        {/* When cloud is disabled, show a note */}
        {!isCloudEnabled && (
          <div className="rounded-lg border p-6">
            <h2 className="text-xl font-semibold mb-2">Additional Settings</h2>
            <p className="text-muted-foreground">
              Cloud sync and other features will be available here when enabled.
            </p>
          </div>
        )}
      </div>
    </div>
  )
}

// Editor Settings Component
function EditorSettings() {
  const [editorSettings, setEditorSettings] = useState<EditorSettings>(getDefaultEditorSettings());
  const [isTestingEditor, setIsTestingEditor] = useState(false);
  const [isSaving, setIsSaving] = useState(false);

  // Load editor settings on mount
  useEffect(() => {
    loadEditorSettings();
  }, []);

  const loadEditorSettings = async () => {
    try {
      const stored = localStorage.getItem('orkee-editor-settings');
      if (stored) {
        const parsed = JSON.parse(stored);
        setEditorSettings({ ...getDefaultEditorSettings(), ...parsed });
      }
    } catch (error) {
      console.error('Failed to load editor settings:', error);
    }
  };

  const saveEditorSettings = async (newSettings: EditorSettings) => {
    setIsSaving(true);
    try {
      localStorage.setItem('orkee-editor-settings', JSON.stringify(newSettings));
      setEditorSettings(newSettings);
    } catch (error) {
      console.error('Failed to save editor settings:', error);
    } finally {
      setIsSaving(false);
    }
  };

  const handleEditorChange = (editorId: string) => {
    const newSettings = {
      ...editorSettings,
      defaultEditor: editorId,
    };
    saveEditorSettings(newSettings);
  };

  const handleCustomCommandChange = (command: string) => {
    const newSettings = {
      ...editorSettings,
      customCommand: command,
    };
    saveEditorSettings(newSettings);
  };

  const handleToggleChange = (field: keyof EditorSettings, value: boolean) => {
    const newSettings = {
      ...editorSettings,
      [field]: value,
    };
    saveEditorSettings(newSettings);
  };

  const handleTestEditor = async () => {
    setIsTestingEditor(true);
    try {
      const response = await fetch('/api/projects/open-in-editor', {
        method: 'GET',
      });
      const result = await response.json();
      
      if (result.success) {
        alert(`✅ ${result.message}\n\nDetected: ${result.detectedCommand || 'N/A'}`);
      } else {
        alert(`❌ ${result.message}`);
      }
    } catch (error) {
      alert('❌ Failed to test editor configuration');
      console.error('Test editor error:', error);
    } finally {
      setIsTestingEditor(false);
    }
  };

  const selectedEditor = findEditorById(editorSettings.defaultEditor);

  return (
    <div className="rounded-lg border p-6">
      <div className="flex items-center gap-2 mb-4">
        <Code2 className="h-5 w-5 text-primary" />
        <h2 className="text-xl font-semibold">Code Editor</h2>
      </div>

      <div className="space-y-6">
        {/* Editor Selection */}
        <div className="space-y-3">
          <Label htmlFor="editor-select">Default Editor</Label>
          <p className="text-sm text-muted-foreground mb-3">
            Choose your preferred code editor for opening projects
          </p>
          <Select 
            value={editorSettings.defaultEditor} 
            onValueChange={handleEditorChange}
            disabled={isSaving}
          >
            <SelectTrigger>
              <SelectValue>
                <div className="flex items-center gap-2">
                  <span>{selectedEditor?.icon}</span>
                  <span>{selectedEditor?.name || "Select editor..."}</span>
                </div>
              </SelectValue>
            </SelectTrigger>
            <SelectContent className="max-h-[300px]">
              {SUPPORTED_EDITORS.map((editor) => (
                <SelectItem key={editor.id} value={editor.id}>
                  <div className="flex items-center gap-2">
                    <span>{editor.icon}</span>
                    <span>{editor.name}</span>
                    {/* Platform indicators */}
                    {editor.platformRestricted?.includes('darwin') && editor.platformRestricted.length === 1 && (
                      <Badge variant="secondary" className="ml-auto text-xs">macOS</Badge>
                    )}
                    {editor.platformRestricted?.includes('win32') && editor.platformRestricted.length === 1 && (
                      <Badge variant="secondary" className="ml-auto text-xs">Windows</Badge>
                    )}
                    {editor.platformRestricted?.includes('linux') && editor.platformRestricted.length === 1 && (
                      <Badge variant="secondary" className="ml-auto text-xs">Linux</Badge>
                    )}
                  </div>
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>

        {/* Custom Command Input */}
        {editorSettings.defaultEditor === 'custom' && (
          <div className="space-y-3">
            <Label htmlFor="custom-command">Custom Command</Label>
            <p className="text-sm text-muted-foreground mb-3">
              Enter the full path or command to launch your editor
            </p>
            <Input
              id="custom-command"
              type="text"
              placeholder="e.g., /usr/local/bin/myeditor"
              value={editorSettings.customCommand}
              onChange={(e) => handleCustomCommandChange(e.target.value)}
              disabled={isSaving}
            />
          </div>
        )}

        {/* Editor Options */}
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label htmlFor="auto-detect">Auto-detect Editor</Label>
              <p className="text-sm text-muted-foreground">
                Automatically detect if the selected editor is installed
              </p>
            </div>
            <Switch
              id="auto-detect"
              checked={editorSettings.autoDetect}
              onCheckedChange={(value) => handleToggleChange('autoDetect', value)}
              disabled={isSaving}
            />
          </div>

          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label htmlFor="new-window">Open in New Window</Label>
              <p className="text-sm text-muted-foreground">
                Always open projects in a new editor window
              </p>
            </div>
            <Switch
              id="new-window"
              checked={editorSettings.openInNewWindow}
              onCheckedChange={(value) => handleToggleChange('openInNewWindow', value)}
              disabled={isSaving}
            />
          </div>
        </div>

        {/* Test Button */}
        <div className="pt-4 border-t">
          <Button 
            variant="outline" 
            onClick={handleTestEditor}
            className="w-full"
            disabled={!editorSettings.defaultEditor || isTestingEditor || isSaving}
          >
            {isTestingEditor ? (
              <>
                <RefreshCw className="mr-2 h-4 w-4 animate-spin" />
                Testing...
              </>
            ) : (
              <>
                <ExternalLink className="mr-2 h-4 w-4" />
                Test Editor Configuration
              </>
            )}
          </Button>
          <p className="text-xs text-muted-foreground mt-2 text-center">
            Tests if your selected editor can be launched successfully
          </p>
        </div>
      </div>
    </div>
  );
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
                  value={import.meta.env.VITE_ORKEE_CLOUD_API_URL || "https://api.orkee.ai"}
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