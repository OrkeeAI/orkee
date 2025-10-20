// ABOUTME: Specifications view displaying list of capabilities with expandable details
// ABOUTME: Integrates with SpecBuilderWizard and SpecDetailsView for management
import { useState } from 'react';
import { FileText, Plus, CheckCircle2, AlertCircle, Layers } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { useSpecs } from '@/hooks/useSpecs';
import { SpecBuilderWizard } from '@/components/SpecBuilderWizard';
import { SpecDetailsView } from '@/components/SpecDetailsView';
import type { SpecCapability } from '@/services/specs';

interface SpecificationsViewProps {
  projectId: string;
}

export function SpecificationsView({ projectId }: SpecificationsViewProps) {
  const [selectedSpec, setSelectedSpec] = useState<SpecCapability | null>(null);
  const [showCreateDialog, setShowCreateDialog] = useState(false);

  const { data: specs, isLoading, error } = useSpecs(projectId);

  const getStatusBadge = (status: string) => {
    switch (status) {
      case 'active':
        return (
          <Badge variant="default" className="flex items-center gap-1">
            <CheckCircle2 className="h-3 w-3" />
            Active
          </Badge>
        );
      case 'deprecated':
        return (
          <Badge variant="secondary" className="flex items-center gap-1">
            <AlertCircle className="h-3 w-3" />
            Deprecated
          </Badge>
        );
      default:
        return <Badge variant="outline">Archived</Badge>;
    }
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-center">
          <div className="h-8 w-8 animate-spin rounded-full border-2 border-primary border-t-transparent mx-auto mb-4" />
          <p className="text-muted-foreground">Loading specifications...</p>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <Alert variant="destructive">
        <AlertDescription>
          {error instanceof Error ? error.message : 'Failed to load specifications'}
        </AlertDescription>
      </Alert>
    );
  }

  if (!specs || specs.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center h-64 space-y-4">
        <Layers className="h-16 w-16 text-muted-foreground" />
        <div className="text-center space-y-2">
          <h3 className="text-lg font-semibold">No Specifications Yet</h3>
          <p className="text-sm text-muted-foreground max-w-md">
            Create specifications to define capabilities, requirements, and scenarios for your
            project.
          </p>
        </div>
        <Button onClick={() => setShowCreateDialog(true)}>
          <Plus className="mr-2 h-4 w-4" />
          Create Specification
        </Button>

        <SpecBuilderWizard
          projectId={projectId}
          open={showCreateDialog}
          onOpenChange={setShowCreateDialog}
        />
      </div>
    );
  }

  return (
    <div className="space-y-4">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-lg font-semibold">Specifications</h3>
          <p className="text-sm text-muted-foreground">
            {specs.length} {specs.length === 1 ? 'specification' : 'specifications'} defined
          </p>
        </div>
        <Button onClick={() => setShowCreateDialog(true)} size="sm">
          <Plus className="mr-2 h-4 w-4" />
          Create Spec
        </Button>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-4">
        {/* Spec List */}
        <div className="space-y-2">
          {specs.map((spec) => (
            <Card
              key={spec.id}
              className={`cursor-pointer transition-colors hover:border-primary/50 ${
                selectedSpec?.id === spec.id ? 'border-primary' : ''
              }`}
              onClick={() => setSelectedSpec(spec)}
            >
              <CardHeader className="pb-3">
                <div className="flex items-start justify-between">
                  <CardTitle className="text-sm font-medium line-clamp-2">{spec.name}</CardTitle>
                  {getStatusBadge(spec.status)}
                </div>
                <CardDescription className="text-xs line-clamp-2">{spec.purpose}</CardDescription>
              </CardHeader>
              <CardContent className="pb-3">
                <div className="flex items-center gap-4 text-xs text-muted-foreground">
                  <div className="flex items-center gap-1">
                    <FileText className="h-3 w-3" />
                    <span>{spec.requirementCount} requirements</span>
                  </div>
                  <div>v{spec.version}</div>
                </div>
              </CardContent>
            </Card>
          ))}
        </div>

        {/* Spec Details Viewer */}
        <div className="lg:col-span-2">
          {selectedSpec ? (
            <SpecDetailsView projectId={projectId} specId={selectedSpec.id} />
          ) : (
            <Card className="h-full flex items-center justify-center">
              <CardContent>
                <div className="text-center text-muted-foreground">
                  <Layers className="h-12 w-12 mx-auto mb-2 opacity-50" />
                  <p>Select a specification to view its details</p>
                </div>
              </CardContent>
            </Card>
          )}
        </div>
      </div>

      <SpecBuilderWizard
        projectId={projectId}
        open={showCreateDialog}
        onOpenChange={setShowCreateDialog}
      />
    </div>
  );
}
