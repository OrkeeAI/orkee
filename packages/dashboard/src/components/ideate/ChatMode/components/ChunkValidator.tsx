// ABOUTME: Chunk validator for 200-300 word PRD sections
// ABOUTME: Displays chunks with word count, validation controls, and inline editing

import React, { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Textarea } from '@/components/ui/textarea';
import { Card, CardContent, CardFooter, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Check, X, Edit3, RotateCcw } from 'lucide-react';
import { cn } from '@/lib/utils';

export interface PrdChunk {
  id: string;
  session_id: string;
  section_name: string;
  chunk_number: number;
  content: string;
  word_count: number;
  status: 'pending' | 'approved' | 'rejected' | 'edited';
  edited_content?: string;
  user_feedback?: string;
}

export interface ChunkValidatorProps {
  chunk: PrdChunk;
  onApprove: () => void;
  onReject: (feedback?: string) => void;
  onEdit: (newContent: string) => void;
  onRegenerate: () => void;
  disabled?: boolean;
  className?: string;
}

export function ChunkValidator({
  chunk,
  onApprove,
  onReject,
  onEdit,
  onRegenerate,
  disabled = false,
  className,
}: ChunkValidatorProps) {
  const [isEditing, setIsEditing] = useState(false);
  const [editedContent, setEditedContent] = useState(chunk.edited_content || chunk.content);
  const [rejectFeedback, setRejectFeedback] = useState('');
  const [showRejectInput, setShowRejectInput] = useState(false);

  const currentContent = chunk.edited_content || chunk.content;
  const currentWordCount = currentContent.split(/\s+/).filter((w) => w.length > 0).length;

  const handleSaveEdit = () => {
    onEdit(editedContent);
    setIsEditing(false);
  };

  const handleCancelEdit = () => {
    setEditedContent(currentContent);
    setIsEditing(false);
  };

  const handleReject = () => {
    if (showRejectInput) {
      onReject(rejectFeedback);
      setShowRejectInput(false);
      setRejectFeedback('');
    } else {
      setShowRejectInput(true);
    }
  };

  const getStatusColor = () => {
    switch (chunk.status) {
      case 'approved':
        return 'bg-green-500/10 text-green-700 border-green-200';
      case 'rejected':
        return 'bg-red-500/10 text-red-700 border-red-200';
      case 'edited':
        return 'bg-blue-500/10 text-blue-700 border-blue-200';
      default:
        return 'bg-yellow-500/10 text-yellow-700 border-yellow-200';
    }
  };

  const getWordCountColor = () => {
    if (currentWordCount < 200) return 'text-orange-600';
    if (currentWordCount > 300) return 'text-orange-600';
    return 'text-green-600';
  };

  return (
    <Card className={cn('relative', className)}>
      <CardHeader className="pb-3">
        <div className="flex items-start justify-between gap-4">
          <div className="space-y-1">
            <CardTitle className="text-lg">
              {chunk.section_name} - Chunk {chunk.chunk_number}
            </CardTitle>
            <div className="flex items-center gap-2">
              <Badge variant="outline" className={cn('text-xs', getWordCountColor())}>
                {currentWordCount} words
              </Badge>
              {currentWordCount < 200 && (
                <span className="text-xs text-muted-foreground">(below 200)</span>
              )}
              {currentWordCount > 300 && (
                <span className="text-xs text-muted-foreground">(over 300)</span>
              )}
            </div>
          </div>
          <Badge className={cn('text-xs', getStatusColor())}>{chunk.status}</Badge>
        </div>
      </CardHeader>

      <CardContent className="space-y-4">
        {/* Content Display/Edit */}
        {isEditing ? (
          <div className="space-y-2">
            <Textarea
              value={editedContent}
              onChange={(e) => setEditedContent(e.target.value)}
              className="min-h-[200px] font-mono text-sm"
              placeholder="Edit the content..."
            />
            <p className="text-xs text-muted-foreground">
              Word count: {editedContent.split(/\s+/).filter((w) => w.length > 0).length}
            </p>
          </div>
        ) : (
          <div className="prose prose-sm max-w-none dark:prose-invert">
            <p className="whitespace-pre-wrap text-sm leading-relaxed">{currentContent}</p>
          </div>
        )}

        {/* Reject Feedback Input */}
        {showRejectInput && !isEditing && (
          <div className="space-y-2 pt-2 border-t">
            <label className="text-sm font-medium text-muted-foreground">
              Why does this need to be regenerated?
            </label>
            <Textarea
              value={rejectFeedback}
              onChange={(e) => setRejectFeedback(e.target.value)}
              placeholder="Optional: Provide feedback for regeneration..."
              className="text-sm"
              rows={3}
            />
          </div>
        )}

        {/* Validation Question */}
        {!isEditing && chunk.status === 'pending' && (
          <div className="text-center pt-2">
            <p className="text-sm font-medium text-muted-foreground">Does this look right?</p>
          </div>
        )}
      </CardContent>

      <CardFooter className="flex flex-wrap gap-2">
        {isEditing ? (
          <>
            <Button onClick={handleSaveEdit} size="sm" className="flex-1">
              <Check className="h-4 w-4 mr-2" />
              Save Changes
            </Button>
            <Button onClick={handleCancelEdit} size="sm" variant="outline">
              Cancel
            </Button>
          </>
        ) : (
          <>
            {chunk.status === 'pending' && (
              <>
                <Button
                  onClick={onApprove}
                  disabled={disabled}
                  size="sm"
                  variant="default"
                  className="flex-1"
                >
                  <Check className="h-4 w-4 mr-2" />
                  Approve
                </Button>
                <Button
                  onClick={() => setIsEditing(true)}
                  disabled={disabled}
                  size="sm"
                  variant="outline"
                >
                  <Edit3 className="h-4 w-4 mr-2" />
                  Edit
                </Button>
                <Button
                  onClick={handleReject}
                  disabled={disabled}
                  size="sm"
                  variant="outline"
                  className="text-destructive hover:text-destructive"
                >
                  {showRejectInput ? (
                    <>
                      <X className="h-4 w-4 mr-2" />
                      Confirm Reject
                    </>
                  ) : (
                    <>
                      <RotateCcw className="h-4 w-4 mr-2" />
                      Regenerate
                    </>
                  )}
                </Button>
              </>
            )}

            {chunk.status !== 'pending' && (
              <>
                <Button
                  onClick={() => setIsEditing(true)}
                  disabled={disabled}
                  size="sm"
                  variant="outline"
                  className="flex-1"
                >
                  <Edit3 className="h-4 w-4 mr-2" />
                  Edit Again
                </Button>
                {chunk.status === 'rejected' && (
                  <Button
                    onClick={onRegenerate}
                    disabled={disabled}
                    size="sm"
                    variant="default"
                  >
                    <RotateCcw className="h-4 w-4 mr-2" />
                    Regenerate
                  </Button>
                )}
              </>
            )}
          </>
        )}
      </CardFooter>

      {showRejectInput && !isEditing && (
        <div className="px-6 pb-4">
          <Button
            onClick={() => {
              setShowRejectInput(false);
              setRejectFeedback('');
            }}
            size="sm"
            variant="ghost"
            className="w-full"
          >
            Cancel
          </Button>
        </div>
      )}
    </Card>
  );
}
