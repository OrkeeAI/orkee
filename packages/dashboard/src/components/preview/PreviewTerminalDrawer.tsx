import { useState, useEffect, useRef, useCallback } from 'react';
import { Terminal, Download, Trash2, RefreshCw } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Sheet, SheetContent, SheetHeader, SheetTitle, SheetDescription } from '@/components/ui/sheet';
import { previewService, DevServerLog } from '@/services/preview';

interface PreviewTerminalDrawerProps {
  projectId: string;
  projectName: string;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function PreviewTerminalDrawer({ projectId, projectName, open, onOpenChange }: PreviewTerminalDrawerProps) {
  const [logs, setLogs] = useState<DevServerLog[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [lastFetchTime, setLastFetchTime] = useState<Date | null>(null);
  const [hasAttemptedLoad, setHasAttemptedLoad] = useState(false);
  const terminalRef = useRef<HTMLDivElement>(null);
  const shouldAutoScroll = useRef(true);

  // Fetch logs
  const fetchLogs = useCallback(async (since?: string) => {
    try {
      setIsLoading(true);
      const newLogs = await previewService.getServerLogs(projectId, {
        since,
        limit: 1000,
      });
      
      if (since) {
        // Append new logs
        setLogs(prev => [...prev, ...newLogs]);
      } else {
        // Replace all logs
        setLogs(newLogs);
      }
      
      setLastFetchTime(new Date());
      setHasAttemptedLoad(true);
    } catch (error) {
      console.error('Failed to fetch logs:', error);
      setHasAttemptedLoad(true);
    } finally {
      setIsLoading(false);
    }
  }, [projectId]);

  // Clear logs
  const handleClearLogs = async () => {
    try {
      await previewService.clearServerLogs(projectId);
      setLogs([]);
      setLastFetchTime(new Date());
    } catch (error) {
      console.error('Failed to clear logs:', error);
    }
  };

  // Download logs
  const handleDownloadLogs = () => {
    const logText = logs.length > 0 
      ? logs.map(log => `[${new Date(log.timestamp).toLocaleString()}] [${log.log_type.toUpperCase()}] ${log.message}`).join('\n')
      : `# ${projectName} Preview Server Logs\n# Generated: ${new Date().toLocaleString()}\n# No logs captured yet\n`;
    
    const blob = new Blob([logText], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `${projectName}-preview-logs-${new Date().toISOString().slice(0, 19)}.txt`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  };

  // Auto-scroll to bottom when new logs arrive
  useEffect(() => {
    if (shouldAutoScroll.current && terminalRef.current) {
      terminalRef.current.scrollTop = terminalRef.current.scrollHeight;
    }
  }, [logs]);

  // Handle scroll to detect if user scrolled up
  const handleScroll = (event: React.UIEvent) => {
    const target = event.target as HTMLElement;
    const isNearBottom = target.scrollTop + target.clientHeight >= target.scrollHeight - 50;
    shouldAutoScroll.current = isNearBottom;
  };

  // Load logs when drawer opens and poll while open
  useEffect(() => {
    if (open) {
      // Load logs immediately when opened
      fetchLogs();
      
      // Poll for new logs every 2 seconds while open
      const interval = setInterval(() => {
        fetchLogs();
      }, 2000);

      return () => clearInterval(interval);
    }
  }, [open, fetchLogs]);

  // Format log message with ANSI colors (basic implementation)
  const formatLogMessage = (message: string) => {
    // eslint-disable-next-line no-control-regex
    return message.replace(/\u001b\[[0-9;]*m/g, '');
  };

  const getLogTypeColor = (logType: string) => {
    switch (logType) {
      case 'system':
        return 'text-blue-400';
      case 'stderr':
        return 'text-red-400';
      case 'stdout':
        return 'text-green-400';
      default:
        return 'text-gray-300';
    }
  };

  return (
    <Sheet open={open} onOpenChange={onOpenChange}>
      <SheetContent side="bottom" className="h-[60vh] flex flex-col">
        <SheetHeader>
          <div className="flex items-center justify-between">
            <div>
              <SheetTitle className="flex items-center gap-2">
                <Terminal className="h-5 w-5" />
                {projectName} - Development Server Terminal
              </SheetTitle>
              <SheetDescription>
                Preview server logs and output
              </SheetDescription>
            </div>
            <div className="flex items-center gap-2">
              <Button
                variant="outline"
                size="sm"
                onClick={() => fetchLogs()}
                disabled={isLoading}
              >
                {isLoading ? (
                  <RefreshCw className="mr-2 h-4 w-4 animate-spin" />
                ) : (
                  <RefreshCw className="mr-2 h-4 w-4" />
                )}
                Refresh
              </Button>
              <Button
                variant="outline"
                size="sm"
                onClick={handleDownloadLogs}
              >
                <Download className="mr-2 h-4 w-4" />
                Download
              </Button>
              <Button
                variant="outline"
                size="sm"
                onClick={handleClearLogs}
                disabled={logs.length === 0}
              >
                <Trash2 className="mr-2 h-4 w-4" />
                Clear
              </Button>
            </div>
          </div>
        </SheetHeader>

        {/* Terminal Content */}
        <div className="flex-1 flex flex-col min-h-0 mt-4">
          <div
            ref={terminalRef}
            className="flex-1 border rounded-md bg-gray-950 text-gray-100 p-4 overflow-auto font-mono text-sm"
            onScroll={handleScroll}
          >
            {!hasAttemptedLoad && isLoading ? (
              <div className="flex items-center justify-center h-32 text-gray-400">
                <RefreshCw className="mr-2 h-4 w-4 animate-spin" />
                Loading logs...
              </div>
            ) : logs.length === 0 ? (
              <div className="flex items-center justify-center h-32 text-gray-400">
                <div className="text-center">
                  <Terminal className="mx-auto h-8 w-8 mb-2 opacity-50" />
                  <div>No logs available yet</div>
                  <div className="text-xs mt-1">Logs will appear as the server generates output</div>
                </div>
              </div>
            ) : (
              <div className="space-y-1">
                {logs.map((log, index) => (
                  <div key={index} className="flex gap-2">
                    <span className="text-gray-500 shrink-0">
                      [{new Date(log.timestamp).toLocaleTimeString()}]
                    </span>
                    <span className={`shrink-0 ${getLogTypeColor(log.log_type)}`}>
                      [{log.log_type.toUpperCase()}]
                    </span>
                    <span className="break-words">
                      {formatLogMessage(log.message)}
                    </span>
                  </div>
                ))}
              </div>
            )}
          </div>

          {/* Status Bar */}
          <div className="mt-2 flex items-center justify-between text-xs text-muted-foreground">
            <span>
              {logs.length > 0 ? `${logs.length} log entries` : 'No logs yet'}
            </span>
            {lastFetchTime && (
              <span>
                Last updated: {lastFetchTime.toLocaleTimeString()}
              </span>
            )}
          </div>
        </div>
      </SheetContent>
    </Sheet>
  );
}