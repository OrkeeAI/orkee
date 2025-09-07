import { useState, useEffect } from 'react';
import { Monitor, Tablet, Smartphone, RefreshCw, AlertCircle, ExternalLink, Maximize2, X } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Alert, AlertDescription } from '@/components/ui/alert';

interface PreviewFrameProps {
  url: string;
  projectName: string;
  refreshKey?: number;
}

type DeviceType = 'desktop' | 'tablet' | 'mobile';

const deviceConfigs = {
  desktop: {
    icon: Monitor,
    name: 'Desktop',
    width: '100%',
    height: '600px',
  },
  tablet: {
    icon: Tablet,
    name: 'Tablet',
    width: '768px',
    height: '600px',
  },
  mobile: {
    icon: Smartphone,
    name: 'Mobile',
    width: '375px',
    height: '600px',
  },
} as const;

export function PreviewFrame({ url, projectName, refreshKey = 0 }: PreviewFrameProps) {
  const [selectedDevice, setSelectedDevice] = useState<DeviceType>('desktop');
  const [isLoading, setIsLoading] = useState(true);
  const [hasError, setHasError] = useState(false);
  const [delayedUrl, setDelayedUrl] = useState<string | null>(null);
  const [retryCount, setRetryCount] = useState(0);
  const [isFullscreen, setIsFullscreen] = useState(false);

  const handleIframeLoad = () => {
    console.log(`[PreviewFrame] Iframe loaded successfully for ${url}`);
    setIsLoading(false);
    setHasError(false);
    setRetryCount(0);
  };

  const handleIframeError = () => {
    console.log(`[PreviewFrame] Iframe error for ${url}, retry count: ${retryCount}`);
    setIsLoading(false);
    setHasError(true);
  };

  const handleRetry = () => {
    console.log(`[PreviewFrame] Manual retry for ${url}`);
    setRetryCount(prev => prev + 1);
    setHasError(false);
    setIsLoading(true);
    setDelayedUrl(null);
    
    // Force iframe reload with a slight delay
    setTimeout(() => {
      setDelayedUrl(url);
    }, 500);
  };

  // Initial URL setting with delay to ensure server is ready
  useEffect(() => {
    setIsLoading(true);
    setHasError(false);
    setDelayedUrl(null);
    
    // Give Next.js server a moment to be fully ready
    const delayTimer = setTimeout(() => {
      console.log(`[PreviewFrame] Setting delayed URL after initial delay: ${url}`);
      setDelayedUrl(url);
    }, 2000); // 2 second delay for initial load

    return () => clearTimeout(delayTimer);
  }, [url, refreshKey]);

  // Fallback timeout for frameworks like Next.js that don't reliably fire onLoad
  useEffect(() => {
    if (!delayedUrl) return;

    const fallbackTimer = setTimeout(() => {
      if (isLoading) {
        console.log(`[PreviewFrame] Fallback timeout - assuming iframe loaded successfully for ${url}`);
        setIsLoading(false);
        setHasError(false);
      }
    }, 5000); // 5 second timeout

    return () => clearTimeout(fallbackTimer);
  }, [delayedUrl, isLoading, url]);

  // Handle escape key to exit fullscreen
  useEffect(() => {
    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && isFullscreen) {
        setIsFullscreen(false);
      }
    };

    if (isFullscreen) {
      document.addEventListener('keydown', handleEscape);
      return () => document.removeEventListener('keydown', handleEscape);
    }
  }, [isFullscreen]);

  const currentConfig = deviceConfigs[selectedDevice];

  return (
    <>
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <CardTitle className="text-lg font-semibold">
              Application Preview
            </CardTitle>
          <div className="flex items-center gap-2">
            {/* Device Size Selector */}
            <div className="flex items-center border rounded-md p-1">
              {Object.entries(deviceConfigs).map(([device, config]) => {
                const IconComponent = config.icon;
                return (
                  <Button
                    key={device}
                    variant={selectedDevice === device ? "default" : "ghost"}
                    size="sm"
                    className="px-2 py-1"
                    onClick={() => setSelectedDevice(device as DeviceType)}
                  >
                    <IconComponent className="h-4 w-4" />
                    <span className="sr-only">{config.name}</span>
                  </Button>
                );
              })}
            </div>
            
            <Button
              variant="outline"
              size="sm"
              onClick={() => setIsFullscreen(true)}
            >
              <Maximize2 className="mr-2 h-4 w-4" />
              Full
            </Button>
            
            <Button
              variant="outline"
              size="sm"
              onClick={() => window.open(url, '_blank')}
            >
              <ExternalLink className="mr-2 h-4 w-4" />
              Open
            </Button>
          </div>
        </div>
      </CardHeader>
      <CardContent>
        {hasError ? (
          <Alert variant="destructive">
            <AlertCircle className="h-4 w-4" />
            <AlertDescription>
              Failed to load the preview. The development server might not be ready yet.
              {retryCount > 0 && (
                <span className="text-xs block mt-1">
                  Retry attempt: {retryCount}
                </span>
              )}
              <Button
                variant="outline"
                size="sm"
                className="ml-2"
                onClick={handleRetry}
              >
                <RefreshCw className="mr-1 h-3 w-3" />
                Retry
              </Button>
            </AlertDescription>
          </Alert>
        ) : (
          <div className="preview-container">
            {/* Device Info */}
            <div className="mb-4 text-sm text-muted-foreground">
              Previewing <strong>{projectName}</strong> in {currentConfig.name} view
            </div>

            {/* Loading Indicator */}
            {(isLoading || !delayedUrl) && (
              <div className="absolute inset-0 flex items-center justify-center bg-background/80 z-10">
                <div className="flex flex-col items-center gap-2">
                  <RefreshCw className="h-6 w-6 animate-spin" />
                  <span className="text-sm text-muted-foreground">
                    {!delayedUrl ? 'Waiting for server to be ready...' : 'Loading preview...'}
                  </span>
                </div>
              </div>
            )}

            {/* Preview Frame Container */}
            <div 
              className="relative mx-auto border rounded-lg overflow-hidden bg-white shadow-lg"
              style={{ 
                width: currentConfig.width,
                maxWidth: '100%',
                height: currentConfig.height,
              }}
            >
              {delayedUrl && (
                <iframe
                  key={`${delayedUrl}-${refreshKey}-${retryCount}`}
                  src={delayedUrl}
                  className="w-full h-full"
                  title={`Preview of ${projectName}`}
                  sandbox="allow-same-origin allow-scripts allow-popups allow-forms"
                  onLoad={handleIframeLoad}
                  onError={handleIframeError}
                  style={{
                    border: 'none',
                    background: 'white',
                  }}
                />
              )}
            </div>

            {/* URL Info */}
            <div className="mt-4 p-3 bg-muted rounded-md">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <span className="text-sm font-medium">Preview URL:</span>
                  <code className="text-sm bg-background px-2 py-1 rounded border">
                    {url}
                  </code>
                </div>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => navigator.clipboard.writeText(url)}
                  className="text-xs"
                >
                  Copy
                </Button>
              </div>
            </div>
          </div>
        )}
      </CardContent>
    </Card>

    {/* Fullscreen Modal */}
    {isFullscreen && (
      <div className="fixed inset-0 z-50 bg-black bg-opacity-75 flex items-center justify-center p-4">
        <div className="w-full h-full max-w-none max-h-none bg-white rounded-lg overflow-hidden shadow-xl relative">
          {/* Fullscreen Header */}
          <div className="flex items-center justify-between p-4 border-b bg-gray-50">
            <div className="flex items-center gap-2">
              <h3 className="font-semibold text-lg">Preview: {projectName}</h3>
            </div>
            <div className="flex items-center gap-2">
              {/* Device Size Selector in Fullscreen */}
              <div className="flex items-center border rounded-md p-1 bg-white">
                {Object.entries(deviceConfigs).map(([device, config]) => {
                  const IconComponent = config.icon;
                  return (
                    <Button
                      key={device}
                      variant={selectedDevice === device ? "default" : "ghost"}
                      size="sm"
                      className="px-2 py-1"
                      onClick={() => setSelectedDevice(device as DeviceType)}
                    >
                      <IconComponent className="h-4 w-4" />
                      <span className="sr-only">{config.name}</span>
                    </Button>
                  );
                })}
              </div>
              
              <Button
                variant="outline"
                size="sm"
                onClick={() => window.open(url, '_blank')}
              >
                <ExternalLink className="mr-2 h-4 w-4" />
                Open
              </Button>
              <Button
                variant="outline"
                size="sm"
                onClick={() => setIsFullscreen(false)}
              >
                <X className="h-4 w-4" />
                <span className="sr-only">Close fullscreen</span>
              </Button>
            </div>
          </div>

          {/* Fullscreen Content */}
          <div className="relative w-full h-[calc(100%-73px)] bg-gray-100 flex items-center justify-center">
            {/* Loading Indicator */}
            {(isLoading || !delayedUrl) && (
              <div className="absolute inset-0 flex items-center justify-center bg-background/80 z-10">
                <div className="flex flex-col items-center gap-2">
                  <RefreshCw className="h-8 w-8 animate-spin" />
                  <span className="text-lg text-muted-foreground">
                    {!delayedUrl ? 'Waiting for server to be ready...' : 'Loading preview...'}
                  </span>
                </div>
              </div>
            )}

            {/* Fullscreen Preview Frame Container */}
            <div 
              className="relative border rounded-lg overflow-hidden bg-white shadow-lg"
              style={{ 
                width: selectedDevice === 'desktop' ? '100%' : currentConfig.width,
                maxWidth: selectedDevice === 'desktop' ? '100%' : currentConfig.width,
                height: selectedDevice === 'desktop' ? '100%' : currentConfig.height,
                maxHeight: selectedDevice === 'desktop' ? '100%' : currentConfig.height,
              }}
            >
              {/* Fullscreen Iframe */}
              {delayedUrl && (
                <iframe
                  key={`fullscreen-${delayedUrl}-${refreshKey}-${retryCount}`}
                  src={delayedUrl}
                  className="w-full h-full border-none"
                  title={`Fullscreen preview of ${projectName}`}
                  sandbox="allow-same-origin allow-scripts allow-popups allow-forms"
                  onLoad={handleIframeLoad}
                  onError={handleIframeError}
                />
              )}
            </div>
          </div>
        </div>
      </div>
    )}
    </>
  );
}