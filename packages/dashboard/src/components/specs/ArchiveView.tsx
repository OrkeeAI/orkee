// ABOUTME: Archive view showing archived/completed changes
// ABOUTME: Historical view of changes that have been implemented and archived

import { useState } from 'react';
import { ChangesList } from '@/components/changes/ChangesList';
import { ChangeDetails } from '@/components/changes/ChangeDetails';

interface ArchiveViewProps {
  projectId: string;
}

export function ArchiveView({ projectId }: ArchiveViewProps) {
  const [selectedChangeId, setSelectedChangeId] = useState<string | null>(null);

  return (
    <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
      <div>
        <ChangesList
          projectId={projectId}
          onSelectChange={setSelectedChangeId}
          statusFilter="archived"
        />
      </div>
      <div>
        {selectedChangeId ? (
          <ChangeDetails
            projectId={projectId}
            changeId={selectedChangeId}
          />
        ) : (
          <div className="flex items-center justify-center h-full">
            <p className="text-muted-foreground">Select an archived change to view details</p>
          </div>
        )}
      </div>
    </div>
  );
}
