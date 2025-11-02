// ABOUTME: Container component for Specs tab with ideation, PRDs, Epics, Tasks, and coverage
// ABOUTME: Integrates IdeateTab, PRDView, EpicsTab, TasksTab, and CoverageView
import { useState } from 'react';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { FileText, BarChart, Lightbulb, Layers, ListTodo } from 'lucide-react';
import { IdeateTab } from '@/components/specs/IdeateTab';
import { PRDView } from '@/components/specs/PRDView';
import { EpicsTab } from '@/components/epics/EpicsTab';
import { TasksTab } from '@/components/TasksTab';
import { CoverageView } from '@/components/specs/CoverageView';

interface SpecsTabProps {
  projectId: string;
  projectPath: string;
  taskSource: string;
}

export function SpecsTab({ projectId, projectPath, taskSource }: SpecsTabProps) {
  const [activeTab, setActiveTab] = useState('ideate');

  return (
    <Tabs value={activeTab} onValueChange={setActiveTab} className="space-y-4">
      <TabsList>
        <TabsTrigger value="ideate" className="flex items-center gap-2">
          <Lightbulb className="h-4 w-4" />
          Ideate
        </TabsTrigger>
        <TabsTrigger value="prds" className="flex items-center gap-2">
          <FileText className="h-4 w-4" />
          PRDs
        </TabsTrigger>
        <TabsTrigger value="epics" className="flex items-center gap-2">
          <Layers className="h-4 w-4" />
          Epics
        </TabsTrigger>
        <TabsTrigger value="tasks" className="flex items-center gap-2">
          <ListTodo className="h-4 w-4" />
          Tasks
        </TabsTrigger>
        <TabsTrigger value="coverage" className="flex items-center gap-2">
          <BarChart className="h-4 w-4" />
          Coverage
        </TabsTrigger>
      </TabsList>

      <TabsContent value="ideate" className="space-y-4">
        <IdeateTab projectId={projectId} />
      </TabsContent>

      <TabsContent value="prds" className="space-y-4">
        <PRDView projectId={projectId} />
      </TabsContent>

      <TabsContent value="epics" className="space-y-4">
        <EpicsTab projectId={projectId} />
      </TabsContent>

      <TabsContent value="tasks" className="space-y-4">
        <TasksTab projectId={projectId} projectPath={projectPath} taskSource={taskSource} />
      </TabsContent>

      <TabsContent value="coverage" className="space-y-4">
        <CoverageView projectId={projectId} />
      </TabsContent>
    </Tabs>
  );
}
