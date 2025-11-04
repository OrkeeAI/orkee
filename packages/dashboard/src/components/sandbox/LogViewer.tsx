// ABOUTME: Real-time log viewer component with SSE streaming
// ABOUTME: Displays execution logs with filtering, search, and auto-scroll functionality

import { useState, useRef, useEffect } from 'react';
import {
  Terminal,
  Search,
  Download,
  Filter,
  ChevronsDown,
  WifiOff,
  Wifi,
  AlertCircle,
  Loader2,
} from 'lucide-react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Alert, AlertDescription } from '@/components/ui/alert';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { useExecutionLogs } from '@/hooks/useExecutionLogs';
import type { LogEntry } from '@/services/execution-stream';

interface LogViewerProps {
  executionId: string;
  autoScroll?: boolean;
  maxHeight?: string;
}

export function LogViewer({
  executionId,
  autoScroll = true,
  maxHeight = '600px',
}: LogViewerProps) {
  const [searchTerm, setSearchTerm] = useState('');
  const [levelFilter, setLevelFilter] = useState<string>('all');
  const [autoScrollEnabled, setAutoScrollEnabled] = useState(autoScroll);
  const scrollAreaRef = useRef<HTMLDivElement>(null);
  const bottomRef = useRef<HTMLDivElement>(null);

  // Use SSE hook for real-time log streaming
  const {
    logs,
    connectionState,
    isComplete,
    isFailed,
    clearLogs,
  } = useExecutionLogs({
    executionId,
    autoConnect: true,
  });

  // Auto-scroll to bottom when new logs arrive
  useEffect(() => {
    if (autoScrollEnabled && bottomRef.current) {
      bottomRef.current.scrollIntoView({ behavior: 'smooth' });
    }
  }, [logs, autoScrollEnabled]);

  // Filter logs
  const filteredLogs = logs.filter((log) => {
    // Level filter
    if (levelFilter !== 'all' && log.log_level !== levelFilter) {
      return false;
    }

    // Search filter
    if (searchTerm && !log.message.toLowerCase().includes(searchTerm.toLowerCase())) {
      return false;
    }

    return true;
  });

  const handleExportLogs = () => {
    const logsText = filteredLogs
      .map((log) => `[${log.timestamp}] [${log.log_level.toUpperCase()}] ${log.message}`)
      .join('\n');

    const blob = new Blob([logsText], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `execution-${executionId}-logs.txt`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  };

  const getLogLevelColor = (level: string) => {
    switch (level) {
      case 'debug':
        return 'text-gray-500';
      case 'info':
        return 'text-blue-600';
      case 'warn':
        return 'text-yellow-600';
      case 'error':
        return 'text-red-600';
      case 'fatal':
        return 'text-red-800';
      default:
        return 'text-gray-600';
    }
  };

  const getLogLevelBadgeVariant = (level: string): "default" | "secondary" | "destructive" | "outline" => {
    switch (level) {
      case 'error':
      case 'fatal':
        return 'destructive';
      case 'warn':
        return 'outline';
      case 'info':
        return 'default';
      case 'debug':
        return 'secondary';
      default:
        return 'outline';
    }
  };

  const getConnectionIcon = () => {
    switch (connectionState) {
      case 'connected':
        return <Wifi className="h-4 w-4 text-green-600" />;
      case 'connecting':
        return <Loader2 className="h-4 w-4 animate-spin text-yellow-600" />;
      case 'disconnected':
        return <WifiOff className="h-4 w-4 text-gray-500" />;
      case 'error':
        return <AlertCircle className="h-4 w-4 text-red-600" />;
      default:
        return <WifiOff className="h-4 w-4" />;
    }
  };

  return (
    <Card>
      <CardHeader>
        <div className="flex items-start justify-between">
          <div className="space-y-1">
            <CardTitle className="flex items-center gap-2">
              <Terminal className="h-5 w-5" />
              Execution Logs
              {getConnectionIcon()}
            </CardTitle>
            <CardDescription>
              {connectionState === 'connected' && 'Streaming logs in real-time'}
              {connectionState === 'connecting' && 'Connecting to log stream...'}
              {connectionState === 'disconnected' && 'Disconnected from log stream'}
              {connectionState === 'error' && 'Error connecting to log stream'}
            </CardDescription>
          </div>
          <div className="flex items-center gap-2">
            <Badge variant={isComplete ? 'default' : 'outline'}>
              {filteredLogs.length} logs
            </Badge>
            {isFailed && (
              <Badge variant="destructive">Failed</Badge>
            )}
          </div>
        </div>
      </CardHeader>
      <CardContent className="space-y-4">
        {/* Controls */}
        <div className="flex flex-col sm:flex-row gap-2">
          <div className="relative flex-1">
            <Search className="absolute left-2 top-2.5 h-4 w-4 text-muted-foreground" />
            <Input
              placeholder="Search logs..."
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              className="pl-8"
            />
          </div>

          <Select value={levelFilter} onValueChange={setLevelFilter}>
            <SelectTrigger className="w-full sm:w-[150px]">
              <Filter className="mr-2 h-4 w-4" />
              <SelectValue placeholder="Log level" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">All Levels</SelectItem>
              <SelectItem value="debug">Debug</SelectItem>
              <SelectItem value="info">Info</SelectItem>
              <SelectItem value="warn">Warning</SelectItem>
              <SelectItem value="error">Error</SelectItem>
              <SelectItem value="fatal">Fatal</SelectItem>
            </SelectContent>
          </Select>

          <Button
            variant="outline"
            size="sm"
            onClick={handleExportLogs}
            disabled={filteredLogs.length === 0}
          >
            <Download className="h-4 w-4" />
          </Button>

          <Button
            variant={autoScrollEnabled ? 'default' : 'outline'}
            size="sm"
            onClick={() => setAutoScrollEnabled(!autoScrollEnabled)}
          >
            <ChevronsDown className="h-4 w-4" />
          </Button>
        </div>

        {/* Connection Error Alert */}
        {connectionState === 'error' && (
          <Alert variant="destructive">
            <AlertCircle className="h-4 w-4" />
            <AlertDescription>
              Failed to connect to log stream. Showing cached logs only.
            </AlertDescription>
          </Alert>
        )}

        {/* Log Display */}
        <div className="border rounded-lg overflow-hidden">
          <ScrollArea
            ref={scrollAreaRef}
            style={{ height: maxHeight }}
            className="bg-slate-950 text-slate-50"
          >
            <div className="p-4 font-mono text-xs space-y-1">
              {filteredLogs.length === 0 ? (
                <div className="text-center text-slate-400 py-8">
                  {connectionState === 'connecting' ? (
                    <div className="flex flex-col items-center gap-2">
                      <Loader2 className="h-6 w-6 animate-spin" />
                      <span>Waiting for logs...</span>
                    </div>
                  ) : (
                    <div className="flex flex-col items-center gap-2">
                      <Terminal className="h-6 w-6" />
                      <span>
                        {searchTerm || levelFilter !== 'all'
                          ? 'No logs match the current filters'
                          : 'No logs available'}
                      </span>
                    </div>
                  )}
                </div>
              ) : (
                <>
                  {filteredLogs.map((log) => (
                    <div
                      key={log.id}
                      className="flex items-start gap-2 py-1 hover:bg-slate-900/50 px-2 rounded"
                    >
                      <span className="text-slate-500 shrink-0">
                        {new Date(log.timestamp).toLocaleTimeString()}
                      </span>
                      <Badge
                        variant={getLogLevelBadgeVariant(log.log_level)}
                        className="shrink-0 uppercase text-xs"
                      >
                        {log.log_level}
                      </Badge>
                      {log.source && (
                        <span className="text-slate-400 shrink-0">[{log.source}]</span>
                      )}
                      <span className={`flex-1 ${getLogLevelColor(log.log_level)}`}>
                        {log.message}
                      </span>
                    </div>
                  ))}
                  <div ref={bottomRef} />
                </>
              )}
            </div>
          </ScrollArea>
        </div>

        {/* Log Count Info */}
        {logs.length !== filteredLogs.length && (
          <p className="text-xs text-muted-foreground">
            Showing {filteredLogs.length} of {logs.length} logs
          </p>
        )}
      </CardContent>
    </Card>
  );
}
