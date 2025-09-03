import { useConnection } from '@/contexts/ConnectionContext';
import { cn } from '@/lib/utils';
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip';

interface ConnectionStatusProps {
  className?: string;
  showText?: boolean;
}

export function ConnectionStatus({ className, showText = false }: ConnectionStatusProps) {
  const { connectionState, lastHealthCheck, error } = useConnection();

  const getStatusColor = () => {
    switch (connectionState) {
      case 'connected':
        return 'bg-green-500';
      case 'connecting':
        return 'bg-yellow-500';
      case 'disconnected':
        return 'bg-red-500';
      default:
        return 'bg-gray-500';
    }
  };

  const getStatusText = () => {
    switch (connectionState) {
      case 'connected':
        return lastHealthCheck ? `Connected (v${lastHealthCheck.version})` : 'Connected';
      case 'connecting':
        return 'Connecting...';
      case 'disconnected':
        return error ? `Disconnected: ${error}` : 'Disconnected';
      default:
        return 'Unknown';
    }
  };

  const statusDot = (
    <div
      className={cn(
        'w-2 h-2 rounded-full',
        getStatusColor(),
        connectionState === 'connecting' && 'animate-pulse'
      )}
    />
  );

  return (
    <TooltipProvider>
      <div className={cn('flex items-center gap-2', className)}>
        {showText ? (
          <>
            {statusDot}
            <span className="text-xs text-muted-foreground">
              {getStatusText()}
            </span>
          </>
        ) : (
          <Tooltip>
            <TooltipTrigger asChild>
              {statusDot}
            </TooltipTrigger>
            <TooltipContent>
              <p>{getStatusText()}</p>
            </TooltipContent>
          </Tooltip>
        )}
      </div>
    </TooltipProvider>
  );
}