import { PRDTemplate } from '@/services/ideate';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Edit, Trash2, Copy, Lock } from 'lucide-react';

interface QuickstartTemplateListProps {
  templates: PRDTemplate[];
  isLoading: boolean;
  onEdit: (template: PRDTemplate) => void;
  onDelete: (template: PRDTemplate) => void;
  onDuplicate: (template: PRDTemplate) => void;
}

export function QuickstartTemplateList({
  templates,
  isLoading,
  onEdit,
  onDelete,
  onDuplicate,
}: QuickstartTemplateListProps) {
  if (isLoading) {
    return (
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
        {[...Array(6)].map((_, i) => (
          <Card key={i} className="animate-pulse">
            <CardHeader className="space-y-2">
              <div className="h-4 bg-muted rounded w-3/4" />
              <div className="h-3 bg-muted rounded w-1/2" />
            </CardHeader>
            <CardContent className="space-y-2">
              <div className="h-3 bg-muted rounded" />
              <div className="h-8 bg-muted rounded" />
            </CardContent>
          </Card>
        ))}
      </div>
    );
  }

  if (templates.length === 0) {
    return (
      <Card>
        <CardContent className="py-12 text-center">
          <p className="text-muted-foreground mb-4">No templates found</p>
        </CardContent>
      </Card>
    );
  }

  return (
    <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
      {templates.map((template) => (
        <Card key={template.id} className="flex flex-col hover:shadow-md transition-shadow">
          <CardHeader>
            <div className="flex items-start justify-between gap-2">
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2">
                  <CardTitle className="text-base truncate">{template.name}</CardTitle>
                  {template.is_system && (
                    <Badge variant="secondary" className="text-xs flex-shrink-0">
                      <Lock className="w-3 h-3 mr-1" />
                      System
                    </Badge>
                  )}
                </div>
                {template.description && (
                  <CardDescription className="mt-2 line-clamp-2">
                    {template.description}
                  </CardDescription>
                )}
              </div>
            </div>
            {template.project_type && (
              <Badge variant="outline" className="w-fit text-xs mt-2">
                {template.project_type}
              </Badge>
            )}
          </CardHeader>

          <CardContent className="flex-1 flex flex-col justify-end">
            <div className="text-xs text-muted-foreground mb-4">
              Created: {new Date(template.created_at).toLocaleDateString()}
            </div>

            <div className="flex gap-2 flex-wrap">
              {!template.is_system && (
                <>
                  <Button
                    variant="outline"
                    size="sm"
                    className="flex-1"
                    onClick={() => onEdit(template)}
                  >
                    <Edit className="w-4 h-4 mr-1" />
                    Edit
                  </Button>
                  <Button
                    variant="outline"
                    size="sm"
                    className="flex-1"
                    onClick={() => onDuplicate(template)}
                  >
                    <Copy className="w-4 h-4 mr-1" />
                    Duplicate
                  </Button>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => onDelete(template)}
                  >
                    <Trash2 className="w-4 h-4" />
                  </Button>
                </>
              )}
              {template.is_system && (
                <>
                  <Button
                    variant="outline"
                    size="sm"
                    className="flex-1"
                    onClick={() => onDuplicate(template)}
                  >
                    <Copy className="w-4 h-4 mr-1" />
                    Duplicate
                  </Button>
                </>
              )}
            </div>
          </CardContent>
        </Card>
      ))}
    </div>
  );
}
