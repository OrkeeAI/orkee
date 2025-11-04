// ABOUTME: Gallery component for viewing and downloading execution artifacts
// ABOUTME: Displays files, screenshots, test reports, and other execution outputs

import { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import {
  File,
  FileText,
  Image as ImageIcon,
  Download,
  Trash2,
  AlertCircle,
  Loader2,
  Grid3x3,
  List,
  CheckSquare,
} from 'lucide-react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Alert, AlertDescription } from '@/components/ui/alert';
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog';
import { sandboxService, type Artifact } from '@/services/sandbox';
import { toast } from 'sonner';

interface ArtifactGalleryProps {
  executionId: string;
}

type ViewMode = 'grid' | 'list';

export function ArtifactGallery({ executionId }: ArtifactGalleryProps) {
  const [viewMode, setViewMode] = useState<ViewMode>('grid');
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const [artifactToDelete, setArtifactToDelete] = useState<Artifact | null>(null);

  // Fetch artifacts
  const {
    data: artifacts = [],
    isLoading,
    error,
    refetch,
  } = useQuery({
    queryKey: ['artifacts', executionId],
    queryFn: () => sandboxService.listArtifacts(executionId),
  });

  const handleDownload = (artifact: Artifact) => {
    const url = sandboxService.getArtifactDownloadUrl(artifact.id);
    window.open(url, '_blank');
  };

  const handleDeleteClick = (artifact: Artifact) => {
    setArtifactToDelete(artifact);
    setDeleteDialogOpen(true);
  };

  const handleDeleteConfirm = async () => {
    if (!artifactToDelete) return;

    try {
      await sandboxService.deleteArtifact(artifactToDelete.id);
      toast.success('Artifact deleted successfully');
      refetch();
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Failed to delete artifact');
    } finally {
      setDeleteDialogOpen(false);
      setArtifactToDelete(null);
    }
  };

  const getArtifactIcon = (artifact: Artifact) => {
    switch (artifact.artifact_type) {
      case 'screenshot':
        return <ImageIcon className="h-5 w-5" />;
      case 'test_report':
      case 'coverage':
        return <CheckSquare className="h-5 w-5" />;
      case 'file':
        return <File className="h-5 w-5" />;
      default:
        return <FileText className="h-5 w-5" />;
    }
  };

  const getArtifactTypeBadgeVariant = (type: string): "default" | "secondary" | "outline" => {
    switch (type) {
      case 'test_report':
      case 'coverage':
        return 'default';
      case 'screenshot':
        return 'secondary';
      default:
        return 'outline';
    }
  };

  const formatFileSize = (bytes?: number) => {
    if (!bytes) return 'Unknown size';
    const kb = bytes / 1024;
    if (kb < 1024) return `${kb.toFixed(1)} KB`;
    const mb = kb / 1024;
    return `${mb.toFixed(1)} MB`;
  };

  if (isLoading) {
    return (
      <Card>
        <CardContent className="flex items-center justify-center py-12">
          <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
        </CardContent>
      </Card>
    );
  }

  if (error) {
    return (
      <Alert variant="destructive">
        <AlertCircle className="h-4 w-4" />
        <AlertDescription>
          {error instanceof Error ? error.message : 'Failed to load artifacts'}
        </AlertDescription>
      </Alert>
    );
  }

  return (
    <>
      <Card>
        <CardHeader>
          <div className="flex items-start justify-between">
            <div className="space-y-1">
              <CardTitle className="flex items-center gap-2">
                <File className="h-5 w-5" />
                Execution Artifacts
              </CardTitle>
              <CardDescription>
                Files, screenshots, and test reports generated during execution
              </CardDescription>
            </div>
            <div className="flex items-center gap-2">
              <Button
                variant={viewMode === 'grid' ? 'default' : 'outline'}
                size="sm"
                onClick={() => setViewMode('grid')}
              >
                <Grid3x3 className="h-4 w-4" />
              </Button>
              <Button
                variant={viewMode === 'list' ? 'default' : 'outline'}
                size="sm"
                onClick={() => setViewMode('list')}
              >
                <List className="h-4 w-4" />
              </Button>
            </div>
          </div>
        </CardHeader>
        <CardContent>
          {artifacts.length === 0 ? (
            <div className="text-center py-12 text-muted-foreground">
              <File className="h-12 w-12 mx-auto mb-4 opacity-50" />
              <p>No artifacts generated yet</p>
            </div>
          ) : (
            <>
              {viewMode === 'grid' ? (
                <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
                  {artifacts.map((artifact) => (
                    <Card key={artifact.id} className="overflow-hidden">
                      <CardContent className="p-4 space-y-3">
                        <div className="flex items-start gap-3">
                          <div className="p-2 bg-muted rounded">
                            {getArtifactIcon(artifact)}
                          </div>
                          <div className="flex-1 min-w-0">
                            <h4 className="text-sm font-medium truncate" title={artifact.file_name}>
                              {artifact.file_name}
                            </h4>
                            <p className="text-xs text-muted-foreground">
                              {formatFileSize(artifact.file_size_bytes)}
                            </p>
                          </div>
                        </div>

                        <Badge variant={getArtifactTypeBadgeVariant(artifact.artifact_type)}>
                          {artifact.artifact_type.replace('_', ' ')}
                        </Badge>

                        {artifact.description && (
                          <p className="text-xs text-muted-foreground line-clamp-2">
                            {artifact.description}
                          </p>
                        )}

                        <div className="flex gap-2">
                          <Button
                            variant="outline"
                            size="sm"
                            className="flex-1"
                            onClick={() => handleDownload(artifact)}
                          >
                            <Download className="mr-2 h-3 w-3" />
                            Download
                          </Button>
                          <Button
                            variant="outline"
                            size="sm"
                            onClick={() => handleDeleteClick(artifact)}
                          >
                            <Trash2 className="h-3 w-3" />
                          </Button>
                        </div>
                      </CardContent>
                    </Card>
                  ))}
                </div>
              ) : (
                <div className="space-y-2">
                  {artifacts.map((artifact) => (
                    <div
                      key={artifact.id}
                      className="flex items-center justify-between p-3 border rounded-lg hover:bg-muted/50"
                    >
                      <div className="flex items-center gap-3 flex-1 min-w-0">
                        <div className="p-2 bg-muted rounded shrink-0">
                          {getArtifactIcon(artifact)}
                        </div>
                        <div className="flex-1 min-w-0">
                          <h4 className="text-sm font-medium truncate" title={artifact.file_name}>
                            {artifact.file_name}
                          </h4>
                          <div className="flex items-center gap-2 text-xs text-muted-foreground">
                            <Badge
                              variant={getArtifactTypeBadgeVariant(artifact.artifact_type)}
                              className="text-xs"
                            >
                              {artifact.artifact_type.replace('_', ' ')}
                            </Badge>
                            <span>{formatFileSize(artifact.file_size_bytes)}</span>
                            {artifact.mime_type && (
                              <span className="truncate">{artifact.mime_type}</span>
                            )}
                          </div>
                        </div>
                      </div>
                      <div className="flex gap-2 shrink-0">
                        <Button
                          variant="outline"
                          size="sm"
                          onClick={() => handleDownload(artifact)}
                        >
                          <Download className="mr-2 h-3 w-3" />
                          Download
                        </Button>
                        <Button
                          variant="outline"
                          size="sm"
                          onClick={() => handleDeleteClick(artifact)}
                        >
                          <Trash2 className="h-3 w-3" />
                        </Button>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </>
          )}
        </CardContent>
      </Card>

      {/* Delete Confirmation Dialog */}
      <AlertDialog open={deleteDialogOpen} onOpenChange={setDeleteDialogOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Delete Artifact?</AlertDialogTitle>
            <AlertDialogDescription>
              Are you sure you want to delete "{artifactToDelete?.file_name}"? This action
              cannot be undone.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction onClick={handleDeleteConfirm}>Delete</AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  );
}
