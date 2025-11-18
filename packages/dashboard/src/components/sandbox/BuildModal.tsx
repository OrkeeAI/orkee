// ABOUTME: Modal dialog for building Docker images
// ABOUTME: Combines build form and progress display in a dialog

import { useState } from 'react';
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { DockerBuildForm } from './DockerBuildForm';
import { BuildProgressDisplay } from './BuildProgressDisplay';
import type { BuildImageResponse } from '@/services/docker';

interface BuildModalProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  username?: string | null;
  onBuildComplete?: () => void;
  onLoginClick?: () => void;
}

export function BuildModal({ open, onOpenChange, username, onBuildComplete, onLoginClick }: BuildModalProps) {
  const [isBuilding, setIsBuilding] = useState(false);
  const [buildOutput, setBuildOutput] = useState<BuildImageResponse | null>(null);

  const handleBuildStart = () => {
    setIsBuilding(true);
    setBuildOutput(null);
  };

  const handleBuildComplete = (response: BuildImageResponse) => {
    setIsBuilding(false);
    setBuildOutput(response);
    onBuildComplete?.();
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-4xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>Build Docker Image</DialogTitle>
          <DialogDescription>
            Build a custom Docker image for your sandboxes
          </DialogDescription>
        </DialogHeader>
        <div className="space-y-6">
          <DockerBuildForm
            username={username}
            onBuildStart={handleBuildStart}
            onBuildComplete={handleBuildComplete}
            onLoginClick={onLoginClick}
          />
          <BuildProgressDisplay
            buildOutput={buildOutput}
            isBuilding={isBuilding}
          />
        </div>
      </DialogContent>
    </Dialog>
  );
}
