import { useState, useEffect } from 'react';
import { PRDTemplate, CreateTemplateInput } from '@/services/ideate';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Loader2, Plus, X } from 'lucide-react';

interface QuickstartTemplateEditorProps {
  template?: PRDTemplate;
  isLoading?: boolean;
  onSave: (data: CreateTemplateInput) => Promise<void>;
  onCancel: () => void;
}

export function QuickstartTemplateEditor({
  template,
  isLoading = false,
  onSave,
  onCancel,
}: QuickstartTemplateEditorProps) {
  const [formData, setFormData] = useState<CreateTemplateInput>({
    name: '',
    description: '',
    project_type: '',
    default_problem_statement: '',
    default_target_audience: '',
    default_value_proposition: '',
    default_ui_considerations: '',
    default_ux_principles: '',
    default_tech_stack_quick: '',
    default_mvp_scope: [],
    default_research_findings: '',
    default_technical_specs: '',
    default_competitors: [],
    default_similar_projects: [],
  });

  const [mvpInput, setMvpInput] = useState('');
  const [competitorInput, setCompetitorInput] = useState('');
  const [projectInput, setProjectInput] = useState('');
  const [isSaving, setIsSaving] = useState(false);

  useEffect(() => {
    if (template) {
      setFormData({
        name: template.name,
        description: template.description || '',
        project_type: template.project_type || '',
        default_problem_statement: template.default_problem_statement || '',
        default_target_audience: template.default_target_audience || '',
        default_value_proposition: template.default_value_proposition || '',
        default_ui_considerations: template.default_ui_considerations || '',
        default_ux_principles: template.default_ux_principles || '',
        default_tech_stack_quick: template.default_tech_stack_quick || '',
        default_mvp_scope: template.default_mvp_scope || [],
        default_research_findings: template.default_research_findings || '',
        default_technical_specs: template.default_technical_specs || '',
        default_competitors: template.default_competitors || [],
        default_similar_projects: template.default_similar_projects || [],
      });
    }
  }, [template]);

  const handleSave = async () => {
    setIsSaving(true);
    try {
      await onSave(formData);
    } finally {
      setIsSaving(false);
    }
  };

  const addMvpItem = () => {
    if (mvpInput.trim()) {
      setFormData({
        ...formData,
        default_mvp_scope: [...(formData.default_mvp_scope || []), mvpInput.trim()],
      });
      setMvpInput('');
    }
  };

  const removeMvpItem = (index: number) => {
    setFormData({
      ...formData,
      default_mvp_scope: (formData.default_mvp_scope || []).filter((_, i) => i !== index),
    });
  };

  const addCompetitor = () => {
    if (competitorInput.trim()) {
      setFormData({
        ...formData,
        default_competitors: [...(formData.default_competitors || []), competitorInput.trim()],
      });
      setCompetitorInput('');
    }
  };

  const removeCompetitor = (index: number) => {
    setFormData({
      ...formData,
      default_competitors: (formData.default_competitors || []).filter((_, i) => i !== index),
    });
  };

  const addProject = () => {
    if (projectInput.trim()) {
      setFormData({
        ...formData,
        default_similar_projects: [...(formData.default_similar_projects || []), projectInput.trim()],
      });
      setProjectInput('');
    }
  };

  const removeProject = (index: number) => {
    setFormData({
      ...formData,
      default_similar_projects: (formData.default_similar_projects || []).filter((_, i) => i !== index),
    });
  };

  return (
    <div className="space-y-6">
      {/* Basic Info */}
      <Card>
        <CardHeader>
          <CardTitle>Template Information</CardTitle>
          <CardDescription>Basic details about your template</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="name">Template Name *</Label>
            <Input
              id="name"
              placeholder="e.g., SaaS Application"
              value={formData.name}
              onChange={(e) => setFormData({ ...formData, name: e.target.value })}
            />
          </div>

          <div className="space-y-2">
            <Label htmlFor="description">Description</Label>
            <Textarea
              id="description"
              placeholder="Brief description of this template"
              rows={3}
              value={formData.description}
              onChange={(e) => setFormData({ ...formData, description: e.target.value })}
            />
          </div>

          <div className="space-y-2">
            <Label htmlFor="project_type">Project Type</Label>
            <Input
              id="project_type"
              placeholder="e.g., saas, mobile, api, marketplace, internal-tool"
              value={formData.project_type}
              onChange={(e) => setFormData({ ...formData, project_type: e.target.value })}
            />
          </div>
        </CardContent>
      </Card>

      {/* Section Defaults */}
      <Tabs defaultValue="overview" className="w-full">
        <TabsList className="grid w-full grid-cols-5">
          <TabsTrigger value="overview">Overview</TabsTrigger>
          <TabsTrigger value="ux">UX</TabsTrigger>
          <TabsTrigger value="technical">Technical</TabsTrigger>
          <TabsTrigger value="roadmap">Roadmap</TabsTrigger>
          <TabsTrigger value="research">Research</TabsTrigger>
        </TabsList>

        {/* Overview Tab */}
        <TabsContent value="overview" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle className="text-base">Overview Defaults</CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="space-y-2">
                <Label>Problem Statement</Label>
                <Textarea
                  placeholder="Default problem statement for this template type"
                  rows={3}
                  value={formData.default_problem_statement || ''}
                  onChange={(e) =>
                    setFormData({ ...formData, default_problem_statement: e.target.value })
                  }
                />
              </div>

              <div className="space-y-2">
                <Label>Target Audience</Label>
                <Textarea
                  placeholder="Default target audience description"
                  rows={3}
                  value={formData.default_target_audience || ''}
                  onChange={(e) =>
                    setFormData({ ...formData, default_target_audience: e.target.value })
                  }
                />
              </div>

              <div className="space-y-2">
                <Label>Value Proposition</Label>
                <Textarea
                  placeholder="Default value proposition"
                  rows={3}
                  value={formData.default_value_proposition || ''}
                  onChange={(e) =>
                    setFormData({ ...formData, default_value_proposition: e.target.value })
                  }
                />
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        {/* UX Tab */}
        <TabsContent value="ux" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle className="text-base">UX Defaults</CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="space-y-2">
                <Label>UI Considerations</Label>
                <Textarea
                  placeholder="Default UI considerations for this template type"
                  rows={4}
                  value={formData.default_ui_considerations || ''}
                  onChange={(e) =>
                    setFormData({ ...formData, default_ui_considerations: e.target.value })
                  }
                />
              </div>

              <div className="space-y-2">
                <Label>UX Principles</Label>
                <Textarea
                  placeholder="Default UX principles"
                  rows={4}
                  value={formData.default_ux_principles || ''}
                  onChange={(e) =>
                    setFormData({ ...formData, default_ux_principles: e.target.value })
                  }
                />
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        {/* Technical Tab */}
        <TabsContent value="technical" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle className="text-base">Technical Defaults</CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="space-y-2">
                <Label>Tech Stack Quick</Label>
                <Textarea
                  placeholder="Default technology stack overview"
                  rows={6}
                  value={formData.default_tech_stack_quick || ''}
                  onChange={(e) =>
                    setFormData({ ...formData, default_tech_stack_quick: e.target.value })
                  }
                />
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        {/* Roadmap Tab */}
        <TabsContent value="roadmap" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle className="text-base">Roadmap Defaults</CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="space-y-2">
                <Label>MVP Scope Items</Label>
                <div className="flex gap-2">
                  <Input
                    placeholder="Add MVP scope item"
                    value={mvpInput}
                    onChange={(e) => setMvpInput(e.target.value)}
                    onKeyPress={(e) => e.key === 'Enter' && addMvpItem()}
                  />
                  <Button onClick={addMvpItem} size="sm">
                    <Plus className="w-4 h-4" />
                  </Button>
                </div>
                <div className="space-y-2">
                  {(formData.default_mvp_scope || []).map((item, idx) => (
                    <div key={idx} className="flex items-center justify-between bg-muted p-2 rounded">
                      <span className="text-sm">{item}</span>
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => removeMvpItem(idx)}
                      >
                        <X className="w-4 h-4" />
                      </Button>
                    </div>
                  ))}
                </div>
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        {/* Research Tab */}
        <TabsContent value="research" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle className="text-base">Research Defaults</CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="space-y-2">
                <Label>Research Findings</Label>
                <Textarea
                  placeholder="Default research findings"
                  rows={3}
                  value={formData.default_research_findings || ''}
                  onChange={(e) =>
                    setFormData({ ...formData, default_research_findings: e.target.value })
                  }
                />
              </div>

              <div className="space-y-2">
                <Label>Technical Specs</Label>
                <Textarea
                  placeholder="Default technical specifications"
                  rows={3}
                  value={formData.default_technical_specs || ''}
                  onChange={(e) =>
                    setFormData({ ...formData, default_technical_specs: e.target.value })
                  }
                />
              </div>

              <div className="space-y-2">
                <Label>Competitors</Label>
                <div className="flex gap-2">
                  <Input
                    placeholder="Add competitor"
                    value={competitorInput}
                    onChange={(e) => setCompetitorInput(e.target.value)}
                    onKeyPress={(e) => e.key === 'Enter' && addCompetitor()}
                  />
                  <Button onClick={addCompetitor} size="sm">
                    <Plus className="w-4 h-4" />
                  </Button>
                </div>
                <div className="space-y-2">
                  {(formData.default_competitors || []).map((item, idx) => (
                    <div key={idx} className="flex items-center justify-between bg-muted p-2 rounded">
                      <span className="text-sm">{item}</span>
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => removeCompetitor(idx)}
                      >
                        <X className="w-4 h-4" />
                      </Button>
                    </div>
                  ))}
                </div>
              </div>

              <div className="space-y-2">
                <Label>Similar Projects</Label>
                <div className="flex gap-2">
                  <Input
                    placeholder="Add similar project"
                    value={projectInput}
                    onChange={(e) => setProjectInput(e.target.value)}
                    onKeyPress={(e) => e.key === 'Enter' && addProject()}
                  />
                  <Button onClick={addProject} size="sm">
                    <Plus className="w-4 h-4" />
                  </Button>
                </div>
                <div className="space-y-2">
                  {(formData.default_similar_projects || []).map((item, idx) => (
                    <div key={idx} className="flex items-center justify-between bg-muted p-2 rounded">
                      <span className="text-sm">{item}</span>
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => removeProject(idx)}
                      >
                        <X className="w-4 h-4" />
                      </Button>
                    </div>
                  ))}
                </div>
              </div>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>

      {/* Action Buttons */}
      <div className="flex gap-2 justify-end">
        <Button variant="outline" onClick={onCancel} disabled={isSaving || isLoading}>
          Cancel
        </Button>
        <Button onClick={handleSave} disabled={isSaving || isLoading || !formData.name.trim()}>
          {isSaving || isLoading ? (
            <>
              <Loader2 className="w-4 h-4 mr-2 animate-spin" />
              Saving...
            </>
          ) : (
            'Save Template'
          )}
        </Button>
      </div>
    </div>
  );
}
