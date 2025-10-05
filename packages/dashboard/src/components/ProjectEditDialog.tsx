import React, { useState, useEffect } from 'react';
import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { DirectorySelector } from '@/components/DirectorySelector';
import { Check } from 'lucide-react';
import { useUpdateProject } from '@/hooks/useProjects';
import { Project, ProjectUpdateInput, ProjectStatus, Priority, TaskSource } from '@/services/projects';

interface ProjectEditDialogProps {
  project: Project | null;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onProjectUpdated: () => void;
}

export function ProjectEditDialog({ project, open, onOpenChange, onProjectUpdated }: ProjectEditDialogProps) {
  const [formData, setFormData] = useState<ProjectUpdateInput>({});
  const [tagsInput, setTagsInput] = useState('');
  const [hasTaskmaster, setHasTaskmaster] = useState(false);
  const [checkingTaskmaster, setCheckingTaskmaster] = useState(false);

  const updateProjectMutation = useUpdateProject();
  const { isPending: loading, error } = updateProjectMutation;

  // Check for taskmaster folder
  const checkTaskmasterFolder = async (projectRoot: string) => {
    if (!projectRoot) return;
    
    setCheckingTaskmaster(true);
    try {
      const response = await fetch('/api/projects/check-taskmaster', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ projectRoot }),
      });
      
      if (response.ok) {
        const result = await response.json();
        if (result.success) {
          setHasTaskmaster(result.data.hasTaskmaster);
          // Don't auto-change task source when editing existing projects
          // The user's current selection should be preserved
        }
      }
    } catch (error) {
      console.error('Failed to check taskmaster folder:', error);
    } finally {
      setCheckingTaskmaster(false);
    }
  };

  // Populate form when project changes
  useEffect(() => {
    if (project) {
      setFormData({
        name: project.name,
        projectRoot: project.projectRoot,
        description: project.description || '',
        status: project.status,
        priority: project.priority,
        setupScript: project.setupScript || '',
        devScript: project.devScript || '',
        cleanupScript: project.cleanupScript || '',
        taskSource: project.taskSource,
      });
      setTagsInput(project.tags?.join(', ') || '');
      updateProjectMutation.reset();
      
      // Check for taskmaster folder when project loads
      checkTaskmasterFolder(project.projectRoot);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [project]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!project) return;

    // Parse tags from comma-separated string
    const tags = tagsInput
      .split(',')
      .map(tag => tag.trim())
      .filter(tag => tag.length > 0);

    const updateData: ProjectUpdateInput = {
      ...formData,
      tags: tags.length > 0 ? tags : [],
      // Send empty strings for cleared fields so backend can properly clear them
      description: formData.description?.trim() || '',
      setupScript: formData.setupScript?.trim() || '',
      devScript: formData.devScript?.trim() || '',
      cleanupScript: formData.cleanupScript?.trim() || '',
    };

    try {
      await updateProjectMutation.mutateAsync({
        id: project.id,
        input: updateData
      });
      onProjectUpdated();
      onOpenChange(false);
    } catch {
      // Error handled by React Query mutation
    }
  };

  if (!project) return null;

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[625px] max-h-[90vh]" aria-describedby="edit-project-description">
        <form onSubmit={handleSubmit} className="flex flex-col max-h-[calc(90vh-3rem)]">
          <DialogHeader className="flex-shrink-0">
            <DialogTitle>Edit Project</DialogTitle>
            <DialogDescription id="edit-project-description">
              Update the project details below.
            </DialogDescription>
          </DialogHeader>

          <div className="py-4 overflow-y-auto min-h-0 px-1">
            {error && (
              <div className="text-sm text-red-600 bg-red-50 p-3 rounded-md mb-4">
                {error.message}
              </div>
            )}

            <Tabs defaultValue="general" className="w-full">
              <TabsList className="grid w-full grid-cols-2">
                <TabsTrigger value="general">General</TabsTrigger>
                <TabsTrigger value="scripts">Scripts</TabsTrigger>
              </TabsList>
              
              <TabsContent value="general" className="space-y-4 mt-4">
                <div className="grid gap-2">
                  <Label htmlFor="name">Project Name *</Label>
                  <Input
                    id="name"
                    value={formData.name || ''}
                    onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                    placeholder="My Awesome Project"
                    required
                  />
                </div>

                <DirectorySelector
                  id="projectRoot"
                  label="Project Root Path"
                  value={formData.projectRoot || ''}
                  onChange={(value) => {
                    setFormData({ ...formData, projectRoot: value });
                    // Check for taskmaster folder when path changes
                    if (value) {
                      checkTaskmasterFolder(value);
                    }
                  }}
                  placeholder="/path/to/project"
                  required
                />

                <div className="grid gap-2">
                  <Label htmlFor="description">Description</Label>
                  <Textarea
                    id="description"
                    value={formData.description || ''}
                    onChange={(e) => setFormData({ ...formData, description: e.target.value })}
                    placeholder="Brief description of the project"
                    rows={3}
                  />
                </div>

                <div className="grid grid-cols-2 gap-4">
                  <div className="grid gap-2">
                    <Label htmlFor="status">Status</Label>
                    <Select
                      value={formData.status}
                      onValueChange={(value: ProjectStatus) => setFormData({ ...formData, status: value })}
                      modal={false}
                    >
                      <SelectTrigger>
                        <SelectValue placeholder="Select status" />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="active">Active</SelectItem>
                        <SelectItem value="archived">Archived</SelectItem>
                      </SelectContent>
                    </Select>
                  </div>

                  <div className="grid gap-2">
                    <Label htmlFor="priority">Priority</Label>
                    <Select
                      value={formData.priority}
                      onValueChange={(value: Priority) => setFormData({ ...formData, priority: value })}
                      modal={false}
                    >
                      <SelectTrigger>
                        <SelectValue placeholder="Select priority" />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="high">High</SelectItem>
                        <SelectItem value="medium">Medium</SelectItem>
                        <SelectItem value="low">Low</SelectItem>
                      </SelectContent>
                    </Select>
                  </div>
                </div>

                <div className="grid gap-2">
                  <Label>Task Management</Label>
                  <Select
                    value={formData.taskSource || 'manual'}
                    onValueChange={(value: TaskSource) => setFormData({ ...formData, taskSource: value })}
                    modal={false}
                  >
                    <SelectTrigger>
                      <SelectValue placeholder="Select task source" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="taskmaster">Taskmaster (.taskmaster folder)</SelectItem>
                      <SelectItem value="manual">Manual Tasks</SelectItem>
                    </SelectContent>
                  </Select>
                  
                  {checkingTaskmaster && (
                    <div className="flex items-center gap-2 text-sm text-muted-foreground">
                      <div className="h-4 w-4 animate-spin rounded-full border-2 border-primary border-t-transparent" />
                      Checking for .taskmaster folder...
                    </div>
                  )}
                  
                  {!checkingTaskmaster && hasTaskmaster && (
                    <div className="flex items-center gap-2 text-sm text-green-600">
                      <Check className="h-4 w-4" />
                      .taskmaster folder detected
                    </div>
                  )}
                  
                  <p className="text-xs text-muted-foreground">
                    {formData.taskSource === 'taskmaster' 
                      ? 'Tasks will be read from the Taskmaster configuration file'
                      : 'Tasks will be managed manually within this application'
                    }
                  </p>
                </div>

                <div className="grid gap-2">
                  <Label htmlFor="tags">Tags</Label>
                  <Input
                    id="tags"
                    value={tagsInput}
                    onChange={(e) => setTagsInput(e.target.value)}
                    placeholder="react, typescript, web (comma separated)"
                  />
                </div>
              </TabsContent>
              
              <TabsContent value="scripts" className="space-y-4 mt-4">

                <div className="grid gap-2">
                  <Label htmlFor="setupScript">Setup Script</Label>
                  <Input
                    id="setupScript"
                    value={formData.setupScript || ''}
                    onChange={(e) => setFormData({ ...formData, setupScript: e.target.value })}
                    placeholder="npm install"
                  />
                  <p className="text-xs text-muted-foreground">
                    Command to set up project dependencies
                  </p>
                </div>

                <div className="grid gap-2">
                  <Label htmlFor="devScript">Development Script</Label>
                  <Input
                    id="devScript"
                    value={formData.devScript || ''}
                    onChange={(e) => setFormData({ ...formData, devScript: e.target.value })}
                    placeholder="npm run dev"
                  />
                  <p className="text-xs text-muted-foreground">
                    Command to start development server
                  </p>
                </div>

                <div className="grid gap-2">
                  <Label htmlFor="cleanupScript">Cleanup Script</Label>
                  <Input
                    id="cleanupScript"
                    value={formData.cleanupScript || ''}
                    onChange={(e) => setFormData({ ...formData, cleanupScript: e.target.value })}
                    placeholder="npm run clean"
                  />
                  <p className="text-xs text-muted-foreground">
                    Command to clean up project build files
                  </p>
                </div>
                
                <div className="text-xs text-muted-foreground mt-4">
                  Created: {new Date(project.createdAt).toLocaleDateString()}<br />
                  Last updated: {new Date(project.updatedAt).toLocaleDateString()}
                </div>
              </TabsContent>
            </Tabs>
          </div>

          <DialogFooter className="flex-shrink-0">
            <Button
              type="button"
              variant="outline"
              onClick={() => onOpenChange(false)}
              disabled={loading}
            >
              Cancel
            </Button>
            <Button type="submit" disabled={loading || !formData.name || !formData.projectRoot}>
              {loading ? 'Updating...' : 'Update Project'}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}