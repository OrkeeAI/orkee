import { Button } from '@/components/ui/button'
import { Settings as SettingsIcon, Key, Bell, Palette, Download, Upload } from 'lucide-react'

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