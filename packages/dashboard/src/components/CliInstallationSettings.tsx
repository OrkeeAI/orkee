import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { Terminal, CheckCircle2, XCircle, RefreshCw, ExternalLink } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Alert, AlertDescription } from '@/components/ui/alert'

export function CliInstallationSettings() {
  const [isInstalled, setIsInstalled] = useState<boolean | null>(null)
  const [isInstalling, setIsInstalling] = useState(false)
  const [installMessage, setInstallMessage] = useState<{ type: 'success' | 'error', text: string } | null>(null)
  const [isChecking, setIsChecking] = useState(false)

  const checkCliStatus = async () => {
    setIsChecking(true)
    try {
      const installed = await invoke<boolean>('check_cli_installed')
      setIsInstalled(installed)
    } catch (error) {
      console.error('Failed to check CLI status:', error)
      setIsInstalled(false)
    } finally {
      setIsChecking(false)
    }
  }

  useEffect(() => {
    checkCliStatus()
  }, [])

  const handleInstall = async () => {
    setIsInstalling(true)
    setInstallMessage(null)

    try {
      const result = await invoke<string>('install_cli_macos')
      setInstallMessage({ type: 'success', text: result })
      setIsInstalled(true)
    } catch (error) {
      setInstallMessage({ type: 'error', text: error as string })
    } finally {
      setIsInstalling(false)
    }
  }

  const handleResetPreference = async () => {
    try {
      await invoke('set_cli_prompt_preference', { preference: 'show' })
      alert('Preference reset! The installation prompt will be shown on next launch.')
    } catch (error) {
      console.error('Failed to reset preference:', error)
      alert('Failed to reset preference: ' + error)
    }
  }

  return (
    <div className="rounded-lg border p-6">
      <div className="flex items-center gap-2 mb-4">
        <Terminal className="h-5 w-5 text-primary" />
        <h2 className="text-xl font-semibold">CLI Installation</h2>
        {isInstalled !== null && (
          <Badge variant={isInstalled ? "default" : "secondary"} className="ml-2">
            {isInstalled ? "Installed" : "Not Installed"}
          </Badge>
        )}
      </div>

      <div className="space-y-6">
        {/* Status Section */}
        <div className="space-y-3">
          <p className="text-sm text-muted-foreground">
            The orkee CLI provides powerful command-line tools and a terminal UI for managing your projects.
          </p>

          <div className="flex items-center gap-3 p-3 bg-muted rounded-md">
            <div className="flex-1">
              {isChecking ? (
                <div className="flex items-center gap-2 text-sm">
                  <RefreshCw className="h-4 w-4 animate-spin" />
                  <span>Checking CLI status...</span>
                </div>
              ) : isInstalled ? (
                <div className="flex items-center gap-2 text-sm">
                  <CheckCircle2 className="h-4 w-4 text-green-600" />
                  <span className="text-green-800">CLI is installed and available in your PATH</span>
                </div>
              ) : (
                <div className="flex items-center gap-2 text-sm">
                  <XCircle className="h-4 w-4 text-orange-600" />
                  <span className="text-orange-800">CLI is not installed</span>
                </div>
              )}
            </div>
            <Button
              variant="ghost"
              size="sm"
              onClick={checkCliStatus}
              disabled={isChecking}
            >
              <RefreshCw className={`h-4 w-4 ${isChecking ? 'animate-spin' : ''}`} />
            </Button>
          </div>
        </div>

        {/* Installation Results */}
        {installMessage && (
          <Alert variant={installMessage.type === 'error' ? 'destructive' : undefined} className={installMessage.type === 'success' ? 'bg-green-50 border-green-200' : ''}>
            {installMessage.type === 'success' ? (
              <CheckCircle2 className="h-4 w-4 text-green-600" />
            ) : (
              <XCircle className="h-4 w-4" />
            )}
            <AlertDescription className={installMessage.type === 'success' ? 'text-green-800' : ''}>
              {installMessage.text}
            </AlertDescription>
          </Alert>
        )}

        {/* Actions */}
        <div className="space-y-3">
          <h3 className="text-sm font-medium">Actions</h3>
          <div className="flex flex-wrap gap-2">
            {!isInstalled && (
              <Button
                onClick={handleInstall}
                disabled={isInstalling || isChecking}
              >
                {isInstalling ? (
                  <>
                    <RefreshCw className="mr-2 h-4 w-4 animate-spin" />
                    Installing...
                  </>
                ) : (
                  <>
                    <Terminal className="mr-2 h-4 w-4" />
                    Install CLI
                  </>
                )}
              </Button>
            )}

            <Button
              variant="outline"
              size="sm"
              onClick={handleResetPreference}
            >
              Reset Install Prompt
            </Button>

            <Button
              variant="outline"
              size="sm"
              asChild
            >
              <a
                href="https://github.com/OrkeeAI/orkee/blob/main/packages/dashboard/src-tauri/INSTALLER_README.md#macos-manual-cli-setup"
                target="_blank"
                rel="noopener noreferrer"
              >
                <ExternalLink className="mr-2 h-4 w-4" />
                Manual Install Guide
              </a>
            </Button>
          </div>
        </div>

        {/* Manual Instructions */}
        {!isInstalled && (
          <div className="space-y-3 pt-3 border-t">
            <h3 className="text-sm font-medium">Manual Installation</h3>
            <p className="text-sm text-muted-foreground">
              If automatic installation fails, you can install manually:
            </p>
            <div className="bg-muted p-3 rounded-md">
              <code className="text-xs block">
                sudo cp /Applications/Orkee.app/Contents/MacOS/orkee /usr/local/bin/orkee
              </code>
              <code className="text-xs block mt-1">
                sudo chmod +x /usr/local/bin/orkee
              </code>
            </div>
          </div>
        )}

        {/* CLI Features */}
        <div className="space-y-3 pt-3 border-t">
          <h3 className="text-sm font-medium">Available Commands</h3>
          <div className="space-y-2 text-sm">
            <div className="flex items-start gap-2">
              <Terminal className="h-4 w-4 mt-0.5 text-muted-foreground" />
              <div>
                <code className="text-xs bg-muted px-2 py-1 rounded">orkee projects list</code>
                <p className="text-muted-foreground text-xs mt-1">Manage your projects from the command line</p>
              </div>
            </div>
            <div className="flex items-start gap-2">
              <Terminal className="h-4 w-4 mt-0.5 text-muted-foreground" />
              <div>
                <code className="text-xs bg-muted px-2 py-1 rounded">orkee tui</code>
                <p className="text-muted-foreground text-xs mt-1">Launch the full-featured terminal UI</p>
              </div>
            </div>
            <div className="flex items-start gap-2">
              <Terminal className="h-4 w-4 mt-0.5 text-muted-foreground" />
              <div>
                <code className="text-xs bg-muted px-2 py-1 rounded">orkee dashboard</code>
                <p className="text-muted-foreground text-xs mt-1">Start the web dashboard with backend server</p>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}
