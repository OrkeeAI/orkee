// ABOUTME: Message bubble component for displaying conversation messages
// ABOUTME: Renders user and assistant messages with appropriate styling and formatting

import React from 'react';
import { cn } from '@/lib/utils';
import { User, Bot } from 'lucide-react';
import type { ConversationMessage } from '@/services/chat';
import type { StreamingMessage } from '../hooks/useStreamingResponse';

export interface MessageBubbleProps {
  message: ConversationMessage | StreamingMessage;
  isStreaming?: boolean;
}

export function MessageBubble({ message, isStreaming = false }: MessageBubbleProps) {
  const isUser = message.role === 'user';
  const isAssistant = message.role === 'assistant';
  const isSystem = message.role === 'system';

  if (isSystem) {
    return (
      <div className="flex justify-center my-4">
        <div className="bg-muted px-4 py-2 rounded-full text-xs text-muted-foreground">
          {message.content}
        </div>
      </div>
    );
  }

  return (
    <div
      className={cn(
        'flex gap-3 mb-4',
        isUser && 'flex-row-reverse',
        isAssistant && 'flex-row'
      )}
    >
      <div
        className={cn(
          'flex-shrink-0 w-8 h-8 rounded-full flex items-center justify-center',
          isUser && 'bg-primary text-primary-foreground',
          isAssistant && 'bg-slate-600 dark:bg-slate-700 text-white'
        )}
      >
        {isUser ? <User className="h-4 w-4" /> : <Bot className="h-4 w-4" />}
      </div>

      <div
        className={cn(
          'flex-1 max-w-[70%]',
          isUser && 'flex justify-end'
        )}
      >
        <div
          className={cn(
            'rounded-lg px-4 py-3 prose prose-sm max-w-none border',
            isUser && 'bg-primary text-primary-foreground border-primary/20',
            isAssistant && 'bg-slate-100 dark:bg-slate-800 text-foreground border-slate-200 dark:border-slate-700',
            isStreaming && 'animate-pulse'
          )}
        >
          <div className="whitespace-pre-wrap break-words">
            {message.content}
            {isStreaming && (
              <span className="inline-block w-2 h-4 ml-1 bg-current animate-pulse" />
            )}
          </div>
        </div>

        {'created_at' in message && (
          <div
            className={cn(
              'text-xs text-muted-foreground mt-1 px-1',
              isUser && 'text-right'
            )}
          >
            {new Date(message.created_at).toLocaleTimeString([], {
              hour: '2-digit',
              minute: '2-digit',
            })}
          </div>
        )}
      </div>
    </div>
  );
}
