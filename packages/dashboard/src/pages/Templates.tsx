// ABOUTME: PRD template management page
// ABOUTME: Create, edit, and delete global PRD templates for ideation
import { useState, useEffect } from 'react';
import { FileText, Plus, Edit, Trash2, Eye, Download } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from '@/components/ui/dialog';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { toast } from 'sonner';

interface PRDTemplate {
  id: string;
  name: string;
  description?: string;
  content: string;
  created_at: string;
  updated_at?: string;
  is_default?: boolean;
}

export function Templates() {
  const [templates, setTemplates] = useState<PRDTemplate[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [showCreateDialog, setShowCreateDialog] = useState(false);
  const [showEditDialog, setShowEditDialog] = useState(false);
  const [showPreviewDialog, setShowPreviewDialog] = useState(false);
  const [selectedTemplate, setSelectedTemplate] = useState<PRDTemplate | null>(null);

  // Form state
  const [templateName, setTemplateName] = useState('');
  const [templateDescription, setTemplateDescription] = useState('');
  const [templateContent, setTemplateContent] = useState('');

  useEffect(() => {
    loadTemplates();
  }, []);

  const loadTemplates = async () => {
    try {
      setIsLoading(true);
      // TODO: Replace with actual API call when backend is ready
      // For now, load from localStorage
      const stored = localStorage.getItem('prd-templates');
      if (stored) {
        setTemplates(JSON.parse(stored));
      } else {
        // Initialize with example templates
        const exampleTemplates: PRDTemplate[] = [
          {
            id: 'standard',
            name: 'Standard PRD',
            description: 'Default template for general product requirements',
            content: '# Product Requirements Document\n\n## Overview\n\n**Problem Statement**: [Describe the problem]\n\n**Target Audience**: [Who are the users?]\n\n**Value Proposition**: [Why is this solution better?]\n\n## Core Features\n\n### Feature 1\n- **What**: [Description]\n- **Why**: [Importance]\n- **How**: [Implementation approach]\n\n## Technical Architecture\n\n[Technical details]\n\n## User Experience\n\n### Personas\n[User personas]\n\n### User Flows\n[Key user journeys]\n\n## Roadmap\n\n### MVP Scope\n[Minimum viable features]\n\n### Future Phases\n[Post-MVP features]\n\n## Risks & Mitigations\n\n[Potential risks and how to address them]\n',
            created_at: new Date().toISOString(),
            is_default: true,
          },
        ];
        localStorage.setItem('prd-templates', JSON.stringify(exampleTemplates));
        setTemplates(exampleTemplates);
      }
    } catch (error) {
      toast.error('Failed to load templates');
      console.error('Failed to load templates:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const handleCreate = () => {
    setTemplateName('');
    setTemplateDescription('');
    setTemplateContent('');
    setShowCreateDialog(true);
  };

  const handleEdit = (template: PRDTemplate) => {
    setSelectedTemplate(template);
    setTemplateName(template.name);
    setTemplateDescription(template.description || '');
    setTemplateContent(template.content);
    setShowEditDialog(true);
  };

  const handlePreview = (template: PRDTemplate) => {
    setSelectedTemplate(template);
    setShowPreviewDialog(true);
  };

  const handleDelete = async (template: PRDTemplate) => {
    if (template.is_default) {
      toast.error('Cannot delete default template');
      return;
    }

    if (!confirm(`Are you sure you want to delete "${template.name}"?`)) {
      return;
    }

    try {
      const updatedTemplates = templates.filter((t) => t.id !== template.id);
      localStorage.setItem('prd-templates', JSON.stringify(updatedTemplates));
      setTemplates(updatedTemplates);
      toast.success('Template deleted successfully');
    } catch (error) {
      toast.error('Failed to delete template');
      console.error('Failed to delete template:', error);
    }
  };

  const handleSaveCreate = async () => {
    if (!templateName.trim()) {
      toast.error('Template name is required');
      return;
    }

    if (!templateContent.trim()) {
      toast.error('Template content is required');
      return;
    }

    try {
      const newTemplate: PRDTemplate = {
        id: `template-${Date.now()}`,
        name: templateName.trim(),
        description: templateDescription.trim() || undefined,
        content: templateContent,
        created_at: new Date().toISOString(),
      };

      const updatedTemplates = [...templates, newTemplate];
      localStorage.setItem('prd-templates', JSON.stringify(updatedTemplates));
      setTemplates(updatedTemplates);
      setShowCreateDialog(false);
      toast.success('Template created successfully');
    } catch (error) {
      toast.error('Failed to create template');
      console.error('Failed to create template:', error);
    }
  };

  const handleSaveEdit = async () => {
    if (!selectedTemplate) return;

    if (!templateName.trim()) {
      toast.error('Template name is required');
      return;
    }

    if (!templateContent.trim()) {
      toast.error('Template content is required');
      return;
    }

    try {
      const updatedTemplate: PRDTemplate = {
        ...selectedTemplate,
        name: templateName.trim(),
        description: templateDescription.trim() || undefined,
        content: templateContent,
        updated_at: new Date().toISOString(),
      };

      const updatedTemplates = templates.map((t) =>
        t.id === selectedTemplate.id ? updatedTemplate : t
      );
      localStorage.setItem('prd-templates', JSON.stringify(updatedTemplates));
      setTemplates(updatedTemplates);
      setShowEditDialog(false);
      toast.success('Template updated successfully');
    } catch (error) {
      toast.error('Failed to update template');
      console.error('Failed to update template:', error);
    }
  };

  const handleExportTemplate = (template: PRDTemplate) => {
    const dataStr = JSON.stringify(template, null, 2);
    const dataUri = 'data:application/json;charset=utf-8,' + encodeURIComponent(dataStr);
    const exportFileDefaultName = `${template.name.replace(/\s+/g, '-').toLowerCase()}.json`;

    const linkElement = document.createElement('a');
    linkElement.setAttribute('href', dataUri);
    linkElement.setAttribute('download', exportFileDefaultName);
    linkElement.click();
    toast.success('Template exported');
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">PRD Templates</h1>
          <p className="text-muted-foreground">
            Manage your Product Requirement Document templates
          </p>
        </div>
        <Button onClick={handleCreate}>
          <Plus className="mr-2 h-4 w-4" />
          Create Template
        </Button>
      </div>

      {isLoading ? (
        <div className="text-center py-12 text-muted-foreground">Loading templates...</div>
      ) : templates.length === 0 ? (
        <Card>
          <CardContent className="py-12 text-center">
            <FileText className="mx-auto h-12 w-12 text-muted-foreground mb-4" />
            <h3 className="text-lg font-semibold mb-2">No templates yet</h3>
            <p className="text-muted-foreground mb-4">
              Create your first PRD template to get started
            </p>
            <Button onClick={handleCreate}>
              <Plus className="mr-2 h-4 w-4" />
              Create Template
            </Button>
          </CardContent>
        </Card>
      ) : (
        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
          {templates.map((template) => (
            <Card key={template.id} className="flex flex-col">
              <CardHeader>
                <div className="flex items-start justify-between">
                  <div className="flex-1">
                    <CardTitle className="flex items-center gap-2">
                      {template.name}
                      {template.is_default && (
                        <Badge variant="secondary" className="text-xs">
                          Default
                        </Badge>
                      )}
                    </CardTitle>
                    {template.description && (
                      <CardDescription className="mt-2">
                        {template.description}
                      </CardDescription>
                    )}
                  </div>
                </div>
              </CardHeader>
              <CardContent className="flex-1 flex flex-col justify-end">
                <div className="text-xs text-muted-foreground mb-4">
                  Created: {new Date(template.created_at).toLocaleDateString()}
                  {template.updated_at && (
                    <> • Updated: {new Date(template.updated_at).toLocaleDateString()}</>
                  )}
                </div>
                <div className="flex gap-2">
                  <Button
                    variant="outline"
                    size="sm"
                    className="flex-1"
                    onClick={() => handlePreview(template)}
                  >
                    <Eye className="mr-2 h-4 w-4" />
                    Preview
                  </Button>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => handleEdit(template)}
                  >
                    <Edit className="h-4 w-4" />
                  </Button>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => handleExportTemplate(template)}
                  >
                    <Download className="h-4 w-4" />
                  </Button>
                  {!template.is_default && (
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => handleDelete(template)}
                    >
                      <Trash2 className="h-4 w-4" />
                    </Button>
                  )}
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      )}

      {/* Create Dialog */}
      <Dialog open={showCreateDialog} onOpenChange={setShowCreateDialog}>
        <DialogContent className="max-w-4xl max-h-[90vh] overflow-hidden flex flex-col">
          <DialogHeader>
            <DialogTitle>Create PRD Template</DialogTitle>
            <DialogDescription>
              Create a new template for generating Product Requirement Documents
            </DialogDescription>
          </DialogHeader>
          <div className="flex-1 overflow-auto space-y-4 py-4">
            <div className="space-y-2">
              <Label htmlFor="name">Template Name *</Label>
              <Input
                id="name"
                placeholder="e.g., Standard PRD, Technical Spec, Feature Brief"
                value={templateName}
                onChange={(e) => setTemplateName(e.target.value)}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="description">Description</Label>
              <Input
                id="description"
                placeholder="Brief description of when to use this template"
                value={templateDescription}
                onChange={(e) => setTemplateDescription(e.target.value)}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="content">Template Content (Markdown) *</Label>
              <Textarea
                id="content"
                placeholder="# PRD Template&#10;&#10;## Overview&#10;[Your template content here...]"
                className="font-mono text-sm min-h-[400px]"
                value={templateContent}
                onChange={(e) => setTemplateContent(e.target.value)}
              />
              <Alert>
                <AlertDescription className="text-xs">
                  Use markdown formatting. Variables will be supported in a future update.
                </AlertDescription>
              </Alert>
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowCreateDialog(false)}>
              Cancel
            </Button>
            <Button onClick={handleSaveCreate}>Create Template</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Edit Dialog */}
      <Dialog open={showEditDialog} onOpenChange={setShowEditDialog}>
        <DialogContent className="max-w-4xl max-h-[90vh] overflow-hidden flex flex-col">
          <DialogHeader>
            <DialogTitle>Edit Template</DialogTitle>
            <DialogDescription>
              Update your PRD template
            </DialogDescription>
          </DialogHeader>
          <div className="flex-1 overflow-auto space-y-4 py-4">
            <div className="space-y-2">
              <Label htmlFor="edit-name">Template Name *</Label>
              <Input
                id="edit-name"
                placeholder="Template name"
                value={templateName}
                onChange={(e) => setTemplateName(e.target.value)}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="edit-description">Description</Label>
              <Input
                id="edit-description"
                placeholder="Brief description"
                value={templateDescription}
                onChange={(e) => setTemplateDescription(e.target.value)}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="edit-content">Template Content (Markdown) *</Label>
              <Textarea
                id="edit-content"
                className="font-mono text-sm min-h-[400px]"
                value={templateContent}
                onChange={(e) => setTemplateContent(e.target.value)}
              />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowEditDialog(false)}>
              Cancel
            </Button>
            <Button onClick={handleSaveEdit}>Save Changes</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Preview Dialog */}
      <Dialog open={showPreviewDialog} onOpenChange={setShowPreviewDialog}>
        <DialogContent className="max-w-4xl max-h-[90vh] overflow-hidden flex flex-col">
          <DialogHeader>
            <DialogTitle>{selectedTemplate?.name}</DialogTitle>
            <DialogDescription>
              {selectedTemplate?.description || 'Template preview'}
            </DialogDescription>
          </DialogHeader>
          <div className="flex-1 overflow-auto">
            <pre className="whitespace-pre-wrap font-mono text-sm p-4 bg-muted rounded-md">
              {selectedTemplate?.content}
            </pre>
          </div>
          <DialogFooter>
            <Button onClick={() => setShowPreviewDialog(false)}>Close</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
