import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { useDeleteProject } from '@/hooks/useProjects';
import { Project } from '@/services/projects';

interface ProjectDeleteDialogProps {
  project: Project | null;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onProjectDeleted: () => void;
}

export function ProjectDeleteDialog({ project, open, onOpenChange, onProjectDeleted }: ProjectDeleteDialogProps) {
  const deleteProjectMutation = useDeleteProject();
  const { isPending: loading, error } = deleteProjectMutation;

  const handleDelete = async () => {
    if (!project) return;

    try {
      await deleteProjectMutation.mutateAsync(project.id);
      onProjectDeleted();
      onOpenChange(false);
    } catch {
      // Error handled by React Query mutation
    }
  };

  if (!project) return null;

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[425px]">
        <DialogHeader>
          <DialogTitle>Delete Project</DialogTitle>
          <DialogDescription>
            Are you sure you want to delete this project? This action cannot be undone.
          </DialogDescription>
        </DialogHeader>
        
        <div className="py-4">
          {error && (
            <div className="text-sm text-red-600 bg-red-50 p-3 rounded-md mb-4">
              {error.message}
            </div>
          )}

          <div className="bg-gray-50 p-4 rounded-md">
            <h4 className="font-medium">{project.name}</h4>
            <p className="text-sm text-muted-foreground">{project.projectRoot}</p>
            {project.description && (
              <p className="text-sm text-muted-foreground mt-1">{project.description}</p>
            )}
          </div>
        </div>

        <DialogFooter>
          <Button
            type="button"
            variant="outline"
            onClick={() => onOpenChange(false)}
            disabled={loading}
          >
            Cancel
          </Button>
          <Button
            type="button"
            variant="destructive"
            onClick={handleDelete}
            disabled={loading}
          >
            {loading ? 'Deleting...' : 'Delete Project'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}