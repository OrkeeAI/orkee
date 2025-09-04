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
import { projectsService, Project, ProjectUpdateInput, ProjectStatus, Priority } from '@/services/projects';

interface ProjectEditDialogProps {
  project: Project | null;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onProjectUpdated: () => void;
}

export function ProjectEditDialog({ project, open, onOpenChange, onProjectUpdated }: ProjectEditDialogProps) {
  const [formData, setFormData] = useState<ProjectUpdateInput>({});
  const [tagsInput, setTagsInput] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

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
      });
      setTagsInput(project.tags?.join(', ') || '');
      setError(null);
    }
  }, [project]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!project) return;
    
    setLoading(true);
    setError(null);

    try {
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

      await projectsService.updateProject(project.id, updateData);
      onProjectUpdated();
      onOpenChange(false);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to update project');
    } finally {
      setLoading(false);
    }
  };

  if (!project) return null;

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[625px]">
        <form onSubmit={handleSubmit}>
          <DialogHeader>
            <DialogTitle>Edit Project</DialogTitle>
            <DialogDescription>
              Update the project details below.
            </DialogDescription>
          </DialogHeader>
          
          <div className="py-4">
            {error && (
              <div className="text-sm text-red-600 bg-red-50 p-3 rounded-md mb-4">
                {error}
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
                  onChange={(value) => setFormData({ ...formData, projectRoot: value })}
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

          <DialogFooter>
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