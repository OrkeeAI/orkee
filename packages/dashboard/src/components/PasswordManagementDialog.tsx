// ABOUTME: Dialog for managing password-based encryption
// ABOUTME: Supports setting, changing, and removing password encryption

import { useState, useCallback } from 'react';
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { AlertTriangle, Lock, Shield, RefreshCw, Check, X } from 'lucide-react';
import { useSetPassword, useChangePassword, useRemovePassword } from '@/hooks/useSecurity';
import { validatePasswordStrength } from '@/lib/password-validation';

interface PasswordManagementDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  mode: 'set' | 'change' | 'remove';
}

export function PasswordManagementDialog({ open, onOpenChange, mode }: PasswordManagementDialogProps) {
  const [currentPassword, setCurrentPassword] = useState('');
  const [newPassword, setNewPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  const setPasswordMutation = useSetPassword();
  const changePasswordMutation = useChangePassword();
  const removePasswordMutation = useRemovePassword();

  const isLoading = setPasswordMutation.isPending || changePasswordMutation.isPending || removePasswordMutation.isPending;

  const handleClose = useCallback(() => {
    // Prevent closing during async operations
    if (isLoading) {
      return;
    }

    setCurrentPassword('');
    setNewPassword('');
    setConfirmPassword('');
    setError(null);
    setSuccess(null);
    onOpenChange(false);
  }, [isLoading, onOpenChange]);

  const handleSetPassword = async () => {
    setError(null);
    setSuccess(null);

    const validation = validatePasswordStrength(newPassword);
    if (!validation.valid) {
      setError(validation.error!);
      return;
    }

    if (newPassword !== confirmPassword) {
      setError('Passwords do not match');
      return;
    }

    try {
      const result = await setPasswordMutation.mutateAsync(newPassword);
      setSuccess(result.message);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to set password');
    }
  };

  const handleChangePassword = async () => {
    setError(null);
    setSuccess(null);

    if (!currentPassword) {
      setError('Current password is required');
      return;
    }

    const validation = validatePasswordStrength(newPassword);
    if (!validation.valid) {
      setError(validation.error!);
      return;
    }

    if (newPassword !== confirmPassword) {
      setError('Passwords do not match');
      return;
    }

    try {
      const result = await changePasswordMutation.mutateAsync({
        currentPassword,
        newPassword,
      });
      setSuccess(result.message);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to change password');
    }
  };

  const handleRemovePassword = async () => {
    setError(null);
    setSuccess(null);

    if (!currentPassword) {
      setError('Current password is required');
      return;
    }

    try {
      const result = await removePasswordMutation.mutateAsync(currentPassword);
      setSuccess(result.message);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to remove password');
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    if (mode === 'set') {
      await handleSetPassword();
    } else if (mode === 'change') {
      await handleChangePassword();
    } else if (mode === 'remove') {
      await handleRemovePassword();
    }
  };

  return (
    <Dialog open={open} onOpenChange={isLoading ? undefined : handleClose}>
      <DialogContent className="sm:max-w-[500px]" onPointerDownOutside={(e) => isLoading && e.preventDefault()} onEscapeKeyDown={(e) => isLoading && e.preventDefault()}>
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            {mode === 'set' && (
              <>
                <Lock className="h-5 w-5" />
                Enable Password Protection
              </>
            )}
            {mode === 'change' && (
              <>
                <RefreshCw className="h-5 w-5" />
                Change Password
              </>
            )}
            {mode === 'remove' && (
              <>
                <AlertTriangle className="h-5 w-5" />
                Remove Password Protection
              </>
            )}
          </DialogTitle>
          <DialogDescription>
            {mode === 'set' && 'Upgrade to password-based encryption for enhanced security'}
            {mode === 'change' && 'Change your encryption password'}
            {mode === 'remove' && 'Downgrade to machine-based encryption'}
          </DialogDescription>
        </DialogHeader>

        <form onSubmit={handleSubmit}>
          <div className="space-y-4 py-4">
            {/* Security Warnings */}
            {mode === 'set' && (
              <Alert>
                <Shield className="h-4 w-4" />
                <AlertDescription>
                  <div className="space-y-1 text-sm">
                    <p className="font-medium">Password-based encryption provides:</p>
                    <ul className="list-disc list-inside space-y-1 ml-2">
                      <li>True at-rest encryption</li>
                      <li>Protection from local file access</li>
                      <li>Suitable for shared machines</li>
                    </ul>
                    <p className="text-amber-600 font-medium mt-2">
                      ⚠️ If you forget your password, encrypted keys cannot be recovered
                    </p>
                  </div>
                </AlertDescription>
              </Alert>
            )}

            {mode === 'remove' && (
              <Alert variant="destructive">
                <AlertTriangle className="h-4 w-4" />
                <AlertDescription>
                  <div className="space-y-1 text-sm">
                    <p className="font-medium">Warning: This will downgrade security</p>
                    <ul className="list-disc list-inside space-y-1 ml-2">
                      <li>Keys will only be encrypted during transfer</li>
                      <li>NOT secure at-rest on local machine</li>
                      <li>Anyone with database file access can decrypt keys</li>
                    </ul>
                  </div>
                </AlertDescription>
              </Alert>
            )}

            {/* Current Password (for change and remove modes) */}
            {(mode === 'change' || mode === 'remove') && (
              <div className="space-y-2">
                <Label htmlFor="current-password">Current Password</Label>
                <Input
                  id="current-password"
                  type="password"
                  value={currentPassword}
                  onChange={(e) => setCurrentPassword(e.target.value)}
                  placeholder="Enter current password"
                  disabled={isLoading}
                  required
                  aria-label="Current password"
                  aria-required="true"
                />
              </div>
            )}

            {/* New Password (for set and change modes) */}
            {(mode === 'set' || mode === 'change') && (
              <>
                <div className="space-y-2">
                  <Label htmlFor="new-password">
                    {mode === 'set' ? 'Password' : 'New Password'}
                  </Label>
                  <Input
                    id="new-password"
                    type="password"
                    value={newPassword}
                    onChange={(e) => setNewPassword(e.target.value)}
                    placeholder="At least 8 characters with uppercase, lowercase, digit, and special character"
                    disabled={isLoading}
                    required
                    aria-label={mode === 'set' ? 'Password' : 'New password'}
                    aria-required="true"
                    aria-describedby="new-password-help"
                  />
                  <p id="new-password-help" className="text-xs text-muted-foreground">
                    Must include: uppercase letter, lowercase letter, digit, and special character
                  </p>
                </div>

                <div className="space-y-2">
                  <Label htmlFor="confirm-password">Confirm Password</Label>
                  <Input
                    id="confirm-password"
                    type="password"
                    value={confirmPassword}
                    onChange={(e) => setConfirmPassword(e.target.value)}
                    placeholder="Confirm your password"
                    disabled={isLoading}
                    required
                    aria-label="Confirm password"
                    aria-required="true"
                  />
                </div>
              </>
            )}

            {/* Error Display */}
            {error && (
              <Alert variant="destructive" role="alert" aria-live="assertive">
                <X className="h-4 w-4" />
                <AlertDescription>{error}</AlertDescription>
              </Alert>
            )}

            {/* Success Display */}
            {success && (
              <Alert role="status" aria-live="polite">
                <Check className="h-4 w-4" />
                <AlertDescription>{success}</AlertDescription>
              </Alert>
            )}
          </div>

          <DialogFooter>
            {success ? (
              <Button type="button" onClick={handleClose}>
                Close
              </Button>
            ) : (
              <>
                <Button type="button" variant="outline" onClick={handleClose} disabled={isLoading}>
                  Cancel
                </Button>
                <Button
                  type="submit"
                  disabled={isLoading}
                  variant={mode === 'remove' ? 'destructive' : 'default'}
                >
                  {isLoading ? (
                    <>
                      <RefreshCw className="mr-2 h-4 w-4 animate-spin" />
                      Processing...
                    </>
                  ) : (
                    <>
                      {mode === 'set' && 'Enable Password Protection'}
                      {mode === 'change' && 'Change Password'}
                      {mode === 'remove' && 'Remove Protection'}
                    </>
                  )}
                </Button>
              </>
            )}
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
