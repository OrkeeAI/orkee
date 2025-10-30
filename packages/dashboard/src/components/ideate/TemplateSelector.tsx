// ABOUTME: Template selection component for PRD quickstart templates
// ABOUTME: Displays template cards with "Start from Scratch" option

import { Card, CardHeader, CardTitle, CardDescription, CardContent } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import {
  FileCode2,
  Smartphone,
  Globe,
  ShoppingBag,
  LayoutDashboard,
  Sparkles,
  X,
} from 'lucide-react';
import { cn } from '@/lib/utils';
import type { PRDTemplate, ProjectType } from '@/services/ideate';

interface TemplateSelectorProps {
  templates: PRDTemplate[];
  selectedTemplateId: string | null;
  onSelectTemplate: (templateId: string | null) => void;
  onConfirm?: () => void;
  isLoading?: boolean;
}

const PROJECT_TYPE_ICONS: Record<ProjectType, React.ReactNode> = {
  'saas': <Globe className="h-5 w-5" />,
  'mobile': <Smartphone className="h-5 w-5" />,
  'api': <FileCode2 className="h-5 w-5" />,
  'marketplace': <ShoppingBag className="h-5 w-5" />,
  'internal-tool': <LayoutDashboard className="h-5 w-5" />,
};

export function TemplateSelector({
  templates,
  selectedTemplateId,
  onSelectTemplate,
  onConfirm,
  isLoading,
}: TemplateSelectorProps) {
  const systemTemplates = templates.filter(t => t.is_system);

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold tracking-tight">Choose a Template</h2>
        <p className="text-muted-foreground mt-2">
          Select a quickstart template to pre-populate your PRD, or start from scratch.
        </p>
      </div>

      {isLoading ? (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {[...Array(6)].map((_, i) => (
            <Card key={i} className="animate-pulse">
              <CardHeader>
                <div className="h-6 bg-muted rounded w-3/4 mb-2"></div>
                <div className="h-4 bg-muted rounded w-full"></div>
              </CardHeader>
              <CardContent>
                <div className="space-y-2">
                  <div className="h-3 bg-muted rounded"></div>
                  <div className="h-3 bg-muted rounded"></div>
                  <div className="h-3 bg-muted rounded w-2/3"></div>
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {/* Start from Scratch Option */}
          <Card
            className={cn(
              'cursor-pointer transition-all hover:shadow-md relative',
              selectedTemplateId === null
                ? 'ring-2 ring-primary border-primary bg-primary/5'
                : 'hover:border-primary/50'
            )}
            onClick={() => onSelectTemplate(null)}
          >
            <CardHeader>
              <div className="flex items-center gap-3 mb-2">
                <div
                  className={cn(
                    'p-2 rounded-lg',
                    selectedTemplateId === null
                      ? 'bg-primary text-primary-foreground'
                      : 'bg-muted'
                  )}
                >
                  <Sparkles className="h-5 w-5" />
                </div>
                <CardTitle className="text-lg">Start from Scratch</CardTitle>
              </div>
              <CardDescription className="text-sm">
                Build your PRD from a blank slate with full customization
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div className="text-sm text-muted-foreground">
                Perfect for unique projects that don't fit standard patterns
              </div>
            </CardContent>
          </Card>

          {/* System Templates */}
          {systemTemplates.map((template) => (
            <Card
              key={template.id}
              className={cn(
                'cursor-pointer transition-all hover:shadow-md relative',
                selectedTemplateId === template.id
                  ? 'ring-2 ring-primary border-primary bg-primary/5'
                  : 'hover:border-primary/50'
              )}
              onClick={() => onSelectTemplate(template.id)}
            >
              <CardHeader>
                <div className="flex items-center justify-between gap-2 mb-2">
                  <div className="flex items-center gap-3">
                    <div
                      className={cn(
                        'p-2 rounded-lg',
                        selectedTemplateId === template.id
                          ? 'bg-primary text-primary-foreground'
                          : 'bg-muted'
                      )}
                    >
                      {template.project_type && PROJECT_TYPE_ICONS[template.project_type]}
                    </div>
                    <CardTitle className="text-lg">{template.name}</CardTitle>
                  </div>
                  {template.project_type && (
                    <Badge variant="secondary" className="text-xs capitalize">
                      {template.project_type}
                    </Badge>
                  )}
                </div>
                <CardDescription className="text-sm line-clamp-2">
                  {template.description}
                </CardDescription>
              </CardHeader>
              <CardContent>
                {template.default_features && template.default_features.length > 0 && (
                  <div className="space-y-2">
                    <div className="text-xs font-medium text-muted-foreground">
                      Included Features:
                    </div>
                    <ul className="space-y-1 text-sm">
                      {template.default_features.slice(0, 3).map((feature, index) => (
                        <li key={index} className="flex items-start gap-2">
                          <span className="text-primary mt-0.5 text-xs">•</span>
                          <span className="text-muted-foreground line-clamp-1">{feature}</span>
                        </li>
                      ))}
                      {template.default_features.length > 3 && (
                        <li className="text-xs text-muted-foreground italic">
                          +{template.default_features.length - 3} more...
                        </li>
                      )}
                    </ul>
                  </div>
                )}
              </CardContent>
            </Card>
          ))}
        </div>
      )}

      {onConfirm && (
        <div className="flex justify-end gap-3">
          <Button
            variant="outline"
            onClick={() => onSelectTemplate(null)}
            disabled={isLoading}
          >
            <X className="h-4 w-4 mr-2" />
            Clear Selection
          </Button>
          <Button
            onClick={onConfirm}
            size="lg"
            disabled={isLoading}
            className="gap-2"
          >
            Continue
            {selectedTemplateId === null ? ' from Scratch' : ' with Template'}
          </Button>
        </div>
      )}
    </div>
  );
}
