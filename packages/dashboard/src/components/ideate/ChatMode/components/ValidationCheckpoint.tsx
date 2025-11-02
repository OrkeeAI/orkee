// ABOUTME: Validation checkpoint modal for periodic review
// ABOUTME: Shows summary and allows inline editing of captured info

import React, { useState } from 'react';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { Textarea } from '@/components/ui/textarea';
import { Badge } from '@/components/ui/badge';
import { CheckCircle2, XCircle, Edit2, Save } from 'lucide-react';

export interface CheckpointSection {
  name: string;
  content: string;
  quality_score?: number;
}

export interface ValidationCheckpointProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  title?: string;
  description?: string;
  sections: CheckpointSection[];
  onApprove: () => void;
  onEdit: (sectionName: string, newContent: string) => void;
  onReject: () => void;
}

export function ValidationCheckpoint({
  open,
  onOpenChange,
  title = 'Review Your Progress',
  description = "Let's review what we've discovered so far. Does this look right?",
  sections,
  onApprove,
  onEdit,
  onReject,
}: ValidationCheckpointProps) {
  const [editingSection, setEditingSection] = useState<string | null>(null);
  const [editedContent, setEditedContent] = useState('');

  const handleStartEdit = (section: CheckpointSection) => {
    setEditingSection(section.name);
    setEditedContent(section.content);
  };

  const handleSaveEdit = (sectionName: string) => {
    onEdit(sectionName, editedContent);
    setEditingSection(null);
    setEditedContent('');
  };

  const handleCancelEdit = () => {
    setEditingSection(null);
    setEditedContent('');
  };

  const getQualityBadge = (score?: number) => {
    if (!score) return null;

    if (score >= 80) {
      return <Badge className="bg-green-500">Excellent</Badge>;
    } else if (score >= 60) {
      return <Badge variant="secondary">Good</Badge>;
    } else {
      return <Badge variant="destructive">Needs Work</Badge>;
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-2xl max-h-[80vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>{title}</DialogTitle>
          <DialogDescription>{description}</DialogDescription>
        </DialogHeader>

        <div className="space-y-4 py-4">
          {sections.map((section) => (
            <div key={section.name} className="border rounded-lg p-4">
              <div className="flex items-center justify-between mb-2">
                <div className="flex items-center gap-2">
                  <h3 className="font-semibold capitalize">{section.name}</h3>
                  {getQualityBadge(section.quality_score)}
                </div>
                {editingSection === section.name ? (
                  <div className="flex gap-2">
                    <Button
                      size="sm"
                      variant="ghost"
                      onClick={() => handleSaveEdit(section.name)}
                    >
                      <Save className="h-4 w-4" />
                    </Button>
                    <Button
                      size="sm"
                      variant="ghost"
                      onClick={handleCancelEdit}
                    >
                      <XCircle className="h-4 w-4" />
                    </Button>
                  </div>
                ) : (
                  <Button
                    size="sm"
                    variant="ghost"
                    onClick={() => handleStartEdit(section)}
                  >
                    <Edit2 className="h-4 w-4" />
                  </Button>
                )}
              </div>

              {editingSection === section.name ? (
                <Textarea
                  value={editedContent}
                  onChange={(e) => setEditedContent(e.target.value)}
                  className="min-h-[100px]"
                  placeholder="Edit the content..."
                />
              ) : (
                <div className="text-sm text-muted-foreground whitespace-pre-wrap">
                  {section.content || <em>No content yet</em>}
                </div>
              )}
            </div>
          ))}
        </div>

        <DialogFooter className="flex gap-2 sm:gap-0">
          <Button variant="outline" onClick={onReject}>
            <XCircle className="h-4 w-4 mr-2" />
            Needs Revision
          </Button>
          <Button onClick={onApprove}>
            <CheckCircle2 className="h-4 w-4 mr-2" />
            Looks Good
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
