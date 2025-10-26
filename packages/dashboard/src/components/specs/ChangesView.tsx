// ABOUTME: OpenSpec changes view showing change proposals for the project
// ABOUTME: Displays list of changes with detail panel for selected change

import { useState } from 'react';
import { ChangesList } from '@/components/changes/ChangesList';
import { ChangeDetails } from '@/components/changes/ChangeDetails';

interface ChangesViewProps {
  projectId: string;
}

export function ChangesView({ projectId }: ChangesViewProps) {
  const [selectedChangeId, setSelectedChangeId] = useState<string | null>(null);

  return (
    <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
      <div>
        <ChangesList
          projectId={projectId}
          onSelectChange={setSelectedChangeId}
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
            <p className="text-muted-foreground">Select a change to view details</p>
          </div>
        )}
      </div>
    </div>
  );
}
