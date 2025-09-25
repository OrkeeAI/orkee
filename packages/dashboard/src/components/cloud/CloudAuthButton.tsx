import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Avatar, AvatarFallback } from '@/components/ui/avatar';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { Badge } from '@/components/ui/badge';
import { useCloudAuth } from '@/hooks/useCloud';
import { Cloud, User, LogOut, Settings, AlertCircle } from 'lucide-react';

interface CloudAuthButtonProps {
  variant?: 'default' | 'header' | 'compact';
  showLabel?: boolean;
}

export function CloudAuthButton({ 
  variant = 'default',
  showLabel = true 
}: CloudAuthButtonProps) {
  const { isAuthenticating, authError, login, logout, isAuthenticated, user } = useCloudAuth();
  const [showError, setShowError] = useState(false);

  const handleLogin = async () => {
    setShowError(false);
    try {
      await login();
    } catch {
      setShowError(true);
    }
  };

  const handleLogout = async () => {
    try {
      await logout();
    } catch (error) {
      console.error('Logout failed:', error);
    }
  };

  // Authenticated state - show user dropdown
  if (isAuthenticated && user) {
    return (
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button
            variant={variant === 'header' ? 'ghost' : 'outline'}
            size={variant === 'compact' ? 'sm' : 'default'}
            className="flex items-center gap-2"
          >
            <Avatar className="h-6 w-6">
              <AvatarFallback className="bg-blue-500 text-white text-xs">
                {user.name?.charAt(0)?.toUpperCase() || user.email?.charAt(0)?.toUpperCase() || 'U'}
              </AvatarFallback>
            </Avatar>
            {showLabel && variant !== 'compact' && (
              <span className="truncate max-w-32">
                {user.name || user.email}
              </span>
            )}
            <Cloud className="h-4 w-4 text-blue-500" />
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end" className="w-64">
          <DropdownMenuLabel>
            <div className="flex flex-col space-y-1">
              <p className="text-sm font-medium">{user.name || 'Cloud User'}</p>
              <p className="text-xs text-muted-foreground">{user.email}</p>
              <div className="flex items-center gap-2">
                <Badge variant="secondary" className="text-xs">
                  {user.tier || 'Free'}
                </Badge>
                <div className="flex items-center gap-1 text-xs text-green-600">
                  <div className="w-2 h-2 bg-green-500 rounded-full" />
                  Connected
                </div>
              </div>
            </div>
          </DropdownMenuLabel>
          <DropdownMenuSeparator />
          <DropdownMenuItem
            onClick={() => {
              // TODO: Navigate to cloud settings or dashboard
              console.log('Navigate to cloud settings');
            }}
          >
            <Settings className="mr-2 h-4 w-4" />
            <span>Cloud Settings</span>
          </DropdownMenuItem>
          <DropdownMenuItem
            onClick={() => {
              // TODO: Navigate to profile settings
              console.log('Navigate to profile');
            }}
          >
            <User className="mr-2 h-4 w-4" />
            <span>Profile</span>
          </DropdownMenuItem>
          <DropdownMenuSeparator />
          <DropdownMenuItem 
            onClick={handleLogout}
            disabled={isAuthenticating}
          >
            <LogOut className="mr-2 h-4 w-4" />
            <span>Sign Out</span>
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    );
  }

  // Unauthenticated state - show login button
  return (
    <div className="flex items-center gap-2">
      <Button
        onClick={handleLogin}
        disabled={isAuthenticating}
        variant={variant === 'header' ? 'ghost' : 'default'}
        size={variant === 'compact' ? 'sm' : 'default'}
        className="flex items-center gap-2"
      >
        {isAuthenticating ? (
          <>
            <div className="animate-spin rounded-full h-4 w-4 border-2 border-gray-300 border-t-gray-600" />
            {showLabel && <span>Connecting...</span>}
          </>
        ) : (
          <>
            <Cloud className="h-4 w-4" />
            {showLabel && <span>Connect Cloud</span>}
          </>
        )}
      </Button>

      {/* Show error state */}
      {authError && showError && (
        <div className="flex items-center gap-1 text-red-600">
          <AlertCircle className="h-4 w-4" />
          {variant !== 'compact' && (
            <span className="text-xs">Failed to connect</span>
          )}
        </div>
      )}
    </div>
  );
}

// Compact version for sidebar or small spaces
export function CloudAuthButtonCompact() {
  return <CloudAuthButton variant="compact" showLabel={false} />;
}

// Header version for navigation bar
export function CloudAuthButtonHeader() {
  return <CloudAuthButton variant="header" showLabel={true} />;
}

// Connection status indicator (just shows status, no actions)
export function CloudConnectionStatus() {
  const { isAuthenticated, user } = useCloudAuth();

  if (!isAuthenticated) {
    return (
      <div className="flex items-center gap-2 text-muted-foreground">
        <div className="w-2 h-2 bg-gray-400 rounded-full" />
        <span className="text-xs">Not connected</span>
      </div>
    );
  }

  return (
    <div className="flex items-center gap-2 text-green-600">
      <div className="w-2 h-2 bg-green-500 rounded-full animate-pulse" />
      <span className="text-xs">Connected as {user?.name || user?.email}</span>
    </div>
  );
}