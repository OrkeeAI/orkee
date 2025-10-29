// ABOUTME: Dialog for selecting template and regenerating PRD with new format
// ABOUTME: Fetches available templates, displays selection, and triggers intelligent regeneration

import { useEffect, useState } from 'react';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { Alert, AlertDescription } from '@/components/ui/alert';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { AlertCircle, Loader2, RefreshCw } from 'lucide-react';
import { ideateService } from '@/services/ideate';
import type { PRDTemplate } from '@/services/ideate';

interface RegenerateTemplateDialogProps {
  sessionId: string;
  onSuccess?: () => void;
  onClose: () => void;
}

export function RegenerateTemplateDialog({
  sessionId,
  onSuccess,
  onClose,
}: RegenerateTemplateDialogProps) {
  const [templates, setTemplates] = useState<PRDTemplate[]>([]);
  const [selectedTemplateId, setSelectedTemplateId] = useState<string>('');
  const [isLoading, setIsLoading] = useState(true);
  const [isRegenerating, setIsRegenerating] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Fetch available output templates on mount
  useEffect(() => {
    const loadTemplates = async () => {
      try {
        setIsLoading(true);
        const data = await ideateService.getTemplates('output');
        setTemplates(data);
        if (data.length > 0) {
          setSelectedTemplateId(data[0].id);
        }
      } catch (err) {
        setError(
          err instanceof Error ? err.message : 'Failed to load templates'
        );
      } finally {
        setIsLoading(false);
      }
    };

    loadTemplates();
  }, []);

  const handleRegenerate = async () => {
    if (!selectedTemplateId) {
      setError('Please select a template');
      return;
    }

    try {
      setIsRegenerating(true);
      setError(null);
      await ideateService.regenerateWithTemplate(sessionId, selectedTemplateId);
      // Give user feedback that regeneration succeeded
      onSuccess?.();
      onClose();
    } catch (err) {
      setError(
        err instanceof Error
          ? err.message
          : 'Failed to regenerate with template'
      );
    } finally {
      setIsRegenerating(false);
    }
  };

  return (
    <Dialog open onOpenChange={onClose}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <RefreshCw className="h-5 w-5" />
            Regenerate with Template
          </DialogTitle>
          <DialogDescription>
            Select a template to reformat your PRD with a different structure
            and style. AI will intelligently reformat your existing content.
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4">
          {/* Template Selection */}
          {isLoading ? (
            <div className="flex items-center justify-center py-8">
              <div className="flex flex-col items-center gap-2 text-muted-foreground">
                <Loader2 className="h-6 w-6 animate-spin" />
                <p className="text-sm">Loading templates...</p>
              </div>
            </div>
          ) : error && templates.length === 0 ? (
            <Alert variant="destructive">
              <AlertCircle className="h-4 w-4" />
              <AlertDescription>{error}</AlertDescription>
            </Alert>
          ) : (
            <>
              <Select value={selectedTemplateId} onValueChange={setSelectedTemplateId}>
                <SelectTrigger>
                  <SelectValue placeholder="Choose a template..." />
                </SelectTrigger>
                <SelectContent>
                  {templates.map((template) => (
                    <SelectItem key={template.id} value={template.id}>
                      <div className="flex flex-col">
                        <span className="font-medium">{template.name}</span>
                        {template.description && (
                          <span className="text-xs text-muted-foreground">
                            {template.description}
                          </span>
                        )}
                      </div>
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>

              {selectedTemplateId && templates.length > 0 && (
                <div className="rounded-lg bg-muted p-3 text-sm">
                  <p className="text-muted-foreground">
                    {
                      templates.find((t) => t.id === selectedTemplateId)
                        ?.description
                    }
                  </p>
                </div>
              )}

              {/* Error Message */}
              {error && (
                <Alert variant="destructive">
                  <AlertCircle className="h-4 w-4" />
                  <AlertDescription>{error}</AlertDescription>
                </Alert>
              )}
            </>
          )}
        </div>

        <DialogFooter className="gap-2 sm:gap-0">
          <Button
            variant="outline"
            onClick={onClose}
            disabled={isRegenerating}
          >
            Cancel
          </Button>
          <Button
            onClick={handleRegenerate}
            disabled={
              isRegenerating ||
              isLoading ||
              !selectedTemplateId ||
              templates.length === 0
            }
            className="gap-2"
          >
            {isRegenerating ? (
              <>
                <Loader2 className="h-4 w-4 animate-spin" />
                Regenerating...
              </>
            ) : (
              <>
                <RefreshCw className="h-4 w-4" />
                Regenerate
              </>
            )}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
