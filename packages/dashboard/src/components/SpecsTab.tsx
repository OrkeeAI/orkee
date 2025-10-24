// ABOUTME: Container component for Specs tab with three sub-sections
// ABOUTME: Integrates PRDView, SpecificationsView, and CoverageView with nested tabs
import { useState } from 'react';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { FileText, Layers, BarChart } from 'lucide-react';
import { PRDView } from '@/components/specs/PRDView';
import { SpecificationsView } from '@/components/specs/SpecificationsView';
import { CoverageView } from '@/components/specs/CoverageView';

interface SpecsTabProps {
  projectId: string;
}

export function SpecsTab({ projectId }: SpecsTabProps) {
  const [activeTab, setActiveTab] = useState('prd');
  const [filterPrdId, setFilterPrdId] = useState<string | null>(null);

  const handleViewSpecs = (prdId: string) => {
    setFilterPrdId(prdId);
    setActiveTab('specs');
  };

  return (
    <Tabs value={activeTab} onValueChange={setActiveTab} className="space-y-4">
      <TabsList>
        <TabsTrigger value="prd" className="flex items-center gap-2">
          <FileText className="h-4 w-4" />
          PRD
        </TabsTrigger>
        <TabsTrigger value="specs" className="flex items-center gap-2">
          <Layers className="h-4 w-4" />
          Specifications
        </TabsTrigger>
        <TabsTrigger value="coverage" className="flex items-center gap-2">
          <BarChart className="h-4 w-4" />
          Coverage
        </TabsTrigger>
      </TabsList>

      <TabsContent value="prd" className="space-y-4">
        <PRDView projectId={projectId} onViewSpecs={handleViewSpecs} />
      </TabsContent>

      <TabsContent value="specs" className="space-y-4">
        <SpecificationsView projectId={projectId} filterPrdId={filterPrdId} onClearFilter={() => setFilterPrdId(null)} />
      </TabsContent>

      <TabsContent value="coverage" className="space-y-4">
        <CoverageView projectId={projectId} />
      </TabsContent>
    </Tabs>
  );
}
