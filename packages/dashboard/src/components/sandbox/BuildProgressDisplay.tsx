// ABOUTME: Docker build progress display component
// ABOUTME: Terminal-style log viewer for build output

import { useEffect, useRef, useState } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Terminal, Copy, Trash2, CheckCircle2, XCircle } from 'lucide-react';
import { useToast } from '@/hooks/use-toast';
import type { BuildImageResponse } from '@/services/docker';

interface BuildProgressDisplayProps {
  buildOutput?: BuildImageResponse | null;
}

export function BuildProgressDisplay({ buildOutput }: BuildProgressDisplayProps) {
  const [status, setStatus] = useState<'idle' | 'building' | 'success' | 'failed'>('idle');
  const [logs, setLogs] = useState<string[]>([]);
  const logsEndRef = useRef<HTMLDivElement>(null);
  const { toast } = useToast();

  useEffect(() => {
    if (buildOutput) {
      setStatus('success');
      setLogs(buildOutput.output.split('\n'));
    }
  }, [buildOutput]);

  useEffect(() => {
    // Auto-scroll to bottom when logs change
    logsEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [logs]);

  const handleCopyLogs = () => {
    navigator.clipboard.writeText(logs.join('\n'));
    toast({
      title: 'Logs copied to clipboard',
    });
  };

  const handleClearLogs = () => {
    setLogs([]);
    setStatus('idle');
  };

  if (status === 'idle' && logs.length === 0) {
    return (
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Terminal className="h-5 w-5" />
            Build Output
          </CardTitle>
          <CardDescription>Build logs will appear here</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex items-center justify-center py-8 text-sm text-muted-foreground">
            No build output yet
          </div>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Terminal className="h-5 w-5" />
            <CardTitle>Build Output</CardTitle>
            {status === 'building' && (
              <Badge variant="secondary">Building...</Badge>
            )}
            {status === 'success' && (
              <Badge variant="default" className="bg-green-500">
                <CheckCircle2 className="h-3 w-3 mr-1" />
                Success
              </Badge>
            )}
            {status === 'failed' && (
              <Badge variant="destructive">
                <XCircle className="h-3 w-3 mr-1" />
                Failed
              </Badge>
            )}
          </div>
          <div className="flex gap-2">
            <Button
              variant="outline"
              size="sm"
              onClick={handleCopyLogs}
              disabled={logs.length === 0}
            >
              <Copy className="h-4 w-4 mr-2" />
              Copy
            </Button>
            <Button
              variant="outline"
              size="sm"
              onClick={handleClearLogs}
              disabled={logs.length === 0}
            >
              <Trash2 className="h-4 w-4 mr-2" />
              Clear
            </Button>
          </div>
        </div>
      </CardHeader>
      <CardContent>
        <div className="bg-black rounded-lg p-4 font-mono text-sm overflow-auto max-h-[400px]">
          {logs.length === 0 ? (
            <div className="text-gray-500">Waiting for build output...</div>
          ) : (
            <div className="space-y-1">
              {logs.map((line, index) => (
                <div
                  key={index}
                  className={`${
                    line.toLowerCase().includes('error')
                      ? 'text-red-400'
                      : line.toLowerCase().includes('warning')
                      ? 'text-yellow-400'
                      : line.toLowerCase().includes('successfully')
                      ? 'text-green-400'
                      : 'text-gray-300'
                  }`}
                >
                  {line || '\u00A0'}
                </div>
              ))}
              <div ref={logsEndRef} />
            </div>
          )}
        </div>
        {buildOutput && (
          <div className="mt-4 text-sm text-muted-foreground">
            Built image: <span className="font-medium">{buildOutput.image_tag}</span>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
