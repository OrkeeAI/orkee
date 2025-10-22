// ABOUTME: Security status display showing current encryption mode
// ABOUTME: Provides buttons to manage password-based encryption

import { Button } from '@/components/ui/button';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Badge } from '@/components/ui/badge';
import { Shield, AlertTriangle, Lock, RefreshCw, Check } from 'lucide-react';
import { useSecurityStatus } from '@/hooks/useSecurity';
import { EncryptionMode } from '@/services/security';

interface SecurityStatusSectionProps {
  onManagePassword: (mode: 'set' | 'change' | 'remove') => void;
}

export function SecurityStatusSection({ onManagePassword }: SecurityStatusSectionProps) {
  const { data: securityStatus, isLoading, error, refetch } = useSecurityStatus();

  if (isLoading) {
    return (
      <div className="rounded-lg border p-4" role="status" aria-live="polite">
        <div className="flex items-center gap-2">
          <RefreshCw className="h-4 w-4 animate-spin" />
          <span className="text-sm">Loading security status...</span>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <Alert variant="destructive" role="alert" aria-live="assertive">
        <AlertTriangle className="h-4 w-4" />
        <AlertDescription>
          Failed to load security status: {error.message}
        </AlertDescription>
      </Alert>
    );
  }

  const isMachineMode = securityStatus?.encryptionMode === EncryptionMode.Machine;
  const isPasswordMode = securityStatus?.encryptionMode === EncryptionMode.Password;

  return (
    <div className="rounded-lg border p-4 bg-gradient-to-r from-slate-50 to-slate-100 dark:from-slate-900 dark:to-slate-800">
      <div className="flex items-start justify-between mb-3">
        <div className="flex items-center gap-2">
          <Shield className={`h-5 w-5 ${isPasswordMode ? 'text-green-600' : 'text-amber-600'}`} />
          <h3 className="font-semibold">Encryption Security Status</h3>
        </div>
        <Badge variant={isPasswordMode ? 'default' : 'secondary'} className={isPasswordMode ? 'bg-green-600' : 'bg-amber-600'}>
          {isPasswordMode ? 'Secure' : 'Limited'}
        </Badge>
      </div>

      {isMachineMode && (
        <>
          <Alert className="mb-4 border-amber-200 bg-amber-50 dark:bg-amber-950 dark:border-amber-800">
            <AlertTriangle className="h-4 w-4 text-amber-600" />
            <AlertDescription className="text-sm">
              <p className="font-medium text-amber-900 dark:text-amber-100 mb-2">Machine-Based Encryption (Transport Only)</p>
              <ul className="list-disc list-inside space-y-1 text-amber-800 dark:text-amber-200 text-xs">
                <li>Keys encrypted during backup/sync only</li>
                <li><strong>NOT secure at-rest</strong> on local machine</li>
                <li>Anyone with database file access can decrypt keys</li>
              </ul>
            </AlertDescription>
          </Alert>
          <div className="flex gap-2">
            <Button
              onClick={() => onManagePassword('set')}
              className="flex-1"
              variant="default"
            >
              <Lock className="mr-2 h-4 w-4" />
              Upgrade to Password Protection
            </Button>
            <Button
              onClick={() => refetch()}
              variant="ghost"
              size="icon"
              aria-label="Refresh security status"
            >
              <RefreshCw className="h-4 w-4" />
            </Button>
          </div>
        </>
      )}

      {isPasswordMode && (
        <>
          <Alert className="mb-4 border-green-200 bg-green-50 dark:bg-green-950 dark:border-green-800">
            <Check className="h-4 w-4 text-green-600" />
            <AlertDescription className="text-sm">
              <p className="font-medium text-green-900 dark:text-green-100 mb-2">Password-Based Encryption (At-Rest)</p>
              <ul className="list-disc list-inside space-y-1 text-green-800 dark:text-green-200 text-xs">
                <li>True at-rest encryption enabled</li>
                <li>Keys cannot be decrypted without password</li>
                <li>Suitable for shared machines and production</li>
              </ul>
            </AlertDescription>
          </Alert>
          <div className="flex gap-2">
            <Button
              onClick={() => onManagePassword('change')}
              className="flex-1"
              variant="outline"
            >
              <RefreshCw className="mr-2 h-4 w-4" />
              Change Password
            </Button>
            <Button
              onClick={() => onManagePassword('remove')}
              variant="outline"
              className="flex-1"
            >
              <AlertTriangle className="mr-2 h-4 w-4" />
              Remove Protection
            </Button>
            <Button
              onClick={() => refetch()}
              variant="ghost"
              size="icon"
              aria-label="Refresh security status"
            >
              <RefreshCw className="h-4 w-4" />
            </Button>
          </div>
        </>
      )}

      {securityStatus?.isLocked && (
        <Alert variant="destructive" className="mt-3" role="alert" aria-live="assertive">
          <AlertTriangle className="h-4 w-4" />
          <AlertDescription className="text-sm">
            Account locked due to failed password attempts. Please wait before trying again.
          </AlertDescription>
        </Alert>
      )}
    </div>
  );
}
