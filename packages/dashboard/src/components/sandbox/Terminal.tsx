// ABOUTME: Terminal component using xterm.js for sandbox command execution
// ABOUTME: Provides interactive terminal with command history and output display

import { useEffect, useRef, useState } from 'react'
import { Terminal as XTerm } from '@xterm/xterm'
import { FitAddon } from '@xterm/addon-fit'
import { WebLinksAddon } from '@xterm/addon-web-links'
import '@xterm/xterm/css/xterm.css'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { useToast } from '@/hooks/use-toast'
import { executeCommand, getSandboxExecutions, type SandboxExecution } from '@/services/sandbox'
import { Send, RotateCcw, Trash2 } from 'lucide-react'

interface TerminalProps {
  sandboxId: string
  agentId?: string | null
  model?: string | null
}

export function Terminal({ sandboxId, agentId, model }: TerminalProps) {
  const { toast } = useToast()
  const terminalRef = useRef<HTMLDivElement>(null)
  const xtermRef = useRef<XTerm | null>(null)
  const fitAddonRef = useRef<FitAddon | null>(null)
  const [command, setCommand] = useState('')
  const [isExecuting, setIsExecuting] = useState(false)
  const [executions, setExecutions] = useState<SandboxExecution[]>([])

  // Initialize terminal
  useEffect(() => {
    if (!terminalRef.current) return

    const term = new XTerm({
      cursorBlink: true,
      fontSize: 14,
      fontFamily: 'Menlo, Monaco, "Courier New", monospace',
      theme: {
        background: '#1e1e1e',
        foreground: '#d4d4d4',
        cursor: '#ffffff',
      },
      rows: 24,
    })

    const fitAddon = new FitAddon()
    const webLinksAddon = new WebLinksAddon()

    term.loadAddon(fitAddon)
    term.loadAddon(webLinksAddon)
    term.open(terminalRef.current)

    fitAddon.fit()
    xtermRef.current = term
    fitAddonRef.current = fitAddon

    term.writeln('Terminal initialized. Enter commands below.')
    term.writeln('')

    // Handle window resize
    const handleResize = () => {
      fitAddon.fit()
    }
    window.addEventListener('resize', handleResize)

    // Load execution history
    loadExecutions()

    return () => {
      window.removeEventListener('resize', handleResize)
      term.dispose()
    }
  }, [sandboxId])

  const loadExecutions = async () => {
    try {
      const execs = await getSandboxExecutions(sandboxId)
      setExecutions(execs)

      // Display recent executions in terminal
      if (xtermRef.current && execs.length > 0) {
        xtermRef.current.writeln('Recent command history:')
        execs.slice(-5).forEach((exec) => {
          xtermRef.current?.writeln(`$ ${exec.command}`)
          if (exec.output) {
            exec.output.split('\n').forEach((line) => {
              xtermRef.current?.writeln(line)
            })
          }
          if (exec.error) {
            xtermRef.current?.writeln(`\x1b[31m${exec.error}\x1b[0m`) // Red text for errors
          }
        })
        xtermRef.current.writeln('')
      }
    } catch (error) {
      console.error('Failed to load executions:', error)
    }
  }

  const handleExecuteCommand = async () => {
    if (!command.trim()) return
    if (!xtermRef.current) return

    setIsExecuting(true)

    try {
      // Write command to terminal
      xtermRef.current.writeln(`$ ${command}`)

      // Execute command
      const execution = await executeCommand(sandboxId, {
        command,
        agent_id: agentId,
        model: model,
      })

      // Write output to terminal
      if (execution.output) {
        execution.output.split('\n').forEach((line) => {
          xtermRef.current?.writeln(line)
        })
      }

      if (execution.error) {
        xtermRef.current.writeln(`\x1b[31m${execution.error}\x1b[0m`) // Red text
      }

      if (execution.exit_code !== 0 && execution.exit_code !== null) {
        xtermRef.current.writeln(`\x1b[31mExit code: ${execution.exit_code}\x1b[0m`)
      }

      xtermRef.current.writeln('')

      // Clear command input
      setCommand('')

      // Reload executions
      await loadExecutions()
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Failed to execute command'
      xtermRef.current.writeln(`\x1b[31mError: ${errorMessage}\x1b[0m`)
      xtermRef.current.writeln('')
      toast({
        title: 'Command execution failed',
        description: errorMessage,
        variant: 'destructive',
      })
    } finally {
      setIsExecuting(false)
    }
  }

  const handleClear = () => {
    if (xtermRef.current) {
      xtermRef.current.clear()
      xtermRef.current.writeln('Terminal cleared.')
      xtermRef.current.writeln('')
    }
  }

  const handleKeyPress = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter' && !isExecuting) {
      handleExecuteCommand()
    }
  }

  return (
    <div className="flex flex-col h-full border rounded-lg bg-[#1e1e1e]">
      <div className="flex-1 p-2">
        <div ref={terminalRef} className="h-full" />
      </div>

      <div className="border-t bg-card p-3">
        <div className="flex gap-2">
          <div className="flex-1 flex gap-2">
            <span className="text-sm text-muted-foreground self-center">$</span>
            <Input
              value={command}
              onChange={(e) => setCommand(e.target.value)}
              onKeyPress={handleKeyPress}
              placeholder="Enter command..."
              disabled={isExecuting}
              className="flex-1"
            />
          </div>
          <Button
            onClick={handleExecuteCommand}
            disabled={isExecuting || !command.trim()}
            size="sm"
          >
            {isExecuting ? (
              <>
                <RotateCcw className="h-4 w-4 mr-2 animate-spin" />
                Running...
              </>
            ) : (
              <>
                <Send className="h-4 w-4 mr-2" />
                Execute
              </>
            )}
          </Button>
          <Button onClick={handleClear} variant="outline" size="sm">
            <Trash2 className="h-4 w-4" />
          </Button>
        </div>
      </div>
    </div>
  )
}
