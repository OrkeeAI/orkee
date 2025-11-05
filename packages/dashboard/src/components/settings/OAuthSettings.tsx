import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { useAuth } from '@/contexts/AuthContext';
import { Terminal, Check, X, Clock, Shield, AlertTriangle } from 'lucide-react';
import { useState } from 'react';

export function OAuthSettings() {
  const { authStatus, logout, isLoading, error } = useAuth();
  const [loggingOut, setLoggingOut] = useState<string | null>(null);

  const providers = [
    {
      id: 'claude',
      name: 'Claude (Anthropic)',
      description: 'Use your Claude Pro/Max subscription - requires Claude CLI',
      icon: '🤖',
    },
  ];

  const handleLogout = async (provider: string) => {
    setLoggingOut(provider);
    try {
      await logout(provider);
    } catch (err) {
      console.error(`Failed to logout from ${provider}:`, err);
      alert(`Failed to logout: ${err instanceof Error ? err.message : 'Unknown error'}`);
    } finally {
      setLoggingOut(null);
    }
  };

  const formatExpiryTime = (expiresAt: number) => {
    const now = Math.floor(Date.now() / 1000);
    const diff = expiresAt - now;

    if (diff < 0) return 'Expired';
    if (diff < 3600) return `${Math.floor(diff / 60)}m`;
    if (diff < 86400) return `${Math.floor(diff / 3600)}h`;
    return `${Math.floor(diff / 86400)}d`;
  };

  if (isLoading) {
    return (
      <div className="space-y-6">
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Shield className="h-5 w-5" />
              AI Provider Authentication
            </CardTitle>
            <CardDescription>Loading authentication status...</CardDescription>
          </CardHeader>
        </Card>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Shield className="h-5 w-5" />
            AI Provider Authentication
          </CardTitle>
          <CardDescription>
            Authenticate with Claude using OAuth tokens from the Claude CLI
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-6">
          {/* Info Alert */}
          <Alert>
            <Terminal className="h-4 w-4" />
            <AlertDescription>
              <p className="font-medium mb-2">How to authenticate with Claude:</p>
              <ol className="text-sm space-y-2 list-decimal list-inside">
                <li className="pl-1">
                  Install Claude CLI:
                  <code className="ml-2 px-1.5 py-0.5 bg-muted border rounded text-xs font-mono">
                    npm install -g @anthropic-ai/claude-code
                  </code>
                </li>
                <li className="pl-1">
                  Run{' '}
                  <code className="px-1.5 py-0.5 bg-muted border rounded text-xs font-mono">
                    orkee auth login claude
                  </code>
                </li>
                <li className="pl-1">Your browser will open for authentication with Anthropic</li>
                <li className="pl-1">The generated OAuth token will be imported automatically</li>
                <li className="pl-1">Return here to see your connection status</li>
              </ol>
            </AlertDescription>
          </Alert>

          {/* Error Display */}
          {error && (
            <Alert variant="destructive">
              <AlertTriangle className="h-4 w-4" />
              <AlertDescription>{error}</AlertDescription>
            </Alert>
          )}

          {/* Provider Cards */}
          <div className="space-y-4">
            {providers.map((provider) => {
              const status = authStatus[provider.id];
              const isAuthenticated = status?.authenticated || false;
              const isExpiringSoon = status?.expiresAt && (status.expiresAt - Math.floor(Date.now() / 1000)) < 3600;

              return (
                <Card key={provider.id} className={isAuthenticated ? 'border-green-200 bg-green-50/30' : ''}>
                  <CardContent className="pt-6">
                    <div className="flex items-start justify-between">
                      <div className="flex items-start gap-3 flex-1">
                        <div className="text-2xl">{provider.icon}</div>
                        <div className="flex-1">
                          <div className="flex items-center gap-2 mb-1">
                            <h3 className="font-medium">{provider.name}</h3>
                            {isAuthenticated ? (
                              <Badge variant="default" className="bg-green-600">
                                <Check className="h-3 w-3 mr-1" />
                                Connected
                              </Badge>
                            ) : (
                              <Badge variant="secondary">
                                <X className="h-3 w-3 mr-1" />
                                Not Connected
                              </Badge>
                            )}
                          </div>
                          <p className="text-sm text-muted-foreground mb-3">
                            {provider.description}
                          </p>

                          {isAuthenticated && status && (
                            <div className="space-y-2">
                              {status.accountEmail && (
                                <div className="flex items-center gap-2 text-sm">
                                  <span className="text-muted-foreground">Account:</span>
                                  <span className="font-mono">{status.accountEmail}</span>
                                </div>
                              )}
                              {status.subscriptionType && (
                                <div className="flex items-center gap-2 text-sm">
                                  <span className="text-muted-foreground">Plan:</span>
                                  <Badge variant="outline" className="text-xs">
                                    {status.subscriptionType}
                                  </Badge>
                                </div>
                              )}
                              {status.expiresAt && (
                                <div className="flex items-center gap-2 text-sm">
                                  <Clock className={`h-3 w-3 ${isExpiringSoon ? 'text-orange-500' : 'text-muted-foreground'}`} />
                                  <span className={isExpiringSoon ? 'text-orange-600 font-medium' : 'text-muted-foreground'}>
                                    Expires in {formatExpiryTime(status.expiresAt)}
                                    {isExpiringSoon && ' (refresh recommended)'}
                                  </span>
                                </div>
                              )}
                            </div>
                          )}

                          {!isAuthenticated && (
                            <div className="mt-2">
                              <code className="text-xs bg-muted border px-2 py-1 rounded font-mono">
                                orkee auth login {provider.id}
                              </code>
                            </div>
                          )}
                        </div>
                      </div>

                      <div className="flex flex-col gap-2">
                        {isAuthenticated && (
                          <Button
                            variant="outline"
                            size="sm"
                            onClick={() => handleLogout(provider.id)}
                            disabled={loggingOut === provider.id}
                          >
                            {loggingOut === provider.id ? 'Disconnecting...' : 'Disconnect'}
                          </Button>
                        )}
                      </div>
                    </div>
                  </CardContent>
                </Card>
              );
            })}
          </div>

          {/* CLI Reference */}
          <Card className="bg-muted/50">
            <CardHeader>
              <CardTitle className="text-sm flex items-center gap-2">
                <Terminal className="h-4 w-4" />
                CLI Reference
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-2">
              <div className="grid grid-cols-2 gap-3 text-sm">
                <div>
                  <p className="text-foreground font-medium mb-1">Authenticate:</p>
                  <code className="text-xs bg-background border px-2 py-1 rounded block font-mono">orkee auth login claude</code>
                </div>
                <div>
                  <p className="text-foreground font-medium mb-1">Check Status:</p>
                  <code className="text-xs bg-background border px-2 py-1 rounded block font-mono">orkee auth status</code>
                </div>
                <div>
                  <p className="text-foreground font-medium mb-1">Import from File:</p>
                  <code className="text-xs bg-background border px-2 py-1 rounded block font-mono">orkee auth login claude --file token.txt</code>
                </div>
                <div>
                  <p className="text-foreground font-medium mb-1">Logout:</p>
                  <code className="text-xs bg-background border px-2 py-1 rounded block font-mono">orkee auth logout claude</code>
                </div>
              </div>
              <div className="mt-3 pt-3 border-t text-xs text-muted-foreground">
                <p>💡 Claude tokens expire after 1 year and cannot be refreshed. Re-authenticate when expired.</p>
              </div>
            </CardContent>
          </Card>
        </CardContent>
      </Card>
    </div>
  );
}
