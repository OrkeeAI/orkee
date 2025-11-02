// ABOUTME: Main conversation view component with message display and input
// ABOUTME: Handles message rendering, auto-scroll, and user input submission

import React, { useRef, useEffect, useState } from 'react';
import { Button } from '@/components/ui/button';
import { Textarea } from '@/components/ui/textarea';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Send, Loader2 } from 'lucide-react';
import { MessageBubble } from './MessageBubble';
import { SuggestedQuestions } from './SuggestedQuestions';
import type { ChatMessage, DiscoveryQuestion } from '@/services/chat';
import type { StreamingMessage } from '../hooks/useStreamingResponse';
import { UI_TEXT } from '../constants';

export interface ChatViewProps {
  messages: ChatMessage[];
  streamingMessage: StreamingMessage | null;
  suggestedQuestions: DiscoveryQuestion[];
  onSendMessage: (content: string) => void;
  isLoading: boolean;
  isSending: boolean;
}

export function ChatView({
  messages,
  streamingMessage,
  suggestedQuestions,
  onSendMessage,
  isLoading,
  isSending,
}: ChatViewProps) {
  const [input, setInput] = useState('');
  const scrollAreaRef = useRef<HTMLDivElement>(null);
  const messagesEndRef = useRef<HTMLDivElement>(null);

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  };

  useEffect(() => {
    scrollToBottom();
  }, [messages, streamingMessage]);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (input.trim() && !isSending) {
      onSendMessage(input.trim());
      setInput('');
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSubmit(e);
    }
  };

  const handleQuestionSelect = (question: string) => {
    setInput(question);
  };

  const allMessages = [...messages];
  if (streamingMessage) {
    allMessages.push(streamingMessage as ChatMessage);
  }

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

      <div className="border-t p-4 space-y-3 bg-background">
        {suggestedQuestions.length > 0 && (
          <SuggestedQuestions
            questions={suggestedQuestions}
            onSelectQuestion={handleQuestionSelect}
            isDisabled={isSending}
          />
        )}

        <form onSubmit={handleSubmit} className="flex gap-2">
          <Textarea
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder={UI_TEXT.INPUT_PLACEHOLDER}
            className="resize-none min-h-[60px] max-h-[120px]"
            disabled={isSending}
          />
          <Button
            type="submit"
            size="icon"
            disabled={!input.trim() || isSending}
            className="h-[60px] w-[60px] flex-shrink-0"
          >
            {isSending ? (
              <Loader2 className="h-5 w-5 animate-spin" />
            ) : (
              <Send className="h-5 w-5" />
            )}
          </Button>
        </form>
      </div>
    </div>
  );
}
