// ABOUTME: Dialog for selecting export format and options for PRD export
// ABOUTME: Supports Markdown, HTML, PDF, and DOCX formats with customization options

import { useState } from 'react';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { Label } from '@/components/ui/label';
import { Input } from '@/components/ui/input';
import { Textarea } from '@/components/ui/textarea';
import { Checkbox } from '@/components/ui/checkbox';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Download, Loader2, FileText, Info } from 'lucide-react';
import { useExportPRD } from '@/hooks/useIdeate';
import type { ExportFormat } from '@/services/ideate';

interface ExportDialogProps {
  sessionId: string;
  onClose: () => void;
}

export function ExportDialog({ sessionId, onClose }: ExportDialogProps) {
  const [format, setFormat] = useState<ExportFormat>('markdown');
  const [includeToc, setIncludeToc] = useState(true);
  const [includeMetadata, setIncludeMetadata] = useState(true);
  const [includePageNumbers, setIncludePageNumbers] = useState(false);
  const [customCss, setCustomCss] = useState('');
  const [title, setTitle] = useState('');

  const exportMutation = useExportPRD(sessionId);

  const handleExport = async () => {
    try {
      const result = await exportMutation.mutateAsync({
        format,
        includeToc,
        includeMetadata,
        includePageNumbers,
        customCss: customCss || undefined,
        title: title || undefined,
      });

      // Trigger download
      const blob = new Blob([result.content], { type: result.mimeType });
      const url = window.URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = result.fileName;
      document.body.appendChild(a);
      a.click();
      window.URL.revokeObjectURL(url);
      document.body.removeChild(a);

      onClose();
    } catch (error) {
      console.error('Export failed:', error);
    }
  };

  const formatDescriptions: Record<ExportFormat, string> = {
    markdown: 'Plain text format with formatting. Great for version control and editing.',
    html: 'Web-ready format with styling. Can be viewed in any browser.',
    pdf: 'Professional document format. Perfect for sharing and presentations.',
    docx: 'Microsoft Word format. Ideal for collaborative editing.',
  };

  const isFormatAvailable = (fmt: ExportFormat): boolean => {
    return fmt === 'markdown' || fmt === 'html';
  };

  return (
    <Dialog open onOpenChange={onClose}>
      <DialogContent className="max-w-2xl">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <FileText className="h-5 w-5" />
            Export PRD
          </DialogTitle>
          <DialogDescription>
            Choose your export format and customize the output options.
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-6 py-4">
          {/* Format Selection */}
          <div className="space-y-2">
            <Label>Export Format</Label>
            <Select value={format} onValueChange={(value) => setFormat(value as ExportFormat)}>
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="markdown">
                  Markdown (.md) {!isFormatAvailable('markdown') && '(Coming Soon)'}
                </SelectItem>
                <SelectItem value="html">
                  HTML (.html) {!isFormatAvailable('html') && '(Coming Soon)'}
                </SelectItem>
                <SelectItem value="pdf">
                  PDF (.pdf) (Coming Soon)
                </SelectItem>
                <SelectItem value="docx">
                  Word (.docx) (Coming Soon)
                </SelectItem>
              </SelectContent>
            </Select>
            <p className="text-sm text-muted-foreground">
              {formatDescriptions[format]}
            </p>
            {!isFormatAvailable(format) && (
              <Alert>
                <Info className="h-4 w-4" />
                <AlertDescription>
                  This format is not yet available. Please use Markdown or HTML for now.
                </AlertDescription>
              </Alert>
            )}
          </div>

          {/* Title Override */}
          <div className="space-y-2">
            <Label htmlFor="title">Custom Title (Optional)</Label>
            <Input
              id="title"
              placeholder="Leave blank to use session title"
              value={title}
              onChange={(e) => setTitle(e.target.value)}
            />
          </div>

          {/* Options */}
          <div className="space-y-3">
            <Label>Export Options</Label>

            <div className="flex items-center space-x-2">
              <Checkbox
                id="toc"
                checked={includeToc}
                onCheckedChange={(checked) => setIncludeToc(checked === true)}
              />
              <label
                htmlFor="toc"
                className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70"
              >
                Include Table of Contents
              </label>
            </div>

            <div className="flex items-center space-x-2">
              <Checkbox
                id="metadata"
                checked={includeMetadata}
                onCheckedChange={(checked) => setIncludeMetadata(checked === true)}
              />
              <label
                htmlFor="metadata"
                className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70"
              >
                Include Metadata (session info, generation date)
              </label>
            </div>

            {(format === 'pdf' || format === 'docx') && (
              <div className="flex items-center space-x-2">
                <Checkbox
                  id="pageNumbers"
                  checked={includePageNumbers}
                  onCheckedChange={(checked) => setIncludePageNumbers(checked === true)}
                />
                <label
                  htmlFor="pageNumbers"
                  className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70"
                >
                  Include Page Numbers
                </label>
              </div>
            )}
          </div>

          {/* Custom CSS (HTML only) */}
          {format === 'html' && (
            <div className="space-y-2">
              <Label htmlFor="customCss">Custom CSS (Optional)</Label>
              <Textarea
                id="customCss"
                placeholder="Add your own CSS styles..."
                value={customCss}
                onChange={(e) => setCustomCss(e.target.value)}
                rows={4}
                className="font-mono text-xs"
              />
              <p className="text-xs text-muted-foreground">
                Override default styles with your own CSS. Leave blank for default styling.
              </p>
            </div>
          )}
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={onClose}>
            Cancel
          </Button>
          <Button
            onClick={handleExport}
            disabled={exportMutation.isPending || !isFormatAvailable(format)}
            className="gap-2"
          >
            {exportMutation.isPending ? (
              <>
                <Loader2 className="h-4 w-4 animate-spin" />
                Exporting...
              </>
            ) : (
              <>
                <Download className="h-4 w-4" />
                Export
              </>
            )}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
