import { useState, useEffect, useCallback, useRef } from 'react';
import { Play, Square, Terminal, Monitor, RefreshCw, ExternalLink, AlertCircle } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { previewService, DevServerInstance } from '@/services/preview';
import { PreviewFrame } from './PreviewFrame';
import { PreviewTerminalDrawer } from './PreviewTerminalDrawer';

interface PreviewPanelProps {
  projectId: string;
  projectName: string;
}

export function PreviewPanel({ projectId, projectName }: PreviewPanelProps) {
  const [serverInstance, setServerInstance] = useState<DevServerInstance | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [showTerminalModal, setShowTerminalModal] = useState(false);
  const [refreshKey, setRefreshKey] = useState(0);
  const isAggressivePollingRef = useRef(false);

  // Poll for server status
  const checkServerStatus = useCallback(async () => {
    try {
      console.log(`[PreviewPanel] Checking server status for project ${projectId}`);
      const instance = await previewService.getServerStatus(projectId);
      console.log(`[PreviewPanel] Server status response:`, instance);
      setServerInstance(instance);
      
      // Update activity if server is running
      if (instance?.status === 'running') {
        console.log(`[PreviewPanel] Server is running, preview_url: ${instance.preview_url}`);
        previewService.updateServerActivity(projectId).catch(console.warn);
      }
    } catch (err) {
      console.error('[PreviewPanel] Failed to check server status:', err);
    }
  }, [projectId]);

  // Start server
  const handleStartServer = async (customPort?: number) => {
    try {
      setIsLoading(true);
      setError(null);
      
      // Auto-open terminal to show startup logs
      setShowTerminalModal(true);
      
      const instance = await previewService.startServer(projectId, customPort);
      setServerInstance(instance);
      
      // Start polling more aggressively after server start (only if not already polling)
      if (!isAggressivePollingRef.current) {
        isAggressivePollingRef.current = true;
        console.log(`[PreviewPanel] Starting aggressive polling for project ${projectId}`);
        
        const pollInterval = setInterval(async () => {
          try {
            console.log(`[PreviewPanel] Aggressive poll check for project ${projectId}`);
            const updatedInstance = await previewService.getServerStatus(projectId);
            console.log(`[PreviewPanel] Aggressive poll response:`, updatedInstance);
            setServerInstance(updatedInstance);
            
            // Stop aggressive polling once server is running
            if (updatedInstance?.status === 'running') {
              console.log(`[PreviewPanel] Server running detected, stopping aggressive polling`);
              clearInterval(pollInterval);
              isAggressivePollingRef.current = false;
            }
          } catch (error) {
            console.error('[PreviewPanel] Failed to poll server status:', error);
          }
        }, 1000); // Poll every 1 second until running
        
        // Cleanup aggressive polling after 30 seconds max
        setTimeout(() => {
          clearInterval(pollInterval);
          isAggressivePollingRef.current = false;
        }, 30000);
      }
      
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to start server';
      setError(errorMessage);
      console.error('Failed to start server:', err);
    } finally {
      setIsLoading(false);
    }
  };

  // Stop server
  const handleStopServer = async () => {
    try {
      setIsLoading(true);
      setError(null);
      
      await previewService.stopServer(projectId);
      setServerInstance(null);
      setShowTerminalModal(false);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to stop server';
      setError(errorMessage);
      console.error('Failed to stop server:', err);
    } finally {
      setIsLoading(false);
    }
  };

  // Refresh preview frame
  const handleRefreshPreview = () => {
    setRefreshKey(prev => prev + 1);
  };

  // Initialize and set up polling
  useEffect(() => {
    // Check initial status
    checkServerStatus();

    // Poll for status updates every 5 seconds
    const interval = setInterval(checkServerStatus, 5000);
    
    return () => clearInterval(interval);
  }, [checkServerStatus]);

  // Note: Terminal stays open until manually closed by user

  const getStatusBadge = (status: string) => {
    switch (status) {
      case 'running':
        return <Badge variant="default" className="bg-green-100 text-green-800">Running</Badge>;
      case 'starting':
        return <Badge variant="default" className="bg-blue-100 text-blue-800">Starting</Badge>;
      case 'stopping':
        return <Badge variant="default" className="bg-yellow-100 text-yellow-800">Stopping</Badge>;
      case 'error':
        return <Badge variant="destructive">Error</Badge>;
      default:
        return <Badge variant="secondary">Stopped</Badge>;
    }
  };

  const isRunning = serverInstance?.status === 'running';
  const isStarting = serverInstance?.status === 'starting';
  const isStopping = serverInstance?.status === 'stopping';
  const hasError = serverInstance?.status === 'error' || !!error;

  // Debug logging for state changes
  console.log(`[PreviewPanel] Current state - serverInstance:`, serverInstance);
  console.log(`[PreviewPanel] Computed states - isRunning: ${isRunning}, isStarting: ${isStarting}, isLoading: ${isLoading}`);

  return (
    <div className="space-y-4">
      {/* Control Panel */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle className="text-lg font-semibold">Development Preview</CardTitle>
              <CardDescription>
                Preview {projectName} with a live development server
              </CardDescription>
            </div>
            <div className="flex items-center gap-2">
              {serverInstance && getStatusBadge(serverInstance.status)}
            </div>
          </div>
        </CardHeader>
        <CardContent>
          <div className="flex flex-col sm:flex-row gap-2">
            {!isRunning ? (
              <Button 
                onClick={() => handleStartServer()} 
                disabled={isLoading || isStarting}
                className="flex-1 sm:flex-none"
              >
                {isStarting ? (
                  <>
                    <RefreshCw className="mr-2 h-4 w-4 animate-spin" />
                    Starting...
                  </>
                ) : (
                  <>
                    <Play className="mr-2 h-4 w-4" />
                    Start Preview
                  </>
                )}
              </Button>
            ) : (
              <>
                <Button 
                  onClick={handleStopServer} 
                  variant="destructive"
                  disabled={isLoading || isStopping}
                  className="flex-1 sm:flex-none"
                >
                  {isStopping ? (
                    <>
                      <RefreshCw className="mr-2 h-4 w-4 animate-spin" />
                      Stopping...
                    </>
                  ) : (
                    <>
                      <Square className="mr-2 h-4 w-4" />
                      Stop Preview
                    </>
                  )}
                </Button>
                <Button 
                  onClick={handleRefreshPreview}
                  variant="outline"
                  className="flex-1 sm:flex-none"
                >
                  <RefreshCw className="mr-2 h-4 w-4" />
                  Refresh
                </Button>
                {serverInstance.preview_url && (
                  <Button 
                    onClick={() => window.open(serverInstance.preview_url, '_blank')}
                    variant="outline"
                    className="flex-1 sm:flex-none"
                  >
                    <ExternalLink className="mr-2 h-4 w-4" />
                    Open in New Tab
                  </Button>
                )}
              </>
            )}
            
            {serverInstance && (
              <Button 
                onClick={() => setShowTerminalModal(true)}
                variant="outline"
                className="flex-1 sm:flex-none"
              >
                <Terminal className="mr-2 h-4 w-4" />
                Show Terminal
              </Button>
            )}
          </div>

          {/* Server Info */}
          {serverInstance && (
            <div className="mt-4 grid grid-cols-1 md:grid-cols-3 gap-4 text-sm">
              <div>
                <span className="font-medium">Status:</span>
                <div className="mt-1">{getStatusBadge(serverInstance.status)}</div>
              </div>
              {serverInstance.config.framework && (
                <div>
                  <span className="font-medium">Framework:</span>
                  <div className="mt-1 text-muted-foreground">
                    {serverInstance.config.framework.name}
                    {serverInstance.config.framework.version && ` v${serverInstance.config.framework.version}`}
                  </div>
                </div>
              )}
              {serverInstance.config.port && (
                <div>
                  <span className="font-medium">Port:</span>
                  <div className="mt-1 text-muted-foreground">{serverInstance.config.port}</div>
                </div>
              )}
            </div>
          )}
        </CardContent>
      </Card>

      {/* Error Alert */}
      {hasError && (
        <Alert variant="destructive">
          <AlertCircle className="h-4 w-4" />
          <AlertDescription>
            {error || serverInstance?.error || 'An error occurred with the preview server'}
          </AlertDescription>
        </Alert>
      )}

      {/* Terminal Drawer */}
      <PreviewTerminalDrawer 
        projectId={projectId}
        projectName={projectName}
        open={showTerminalModal}
        onOpenChange={setShowTerminalModal}
      />

      {/* Preview Frame */}
      {isRunning && serverInstance?.preview_url ? (
        <PreviewFrame 
          url={serverInstance.preview_url}
          projectName={projectName}
          refreshKey={refreshKey}
        />
      ) : !serverInstance ? (
        <Card>
          <CardContent className="py-12">
            <div className="text-center">
              <Play className="h-12 w-12 mx-auto mb-4 text-muted-foreground" />
              <h3 className="text-lg font-medium mb-2">Preview Not Running</h3>
              <p className="text-muted-foreground mb-4">
                Start the dev server to see a live preview of your application.
              </p>
            </div>
          </CardContent>
        </Card>
      ) : (
        <Card>
          <CardContent className="py-12">
            <div className="text-center">
              <Monitor className="h-12 w-12 mx-auto mb-4 text-muted-foreground" />
              <h3 className="text-lg font-medium mb-2">Starting Preview Server</h3>
              <p className="text-muted-foreground">
                Please wait while the development server starts...
              </p>
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  );
}