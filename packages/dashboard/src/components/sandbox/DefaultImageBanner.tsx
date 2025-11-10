// ABOUTME: Banner displaying the current default Docker image for new sandboxes
// ABOUTME: Shows image name prominently with visual indicator

import { useEffect, useState } from 'react';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Package } from 'lucide-react';
import { getSandboxSettings, type SandboxSettings } from '@/services/sandbox';
import { useToast } from '@/hooks/use-toast';

interface DefaultImageBannerProps {
  refreshTrigger?: number;
}

export function DefaultImageBanner({ refreshTrigger }: DefaultImageBannerProps) {
  const [settings, setSettings] = useState<SandboxSettings | null>(null);
  const { toast } = useToast();

  useEffect(() => {
    const loadSettings = async () => {
      try {
        const data = await getSandboxSettings();
        setSettings(data);
      } catch (error) {
        toast({
          title: 'Failed to load sandbox settings',
          description: error instanceof Error ? error.message : 'Unknown error',
          variant: 'destructive',
        });
      }
    };

    loadSettings();
  }, [refreshTrigger, toast]);

  if (!settings) {
    return null;
  }

  return (
    <Alert>
      <Package className="h-4 w-4" />
      <AlertDescription>
        <span className="text-sm text-muted-foreground">Default image for new sandboxes: </span>
        <span className="font-mono font-semibold">{settings.default_image}</span>
      </AlertDescription>
    </Alert>
  );
}
