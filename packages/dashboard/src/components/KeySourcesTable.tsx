// ABOUTME: Table displaying API key configuration sources
// ABOUTME: Shows which keys are configured and whether they come from database or environment

import { Badge } from '@/components/ui/badge';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Database, Server, AlertTriangle, RefreshCw, Check, X, Info } from 'lucide-react';
import { useKeysStatus } from '@/hooks/useSecurity';
import { Button } from '@/components/ui/button';

export function KeySourcesTable() {
  const { data: keysStatus, isLoading, error, refetch } = useKeysStatus();

  if (isLoading) {
    return (
      <div className="rounded-lg border p-4" role="status" aria-live="polite">
        <div className="flex items-center gap-2">
          <RefreshCw className="h-4 w-4 animate-spin" />
          <span className="text-sm">Loading API key status...</span>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <Alert variant="destructive" role="alert" aria-live="assertive">
        <AlertTriangle className="h-4 w-4" />
        <AlertDescription>
          Failed to load key status: {error.message}
        </AlertDescription>
      </Alert>
    );
  }

  const getKeyLabel = (key: string): string => {
    const labels: Record<string, string> = {
      'openai': 'OpenAI',
      'anthropic': 'Anthropic (Claude)',
      'google': 'Google AI',
      'xai': 'xAI (Grok)',
      'ai_gateway': 'AI Gateway',
    };
    return labels[key] || key;
  };

  const hasEnvironmentKeys = keysStatus?.keys.some(k => k.source === 'environment');

  return (
    <div className="space-y-3">
      <div className="flex items-center justify-between">
        <h3 className="text-sm font-medium">API Keys Configuration</h3>
        <Button
          onClick={() => refetch()}
          variant="ghost"
          size="sm"
          aria-label="Refresh API key status"
        >
          <RefreshCw className="h-3 w-3 mr-1" />
          Refresh
        </Button>
      </div>

      {hasEnvironmentKeys && (
        <Alert>
          <Info className="h-4 w-4" />
          <AlertDescription className="text-xs">
            Environment variables override database keys. Keys from environment are not encrypted (rely on OS security).
          </AlertDescription>
        </Alert>
      )}

      <div className="rounded-lg border overflow-hidden">
        <table className="w-full">
          <thead className="bg-muted/50">
            <tr>
              <th className="text-left text-xs font-medium p-3">Provider</th>
              <th className="text-center text-xs font-medium p-3">Status</th>
              <th className="text-center text-xs font-medium p-3">Source</th>
            </tr>
          </thead>
          <tbody className="divide-y">
            {keysStatus?.keys.map((keyStatus) => {
              const isConfigured = keyStatus.configured;
              const source = keyStatus.source;

              return (
                <tr key={keyStatus.key} className="hover:bg-muted/30 transition-colors">
                  <td className="p-3">
                    <div className="font-medium text-sm">{getKeyLabel(keyStatus.key)}</div>
                  </td>
                  <td className="p-3 text-center">
                    {isConfigured ? (
                      <Badge variant="secondary" className="text-xs">
                        <Check className="h-3 w-3 mr-1" />
                        Configured
                      </Badge>
                    ) : (
                      <Badge variant="outline" className="text-xs text-muted-foreground">
                        <X className="h-3 w-3 mr-1" />
                        Not Set
                      </Badge>
                    )}
                  </td>
                  <td className="p-3 text-center">
                    {source === 'database' && (
                      <Badge variant="default" className="text-xs bg-blue-600">
                        <Database className="h-3 w-3 mr-1" />
                        Database
                      </Badge>
                    )}
                    {source === 'environment' && (
                      <Badge variant="default" className="text-xs bg-purple-600">
                        <Server className="h-3 w-3 mr-1" />
                        Environment
                      </Badge>
                    )}
                    {source === 'none' && (
                      <span className="text-xs text-muted-foreground">-</span>
                    )}
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>

      <div className="flex items-start gap-2 text-xs text-muted-foreground pt-2">
        <Info className="h-3 w-3 mt-0.5 flex-shrink-0" />
        <div>
          <p>
            <Badge variant="default" className="text-xs bg-blue-600 mr-1 inline-flex items-center">
              <Database className="h-2 w-2 mr-1" />
              Database
            </Badge>
            Keys stored encrypted in local database
          </p>
          <p className="mt-1">
            <Badge variant="default" className="text-xs bg-purple-600 mr-1 inline-flex items-center">
              <Server className="h-2 w-2 mr-1" />
              Environment
            </Badge>
            Keys from environment variables (override database)
          </p>
        </div>
      </div>
    </div>
  );
}
