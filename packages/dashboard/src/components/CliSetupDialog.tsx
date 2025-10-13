import { useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { Terminal, RefreshCw, CheckCircle2, AlertCircle } from 'lucide-react'
import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Alert, AlertDescription } from '@/components/ui/alert'

interface CliSetupDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
}

type InstallState = 'idle' | 'installing' | 'success' | 'error'

export function CliSetupDialog({ open, onOpenChange }: CliSetupDialogProps) {
  const [installState, setInstallState] = useState<InstallState>('idle')
  const [errorMessage, setErrorMessage] = useState<string>('')

  const handleInstallNow = async () => {
    setInstallState('installing')
    setErrorMessage('')

    try {
      await invoke<string>('install_cli_macos')
      setInstallState('success')

      // Save preference to "never" show again after successful install
      await invoke('set_cli_prompt_preference', { preference: 'never' })

      // Close dialog after short delay to show success message
      setTimeout(() => {
        onOpenChange(false)
      }, 2000)
    } catch (error) {
      setInstallState('error')
      setErrorMessage(error as string)
    }
  }

  const handleRemindLater = async () => {
    await invoke('set_cli_prompt_preference', { preference: 'later' })
    onOpenChange(false)
  }

  const handleDontShowAgain = async () => {
    await invoke('set_cli_prompt_preference', { preference: 'never' })
    onOpenChange(false)
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <div className="flex items-center gap-2">
            <Terminal className="h-5 w-5 text-primary" />
            <DialogTitle>Enable CLI Access</DialogTitle>
          </div>
          <DialogDescription className="space-y-2 pt-2">
            <p>
              Orkee includes powerful command-line tools that can be accessed from any terminal.
            </p>
            <div className="bg-muted p-3 rounded-md text-sm space-y-1">
              <p className="font-medium">Available features:</p>
              <ul className="list-disc list-inside space-y-1 text-muted-foreground">
                <li>CLI commands: <code className="text-xs bg-background px-1 rounded">orkee projects list</code></li>
                <li>TUI interface: <code className="text-xs bg-background px-1 rounded">orkee tui</code></li>
                <li>Full dashboard: <code className="text-xs bg-background px-1 rounded">orkee dashboard</code></li>
              </ul>
            </div>
            <p className="text-sm">
              This will copy the orkee binary to <code className="text-xs bg-muted px-1 rounded">/usr/local/bin</code> and requires admin privileges.
            </p>
          </DialogDescription>
        </DialogHeader>

        {installState === 'success' && (
          <Alert className="bg-green-50 border-green-200">
            <CheckCircle2 className="h-4 w-4 text-green-600" />
            <AlertDescription className="text-green-800">
              CLI successfully installed! You can now use <code className="text-xs bg-green-100 px-1 rounded">orkee</code> commands in any terminal.
            </AlertDescription>
          </Alert>
        )}

        {installState === 'error' && (
          <Alert variant="destructive">
            <AlertCircle className="h-4 w-4" />
            <AlertDescription>
              <p className="font-medium mb-1">Installation failed</p>
              <p className="text-sm">{errorMessage}</p>
              <p className="text-sm mt-2">
                You can install manually by running:
              </p>
              <code className="block mt-1 text-xs bg-destructive/10 p-2 rounded">
                sudo cp /Applications/Orkee.app/Contents/MacOS/orkee /usr/local/bin/orkee
              </code>
            </AlertDescription>
          </Alert>
        )}

        <DialogFooter className="flex-col sm:flex-row gap-2">
          {installState === 'idle' || installState === 'error' ? (
            <>
              <Button
                variant="outline"
                onClick={handleDontShowAgain}
                className="sm:w-auto w-full"
              >
                Don't Show Again
              </Button>
              <Button
                variant="ghost"
                onClick={handleRemindLater}
                className="sm:w-auto w-full"
              >
                Remind Me Later
              </Button>
              <Button
                onClick={handleInstallNow}
                disabled={installState === 'installing'}
                className="sm:w-auto w-full"
              >
                {installState === 'installing' ? (
                  <>
                    <RefreshCw className="mr-2 h-4 w-4 animate-spin" />
                    Installing...
                  </>
                ) : (
                  <>
                    <Terminal className="mr-2 h-4 w-4" />
                    Install Now
                  </>
                )}
              </Button>
            </>
          ) : null}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
