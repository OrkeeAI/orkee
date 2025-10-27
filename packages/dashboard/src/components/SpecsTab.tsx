// ABOUTME: Container component for Specs tab with OpenSpec workflow sections
// ABOUTME: Integrates PRDView, ChangesView, SpecificationsView, ArchiveView, and CoverageView with nested tabs
import { useState } from 'react';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { FileText, Layers, BarChart, GitBranch, Archive } from 'lucide-react';
import { PRDView } from '@/components/specs/PRDView';
import { ChangesView } from '@/components/specs/ChangesView';
import { SpecificationsView } from '@/components/specs/SpecificationsView';
import { ArchiveView } from '@/components/specs/ArchiveView';
import { CoverageView } from '@/components/specs/CoverageView';

interface SpecsTabProps {
  projectId: string;
}

export function SpecsTab({ projectId }: SpecsTabProps) {
  const [activeTab, setActiveTab] = useState('prds');
  const [filterPrdId, setFilterPrdId] = useState<string | null>(null);

  const handleViewSpecs = (prdId: string) => {
    setFilterPrdId(prdId);
    setActiveTab('specs');
  };

  return (
    <Tabs value={activeTab} onValueChange={setActiveTab} className="space-y-4">
      <TabsList>
        <TabsTrigger value="prds" className="flex items-center gap-2">
          <FileText className="h-4 w-4" />
          PRDs
        </TabsTrigger>
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
        <TabsTrigger value="coverage" className="flex items-center gap-2">
          <BarChart className="h-4 w-4" />
          Coverage
        </TabsTrigger>
      </TabsList>

      <TabsContent value="prds" className="space-y-4">
        <PRDView projectId={projectId} onViewSpecs={handleViewSpecs} />
      </TabsContent>

      <TabsContent value="changes" className="space-y-4">
        <ChangesView projectId={projectId} />
      </TabsContent>

      <TabsContent value="specs" className="space-y-4">
        <SpecificationsView projectId={projectId} filterPrdId={filterPrdId} onClearFilter={() => setFilterPrdId(null)} />
      </TabsContent>

      <TabsContent value="archive" className="space-y-4">
        <ArchiveView projectId={projectId} />
      </TabsContent>

      <TabsContent value="coverage" className="space-y-4">
        <CoverageView projectId={projectId} />
      </TabsContent>
    </Tabs>
  );
}
