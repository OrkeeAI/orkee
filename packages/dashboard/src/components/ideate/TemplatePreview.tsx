// ABOUTME: Template preview dialog showing detailed template information
// ABOUTME: Displays features, prompts, and dependencies for a selected template

import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Separator } from '@/components/ui/separator';
import {
  FileCode2,
  Smartphone,
  Globe,
  ShoppingBag,
  LayoutDashboard,
  CheckCircle2,
  GitBranch,
  MessageSquare,
  Loader2,
} from 'lucide-react';
import type { PRDTemplate, ProjectType } from '@/services/ideate';

interface TemplatePreviewProps {
  template: PRDTemplate | null;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSelectTemplate?: () => void;
  isLoading?: boolean;
}

const PROJECT_TYPE_ICONS: Record<ProjectType, React.ReactNode> = {
  'saas': <Globe className="h-6 w-6" />,
  'mobile': <Smartphone className="h-6 w-6" />,
  'api': <FileCode2 className="h-6 w-6" />,
  'marketplace': <ShoppingBag className="h-6 w-6" />,
  'internal-tool': <LayoutDashboard className="h-6 w-6" />,
};

export function TemplatePreview({
  template,
  open,
  onOpenChange,
  onSelectTemplate,
  isLoading,
}: TemplatePreviewProps) {
  if (!template) return null;

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-2xl max-h-[85vh]">
        <DialogHeader>
          <div className="flex items-center gap-3">
            <div className="p-2 rounded-lg bg-primary/10 text-primary">
              {template.project_type && PROJECT_TYPE_ICONS[template.project_type]}
            </div>
            <div className="flex-1">
              <DialogTitle className="flex items-center gap-2">
                {template.name}
                {template.project_type && (
                  <Badge variant="secondary" className="text-xs capitalize">
                    {template.project_type}
                  </Badge>
                )}
              </DialogTitle>
              <DialogDescription>{template.description}</DialogDescription>
            </div>
          </div>
        </DialogHeader>

        <ScrollArea className="max-h-[55vh] pr-4">
          <div className="space-y-6">
            {/* AI Prompts Section */}
            {template.one_liner_prompts && template.one_liner_prompts.length > 0 && (
              <div className="space-y-3">
                <div className="flex items-center gap-2">
                  <MessageSquare className="h-4 w-4 text-muted-foreground" />
                  <h3 className="font-semibold text-sm">AI Guidance Prompts</h3>
                </div>
                <div className="space-y-2">
                  {template.one_liner_prompts.map((prompt, index) => (
                    <div
                      key={index}
                      className="text-sm p-3 rounded-lg bg-muted/50 border border-border"
                    >
                      <span className="text-muted-foreground">Q{index + 1}:</span>{' '}
                      <span className="text-foreground">{prompt}</span>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {template.one_liner_prompts &&
              template.one_liner_prompts.length > 0 &&
              template.default_features &&
              template.default_features.length > 0 && <Separator />}

            {/* Default Features Section */}
            {template.default_features && template.default_features.length > 0 && (
              <div className="space-y-3">
                <div className="flex items-center gap-2">
                  <CheckCircle2 className="h-4 w-4 text-muted-foreground" />
                  <h3 className="font-semibold text-sm">
                    Included Features ({template.default_features.length})
                  </h3>
                </div>
                <ul className="grid grid-cols-1 gap-2">
                  {template.default_features.map((feature, index) => (
                    <li
                      key={index}
                      className="flex items-start gap-2 text-sm p-2 rounded-md hover:bg-muted/30 transition-colors"
                    >
                      <CheckCircle2 className="h-4 w-4 text-primary mt-0.5 flex-shrink-0" />
                      <span className="text-foreground">{feature}</span>
                    </li>
                  ))}
                </ul>
              </div>
            )}

            {template.default_features &&
              template.default_features.length > 0 &&
              template.default_dependencies && <Separator />}

            {/* Dependencies Section */}
            {template.default_dependencies && (
              <div className="space-y-3">
                <div className="flex items-center gap-2">
                  <GitBranch className="h-4 w-4 text-muted-foreground" />
                  <h3 className="font-semibold text-sm">Feature Dependencies</h3>
                </div>
                <div className="space-y-2">
                  {Object.entries(template.default_dependencies).map(([key, deps]) => (
                    <div
                      key={key}
                      className="text-sm p-3 rounded-lg bg-muted/30 border border-border"
                    >
                      <div className="font-medium text-foreground mb-1 capitalize">
                        {key.replace(/-/g, ' ')}
                      </div>
                      <div className="text-muted-foreground text-xs">
                        Depends on:{' '}
                        {Array.isArray(deps)
                          ? deps.map((d) => d.replace(/-/g, ' ')).join(', ')
                          : typeof deps === 'string'
                          ? deps.replace(/-/g, ' ')
                          : 'N/A'}
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            )}
          </div>
        </ScrollArea>

        {onSelectTemplate && (
          <DialogFooter>
            <Button variant="outline" onClick={() => onOpenChange(false)} disabled={isLoading}>
              Cancel
            </Button>
            <Button onClick={onSelectTemplate} disabled={isLoading} className="gap-2">
              {isLoading && <Loader2 className="h-4 w-4 animate-spin" />}
              Use This Template
            </Button>
          </DialogFooter>
        )}
      </DialogContent>
    </Dialog>
  );
}
