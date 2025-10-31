// ABOUTME: Main Epics tab integrating list, detail, and generator views
// ABOUTME: Provides filtering by PRD and status, with create/edit/delete operations

import { useState } from 'react';
import { Plus, FileText, Loader2, Filter } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { EpicList } from './EpicList';
import { EpicDetail } from './EpicDetail';
import { EpicGenerator } from './EpicGenerator';
import { useEpics, useEpicsByPRD, useCreateEpic, useDeleteEpic } from '@/hooks/useEpics';
import { usePRDs } from '@/hooks/usePRDs';
import type { Epic, CreateEpicInput, EpicStatus } from '@/services/epics';

interface EpicsTabProps {
  projectId: string;
}

export function EpicsTab({ projectId }: EpicsTabProps) {
  const [selectedEpic, setSelectedEpic] = useState<Epic | null>(null);
  const [showGenerator, setShowGenerator] = useState(false);
  const [selectedPRDId, setSelectedPRDId] = useState<string>('all');
  const [statusFilter, setStatusFilter] = useState<EpicStatus | 'all'>('all');

  // Fetch data
  const { data: allEpics, isLoading: epicsLoading, error: epicsError } = useEpics(projectId);
  const { data: prdEpics, isLoading: prdEpicsLoading } = useEpicsByPRD(
    projectId,
    selectedPRDId !== 'all' ? selectedPRDId : ''
  );
  const { data: prds } = usePRDs(projectId);

  // Mutations
  const createEpicMutation = useCreateEpic(projectId);
  const deleteEpicMutation = useDeleteEpic(projectId);

  // Determine which epics to display
  const epics = selectedPRDId !== 'all' ? prdEpics : allEpics;
  const isLoading = selectedPRDId !== 'all' ? prdEpicsLoading : epicsLoading;

  // Apply status filter
  const filteredEpics = epics?.filter((epic) =>
    statusFilter === 'all' || epic.status === statusFilter
  ) || [];

  const handleSelect = (epic: Epic) => {
    setSelectedEpic(epic);
  };

  const handleDelete = async (epicId: string) => {
    await deleteEpicMutation.mutateAsync(epicId);
    if (selectedEpic?.id === epicId) {
      setSelectedEpic(null);
    }
  };

  const handleGenerateEpic = async (input: CreateEpicInput, _generateTasks: boolean) => {
    const newEpic = await createEpicMutation.mutateAsync(input);
    setSelectedEpic(newEpic);
  };

  const handleCreateFromPRD = (prdId: string) => {
    setSelectedPRDId(prdId);
    setShowGenerator(true);
  };

  // Get PRD title for selected PRD
  const selectedPRD = prds?.find((prd) => prd.id === selectedPRDId);
  const prdTitle = selectedPRD?.title || 'Unknown PRD';

  if (epicsError) {
    return (
      <Alert variant="destructive">
        <AlertTitle>Error loading epics</AlertTitle>
        <AlertDescription>{epicsError.message}</AlertDescription>
      </Alert>
    );
  }

  return (
    <div className="h-full flex flex-col">
      {/* Header */}
      <div className="flex items-center justify-between mb-6">
        <div>
          <h1 className="text-3xl font-bold">Epics</h1>
          <p className="text-muted-foreground mt-1">
            Manage implementation epics generated from PRDs
          </p>
        </div>
        <Button onClick={() => setShowGenerator(true)}>
          <Plus className="h-4 w-4 mr-2" />
          Create Epic
        </Button>
      </div>

      {/* Filters */}
      <div className="flex items-center gap-4 mb-6">
        <div className="flex items-center gap-2 flex-1">
          <Filter className="h-4 w-4 text-muted-foreground" />
          <Select value={selectedPRDId} onValueChange={setSelectedPRDId}>
            <SelectTrigger className="w-[250px]">
              <SelectValue placeholder="Filter by PRD" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">All PRDs</SelectItem>
              {prds?.map((prd) => (
                <SelectItem key={prd.id} value={prd.id}>
                  {prd.title}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>

          <Select value={statusFilter} onValueChange={(value) => setStatusFilter(value as EpicStatus | 'all')}>
            <SelectTrigger className="w-[200px]">
              <SelectValue placeholder="Filter by status" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">All Statuses</SelectItem>
              <SelectItem value="draft">Draft</SelectItem>
              <SelectItem value="ready">Ready</SelectItem>
              <SelectItem value="in_progress">In Progress</SelectItem>
              <SelectItem value="blocked">Blocked</SelectItem>
              <SelectItem value="completed">Completed</SelectItem>
              <SelectItem value="cancelled">Cancelled</SelectItem>
            </SelectContent>
          </Select>
        </div>

        {filteredEpics.length > 0 && (
          <span className="text-sm text-muted-foreground">
            {filteredEpics.length} {filteredEpics.length === 1 ? 'epic' : 'epics'}
          </span>
        )}
      </div>

      {/* Main Content */}
      <div className="flex-1 grid grid-cols-1 lg:grid-cols-2 gap-6 overflow-hidden">
        {/* Epic List */}
        <div className="overflow-y-auto">
          <EpicList
            epics={filteredEpics}
            isLoading={isLoading}
            onSelect={handleSelect}
            onDelete={handleDelete}
            selectedEpicId={selectedEpic?.id}
          />
        </div>

        {/* Epic Detail */}
        <div className="overflow-y-auto">
          {selectedEpic ? (
            <EpicDetail
              epic={selectedEpic}
              onEdit={() => {
                // TODO: Implement edit dialog
                alert('Edit functionality coming soon');
              }}
              onGenerateTasks={() => {
                // TODO: Implement task generation (Phase 4)
                alert('Task generation coming in Phase 4');
              }}
              onSyncSuccess={() => {
                // Refetch the epic to update sync status
                if (selectedEpic) {
                  setSelectedEpic({ ...selectedEpic });
                }
              }}
            />
          ) : (
            <div className="h-full flex items-center justify-center">
              <div className="text-center space-y-4">
                <FileText className="h-16 w-16 text-muted-foreground mx-auto" />
                <div>
                  <p className="text-lg font-medium text-muted-foreground">
                    No epic selected
                  </p>
                  <p className="text-sm text-muted-foreground mt-1">
                    Select an epic from the list to view details
                  </p>
                </div>
              </div>
            </div>
          )}
        </div>
      </div>

      {/* Epic Generator Dialog */}
      {showGenerator && selectedPRDId !== 'all' && (
        <EpicGenerator
          open={showGenerator}
          onOpenChange={setShowGenerator}
          prdId={selectedPRDId}
          prdTitle={prdTitle}
          onGenerate={handleGenerateEpic}
          isGenerating={createEpicMutation.isPending}
        />
      )}

      {/* Show message if trying to create without selecting a PRD */}
      {showGenerator && selectedPRDId === 'all' && (
        <Alert>
          <AlertDescription>
            Please select a PRD first to create an epic from it.
          </AlertDescription>
        </Alert>
      )}
    </div>
  );
}
