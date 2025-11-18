// ABOUTME: Docker Hub section for sidebar
// ABOUTME: Shows Docker status, daemon status, user images, and build button

import { useEffect, useState, useRef } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Input } from '@/components/ui/input';
import { Separator } from '@/components/ui/separator';
import { RefreshCw, CheckCircle2, XCircle, Hammer, Activity } from 'lucide-react';
import {
  getDockerStatus,
  getDockerDaemonStatus,
  type DockerStatus,
  type DockerDaemonStatus,
} from '@/services/docker';
import { getSandboxSettings, updateSandboxSettings } from '@/services/sandbox';
import { RemoteImagesList, type RemoteImagesListRef } from './RemoteImagesList';
import { BuildModal } from './BuildModal';
import { useToast } from '@/hooks/use-toast';

interface DockerHubSectionProps {
  onLoginClick?: () => void;
  onLogoutClick?: () => void;
  onImagePulled?: () => void;
  onBuildComplete?: () => void;
}

export function DockerHubSection({
  onLoginClick,
  onLogoutClick,
  onImagePulled,
  onBuildComplete,
}: DockerHubSectionProps) {
  const [status, setStatus] = useState<DockerStatus | null>(null);
  const [daemonStatus, setDaemonStatus] = useState<DockerDaemonStatus | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [isBuildModalOpen, setIsBuildModalOpen] = useState(false);
  const [inputUsername, setInputUsername] = useState<string>('');
  const [savedUsername, setSavedUsername] = useState<string>('');
  const [isEditingUsername, setIsEditingUsername] = useState(false);
  const { toast } = useToast();
  const remoteImagesListRef = useRef<RemoteImagesListRef>(null);

  // Use saved username if provided, otherwise use status username
  const effectiveUsername = savedUsername || status?.username;

  const handleSaveUsername = async () => {
    try {
      // Get current settings
      const currentSettings = await getSandboxSettings();

      // Update with new username
      await updateSandboxSettings({
        ...currentSettings,
        docker_username: inputUsername || null,
      });

      setSavedUsername(inputUsername);
      setIsEditingUsername(false);

      toast({
        title: 'Username saved',
        description: 'Docker Hub username has been saved',
      });
    } catch (error) {
      toast({
        title: 'Failed to save username',
        description: error instanceof Error ? error.message : 'Unknown error',
        variant: 'destructive',
      });
    }
  };

  const handleStartEditing = () => {
    setInputUsername(effectiveUsername || '');
    setIsEditingUsername(true);
  };

  const loadStatus = async () => {
    try {
      setIsLoading(true);
      const [dockerStatus, dockerDaemonStatus] = await Promise.all([
        getDockerStatus(),
        getDockerDaemonStatus(),
      ]);
      setStatus(dockerStatus);
      setDaemonStatus(dockerDaemonStatus);
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
    // Load saved username from settings
    getSandboxSettings()
      .then((settings) => {
        if (settings.docker_username) {
          setSavedUsername(settings.docker_username);
        }
      })
      .catch((error) => {
        console.error('Failed to load saved username:', error);
      });
  }, []);

  const handleRefresh = () => {
    loadStatus();
  };

  const handleLogout = async () => {
    await onLogoutClick?.();
    // Refresh status after logout to update UI
    loadStatus();
  };

  const handleBuildComplete = () => {
    setIsBuildModalOpen(false);
    // Reload Docker Hub images list after successful build
    remoteImagesListRef.current?.reload();
    onBuildComplete?.();
  };

  return (
    <>
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle>Docker Hub</CardTitle>
              <CardDescription>Docker status and images</CardDescription>
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
          ) : (
            <div className="space-y-4">
              {/* Docker Daemon Status */}
              <div className="space-y-2">
                <div className="text-sm font-medium">Docker Daemon</div>
                <div className="flex items-center gap-2">
                  {daemonStatus?.running ? (
                    <>
                      <Activity className="h-4 w-4 text-green-500" />
                      <Badge variant="default" className="bg-green-500">
                        Running
                      </Badge>
                      {daemonStatus.version && (
                        <span className="text-xs text-muted-foreground">
                          v{daemonStatus.version}
                        </span>
                      )}
                    </>
                  ) : (
                    <>
                      <XCircle className="h-4 w-4 text-destructive" />
                      <Badge variant="destructive">Not Running</Badge>
                    </>
                  )}
                </div>
                {daemonStatus?.error && (
                  <p className="text-xs text-destructive">{daemonStatus.error}</p>
                )}
              </div>

              <Separator />

              {/* Docker Hub Login Status */}
              <div className="space-y-2">
                <div className="text-sm font-medium">Docker Hub</div>
                <div className="space-y-2">
                  <div className="flex items-center gap-2">
                    {status?.logged_in ? (
                      <>
                        <CheckCircle2 className="h-4 w-4 text-green-500" />
                        <Badge variant="default" className="bg-green-500">
                          Logged In
                        </Badge>
                      </>
                    ) : (
                      <>
                        <XCircle className="h-4 w-4 text-muted-foreground" />
                        <Badge variant="secondary">Not Logged In</Badge>
                      </>
                    )}
                  </div>

                  {status?.logged_in && (
                    <>
                      {!isEditingUsername && effectiveUsername ? (
                        <div className="text-sm flex items-center gap-2">
                          <span className="font-medium">User:</span>
                          <span className="text-muted-foreground">{effectiveUsername}</span>
                          <Button
                            variant="ghost"
                            size="sm"
                            onClick={handleStartEditing}
                            className="h-6 px-2 text-xs"
                          >
                            Change
                          </Button>
                        </div>
                      ) : isEditingUsername ? (
                        <div className="flex items-center gap-2">
                          <Input
                            placeholder="Docker Hub username"
                            value={inputUsername}
                            onChange={(e) => setInputUsername(e.target.value)}
                            onKeyDown={(e) => {
                              if (e.key === 'Enter') {
                                handleSaveUsername();
                              }
                            }}
                            className="h-8 text-sm"
                            autoFocus
                          />
                          <Button
                            size="sm"
                            onClick={handleSaveUsername}
                            className="h-8 px-3"
                          >
                            Done
                          </Button>
                        </div>
                      ) : (
                        <div className="flex items-center gap-2">
                          <span className="text-xs text-muted-foreground">
                            Credentials stored securely
                          </span>
                          <Button
                            variant="outline"
                            size="sm"
                            onClick={handleStartEditing}
                            className="h-7 px-2 text-xs"
                          >
                            Set Username
                          </Button>
                        </div>
                      )}
                    </>
                  )}
                </div>

                <div>
                  {status?.logged_in ? (
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={handleLogout}
                      className="w-full"
                    >
                      Logout
                    </Button>
                  ) : (
                    <Button
                      size="sm"
                      onClick={onLoginClick}
                      className="w-full"
                    >
                      Login
                    </Button>
                  )}
                </div>
              </div>

              <Separator />

              {/* Docker Hub Images */}
              <div className="space-y-2">
                <div className="text-sm font-medium">Your Images</div>
                <RemoteImagesList
                  ref={remoteImagesListRef}
                  username={effectiveUsername}
                  isLoggedIn={status?.logged_in}
                  onImagePulled={onImagePulled}
                />
              </div>

              <Separator />

              {/* Build Image Button */}
              <Button
                className="w-full"
                onClick={() => setIsBuildModalOpen(true)}
                disabled={!daemonStatus?.running}
              >
                <Hammer className="h-4 w-4 mr-2" />
                Build Image
              </Button>
              {!daemonStatus?.running && (
                <p className="text-xs text-muted-foreground text-center">
                  Docker daemon must be running to build images
                </p>
              )}
            </div>
          )}
        </CardContent>
      </Card>

      <BuildModal
        open={isBuildModalOpen}
        onOpenChange={setIsBuildModalOpen}
        username={effectiveUsername}
        onBuildComplete={handleBuildComplete}
        onLoginClick={() => {
          setIsBuildModalOpen(false);
          onLoginClick?.();
        }}
      />
    </>
  );
}
