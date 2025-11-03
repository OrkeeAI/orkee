// ABOUTME: AI SDK-inspired PromptInput component for chat interfaces
// ABOUTME: Provides auto-resizing textarea, file attachments, and status-aware submit button

import React, { useRef, useState, useCallback, useEffect } from 'react';
import { Button } from '@/components/ui/button';
import { Textarea } from '@/components/ui/textarea';
import { Send, Loader2, Paperclip, X } from 'lucide-react';
import { cn } from '@/lib/utils';

export interface PromptInputFile {
  name: string;
  size: number;
  type: string;
  file: File;
}

export interface PromptInputProps {
  onSubmit: (message: string, files?: PromptInputFile[]) => void;
  placeholder?: string;
  disabled?: boolean;
  status?: 'ready' | 'submitted' | 'streaming' | 'error';
  accept?: string;
  multiple?: boolean;
  maxFiles?: number;
  maxFileSize?: number; // in bytes
  className?: string;
}

export function PromptInput({
  onSubmit,
  placeholder = 'Type your message... (Shift+Enter for new line)',
  disabled = false,
  status = 'ready',
  accept,
  multiple = false,
  maxFiles = 5,
  maxFileSize = 10 * 1024 * 1024, // 10MB default
  className,
}: PromptInputProps) {
  const [input, setInput] = useState('');
  const [attachments, setAttachments] = useState<PromptInputFile[]>([]);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);

  // Auto-resize textarea
  const adjustTextareaHeight = useCallback(() => {
    const textarea = textareaRef.current;
    if (!textarea) return;

    textarea.style.height = 'auto';
    const scrollHeight = textarea.scrollHeight;
    const maxHeight = 120; // max-h-[120px]
    textarea.style.height = `${Math.min(scrollHeight, maxHeight)}px`;
  }, []);

  useEffect(() => {
    adjustTextareaHeight();
  }, [input, adjustTextareaHeight]);

  const handleSubmit = useCallback(
    (e?: React.FormEvent) => {
      e?.preventDefault();
      const trimmedInput = input.trim();

      if (trimmedInput || attachments.length > 0) {
        onSubmit(trimmedInput, attachments.length > 0 ? attachments : undefined);
        setInput('');
        setAttachments([]);

        // Reset textarea height
        if (textareaRef.current) {
          textareaRef.current.style.height = 'auto';
        }
      }
    },
    [input, attachments, onSubmit]
  );

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
      if (e.key === 'Enter' && !e.shiftKey) {
        e.preventDefault();
        handleSubmit();
      } else if (e.key === 'Backspace' && input === '' && attachments.length > 0) {
        // Remove last attachment when backspace is pressed with empty input
        setAttachments((prev) => prev.slice(0, -1));
      }
    },
    [input, attachments.length, handleSubmit]
  );

  const handleFileSelect = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const files = Array.from(e.target.files || []);

      // Validate file count
      if (attachments.length + files.length > maxFiles) {
        console.warn(`Maximum ${maxFiles} files allowed`);
        return;
      }

      // Validate and convert files
      const validFiles: PromptInputFile[] = files
        .filter((file) => {
          if (file.size > maxFileSize) {
            console.warn(`File ${file.name} exceeds maximum size of ${maxFileSize} bytes`);
            return false;
          }
          return true;
        })
        .map((file) => ({
          name: file.name,
          size: file.size,
          type: file.type,
          file,
        }));

      setAttachments((prev) => [...prev, ...validFiles]);

      // Reset file input
      if (fileInputRef.current) {
        fileInputRef.current.value = '';
      }
    },
    [attachments.length, maxFiles, maxFileSize]
  );

  const removeAttachment = useCallback((index: number) => {
    setAttachments((prev) => prev.filter((_, i) => i !== index));
  }, []);

  const isDisabled = disabled || status === 'streaming' || status === 'submitted';
  const canSubmit = (input.trim() || attachments.length > 0) && !isDisabled;

  const getSubmitIcon = () => {
    if (status === 'streaming' || status === 'submitted') {
      return <Loader2 className="h-5 w-5 animate-spin" />;
    }
    return <Send className="h-5 w-5" />;
  };

  return (
    <div className={cn('border-t bg-background', className)}>
      <form onSubmit={handleSubmit} className="flex flex-col">
        {/* Attachments Header */}
        {attachments.length > 0 && (
          <div className="px-4 pt-3 pb-2 border-b">
            <div className="flex flex-wrap gap-2">
              {attachments.map((attachment, index) => (
                <div
                  key={index}
                  className="flex items-center gap-2 px-3 py-1.5 bg-muted rounded-full text-sm group"
                >
                  <Paperclip className="h-3.5 w-3.5 text-muted-foreground" />
                  <span className="max-w-[200px] truncate">{attachment.name}</span>
                  <button
                    type="button"
                    onClick={() => removeAttachment(index)}
                    className="opacity-0 group-hover:opacity-100 transition-opacity hover:text-destructive"
                    disabled={isDisabled}
                  >
                    <X className="h-3.5 w-3.5" />
                  </button>
                </div>
              ))}
            </div>
          </div>
        )}

        {/* Input Body */}
        <div className="flex items-end gap-2 p-4">
          <div className="flex-1 relative">
            <Textarea
              ref={textareaRef}
              value={input}
              onChange={(e) => setInput(e.target.value)}
              onKeyDown={handleKeyDown}
              placeholder={placeholder}
              className="resize-none min-h-[60px] max-h-[120px] pr-10"
              disabled={isDisabled}
              rows={1}
            />

            {/* Attachment Button (inside textarea, bottom-right) */}
            {accept && (
              <Button
                type="button"
                variant="ghost"
                size="icon"
                className="absolute right-2 bottom-2 h-7 w-7"
                onClick={() => fileInputRef.current?.click()}
                disabled={isDisabled || attachments.length >= maxFiles}
              >
                <Paperclip className="h-4 w-4" />
              </Button>
            )}
          </div>

          {/* Submit Button - Smaller, status-aware */}
          <Button
            type="submit"
            size="icon"
            disabled={!canSubmit}
            className="h-10 w-10 flex-shrink-0 rounded-full"
          >
            {getSubmitIcon()}
          </Button>
        </div>
      </form>

      {/* Hidden File Input */}
      {accept && (
        <input
          ref={fileInputRef}
          type="file"
          accept={accept}
          multiple={multiple}
          onChange={handleFileSelect}
          className="hidden"
        />
      )}
    </div>
  );
}
