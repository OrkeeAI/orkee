import { useState } from 'react';
import { PRDTemplate } from '@/services/ideate';
import { useQuickstartTemplates, useCreateTemplate, useUpdateTemplate, useDeleteTemplate } from '@/hooks/useQuickstartTemplates';
import { Button } from '@/components/ui/button';
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { Plus } from 'lucide-react';
import { toast } from 'sonner';
import { QuickstartTemplateList } from './QuickstartTemplateList';
import { QuickstartTemplateEditor } from './QuickstartTemplateEditor';

export function QuickstartTemplateManager() {
  const { data: templates = [], isLoading } = useQuickstartTemplates();
  const createMutation = useCreateTemplate();
  const updateMutation = useUpdateTemplate('');
  const deleteMutation = useDeleteTemplate();

  const [showCreateDialog, setShowCreateDialog] = useState(false);
  const [showEditDialog, setShowEditDialog] = useState(false);
  const [selectedTemplate, setSelectedTemplate] = useState<PRDTemplate | null>(null);

  const handleCreate = async (data: any) => {
    try {
      await createMutation.mutateAsync(data);
      setShowCreateDialog(false);
      toast.success('Template created successfully');
    } catch (error) {
      toast.error('Failed to create template');
      console.error(error);
    }
  };

  const handleEdit = (template: PRDTemplate) => {
    setSelectedTemplate(template);
    setShowEditDialog(true);
  };

  const handleUpdate = async (data: any) => {
    if (!selectedTemplate) return;
    try {
      await updateMutation.mutateAsync(data);
      setShowEditDialog(false);
      setSelectedTemplate(null);
      toast.success('Template updated successfully');
    } catch (error) {
      toast.error('Failed to update template');
      console.error(error);
    }
  };

  const handleDelete = async (template: PRDTemplate) => {
    if (!confirm(`Are you sure you want to delete "${template.name}"?`)) {
      return;
    }

    try {
      await deleteMutation.mutateAsync(template.id);
      toast.success('Template deleted successfully');
    } catch (error) {
      toast.error('Failed to delete template');
      console.error(error);
    }
  };

  const handleDuplicate = (template: PRDTemplate) => {
    const duplicateData = {
      name: `${template.name} (Copy)`,
      description: template.description,
      project_type: template.project_type,
      default_problem_statement: template.default_problem_statement,
      default_target_audience: template.default_target_audience,
      default_value_proposition: template.default_value_proposition,
      default_ui_considerations: template.default_ui_considerations,
      default_ux_principles: template.default_ux_principles,
      default_tech_stack_quick: template.default_tech_stack_quick,
      default_mvp_scope: template.default_mvp_scope,
      default_research_findings: template.default_research_findings,
      default_technical_specs: template.default_technical_specs,
      default_competitors: template.default_competitors,
      default_similar_projects: template.default_similar_projects,
    };

    handleCreate(duplicateData);
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold tracking-tight">Quickstart Templates</h2>
          <p className="text-muted-foreground">
            Manage PRD quickstart templates for guided mode ideation
          </p>
        </div>
        <Button onClick={() => setShowCreateDialog(true)}>
          <Plus className="mr-2 h-4 w-4" />
          Create Template
        </Button>
      </div>

      <QuickstartTemplateList
        templates={templates}
        isLoading={isLoading}
        onEdit={handleEdit}
        onDelete={handleDelete}
        onDuplicate={handleDuplicate}
      />

      {/* Create Dialog */}
      <Dialog open={showCreateDialog} onOpenChange={setShowCreateDialog}>
        <DialogContent className="max-w-4xl max-h-[90vh] overflow-hidden flex flex-col">
          <DialogHeader>
            <DialogTitle>Create Quickstart Template</DialogTitle>
            <DialogDescription>
              Create a new template with default values for all Guided Mode sections
            </DialogDescription>
          </DialogHeader>
          <div className="flex-1 overflow-auto">
            <QuickstartTemplateEditor
              onSave={handleCreate}
              onCancel={() => setShowCreateDialog(false)}
              isLoading={createMutation.isPending}
            />
          </div>
        </DialogContent>
      </Dialog>

      {/* Edit Dialog */}
      <Dialog open={showEditDialog} onOpenChange={setShowEditDialog}>
        <DialogContent className="max-w-4xl max-h-[90vh] overflow-hidden flex flex-col">
          <DialogHeader>
            <DialogTitle>Edit Template</DialogTitle>
            <DialogDescription>
              Update template defaults
            </DialogDescription>
          </DialogHeader>
          <div className="flex-1 overflow-auto">
            <QuickstartTemplateEditor
              template={selectedTemplate || undefined}
              onSave={handleUpdate}
              onCancel={() => {
                setShowEditDialog(false);
                setSelectedTemplate(null);
              }}
              isLoading={updateMutation.isPending}
            />
          </div>
        </DialogContent>
      </Dialog>
    </div>
  );
}
