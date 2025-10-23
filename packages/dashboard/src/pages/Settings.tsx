import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { useCloudAuth, useCloudSync } from '@/hooks/useCloud'
import { cloudService, formatLastSync } from '@/services/cloud'
import { fetchConfig } from '@/services/config'
import { exportDatabase, importDatabase, type ImportResult } from '@/services/database'
import { Cloud, User, RefreshCw, Download, Upload, Code2, ExternalLink, Database, AlertTriangle, Shield, Trash2, Key, Check, Terminal, Sliders, LayoutGrid } from 'lucide-react'
import { useState, useEffect, useRef } from 'react'
import { SUPPORTED_EDITORS, getDefaultEditorSettings, findEditorById } from '@/lib/editor-utils'
import type { EditorSettings } from '@/lib/editor-utils'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Switch } from '@/components/ui/switch'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { CliInstallationSettings } from '@/components/CliInstallationSettings'
import { SecurityStatusSection } from '@/components/SecurityStatusSection'
import { KeySourcesTable } from '@/components/KeySourcesTable'
import { PasswordManagementDialog } from '@/components/PasswordManagementDialog'
import { useTelemetry } from '@/contexts/TelemetryContext'
import { usersService } from '@/services/users'
import type { MaskedUser } from '@/services/users'
import { updateSetting, getSettingsByCategory, type SystemSetting } from '@/services/settings'
import { clearConfigCache } from '@/services/config'

export function Settings() {
  const [isMacOS, setIsMacOS] = useState(false)

  useEffect(() => {
    setIsMacOS(navigator.platform.toLowerCase().includes('mac'))
  }, [])

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold tracking-tight">Settings</h1>
        <p className="text-muted-foreground">
          Configure your Orkee dashboard preferences and integrations.
        </p>
      </div>

      <Tabs defaultValue="general" className="w-full">
        <TabsList className="grid w-full grid-cols-6">
          <TabsTrigger value="general" className="flex items-center gap-2">
            <LayoutGrid className="h-4 w-4" />
            General
          </TabsTrigger>
          <TabsTrigger value="security" className="flex items-center gap-2">
            <Key className="h-4 w-4" />
            Security
          </TabsTrigger>
          <TabsTrigger value="database" className="flex items-center gap-2">
            <Database className="h-4 w-4" />
            Database
          </TabsTrigger>
          <TabsTrigger value="privacy" className="flex items-center gap-2">
            <Shield className="h-4 w-4" />
            Privacy
          </TabsTrigger>
          <TabsTrigger value="cloud" className="flex items-center gap-2">
            <Cloud className="h-4 w-4" />
            Cloud
          </TabsTrigger>
          <TabsTrigger value="advanced" className="flex items-center gap-2">
            <Sliders className="h-4 w-4" />
            Advanced
          </TabsTrigger>
        </TabsList>

        <TabsContent value="general" className="space-y-6 mt-6">
          {/* General Settings - Editor & CLI */}
          <GeneralSettings isMacOS={isMacOS} />
        </TabsContent>

        <TabsContent value="security" className="space-y-6 mt-6">
          {/* API Keys Settings */}
          <ApiKeysSettings />
        </TabsContent>

        <TabsContent value="database" className="space-y-6 mt-6">
          {/* Database Settings */}
          <DatabaseSettings />
        </TabsContent>

        <TabsContent value="privacy" className="space-y-6 mt-6">
          {/* Privacy & Telemetry Settings */}
          <PrivacySettings />
        </TabsContent>

        <TabsContent value="cloud" className="space-y-6 mt-6">
          {/* Cloud Settings - Always shown now */}
          <CloudSettings />
        </TabsContent>

        <TabsContent value="advanced" className="space-y-6 mt-6">
          {/* Advanced Configuration Settings */}
          <AdvancedSettings />
        </TabsContent>
      </Tabs>
    </div>
  )
}

// General Settings Component (combines Editor and CLI)
function GeneralSettings({ isMacOS }: { isMacOS: boolean }) {
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

  return (
    <div className="space-y-6">
      {/* Editor Settings Section */}
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
              <SelectTrigger id="editor-select">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {SUPPORTED_EDITORS.map((editor) => (
                  <SelectItem key={editor.id} value={editor.id}>
                    <div className="flex items-center gap-2">
                      <span>{editor.icon}</span>
                      <span>{editor.name}</span>
                    </div>
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          {/* Custom Command */}
          {editorSettings.defaultEditor === 'custom' && (
            <div className="space-y-3">
              <Label htmlFor="custom-command">Custom Command</Label>
              <p className="text-sm text-muted-foreground">
                Enter the command to open your editor (e.g., "subl", "atom", "emacs")
              </p>
              <Input
                id="custom-command"
                value={editorSettings.customCommand}
                onChange={(e) => handleCustomCommandChange(e.target.value)}
                placeholder="e.g., subl, atom, emacs"
                disabled={isSaving}
              />
            </div>
          )}

          {/* Auto-detect Editor */}
          <div className="flex items-center justify-between p-3 border rounded-md">
            <div className="space-y-0.5">
              <Label htmlFor="auto-detect">Auto-detect Editor</Label>
              <p className="text-sm text-muted-foreground">
                Automatically detect if the selected editor is installed
              </p>
            </div>
            <Switch
              id="auto-detect"
              checked={editorSettings.autoDetect}
              onCheckedChange={(checked) => handleToggleChange('autoDetect', checked)}
              disabled={isSaving}
            />
          </div>

          {/* Open in New Window */}
          <div className="flex items-center justify-between p-3 border rounded-md">
            <div className="space-y-0.5">
              <Label htmlFor="new-window">Open in New Window</Label>
              <p className="text-sm text-muted-foreground">
                Always open projects in a new editor window
              </p>
            </div>
            <Switch
              id="new-window"
              checked={editorSettings.openInNewWindow}
              onCheckedChange={(checked) => handleToggleChange('openInNewWindow', checked)}
              disabled={isSaving}
            />
          </div>

          {/* Test Button */}
          <div className="flex items-center justify-between pt-3 border-t">
            <div>
              <p className="text-sm font-medium">Test Editor Configuration</p>
              <p className="text-sm text-muted-foreground">
                Verify your editor can be opened from Orkee
              </p>
            </div>
            <Button
              variant="outline"
              onClick={handleTestEditor}
              disabled={isTestingEditor}
            >
              {isTestingEditor ? (
                <>
                  <RefreshCw className="mr-2 h-4 w-4 animate-spin" />
                  Testing...
                </>
              ) : (
                <>
                  <ExternalLink className="mr-2 h-4 w-4" />
                  Test Editor
                </>
              )}
            </Button>
          </div>
        </div>
      </div>

      {/* CLI Installation Section (macOS only) */}
      {isMacOS && (
        <div className="rounded-lg border p-6">
          <div className="flex items-center gap-2 mb-4">
            <Terminal className="h-5 w-5 text-primary" />
            <h2 className="text-xl font-semibold">CLI Installation</h2>
          </div>
          <CliInstallationSettings />
        </div>
      )}
    </div>
  );
}

// Editor Settings Component (kept for legacy, not directly used)
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

// API Keys Settings Component
function ApiKeysSettings() {
  const [user, setUser] = useState<MaskedUser | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [isSaving, setIsSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);

  // Form state
  const [openaiKey, setOpenaiKey] = useState('');
  const [anthropicKey, setAnthropicKey] = useState('');
  const [googleKey, setGoogleKey] = useState('');
  const [xaiKey, setXaiKey] = useState('');
  const [gatewayEnabled, setGatewayEnabled] = useState(false);
  const [gatewayUrl, setGatewayUrl] = useState('');
  const [gatewayKey, setGatewayKey] = useState('');

  // Password management dialog state
  const [passwordDialogOpen, setPasswordDialogOpen] = useState(false);
  const [passwordDialogMode, setPasswordDialogMode] = useState<'set' | 'change' | 'remove'>('set');

  // Load user credentials on mount
  useEffect(() => {
    loadUser();
  }, []);

  const loadUser = async () => {
    setIsLoading(true);
    setError(null);
    try {
      const userData = await usersService.getCurrentUser();
      setUser(userData);
      setGatewayEnabled(userData.ai_gateway_enabled);
      setGatewayUrl(userData.ai_gateway_url || '');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load credentials');
      console.error('Failed to load user:', err);
    } finally {
      setIsLoading(false);
    }
  };

  const handleSave = async () => {
    setIsSaving(true);
    setError(null);
    setSuccessMessage(null);

    try {
      // Build update payload with only non-empty values
      const updates: Record<string, string | boolean> = {};

      if (openaiKey.trim()) updates.openai_api_key = openaiKey.trim();
      if (anthropicKey.trim()) updates.anthropic_api_key = anthropicKey.trim();
      if (googleKey.trim()) updates.google_api_key = googleKey.trim();
      if (xaiKey.trim()) updates.xai_api_key = xaiKey.trim();

      updates.ai_gateway_enabled = gatewayEnabled;
      if (gatewayUrl.trim()) updates.ai_gateway_url = gatewayUrl.trim();
      if (gatewayKey.trim()) updates.ai_gateway_key = gatewayKey.trim();

      const updatedUser = await usersService.updateCredentials(updates);
      setUser(updatedUser);

      // Clear form fields
      setOpenaiKey('');
      setAnthropicKey('');
      setGoogleKey('');
      setXaiKey('');
      setGatewayKey('');

      setSuccessMessage('API keys updated successfully');
      setTimeout(() => setSuccessMessage(null), 3000);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to save credentials');
      console.error('Failed to save credentials:', err);
    } finally {
      setIsSaving(false);
    }
  };

  const openPasswordDialog = (mode: 'set' | 'change' | 'remove') => {
    setPasswordDialogMode(mode);
    setPasswordDialogOpen(true);
  };

  if (isLoading) {
    return (
      <div className="rounded-lg border p-6">
        <div className="flex items-center gap-2 mb-4">
          <Key className="h-5 w-5 text-primary" />
          <h2 className="text-xl font-semibold">API Keys</h2>
        </div>
        <p className="text-muted-foreground">Loading...</p>
      </div>
    );
  }

  return (
    <div className="rounded-lg border p-6">
      <div className="flex items-center gap-2 mb-4">
        <Key className="h-5 w-5 text-primary" />
        <h2 className="text-xl font-semibold">API Keys & Security</h2>
      </div>

      <div className="space-y-6">
        {/* Security Status Section */}
        <SecurityStatusSection onManagePassword={openPasswordDialog} />

        {/* Key Sources Table */}
        <KeySourcesTable />

        <Alert>
          <Shield className="h-4 w-4" />
          <AlertDescription>
            API keys are encrypted and stored in your local database. Environment variables override database keys.
            Leave fields empty to keep existing keys unchanged.
          </AlertDescription>
        </Alert>

        {/* Error Display */}
        {error && (
          <Alert variant="destructive">
            <AlertTriangle className="h-4 w-4" />
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        )}

        {/* Success Display */}
        {successMessage && (
          <Alert>
            <Check className="h-4 w-4" />
            <AlertDescription>{successMessage}</AlertDescription>
          </Alert>
        )}

        {/* AI Provider Keys */}
        <div className="space-y-4">
          <h3 className="text-sm font-medium">Update API Keys</h3>

          {/* OpenAI */}
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <Label htmlFor="openai-key">OpenAI API Key</Label>
              {user?.has_openai_api_key && (
                <Badge variant="secondary" className="text-xs">
                  <Check className="h-3 w-3 mr-1" />
                  Configured
                </Badge>
              )}
            </div>
            <Input
              id="openai-key"
              type="password"
              placeholder="sk-..."
              value={openaiKey}
              onChange={(e) => setOpenaiKey(e.target.value)}
              disabled={isSaving}
            />
          </div>

          {/* Anthropic */}
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <Label htmlFor="anthropic-key">Anthropic API Key</Label>
              {user?.has_anthropic_api_key && (
                <Badge variant="secondary" className="text-xs">
                  <Check className="h-3 w-3 mr-1" />
                  Configured
                </Badge>
              )}
            </div>
            <Input
              id="anthropic-key"
              type="password"
              placeholder="sk-ant-..."
              value={anthropicKey}
              onChange={(e) => setAnthropicKey(e.target.value)}
              disabled={isSaving}
            />
          </div>

          {/* Google */}
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <Label htmlFor="google-key">Google AI API Key</Label>
              {user?.has_google_api_key && (
                <Badge variant="secondary" className="text-xs">
                  <Check className="h-3 w-3 mr-1" />
                  Configured
                </Badge>
              )}
            </div>
            <Input
              id="google-key"
              type="password"
              placeholder="AIza..."
              value={googleKey}
              onChange={(e) => setGoogleKey(e.target.value)}
              disabled={isSaving}
            />
          </div>

          {/* xAI */}
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <Label htmlFor="xai-key">xAI API Key</Label>
              {user?.has_xai_api_key && (
                <Badge variant="secondary" className="text-xs">
                  <Check className="h-3 w-3 mr-1" />
                  Configured
                </Badge>
              )}
            </div>
            <Input
              id="xai-key"
              type="password"
              placeholder="xai-..."
              value={xaiKey}
              onChange={(e) => setXaiKey(e.target.value)}
              disabled={isSaving}
            />
          </div>
        </div>

        {/* AI Gateway Settings */}
        <div className="space-y-4 pt-4 border-t">
          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label htmlFor="gateway-enabled">Vercel AI Gateway</Label>
              <p className="text-sm text-muted-foreground">
                Route AI requests through Vercel AI Gateway for monitoring and caching
              </p>
            </div>
            <Switch
              id="gateway-enabled"
              checked={gatewayEnabled}
              onCheckedChange={setGatewayEnabled}
              disabled={isSaving}
            />
          </div>

          {gatewayEnabled && (
            <>
              <div className="space-y-2">
                <Label htmlFor="gateway-url">Gateway URL</Label>
                <Input
                  id="gateway-url"
                  type="url"
                  placeholder="https://gateway.vercel.com/..."
                  value={gatewayUrl}
                  onChange={(e) => setGatewayUrl(e.target.value)}
                  disabled={isSaving}
                />
              </div>

              <div className="space-y-2">
                <div className="flex items-center justify-between">
                  <Label htmlFor="gateway-key">Gateway Key</Label>
                  {user?.has_ai_gateway_key && (
                    <Badge variant="secondary" className="text-xs">
                      <Check className="h-3 w-3 mr-1" />
                      Configured
                    </Badge>
                  )}
                </div>
                <Input
                  id="gateway-key"
                  type="password"
                  placeholder="Gateway authentication key"
                  value={gatewayKey}
                  onChange={(e) => setGatewayKey(e.target.value)}
                  disabled={isSaving}
                />
              </div>
            </>
          )}
        </div>

        {/* Save Button */}
        <div className="pt-4 border-t">
          <Button
            onClick={handleSave}
            disabled={isSaving}
            className="w-full"
          >
            {isSaving ? (
              <>
                <RefreshCw className="mr-2 h-4 w-4 animate-spin" />
                Saving...
              </>
            ) : (
              <>
                <Key className="mr-2 h-4 w-4" />
                Save API Keys
              </>
            )}
          </Button>
        </div>
      </div>

      {/* Password Management Dialog */}
      <PasswordManagementDialog
        open={passwordDialogOpen}
        onOpenChange={setPasswordDialogOpen}
        mode={passwordDialogMode}
      />
    </div>
  );
}

// Database Settings Component
function DatabaseSettings() {
  const [isExporting, setIsExporting] = useState(false);
  const [isImporting, setIsImporting] = useState(false);
  const [selectedFile, setSelectedFile] = useState<File | null>(null);
  const [importResult, setImportResult] = useState<ImportResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const handleExport = async () => {
    setIsExporting(true);
    setError(null);

    try {
      const result = await exportDatabase();

      if (!result.success) {
        setError(result.error || 'Failed to export database');
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Export failed');
    } finally {
      setIsExporting(false);
    }
  };

  const handleFileSelect = (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (file) {
      // Validate file extension
      if (!file.name.endsWith('.gz')) {
        setError('Please select a valid backup file (.gz)');
        setSelectedFile(null);
        return;
      }

      setSelectedFile(file);
      setError(null);
      setImportResult(null);
    }
  };

  const handleImport = async () => {
    if (!selectedFile) return;

    setIsImporting(true);
    setError(null);
    setImportResult(null);

    try {
      const result = await importDatabase(selectedFile);

      if (result.success && result.data) {
        setImportResult(result.data);
        setSelectedFile(null);
        if (fileInputRef.current) {
          fileInputRef.current.value = '';
        }
      } else {
        setError(result.error || 'Import failed');
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Import failed');
    } finally {
      setIsImporting(false);
    }
  };

  return (
    <div className="rounded-lg border p-6">
      <div className="flex items-center gap-2 mb-4">
        <Database className="h-5 w-5 text-primary" />
        <h2 className="text-xl font-semibold">Database Backup</h2>
      </div>

      <div className="space-y-6">
        {/* Export Section */}
        <div className="space-y-3">
          <h3 className="text-sm font-medium">Export Database</h3>
          <p className="text-sm text-muted-foreground">
            Download a compressed backup of your Orkee database. This includes all projects and their configurations.
          </p>
          <Button
            onClick={handleExport}
            disabled={isExporting}
            className="w-full sm:w-auto"
          >
            {isExporting ? (
              <>
                <RefreshCw className="mr-2 h-4 w-4 animate-spin" />
                Exporting...
              </>
            ) : (
              <>
                <Download className="mr-2 h-4 w-4" />
                Export Database
              </>
            )}
          </Button>
        </div>

        {/* Import Section */}
        <div className="space-y-3 pt-3 border-t">
          <h3 className="text-sm font-medium">Import Database</h3>
          <p className="text-sm text-muted-foreground">
            Restore your database from a backup file. This will merge imported projects with existing ones.
          </p>

          {/* Warning */}
          <Alert>
            <AlertTriangle className="h-4 w-4" />
            <AlertDescription>
              Importing will merge data with your current database. Projects with duplicate names or paths may be skipped.
            </AlertDescription>
          </Alert>

          {/* File Input */}
          <div className="flex flex-col sm:flex-row gap-2">
            <Input
              ref={fileInputRef}
              type="file"
              accept=".gz"
              onChange={handleFileSelect}
              className="flex-1"
            />
            <Button
              onClick={handleImport}
              disabled={!selectedFile || isImporting}
              className="w-full sm:w-auto"
            >
              {isImporting ? (
                <>
                  <RefreshCw className="mr-2 h-4 w-4 animate-spin" />
                  Importing...
                </>
              ) : (
                <>
                  <Upload className="mr-2 h-4 w-4" />
                  Import Database
                </>
              )}
            </Button>
          </div>

          {/* Selected File Info */}
          {selectedFile && (
            <p className="text-sm text-muted-foreground">
              Selected: {selectedFile.name} ({(selectedFile.size / 1024).toFixed(2)} KB)
            </p>
          )}
        </div>

        {/* Error Display */}
        {error && (
          <Alert variant="destructive">
            <AlertTriangle className="h-4 w-4" />
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        )}

        {/* Import Results */}
        {importResult && (
          <Alert>
            <Database className="h-4 w-4" />
            <AlertDescription>
              <div className="space-y-1">
                <p className="font-medium">Import completed successfully!</p>
                <ul className="text-sm space-y-1 mt-2">
                  <li>✓ {importResult.projectsImported} projects imported</li>
                  {importResult.projectsSkipped > 0 && (
                    <li>⊘ {importResult.projectsSkipped} projects skipped</li>
                  )}
                  {importResult.conflictsCount > 0 && (
                    <li className="text-orange-600">⚠ {importResult.conflictsCount} conflicts detected</li>
                  )}
                </ul>
                {importResult.conflicts.length > 0 && (
                  <details className="mt-3">
                    <summary className="cursor-pointer text-sm font-medium">View conflicts</summary>
                    <ul className="mt-2 space-y-1 text-sm">
                      {importResult.conflicts.map((conflict, idx) => (
                        <li key={idx}>
                          {conflict.projectName} ({conflict.conflictType})
                        </li>
                      ))}
                    </ul>
                  </details>
                )}
              </div>
            </AlertDescription>
          </Alert>
        )}
      </div>
    </div>
  );
}

// Privacy & Telemetry Settings Component
function PrivacySettings() {
  const { settings, updateSettings, deleteAllData, trackAction, loading, error: contextError } = useTelemetry();
  const [isSaving, setIsSaving] = useState(false);
  const [isDeleting, setIsDeleting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleToggle = async (field: keyof typeof settings, value: boolean) => {
    if (!settings) return;

    setIsSaving(true);
    setError(null);
    try {
      const newSettings = {
        ...settings,
        [field]: value,
      };

      // If disabling non-anonymous metrics, ensure it's disabled
      if (field === 'usage_metrics' && !value) {
        newSettings.non_anonymous_metrics = false;
      }

      await updateSettings(newSettings);

      // Track the change
      trackAction('telemetry_settings_changed', {
        setting: field,
        enabled: value,
      });
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to update telemetry settings';
      setError(errorMessage);
      console.error('Failed to update telemetry settings:', err);
    } finally {
      setIsSaving(false);
    }
  };

  const handleDeleteAllData = async () => {
    const confirmed = window.confirm(
      'Are you sure you want to delete all telemetry data? This action cannot be undone.'
    );

    if (!confirmed) return;

    setIsDeleting(true);
    setError(null);
    try {
      await deleteAllData();
      alert('All telemetry data has been deleted successfully.');
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to delete telemetry data';
      setError(errorMessage);
      alert(`Failed to delete telemetry data: ${errorMessage}`);
      console.error('Failed to delete telemetry data:', err);
    } finally {
      setIsDeleting(false);
    }
  };

  // Show loading state while fetching settings
  if (loading) {
    return (
      <div className="rounded-lg border p-6">
        <div className="flex items-center gap-2 mb-4">
          <Shield className="h-5 w-5 text-primary" />
          <h2 className="text-xl font-semibold">Privacy & Telemetry</h2>
        </div>
        <div className="flex items-center justify-center py-8">
          <RefreshCw className="h-6 w-6 animate-spin text-muted-foreground" />
          <span className="ml-2 text-muted-foreground">Loading settings...</span>
        </div>
      </div>
    );
  }

  // Show error state if context failed to load
  if (contextError) {
    return (
      <div className="rounded-lg border p-6">
        <div className="flex items-center gap-2 mb-4">
          <Shield className="h-5 w-5 text-primary" />
          <h2 className="text-xl font-semibold">Privacy & Telemetry</h2>
        </div>
        <Alert variant="destructive">
          <AlertTriangle className="h-4 w-4" />
          <AlertDescription>
            Failed to load telemetry settings: {contextError}
          </AlertDescription>
        </Alert>
      </div>
    );
  }

  if (!settings) {
    return null;
  }

  return (
    <div className="rounded-lg border p-6">
      <div className="flex items-center gap-2 mb-4">
        <Shield className="h-5 w-5 text-primary" />
        <h2 className="text-xl font-semibold">Privacy & Telemetry</h2>
      </div>

      <div className="space-y-6">
        <Alert>
          <Shield className="h-4 w-4" />
          <AlertDescription>
            We respect your privacy. All telemetry is opt-in and you can change your preferences anytime.
            We never collect project names, file paths, or personal data.
          </AlertDescription>
        </Alert>

        {/* Error Display */}
        {error && (
          <Alert variant="destructive">
            <AlertTriangle className="h-4 w-4" />
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        )}

        {/* Telemetry Options */}
        <div className="space-y-4">
          {/* Error Reporting */}
          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label htmlFor="privacy-error">Error Reporting</Label>
              <p className="text-sm text-muted-foreground">
                Help us fix bugs by sharing crash reports and error logs
              </p>
            </div>
            <Switch
              id="privacy-error"
              checked={settings.error_reporting}
              onCheckedChange={(value) => handleToggle('error_reporting', value)}
              disabled={isSaving}
            />
          </div>

          {/* Usage Metrics */}
          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label htmlFor="privacy-usage">Usage Metrics</Label>
              <p className="text-sm text-muted-foreground">
                Share anonymous usage statistics to help improve the product
              </p>
            </div>
            <Switch
              id="privacy-usage"
              checked={settings.usage_metrics}
              onCheckedChange={(value) => handleToggle('usage_metrics', value)}
              disabled={isSaving}
            />
          </div>

          {/* Non-anonymous Metrics */}
          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label htmlFor="privacy-non-anonymous">Non-anonymous Metrics</Label>
              <p className="text-sm text-muted-foreground">
                Include an identifier to help us provide better support
              </p>
            </div>
            <Switch
              id="privacy-non-anonymous"
              checked={settings.non_anonymous_metrics}
              onCheckedChange={(value) => handleToggle('non_anonymous_metrics', value)}
              disabled={isSaving || !settings.usage_metrics}
            />
          </div>
        </div>

        {/* Data Management */}
        <div className="pt-4 border-t">
          <h3 className="text-sm font-medium mb-3">Data Management</h3>
          <div className="flex gap-2">
            <Button
              variant="destructive"
              size="sm"
              onClick={handleDeleteAllData}
              disabled={isDeleting}
            >
              {isDeleting ? (
                <>
                  <RefreshCw className="mr-2 h-4 w-4 animate-spin" />
                  Deleting...
                </>
              ) : (
                <>
                  <Trash2 className="mr-2 h-4 w-4" />
                  Delete All Telemetry Data
                </>
              )}
            </Button>
          </div>
          <p className="text-xs text-muted-foreground mt-2">
            Permanently remove all collected telemetry data from our servers
          </p>
        </div>

        {/* Status */}
        <div className="text-xs text-muted-foreground">
          <p>
            Status:{' '}
            {settings.error_reporting || settings.usage_metrics
              ? 'Telemetry is active'
              : 'All telemetry is disabled'}
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
  const [cloudEnabled, setCloudEnabled] = useState(false);
  const [isEnabling, setIsEnabling] = useState(false);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    fetchConfig().then(config => {
      setCloudEnabled(config.cloud_enabled);
      setIsLoading(false);
    }).catch(() => {
      setIsLoading(false);
    });
  }, []);

  const handleToggleCloud = async (enabled: boolean) => {
    setIsEnabling(true);
    try {
      await updateSetting('cloud_enabled', enabled ? 'true' : 'false');
      setCloudEnabled(enabled);
      // Clear config cache to force reload
      clearConfigCache();
      // Optionally reload the page to apply changes
      // window.location.reload();
    } catch (error) {
      console.error('Failed to toggle cloud:', error);
      alert('Failed to update cloud setting');
    } finally {
      setIsEnabling(false);
    }
  };

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

  if (isLoading) {
    return (
      <div className="rounded-lg border p-6">
        <div className="flex items-center gap-2 mb-4">
          <Cloud className="h-5 w-5 text-primary" />
          <h2 className="text-xl font-semibold">Cloud Sync</h2>
        </div>
        <p className="text-muted-foreground">Loading...</p>
      </div>
    );
  }

  return (
    <div className="rounded-lg border p-6">
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-2">
          <Cloud className="h-5 w-5 text-primary" />
          <h2 className="text-xl font-semibold">Cloud Sync</h2>
          {isAuthenticated && cloudEnabled && (
            <Badge variant="secondary" className="ml-2">
              Connected
            </Badge>
          )}
          {!cloudEnabled && (
            <Badge variant="outline" className="ml-2 text-gray-500">
              Disabled
            </Badge>
          )}
        </div>
        <div className="flex items-center gap-2">
          <Label htmlFor="cloud-toggle" className="text-sm font-medium">Enable Cloud Sync</Label>
          <Switch
            id="cloud-toggle"
            checked={cloudEnabled}
            onCheckedChange={handleToggleCloud}
            disabled={isEnabling}
          />
        </div>
      </div>
      
      {!cloudEnabled ? (
        <div className="space-y-4">
          <Alert>
            <Cloud className="h-4 w-4" />
            <AlertDescription>
              <p className="font-medium mb-2">Cloud Sync is currently disabled</p>
              <p className="text-sm text-muted-foreground mb-3">
                Enable cloud sync to back up and synchronize your projects across devices. 
                Your data stays secure with end-to-end encryption.
              </p>
              <div className="space-y-2 text-sm">
                <p className="font-medium">Features:</p>
                <ul className="list-disc list-inside space-y-1 text-muted-foreground">
                  <li>Automatic project backup</li>
                  <li>Multi-device synchronization</li>
                  <li>Secure cloud storage</li>
                  <li>Access from anywhere</li>
                  <li>Version history</li>
                </ul>
              </div>
            </AlertDescription>
          </Alert>
          <div className="p-4 bg-blue-50 border border-blue-200 rounded-md">
            <p className="text-sm text-blue-800">
              <strong>Free Plan:</strong> Up to 2 projects, 100MB storage<br/>
              <strong>Starter:</strong> 10 projects, 5GB storage<br/>
              <strong>Pro:</strong> Unlimited projects, 50GB storage
            </p>
          </div>
        </div>
      ) : (
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
      )}
    </div>
  );
}

// Advanced Settings Component with nested tabs
function AdvancedSettings() {
  return (
    <div className="rounded-lg border p-6">
      <div className="flex items-center gap-2 mb-4">
        <Sliders className="h-5 w-5 text-primary" />
        <h2 className="text-xl font-semibold">Advanced Configuration</h2>
      </div>
      
      <div className="text-sm text-muted-foreground mb-6">
        Configure server, security, and runtime settings for Orkee. 
        Changes marked with <Badge variant="secondary" className="text-xs mx-1">Requires Restart</Badge> need an application restart to take effect.
      </div>

      <Tabs defaultValue="server" className="w-full">
        <TabsList className="grid w-full grid-cols-4">
          <TabsTrigger value="server">Server</TabsTrigger>
          <TabsTrigger value="security">Security</TabsTrigger>
          <TabsTrigger value="rate_limiting">Rate Limiting</TabsTrigger>
          <TabsTrigger value="tls">TLS/HTTPS</TabsTrigger>
        </TabsList>

        <TabsContent value="server" className="space-y-4 mt-4">
          <ServerConfigSection />
        </TabsContent>

        <TabsContent value="security" className="space-y-4 mt-4">
          <SecurityConfigSection />
        </TabsContent>

        <TabsContent value="rate_limiting" className="space-y-4 mt-4">
          <RateLimitingConfigSection />
        </TabsContent>

        <TabsContent value="tls" className="space-y-4 mt-4">
          <TlsConfigSection />
        </TabsContent>
      </Tabs>
    </div>
  );
}

// Server Config Section
function ServerConfigSection() {
  const [settings, setSettings] = useState<SystemSetting[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [isSaving, setIsSaving] = useState(false);

  useEffect(() => {
    loadSettings();
  }, []);

  const loadSettings = async () => {
    try {
      const response = await getSettingsByCategory('server');
      setSettings(response.settings);
    } catch (error) {
      console.error('Failed to load server settings:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const validateSettingValue = (key: string, value: string, dataType: string): string | null => {
    // Check for empty value
    if (!value || value.trim() === '') {
      return 'Value cannot be empty';
    }

    // Validate by data type
    if (dataType === 'boolean') {
      if (value !== 'true' && value !== 'false') {
        return 'Must be "true" or "false"';
      }
    } else if (dataType === 'integer') {
      const num = parseInt(value, 10);
      if (isNaN(num)) {
        return 'Must be a valid integer';
      }
    }

    // Setting-specific validation
    if (key === 'api_port' || key === 'ui_port') {
      const port = parseInt(value, 10);
      if (isNaN(port) || port < 1 || port > 65535) {
        return 'Port must be between 1 and 65535';
      }
    } else if (key === 'browse_sandbox_mode') {
      if (!['strict', 'relaxed', 'disabled'].includes(value)) {
        return 'Must be one of: strict, relaxed, disabled';
      }
    } else if (key.startsWith('rate_limit_') || key === 'rate_limit_burst_size') {
      const num = parseInt(value, 10);
      if (isNaN(num) || num < 1 || num > 10000) {
        return 'Must be between 1 and 10,000';
      }
    } else if (key === 'cloud_api_url') {
      try {
        new URL(value);
      } catch {
        return 'Must be a valid URL';
      }
    }

    return null;
  };

  const handleUpdateSetting = async (key: string, value: string) => {
    // Find the setting to get its data type
    const setting = settings.find(s => s.key === key);
    if (!setting) {
      alert('Setting not found');
      return;
    }

    // Validate before sending to server
    const validationError = validateSettingValue(key, value, setting.data_type);
    if (validationError) {
      alert(`Validation error: ${validationError}`);
      return;
    }

    setIsSaving(true);
    try {
      await updateSetting(key, value);
      await loadSettings();
    } catch (error) {
      console.error('Failed to update setting:', error);
      const errorMessage = error instanceof Error ? error.message : 'Failed to update setting';
      alert(errorMessage);
    } finally {
      setIsSaving(false);
    }
  };

  if (isLoading) {
    return <div className="text-muted-foreground">Loading...</div>;
  }

  return (
    <div className="space-y-4">
      {settings.length > 0 && settings.some(s => s.is_env_only) && (
        <Alert>
          <AlertTriangle className="h-4 w-4" />
          <AlertDescription>
            Settings marked as <Badge variant="outline" className="text-xs text-amber-600 border-amber-600 inline-flex items-center">Read-Only (.env)</Badge> must be configured in your <code className="text-xs bg-gray-100 px-1 py-0.5 rounded">.env</code> file before server startup.
          </AlertDescription>
        </Alert>
      )}
      {settings.map((setting) => (
        <div key={setting.key} className="space-y-2">
          <div className="flex items-center justify-between">
            <Label htmlFor={setting.key} className="text-sm font-medium">
              {setting.key.split('_').map(w => w.charAt(0).toUpperCase() + w.slice(1)).join(' ')}
              {setting.requires_restart && (
                <Badge variant="secondary" className="ml-2 text-xs">Requires Restart</Badge>
              )}
              {setting.is_env_only && (
                <Badge variant="outline" className="ml-2 text-xs text-amber-600 border-amber-600">Read-Only (.env)</Badge>
              )}
            </Label>
          </div>
          <Input
            id={setting.key}
            type={setting.data_type === 'integer' ? 'number' : 'text'}
            value={setting.value}
            onChange={(e) => handleUpdateSetting(setting.key, e.target.value)}
            disabled={isSaving || setting.is_env_only}
            className="w-full"
            readOnly={setting.is_env_only}
          />
          {setting.description && (
            <p className="text-xs text-muted-foreground">{setting.description}</p>
          )}
        </div>
      ))}
    </div>
  );
}

// Security Config Section
function SecurityConfigSection() {
  const [settings, setSettings] = useState<SystemSetting[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [isSaving, setIsSaving] = useState(false);

  useEffect(() => {
    loadSettings();
  }, []);

  const loadSettings = async () => {
    try {
      const response = await getSettingsByCategory('security');
      setSettings(response.settings);
    } catch (error) {
      console.error('Failed to load security settings:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const handleUpdateSetting = async (key: string, value: string) => {
    setIsSaving(true);
    try {
      await updateSetting(key, value);
      await loadSettings();
    } catch (error) {
      console.error('Failed to update setting:', error);
      alert('Failed to update setting');
    } finally {
      setIsSaving(false);
    }
  };

  const handleToggleSetting = async (key: string, checked: boolean) => {
    await handleUpdateSetting(key, checked ? 'true' : 'false');
  };

  if (isLoading) {
    return <div className="text-muted-foreground">Loading...</div>;
  }

  return (
    <div className="space-y-4">
      {settings.map((setting) => (
        <div key={setting.key} className="space-y-2">
          {setting.data_type === 'boolean' ? (
            <div className="flex items-center justify-between p-3 border rounded-md">
              <div className="space-y-0.5">
                <Label htmlFor={setting.key} className="text-sm font-medium">
                  {setting.key.split('_').map(w => w.charAt(0).toUpperCase() + w.slice(1)).join(' ')}
                  {setting.requires_restart && (
                    <Badge variant="secondary" className="ml-2 text-xs">Requires Restart</Badge>
                  )}
                </Label>
                {setting.description && (
                  <p className="text-xs text-muted-foreground">{setting.description}</p>
                )}
              </div>
              <Switch
                id={setting.key}
                checked={setting.value === 'true'}
                onCheckedChange={(checked) => handleToggleSetting(setting.key, checked)}
                disabled={isSaving}
              />
            </div>
          ) : (
            <div className="space-y-2">
              <Label htmlFor={setting.key} className="text-sm font-medium">
                {setting.key.split('_').map(w => w.charAt(0).toUpperCase() + w.slice(1)).join(' ')}
                {setting.requires_restart && (
                  <Badge variant="secondary" className="ml-2 text-xs">Requires Restart</Badge>
                )}
              </Label>
              <Input
                id={setting.key}
                type="text"
                value={setting.value}
                onChange={(e) => handleUpdateSetting(setting.key, e.target.value)}
                disabled={isSaving}
                className="w-full"
              />
              {setting.description && (
                <p className="text-xs text-muted-foreground">{setting.description}</p>
              )}
            </div>
          )}
        </div>
      ))}
    </div>
  );
}

// Rate Limiting Config Section
function RateLimitingConfigSection() {
  const [settings, setSettings] = useState<SystemSetting[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [isSaving, setIsSaving] = useState(false);

  useEffect(() => {
    loadSettings();
  }, []);

  const loadSettings = async () => {
    try {
      const response = await getSettingsByCategory('rate_limiting');
      setSettings(response.settings);
    } catch (error) {
      console.error('Failed to load rate limiting settings:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const handleUpdateSetting = async (key: string, value: string) => {
    setIsSaving(true);
    try {
      await updateSetting(key, value);
      await loadSettings();
    } catch (error) {
      console.error('Failed to update setting:', error);
      alert('Failed to update setting');
    } finally {
      setIsSaving(false);
    }
  };

  const handleToggleSetting = async (key: string, checked: boolean) => {
    await handleUpdateSetting(key, checked ? 'true' : 'false');
  };

  if (isLoading) {
    return <div className="text-muted-foreground">Loading...</div>;
  }

  return (
    <div className="space-y-4">
      <Alert>
        <AlertTriangle className="h-4 w-4" />
        <AlertDescription>
          Rate limiting helps prevent API abuse. Values are in requests per minute (RPM).
        </AlertDescription>
      </Alert>
      {settings.map((setting) => (
        <div key={setting.key} className="space-y-2">
          {setting.data_type === 'boolean' ? (
            <div className="flex items-center justify-between p-3 border rounded-md">
              <div className="space-y-0.5">
                <Label htmlFor={setting.key} className="text-sm font-medium">
                  {setting.key.split('_').slice(2).map(w => w.charAt(0).toUpperCase() + w.slice(1)).join(' ')}
                  {setting.requires_restart && (
                    <Badge variant="secondary" className="ml-2 text-xs">Requires Restart</Badge>
                  )}
                </Label>
                {setting.description && (
                  <p className="text-xs text-muted-foreground">{setting.description}</p>
                )}
              </div>
              <Switch
                id={setting.key}
                checked={setting.value === 'true'}
                onCheckedChange={(checked) => handleToggleSetting(setting.key, checked)}
                disabled={isSaving}
              />
            </div>
          ) : (
            <div className="space-y-2">
              <Label htmlFor={setting.key} className="text-sm font-medium">
                {setting.key.split('_').slice(2).map(w => w.charAt(0).toUpperCase() + w.slice(1)).join(' ')}
                {setting.requires_restart && (
                  <Badge variant="secondary" className="ml-2 text-xs">Requires Restart</Badge>
                )}
              </Label>
              <Input
                id={setting.key}
                type="number"
                value={setting.value}
                onChange={(e) => handleUpdateSetting(setting.key, e.target.value)}
                disabled={isSaving}
                className="w-full"
              />
              {setting.description && (
                <p className="text-xs text-muted-foreground">{setting.description}</p>
              )}
            </div>
          )}
        </div>
      ))}
    </div>
  );
}

// TLS Config Section
function TlsConfigSection() {
  const [settings, setSettings] = useState<SystemSetting[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [isSaving, setIsSaving] = useState(false);

  useEffect(() => {
    loadSettings();
  }, []);

  const loadSettings = async () => {
    try {
      const response = await getSettingsByCategory('tls');
      setSettings(response.settings);
    } catch (error) {
      console.error('Failed to load TLS settings:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const handleUpdateSetting = async (key: string, value: string) => {
    setIsSaving(true);
    try {
      await updateSetting(key, value);
      await loadSettings();
    } catch (error) {
      console.error('Failed to update setting:', error);
      alert('Failed to update setting');
    } finally {
      setIsSaving(false);
    }
  };

  const handleToggleSetting = async (key: string, checked: boolean) => {
    await handleUpdateSetting(key, checked ? 'true' : 'false');
  };

  if (isLoading) {
    return <div className="text-muted-foreground">Loading...</div>;
  }

  return (
    <div className="space-y-4">
      <Alert>
        <Shield className="h-4 w-4" />
        <AlertDescription>
          TLS/HTTPS configuration for secure connections. Development mode uses self-signed certificates.
        </AlertDescription>
      </Alert>
      {settings.map((setting) => (
        <div key={setting.key} className="space-y-2">
          {setting.data_type === 'boolean' ? (
            <div className="flex items-center justify-between p-3 border rounded-md">
              <div className="space-y-0.5">
                <Label htmlFor={setting.key} className="text-sm font-medium">
                  {setting.key.split('_').map(w => w.charAt(0).toUpperCase() + w.slice(1)).join(' ')}
                  {setting.requires_restart && (
                    <Badge variant="secondary" className="ml-2 text-xs">Requires Restart</Badge>
                  )}
                </Label>
                {setting.description && (
                  <p className="text-xs text-muted-foreground">{setting.description}</p>
                )}
              </div>
              <Switch
                id={setting.key}
                checked={setting.value === 'true'}
                onCheckedChange={(checked) => handleToggleSetting(setting.key, checked)}
                disabled={isSaving}
              />
            </div>
          ) : (
            <div className="space-y-2">
              <Label htmlFor={setting.key} className="text-sm font-medium">
                {setting.key.split('_').map(w => w.charAt(0).toUpperCase() + w.slice(1)).join(' ')}
                {setting.requires_restart && (
                  <Badge variant="secondary" className="ml-2 text-xs">Requires Restart</Badge>
                )}
              </Label>
              <Input
                id={setting.key}
                type="text"
                value={setting.value}
                onChange={(e) => handleUpdateSetting(setting.key, e.target.value)}
                disabled={isSaving}
                className="w-full"
              />
              {setting.description && (
                <p className="text-xs text-muted-foreground">{setting.description}</p>
              )}
            </div>
          )}
        </div>
      ))}
    </div>
  );
}