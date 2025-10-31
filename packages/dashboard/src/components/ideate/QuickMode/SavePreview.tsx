// ABOUTME: Final PRD preview and confirmation dialog before saving
// ABOUTME: Displays read-only markdown view with project name editing
import { useState } from 'react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import rehypeHighlight from 'rehype-highlight';
import rehypeSanitize from 'rehype-sanitize';
import { FileText } from 'lucide-react';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { ScrollArea } from '@/components/ui/scroll-area';

interface SavePreviewProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  prdContent: string;
  projectName?: string;
  onConfirmSave: (name: string) => void;
  isSaving?: boolean;
}

export function SavePreview({
  open,
  onOpenChange,
  prdContent,
  projectName = '',
  onConfirmSave,
  isSaving = false,
}: SavePreviewProps) {
  const [name, setName] = useState(projectName);

  const handleConfirm = () => {
    if (name.trim()) {
      onConfirmSave(name.trim());
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-4xl max-h-[90vh]">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <FileText className="h-5 w-5" />
            Save PRD
          </DialogTitle>
          <DialogDescription>
            Review your PRD before saving it. You can edit the PRD name below.
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="prd-name">PRD Name *</Label>
            <Input
              id="prd-name"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="My Project PRD"
              disabled={isSaving}
              required
            />
          </div>

          <div className="space-y-2">
            <Label>PRD Preview</Label>
            <ScrollArea className="h-[400px] w-full border rounded-md p-4">
              <div className="prose prose-sm dark:prose-invert max-w-none">
                <ReactMarkdown
                  remarkPlugins={[remarkGfm]}
                  rehypePlugins={[rehypeHighlight, rehypeSanitize]}
                >
                  {prdContent}
                </ReactMarkdown>
              </div>
            </ScrollArea>
          </div>
        </div>

        <DialogFooter>
          <Button
            variant="outline"
            onClick={() => onOpenChange(false)}
            disabled={isSaving}
          >
            Cancel
          </Button>
          <Button
            onClick={handleConfirm}
            disabled={!name.trim() || isSaving}
          >
            {isSaving ? 'Saving...' : 'Confirm & Save PRD'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
