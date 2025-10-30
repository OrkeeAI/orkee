// ABOUTME: PRD editor with markdown rendering, per-section editing, and regeneration
// ABOUTME: Collapsible sections with view/edit mode toggle
import { useState } from 'react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import rehypeHighlight from 'rehype-highlight';
import rehypeSanitize from 'rehype-sanitize';
import { Edit2, RefreshCw, Save, ChevronDown, ChevronRight } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Textarea } from '@/components/ui/textarea';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible';
import { ScrollArea } from '@/components/ui/scroll-area';
import { cn } from '@/lib/utils';
import { PRD_SECTIONS } from './SectionSelector';

/**
 * Convert section data to displayable string
 * Handles both string and object section formats from database
 */
function sectionDataToString(data: string | object | undefined): string {
  if (!data) return '';

  // If it's already a string, use it
  if (typeof data === 'string') {
    // If it looks like JSON, try to pretty-print it
    if (data.trim().startsWith('{') || data.trim().startsWith('[')) {
      try {
        const parsed = JSON.parse(data);
        return JSON.stringify(parsed, null, 2);
      } catch {
        return data;
      }
    }
    return data;
  }

  // If it's an object, convert to pretty JSON
  return JSON.stringify(data, null, 2);
}

interface PRDEditorProps {
  prdContent: string;
  sections: Record<string, string>;
  onSectionUpdate: (sectionId: string, content: string) => void;
  onRegenerateSection: (sectionId: string) => void;
  onGeneratePRD: () => void;
  onSave: () => void;
  isRegenerating?: Record<string, boolean>;
}

export function PRDEditor({
  prdContent: _prdContent, // eslint-disable-line @typescript-eslint/no-unused-vars
  sections,
  onSectionUpdate,
  onRegenerateSection,
  onGeneratePRD,
  onSave,
  isRegenerating = {},
}: PRDEditorProps) {
  const [editingSection, setEditingSection] = useState<string | null>(null);
  const [openSections, setOpenSections] = useState<Set<string>>(
    new Set(PRD_SECTIONS.map((s) => s.id))
  );
  const [editedContent, setEditedContent] = useState<Record<string, string>>({});

  const toggleSection = (sectionId: string) => {
    const newOpen = new Set(openSections);
    if (newOpen.has(sectionId)) {
      newOpen.delete(sectionId);
    } else {
      newOpen.add(sectionId);
    }
    setOpenSections(newOpen);
  };

  const startEditing = (sectionId: string) => {
    setEditingSection(sectionId);
    setEditedContent((prev) => ({
      ...prev,
      [sectionId]: sectionDataToString(sections[sectionId]),
    }));
  };

  const cancelEditing = () => {
    setEditingSection(null);
    setEditedContent({});
  };

  const saveEdit = (sectionId: string) => {
    const content = editedContent[sectionId];
    if (content !== undefined) {
      onSectionUpdate(sectionId, content);
    }
    setEditingSection(null);
    setEditedContent((prev) => {
      const newContent = { ...prev };
      delete newContent[sectionId];
      return newContent;
    });
  };

  const expandAll = () => {
    setOpenSections(new Set(PRD_SECTIONS.map((s) => s.id)));
  };

  const collapseAll = () => {
    setOpenSections(new Set());
  };

  const allExpanded = openSections.size === PRD_SECTIONS.length;

  return (
    <div className="space-y-4">
      {/* Header Actions */}
      <div className="flex items-center justify-between">
        <div className="flex gap-2">
          <Button variant="outline" size="sm" onClick={expandAll} disabled={allExpanded}>
            Expand All
          </Button>
          <Button variant="outline" size="sm" onClick={collapseAll} disabled={openSections.size === 0}>
            Collapse All
          </Button>
        </div>
        <div className="flex gap-2">
          <Button variant="outline" onClick={onGeneratePRD} className="gap-2">
            <RefreshCw className="h-4 w-4" />
            Generate PRD
          </Button>
          <Button onClick={onSave} className="gap-2">
            <Save className="h-4 w-4" />
            Save
          </Button>
        </div>
      </div>

      {/* Sections */}
      <ScrollArea className="h-[600px]">
        <div className="space-y-3 pr-4">
          {PRD_SECTIONS.map((section) => {
            const isOpen = openSections.has(section.id);
            const isEditing = editingSection === section.id;
            const content = sectionDataToString(sections[section.id]);
            const isRegeneratingThis = isRegenerating[section.id] || false;

            return (
              <Collapsible
                key={section.id}
                open={isOpen}
                onOpenChange={() => toggleSection(section.id)}
              >
                <Card className={cn(isOpen && 'ring-1 ring-primary/20')}>
                  <CardHeader className="p-4">
                    <div className="flex items-center justify-between">
                      <CollapsibleTrigger asChild>
                        <Button variant="ghost" className="p-0 h-auto hover:bg-transparent">
                          <div className="flex items-center gap-2">
                            {isOpen ? (
                              <ChevronDown className="h-4 w-4" />
                            ) : (
                              <ChevronRight className="h-4 w-4" />
                            )}
                            <CardTitle className="text-base">{section.name}</CardTitle>
                          </div>
                        </Button>
                      </CollapsibleTrigger>

                      {isOpen && !isEditing && (
                        <div className="flex gap-2">
                          <Button
                            variant="outline"
                            size="sm"
                            onClick={() => onRegenerateSection(section.id)}
                            disabled={isRegeneratingThis}
                            className="gap-2"
                          >
                            <RefreshCw className={cn("h-3 w-3", isRegeneratingThis && "animate-spin")} />
                            {isRegeneratingThis ? 'Regenerating...' : 'Regenerate'}
                          </Button>
                          <Button
                            variant="outline"
                            size="sm"
                            onClick={() => startEditing(section.id)}
                            className="gap-2"
                          >
                            <Edit2 className="h-3 w-3" />
                            Edit
                          </Button>
                        </div>
                      )}

                      {isEditing && (
                        <div className="flex gap-2">
                          <Button
                            variant="outline"
                            size="sm"
                            onClick={cancelEditing}
                          >
                            Cancel
                          </Button>
                          <Button
                            variant="default"
                            size="sm"
                            onClick={() => saveEdit(section.id)}
                            className="gap-2"
                          >
                            <Save className="h-3 w-3" />
                            Save
                          </Button>
                        </div>
                      )}
                    </div>
                  </CardHeader>

                  <CollapsibleContent>
                    <CardContent className="p-4 pt-0">
                      {isEditing ? (
                        <Textarea
                          value={editedContent[section.id] || ''}
                          onChange={(e) =>
                            setEditedContent((prev) => ({
                              ...prev,
                              [section.id]: e.target.value,
                            }))
                          }
                          rows={15}
                          className="font-mono text-sm"
                          placeholder={`Enter ${section.name} content in markdown...`}
                        />
                      ) : (
                        <div className="prose prose-sm dark:prose-invert max-w-none">
                          {content ? (
                            <ReactMarkdown
                              remarkPlugins={[remarkGfm]}
                              rehypePlugins={[rehypeHighlight, rehypeSanitize]}
                            >
                              {content}
                            </ReactMarkdown>
                          ) : (
                            <p className="text-muted-foreground italic">
                              This section has not been generated yet. Click "Regenerate" to create content.
                            </p>
                          )}
                        </div>
                      )}
                    </CardContent>
                  </CollapsibleContent>
                </Card>
              </Collapsible>
            );
          })}
        </div>
      </ScrollArea>
    </div>
  );
}
