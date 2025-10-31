// ABOUTME: OpenSpec parent tab containing Changes, Specs, and Archive as nested subtabs
// ABOUTME: Manages change proposals, specifications, and archived items as grouped workflow
import { useState } from 'react';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { GitBranch, Layers, Archive } from 'lucide-react';
import { ChangesView } from './ChangesView';
import { SpecificationsView } from './SpecificationsView';
import { ArchiveView } from './ArchiveView';

interface OpenSpecTabProps {
  projectId: string;
}

export function OpenSpecTab({ projectId }: OpenSpecTabProps) {
  const [activeSubTab, setActiveSubTab] = useState('changes');
  const [filterPrdId, setFilterPrdId] = useState<string | null>(null);

  return (
    <Tabs value={activeSubTab} onValueChange={setActiveSubTab} className="space-y-4">
      <TabsList>
        <TabsTrigger value="changes" className="flex items-center gap-2">
          <GitBranch className="h-4 w-4" />
          Changes
        </TabsTrigger>
        <TabsTrigger value="specs" className="flex items-center gap-2">
          <Layers className="h-4 w-4" />
          Specs
        </TabsTrigger>
        <TabsTrigger value="archive" className="flex items-center gap-2">
          <Archive className="h-4 w-4" />
          Archive
        </TabsTrigger>
      </TabsList>

      <TabsContent value="changes" className="space-y-4">
        <ChangesView projectId={projectId} />
      </TabsContent>

      <TabsContent value="specs" className="space-y-4">
        <SpecificationsView projectId={projectId} filterPrdId={filterPrdId} onClearFilter={() => setFilterPrdId(null)} />
      </TabsContent>

      <TabsContent value="archive" className="space-y-4">
        <ArchiveView projectId={projectId} />
      </TabsContent>
    </Tabs>
  );
}
