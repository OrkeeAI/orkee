import { useState, useEffect, useCallback } from 'react';
import { AlertCircle } from 'lucide-react';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { previewService, DevServerInstance } from '@/services/preview';
import { PreviewFrame } from './PreviewFrame';
import { PreviewTerminalDrawer } from './PreviewTerminalDrawer';
import { useServerEvents } from '@/hooks/useServerEvents';

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
  const [terminalAutoOpened, setTerminalAutoOpened] = useState(false);

  // Use SSE for real-time server state updates
  const { activeServers, serverErrors } = useServerEvents();

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
      setTerminalAutoOpened(true);

      const instance = await previewService.startServer(projectId, customPort);
      setServerInstance(instance);

      // SSE will automatically notify us when the server reaches 'running' state
      console.log(`[PreviewPanel] Server start initiated, waiting for SSE 'server_started' event`);

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
      setTerminalAutoOpened(false);
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

  // Listen for SSE updates and refresh server status when this project's state changes
  useEffect(() => {
    const isActiveNow = activeServers.has(projectId);
    const hasError = serverErrors.has(projectId);

    if (isActiveNow || hasError) {
      console.log(`[PreviewPanel] SSE detected state change for project ${projectId}, refreshing status`);
      checkServerStatus();
    }
  }, [activeServers, serverErrors, projectId, checkServerStatus]);

  // Auto-close terminal when server finishes starting (only if auto-opened)
  useEffect(() => {
    if (serverInstance?.status === 'running' && showTerminalModal && terminalAutoOpened) {
      // Wait a moment to let users see the success message, then close
      const timer = setTimeout(() => {
        setShowTerminalModal(false);
        setTerminalAutoOpened(false);
      }, 3000); // 3 seconds delay

      return () => clearTimeout(timer);
    }
  }, [serverInstance?.status, showTerminalModal, terminalAutoOpened]);


  const isRunning = serverInstance?.status === 'running';
  const isStarting = serverInstance?.status === 'starting';
  const isStopping = serverInstance?.status === 'stopping';
  const hasError = serverInstance?.status === 'error' || !!error;

  // Debug logging for state changes
  console.log(`[PreviewPanel] Current state - serverInstance:`, serverInstance);
  console.log(`[PreviewPanel] Computed states - isRunning: ${isRunning}, isStarting: ${isStarting}, isLoading: ${isLoading}`);

  return (
    <div className="space-y-4">
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

      {/* Consolidated Preview Frame with Server Controls */}
      <PreviewFrame 
        url={serverInstance?.preview_url || ''}
        projectName={projectName}
        refreshKey={refreshKey}
        serverStatus={serverInstance?.status}
        serverFramework={serverInstance?.config.framework?.name}
        serverPort={serverInstance?.config.port ?? undefined}
        isLoading={isLoading}
        isStarting={isStarting}
        isStopping={isStopping}
        onStartServer={() => handleStartServer()}
        onStopServer={handleStopServer}
        onRefreshPreview={handleRefreshPreview}
        onShowTerminal={() => {
          setShowTerminalModal(true);
          setTerminalAutoOpened(false); // Mark as manually opened
        }}
      />
    </div>
  );
}