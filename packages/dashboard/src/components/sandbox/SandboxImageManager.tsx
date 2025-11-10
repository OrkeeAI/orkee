// ABOUTME: Main Docker image management container component
// ABOUTME: Provides tabbed interface for images, build, and authentication

import { useState, useCallback, useEffect } from 'react';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Package, Hammer } from 'lucide-react';
import { DockerStatusCard } from './DockerStatusCard';
import { LocalImagesList } from './LocalImagesList';
import { RemoteImagesList } from './RemoteImagesList';
import { DockerBuildForm } from './DockerBuildForm';
import { BuildProgressDisplay } from './BuildProgressDisplay';
import { DockerAuthDialog } from './DockerAuthDialog';
import { getDockerStatus, dockerLogout, type DockerStatus, type BuildImageResponse } from '@/services/docker';
import { useToast } from '@/hooks/use-toast';

export function SandboxImageManager() {
  const [refreshTrigger, setRefreshTrigger] = useState(0);
  const [dockerStatus, setDockerStatus] = useState<DockerStatus | null>(null);
  const [showAuthDialog, setShowAuthDialog] = useState(false);
  const [buildOutput, setBuildOutput] = useState<BuildImageResponse | null>(null);
  const { toast } = useToast();

  const loadDockerStatus = useCallback(async () => {
    try {
      const status = await getDockerStatus();
      setDockerStatus(status);
    } catch (error) {
      toast({
        title: 'Failed to load Docker status',
        description: error instanceof Error ? error.message : 'Unknown error',
        variant: 'destructive',
      });
    }
  }, [toast]);

  useEffect(() => {
    loadDockerStatus();
  }, [loadDockerStatus]);

  const handleRefresh = useCallback(() => {
    setRefreshTrigger((prev) => prev + 1);
    loadDockerStatus();
  }, [loadDockerStatus]);

  const handleLoginClick = useCallback(() => {
    setShowAuthDialog(true);
  }, []);

  const handleLogoutClick = useCallback(async () => {
    try {
      await dockerLogout();
      toast({
        title: 'Logged out successfully',
        description: 'You have been logged out of Docker Hub',
      });
      loadDockerStatus();
      handleRefresh();
    } catch (error) {
      toast({
        title: 'Failed to logout',
        description: error instanceof Error ? error.message : 'Unknown error',
        variant: 'destructive',
      });
    }
  }, [toast, loadDockerStatus, handleRefresh]);

  const handleLoginSuccess = useCallback(() => {
    loadDockerStatus();
    handleRefresh();
  }, [loadDockerStatus, handleRefresh]);

  const handleBuildComplete = useCallback((response: BuildImageResponse) => {
    setBuildOutput(response);
    handleRefresh();
  }, [handleRefresh]);

  return (
    <div className="h-full w-full">
      <Tabs defaultValue="images" className="h-full flex flex-col">
        <TabsList className="grid w-full grid-cols-2">
          <TabsTrigger value="images" className="flex items-center gap-2">
            <Package className="h-4 w-4" />
            Images
          </TabsTrigger>
          <TabsTrigger value="build" className="flex items-center gap-2">
            <Hammer className="h-4 w-4" />
            Build
          </TabsTrigger>
        </TabsList>

        <TabsContent value="images" className="flex-1 overflow-auto">
          <div className="p-4 space-y-4">
            <DockerStatusCard
              onRefresh={handleRefresh}
              onLoginClick={handleLoginClick}
              onLogoutClick={handleLogoutClick}
            />
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-4">
                <h3 className="text-lg font-semibold">Local Images</h3>
                <LocalImagesList refreshTrigger={refreshTrigger} />
              </div>
              <div className="space-y-4">
                <h3 className="text-lg font-semibold">Docker Hub Images</h3>
                <RemoteImagesList
                  username={dockerStatus?.username}
                  isLoggedIn={dockerStatus?.logged_in}
                />
              </div>
            </div>
          </div>
        </TabsContent>

        <TabsContent value="build" className="flex-1 overflow-auto">
          <div className="p-4 space-y-4">
            <DockerBuildForm
              username={dockerStatus?.username}
              onBuildComplete={handleBuildComplete}
            />
            <BuildProgressDisplay buildOutput={buildOutput} />
          </div>
        </TabsContent>
      </Tabs>

      <DockerAuthDialog
        open={showAuthDialog}
        onOpenChange={setShowAuthDialog}
        onLoginSuccess={handleLoginSuccess}
      />
    </div>
  );
}
