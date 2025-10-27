// ABOUTME: User input component for sending messages to the roundtable discussion
// ABOUTME: Provides textarea with send button and loading state

import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Textarea } from '@/components/ui/textarea';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Send, Loader2, MessageCircle } from 'lucide-react';
import { useSendInterjection } from '@/hooks/useIdeate';
import { toast } from 'sonner';

interface UserInterjectionInputProps {
  roundtableId: string;
  disabled?: boolean;
}

export function UserInterjectionInput({
  roundtableId,
  disabled = false,
}: UserInterjectionInputProps) {
  const [message, setMessage] = useState('');
  const sendInterjectionMutation = useSendInterjection(roundtableId);

  const handleSend = async () => {
    if (!message.trim()) {
      toast.error('Please enter a message');
      return;
    }

    try {
      await sendInterjectionMutation.mutateAsync({ message: message.trim() });
      toast.success('Message sent to discussion');
      setMessage('');
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error';
      toast.error('Failed to send message', { description: errorMessage });
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) {
      e.preventDefault();
      handleSend();
    }
  };

  return (
    <Card>
      <CardHeader className="pb-3">
        <CardTitle className="flex items-center gap-2 text-base">
          <MessageCircle className="h-4 w-4" />
          Your Input
        </CardTitle>
        <CardDescription className="text-sm">
          Interject with questions, feedback, or additional context
        </CardDescription>
      </CardHeader>
      <CardContent>
        <div className="space-y-3">
          <Textarea
            placeholder="Type your message... (Cmd/Ctrl + Enter to send)"
            value={message}
            onChange={(e) => setMessage(e.target.value)}
            onKeyDown={handleKeyDown}
            rows={4}
            disabled={disabled || sendInterjectionMutation.isPending}
            className="resize-none"
          />
          <div className="flex items-center justify-between">
            <span className="text-xs text-muted-foreground">
              {message.length} characters
            </span>
            <Button
              onClick={handleSend}
              disabled={disabled || !message.trim() || sendInterjectionMutation.isPending}
              size="sm"
            >
              {sendInterjectionMutation.isPending ? (
                <>
                  <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                  Sending...
                </>
              ) : (
                <>
                  <Send className="h-4 w-4 mr-2" />
                  Send Message
                </>
              )}
            </Button>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
