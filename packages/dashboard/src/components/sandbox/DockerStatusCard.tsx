// ABOUTME: Docker authentication status card component
// ABOUTME: Displays login status, username, and provides login/logout actions

import { useEffect, useState } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { RefreshCw, CheckCircle2, XCircle } from 'lucide-react';
import { getDockerStatus, type DockerStatus } from '@/services/docker';
import { useToast } from '@/hooks/use-toast';

interface DockerStatusCardProps {
  onRefresh?: () => void;
  onLoginClick?: () => void;
  onLogoutClick?: () => void;
}

export function DockerStatusCard({ onRefresh, onLoginClick, onLogoutClick }: DockerStatusCardProps) {
  const [status, setStatus] = useState<DockerStatus | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const { toast } = useToast();

  const loadStatus = async () => {
    try {
      setIsLoading(true);
      const data = await getDockerStatus();
      setStatus(data);
    } catch (error) {
      toast({
        title: 'Failed to load Docker status',
        description: error instanceof Error ? error.message : 'Unknown error',
        variant: 'destructive',
      });
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    loadStatus();
  }, []);

  const handleRefresh = () => {
    loadStatus();
    onRefresh?.();
  };

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <div>
            <CardTitle>Docker Hub Authentication</CardTitle>
            <CardDescription>Manage your Docker Hub login status</CardDescription>
          </div>
          <Button
            variant="outline"
            size="icon"
            onClick={handleRefresh}
            disabled={isLoading}
          >
            <RefreshCw className={`h-4 w-4 ${isLoading ? 'animate-spin' : ''}`} />
          </Button>
        </div>
      </CardHeader>
      <CardContent>
        {isLoading ? (
          <div className="flex items-center justify-center py-4">
            <RefreshCw className="h-6 w-6 animate-spin text-muted-foreground" />
          </div>
        ) : status ? (
          <div className="space-y-4">
            <div className="flex items-center gap-2">
              {status.logged_in ? (
                <>
                  <CheckCircle2 className="h-5 w-5 text-green-500" />
                  <Badge variant="default" className="bg-green-500">
                    Logged In
                  </Badge>
                </>
              ) : (
                <>
                  <XCircle className="h-5 w-5 text-muted-foreground" />
                  <Badge variant="secondary">Not Logged In</Badge>
                </>
              )}
            </div>

            {status.logged_in && status.username && (
              <div className="space-y-2">
                <div className="text-sm">
                  <span className="font-medium">Username:</span>{' '}
                  <span className="text-muted-foreground">{status.username}</span>
                </div>
                {status.email && (
                  <div className="text-sm">
                    <span className="font-medium">Email:</span>{' '}
                    <span className="text-muted-foreground">{status.email}</span>
                  </div>
                )}
                {status.server_address && (
                  <div className="text-sm">
                    <span className="font-medium">Server:</span>{' '}
                    <span className="text-muted-foreground">{status.server_address}</span>
                  </div>
                )}
              </div>
            )}

            <div className="pt-2">
              {status.logged_in ? (
                <Button
                  variant="outline"
                  onClick={onLogoutClick}
                  className="w-full"
                >
                  Logout from Docker Hub
                </Button>
              ) : (
                <Button
                  onClick={onLoginClick}
                  className="w-full"
                >
                  Login to Docker Hub
                </Button>
              )}
            </div>
          </div>
        ) : (
          <div className="text-sm text-muted-foreground">
            Failed to load Docker status
          </div>
        )}
      </CardContent>
    </Card>
  );
}
