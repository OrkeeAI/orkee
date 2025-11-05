// ABOUTME: Execution section for task sheets with agent execution UI
// ABOUTME: Provides Execute button, modal, tabs for viewer/logs/artifacts/history

import { useState } from 'react';
import {
  PlayCircle,
  Activity,
  Terminal,
  FileText,
  History,
  ChevronDown,
  ChevronUp,
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import {
  ExecutionModal,
  ExecutionViewer,
  LogViewer,
  ArtifactGallery,
  ExecutionHistory,
} from '@/components/sandbox';
import type { AgentExecution } from '@/services/executions';

interface TaskExecutionSectionProps {
  taskId: string;
  defaultPrompt?: string;
}

export function TaskExecutionSection({ taskId, defaultPrompt }: TaskExecutionSectionProps) {
  const [showExecution, setShowExecution] = useState(false);
  const [isExecutionModalOpen, setIsExecutionModalOpen] = useState(false);
  const [activeExecutionId, setActiveExecutionId] = useState<string | null>(null);

  const handleExecutionCreated = (executionId: string) => {
    setActiveExecutionId(executionId);
    setShowExecution(true);
  };

  const handleViewExecution = (executionId: string) => {
    setActiveExecutionId(executionId);
    setShowExecution(true);
  };

  const handleRetryExecution = (execution: AgentExecution) => {
    // Open modal with the same prompt from the failed execution
    setIsExecutionModalOpen(true);
  };

  return (
    <div className="space-y-3">
      {/* Header with Execute Button */}
      <div className="flex items-center justify-between">
        <button
          onClick={() => setShowExecution(!showExecution)}
          className="flex items-center gap-2 text-sm font-medium text-gray-700 dark:text-gray-300 hover:text-gray-900 dark:hover:text-gray-100 transition-colors"
        >
          {showExecution ? <ChevronUp className="w-4 h-4" /> : <ChevronDown className="w-4 h-4" />}
          <Activity className="w-4 h-4" />
          Agent Execution
        </button>
        <Button
          size="sm"
          variant="default"
          onClick={() => setIsExecutionModalOpen(true)}
          className="flex items-center gap-2"
        >
          <PlayCircle className="w-4 h-4" />
          Execute with Agent
        </Button>
      </div>

      {/* Execution UI (collapsible) */}
      {showExecution && (
        <div className="space-y-3 pt-2">
          {activeExecutionId ? (
            <Tabs defaultValue="viewer" className="w-full">
              <TabsList className="grid w-full grid-cols-4">
                <TabsTrigger value="viewer" className="flex items-center gap-1">
                  <Activity className="w-3 h-3" />
                  <span className="hidden sm:inline">Viewer</span>
                </TabsTrigger>
                <TabsTrigger value="logs" className="flex items-center gap-1">
                  <Terminal className="w-3 h-3" />
                  <span className="hidden sm:inline">Logs</span>
                </TabsTrigger>
                <TabsTrigger value="artifacts" className="flex items-center gap-1">
                  <FileText className="w-3 h-3" />
                  <span className="hidden sm:inline">Artifacts</span>
                </TabsTrigger>
                <TabsTrigger value="history" className="flex items-center gap-1">
                  <History className="w-3 h-3" />
                  <span className="hidden sm:inline">History</span>
                </TabsTrigger>
              </TabsList>

              <TabsContent value="viewer" className="mt-3">
                <ExecutionViewer
                  executionId={activeExecutionId}
                  onRetry={() => setIsExecutionModalOpen(true)}
                />
              </TabsContent>

              <TabsContent value="logs" className="mt-3">
                <LogViewer executionId={activeExecutionId} />
              </TabsContent>

              <TabsContent value="artifacts" className="mt-3">
                <ArtifactGallery executionId={activeExecutionId} />
              </TabsContent>

              <TabsContent value="history" className="mt-3">
                <ExecutionHistory
                  taskId={taskId}
                  onViewExecution={handleViewExecution}
                  onRetryExecution={handleRetryExecution}
                />
              </TabsContent>
            </Tabs>
          ) : (
            <div className="text-center py-8 text-sm text-gray-500 dark:text-gray-400">
              <PlayCircle className="w-8 h-8 mx-auto mb-2 opacity-50" />
              <p>No active execution</p>
              <p className="text-xs mt-1">Click "Execute with Agent" to start</p>
            </div>
          )}
        </div>
      )}

      {/* Execution Modal */}
      <ExecutionModal
        open={isExecutionModalOpen}
        onOpenChange={setIsExecutionModalOpen}
        taskId={taskId}
        defaultPrompt={defaultPrompt}
        onExecutionCreated={handleExecutionCreated}
      />
    </div>
  );
}
