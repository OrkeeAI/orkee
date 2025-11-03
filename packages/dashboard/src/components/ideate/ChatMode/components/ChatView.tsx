// ABOUTME: Main chat view component with message display and input
// ABOUTME: Handles message rendering, auto-scroll, and user input submission

import { useRef, useEffect } from 'react';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Loader2 } from 'lucide-react';
import {
  PromptInput,
  PromptInputHeader,
  PromptInputAttachments,
  PromptInputAttachment,
  PromptInputBody,
  PromptInputTextarea,
  PromptInputFooter,
  PromptInputTools,
  PromptInputSubmit,
  PromptInputActionMenu,
  PromptInputActionMenuTrigger,
  PromptInputActionMenuContent,
  PromptInputActionAddAttachments,
} from '@/components/ai-elements/prompt-input';
import { MessageBubble } from './MessageBubble';
import type { ChatMessage } from '@/services/chat';
import type { StreamingMessage } from '../hooks/useStreamingResponse';
import { UI_TEXT } from '../constants';

export interface ChatViewProps {
  messages: ChatMessage[];
  streamingMessage: StreamingMessage | null;
  onSendMessage: (content: string) => void;
  isLoading: boolean;
  isSending: boolean;
}

export function ChatView({
  messages,
  streamingMessage,
  onSendMessage,
  isLoading,
  isSending,
}: ChatViewProps) {
  const scrollAreaRef = useRef<HTMLDivElement>(null);
  const messagesEndRef = useRef<HTMLDivElement>(null);

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  };

  useEffect(() => {
    scrollToBottom();
  }, [messages, streamingMessage]);

  const handleSubmit = (message: { text?: string; files?: any[] }) => {
    if (message.text?.trim() && !isSending) {
      onSendMessage(message.text);
    }
  };

  const allMessages = [...messages];
  if (streamingMessage) {
    allMessages.push(streamingMessage as unknown as ChatMessage);
  }

  // Determine input status based on current state
  const status: 'ready' | 'submitted' | 'streaming' = isSending ? 'streaming' : 'ready';

  return (
    <div className="flex flex-col h-full overflow-hidden">
      <ScrollArea ref={scrollAreaRef} className="flex-1 px-4 overflow-y-auto">
        <div className="py-4 space-y-4">
          {isLoading ? (
            <div className="flex items-center justify-center py-8">
              <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
            </div>
          ) : allMessages.length === 0 ? (
            <div className="text-center py-8 text-muted-foreground">
              <p className="text-lg font-medium mb-2">{UI_TEXT.EMPTY_STATE_TITLE}</p>
              <p className="text-sm">{UI_TEXT.EMPTY_STATE_DESCRIPTION}</p>
            </div>
          ) : (
            allMessages.map((message) => (
              <MessageBubble
                key={message.id}
                message={message}
                isStreaming={streamingMessage?.id === message.id && !streamingMessage.isComplete}
              />
            ))
          )}
          <div ref={messagesEndRef} />
        </div>
      </ScrollArea>

      <PromptInput onSubmit={handleSubmit}>
        <PromptInputHeader>
          <PromptInputAttachments>
            {(attachment) => <PromptInputAttachment data={attachment} />}
          </PromptInputAttachments>
        </PromptInputHeader>
        <PromptInputBody>
          <PromptInputTextarea placeholder={UI_TEXT.INPUT_PLACEHOLDER} disabled={isSending} />
        </PromptInputBody>
        <PromptInputFooter>
          <PromptInputTools>
            <PromptInputActionMenu>
              <PromptInputActionMenuTrigger />
              <PromptInputActionMenuContent>
                <PromptInputActionAddAttachments />
              </PromptInputActionMenuContent>
            </PromptInputActionMenu>
            <PromptInputSubmit status={status} />
          </PromptInputTools>
        </PromptInputFooter>
      </PromptInput>
    </div>
  );
}
