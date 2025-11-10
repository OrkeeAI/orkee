// ABOUTME: Main Docker image management container component
// ABOUTME: Provides tabbed interface for images, build, and authentication

import { useState, useCallback } from 'react';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Package, Hammer, Key } from 'lucide-react';
import { DockerStatusCard } from './DockerStatusCard';

export function SandboxImageManager() {
  const [refreshTrigger, setRefreshTrigger] = useState(0);

  const handleRefresh = useCallback(() => {
    setRefreshTrigger((prev) => prev + 1);
  }, []);

  const handleLoginClick = useCallback(() => {
    // TODO: Open DockerAuthDialog when implemented
    console.log('Login clicked');
  }, []);

  const handleLogoutClick = useCallback(() => {
    // TODO: Call logout endpoint when implemented
    console.log('Logout clicked');
  }, []);

  return (
    <div className="h-full w-full">
      <Tabs defaultValue="images" className="h-full flex flex-col">
        <TabsList className="grid w-full grid-cols-3">
          <TabsTrigger value="images" className="flex items-center gap-2">
            <Package className="h-4 w-4" />
            Images
          </TabsTrigger>
          <TabsTrigger value="build" className="flex items-center gap-2">
            <Hammer className="h-4 w-4" />
            Build
          </TabsTrigger>
          <TabsTrigger value="auth" className="flex items-center gap-2">
            <Key className="h-4 w-4" />
            Docker Login
          </TabsTrigger>
        </TabsList>

        <TabsContent value="images" className="flex-1 overflow-auto">
          <div className="grid grid-cols-2 gap-4 p-4">
            <div className="space-y-4">
              <h3 className="text-lg font-semibold">Local Images</h3>
              {/* TODO: LocalImagesList component */}
              <div className="text-sm text-muted-foreground">
                LocalImagesList component will go here
              </div>
            </div>
            <div className="space-y-4">
              <h3 className="text-lg font-semibold">Docker Hub Images</h3>
              {/* TODO: RemoteImagesList component */}
              <div className="text-sm text-muted-foreground">
                RemoteImagesList component will go here
              </div>
            </div>
          </div>
        </TabsContent>

        <TabsContent value="build" className="flex-1 overflow-auto">
          <div className="p-4 space-y-4">
            {/* TODO: DockerBuildForm component */}
            <div className="text-sm text-muted-foreground">
              DockerBuildForm component will go here
            </div>
            {/* TODO: BuildProgressDisplay component */}
            <div className="text-sm text-muted-foreground">
              BuildProgressDisplay component will go here
            </div>
          </div>
        </TabsContent>

        <TabsContent value="auth" className="flex-1 overflow-auto">
          <div className="p-4 max-w-2xl">
            <DockerStatusCard
              onRefresh={handleRefresh}
              onLoginClick={handleLoginClick}
              onLogoutClick={handleLogoutClick}
            />
            {/* TODO: DockerAuthDialog will be added here */}
          </div>
        </TabsContent>
      </Tabs>
    </div>
  );
}
