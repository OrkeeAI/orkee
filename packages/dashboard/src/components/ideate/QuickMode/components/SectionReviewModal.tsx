// ABOUTME: Modal for reviewing individual PRD sections before saving in Quick Mode
// ABOUTME: Displays section content, quality score, and allows regeneration or edits

import { useState } from 'react';
import { RefreshCw, Check, X } from 'lucide-react';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { Progress } from '@/components/ui/progress';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Textarea } from '@/components/ui/textarea';
import { Badge } from '@/components/ui/badge';
import type { SectionValidationResult } from '@/services/ideate';

interface SectionReviewModalProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  sectionName: string;
  sectionContent: string;
  validationResult: SectionValidationResult | null;
  onApprove: () => void;
  onRegenerate: () => void;
  onEdit: (content: string) => void;
  isRegenerating?: boolean;
}

export function SectionReviewModal({
  open,
  onOpenChange,
  sectionName,
  sectionContent,
  validationResult,
  onApprove,
  onRegenerate,
  onEdit,
  isRegenerating = false,
}: SectionReviewModalProps) {
  const [isEditing, setIsEditing] = useState(false);
  const [editedContent, setEditedContent] = useState(sectionContent);

  const handleSaveEdit = () => {
    onEdit(editedContent);
    setIsEditing(false);
  };

  const handleCancelEdit = () => {
    setEditedContent(sectionContent);
    setIsEditing(false);
  };

  const getQualityColor = (score: number) => {
    if (score >= 80) return 'text-green-600';
    if (score >= 60) return 'text-yellow-600';
    return 'text-red-600';
  };

  const getQualityLabel = (score: number) => {
    if (score >= 80) return 'Excellent';
    if (score >= 60) return 'Good';
    return 'Needs Improvement';
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-3xl max-h-[80vh] flex flex-col">
        <DialogHeader>
          <DialogTitle className="flex items-center justify-between">
            <span>Review: {sectionName}</span>
            {validationResult && (
              <Badge variant={validationResult.is_valid ? 'default' : 'destructive'}>
                {validationResult.is_valid ? 'Valid' : 'Invalid'}
              </Badge>
            )}
          </DialogTitle>
          <DialogDescription>
            Review the generated content and quality score before proceeding
          </DialogDescription>
        </DialogHeader>

        <div className="flex-1 overflow-y-auto space-y-4">
          {/* Quality Score */}
          {validationResult && (
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <span className="text-sm font-medium">Quality Score</span>
                <span className={`text-sm font-bold ${getQualityColor(validationResult.quality_score)}`}>
                  {validationResult.quality_score}/100 - {getQualityLabel(validationResult.quality_score)}
                </span>
              </div>
              <Progress value={validationResult.quality_score} className="h-2" />
            </div>
          )}

          {/* Issues */}
          {validationResult && validationResult.issues.length > 0 && (
            <Alert variant="destructive">
              <AlertDescription>
                <div className="font-semibold mb-2">Issues Found:</div>
                <ul className="list-disc pl-5 space-y-1">
                  {validationResult.issues.map((issue, index) => (
                    <li key={index} className="text-sm">{issue}</li>
                  ))}
                </ul>
              </AlertDescription>
            </Alert>
          )}

          {/* Suggestions */}
          {validationResult && validationResult.suggestions.length > 0 && (
            <Alert>
              <AlertDescription>
                <div className="font-semibold mb-2">Suggestions:</div>
                <ul className="list-disc pl-5 space-y-1">
                  {validationResult.suggestions.map((suggestion, index) => (
                    <li key={index} className="text-sm">{suggestion}</li>
                  ))}
                </ul>
              </AlertDescription>
            </Alert>
          )}

          {/* Content */}
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <span className="text-sm font-medium">Content</span>
              {!isEditing && (
                <Button variant="outline" size="sm" onClick={() => setIsEditing(true)}>
                  Edit
                </Button>
              )}
            </div>

            {isEditing ? (
              <Textarea
                value={editedContent}
                onChange={(e) => setEditedContent(e.target.value)}
                className="min-h-[200px] font-mono text-sm"
              />
            ) : (
              <div className="p-4 border rounded-md bg-muted/50">
                <pre className="whitespace-pre-wrap text-sm">{sectionContent}</pre>
              </div>
            )}
          </div>
        </div>

        <DialogFooter className="flex items-center justify-between">
          {isEditing ? (
            <div className="flex gap-2 ml-auto">
              <Button variant="outline" onClick={handleCancelEdit}>
                Cancel
              </Button>
              <Button onClick={handleSaveEdit}>
                Save Changes
              </Button>
            </div>
          ) : (
            <>
              <Button
                variant="outline"
                onClick={onRegenerate}
                disabled={isRegenerating}
                className="gap-2"
              >
                <RefreshCw className={`h-4 w-4 ${isRegenerating ? 'animate-spin' : ''}`} />
                Regenerate
              </Button>

              <div className="flex gap-2">
                <Button variant="outline" onClick={() => onOpenChange(false)}>
                  <X className="h-4 w-4 mr-2" />
                  Skip
                </Button>
                <Button onClick={onApprove} variant="default">
                  <Check className="h-4 w-4 mr-2" />
                  Approve
                </Button>
              </div>
            </>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
