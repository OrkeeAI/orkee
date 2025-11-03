// ABOUTME: Main chat view component with message display and input
// ABOUTME: Handles message rendering, auto-scroll, and user input submission

import React, { useRef, useEffect } from 'react';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Loader2 } from 'lucide-react';
import { useModels } from '@/hooks/useModels';
import { useCurrentUser } from '@/hooks/useUsers';
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
  PromptInputModelSelect,
  PromptInputModelSelectTrigger,
  PromptInputModelSelectContent,
  PromptInputModelSelectItem,
  PromptInputModelSelectValue,
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

  const { data: currentUser } = useCurrentUser();
  const { data: models } = useModels();

  // Get available providers (matching ModelSelectionDialog pattern)
  const availableProviders = React.useMemo(() => {
    if (!currentUser) return [];

    const providers = [
      {
        value: 'anthropic',
        label: 'Anthropic (Claude)',
        hasKey: !!currentUser.has_anthropic_api_key,
      },
      {
        value: 'openai',
        label: 'OpenAI (GPT)',
        hasKey: !!currentUser.has_openai_api_key,
      },
      {
        value: 'google',
        label: 'Google (Gemini)',
        hasKey: !!currentUser.has_google_api_key,
      },
      {
        value: 'xai',
        label: 'xAI (Grok)',
        hasKey: !!currentUser.has_xai_api_key,
      },
    ];

    return providers.filter((p) => p.hasKey);
  }, [currentUser]);

  // Get models for selected provider
  const [selectedProvider, setSelectedProvider] = React.useState('anthropic');
  const [selectedModel, setSelectedModel] = React.useState('');

  const availableModels = React.useMemo(() => {
    if (!models || !selectedProvider) return [];

    return models.filter((model) => model.provider === selectedProvider);
  }, [models, selectedProvider]);

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  };

  useEffect(() => {
    scrollToBottom();
  }, [messages, streamingMessage]);

  // Auto-select first model when provider changes or models load
  useEffect(() => {
    if (availableModels.length > 0 && !selectedModel) {
      setSelectedModel(availableModels[0].model);
    }
  }, [availableModels, selectedModel]);

  // Reset model when provider changes
  useEffect(() => {
    if (availableModels.length > 0) {
      const currentModel = availableModels.find(m => m.model === selectedModel);
      if (!currentModel) {
        setSelectedModel(availableModels[0].model);
      }
    }
  }, [selectedProvider, availableModels, selectedModel]);

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
            {/* Provider Selector */}
            <PromptInputModelSelect value={selectedProvider} onValueChange={setSelectedProvider}>
              <PromptInputModelSelectTrigger>
                <PromptInputModelSelectValue placeholder="Provider..." />
              </PromptInputModelSelectTrigger>
              <PromptInputModelSelectContent>
                {availableProviders.map((provider) => (
                  <PromptInputModelSelectItem key={provider.value} value={provider.value}>
                    {provider.label}
                  </PromptInputModelSelectItem>
                ))}
              </PromptInputModelSelectContent>
            </PromptInputModelSelect>

            {/* Model Selector */}
            <PromptInputModelSelect value={selectedModel} onValueChange={setSelectedModel}>
              <PromptInputModelSelectTrigger>
                <PromptInputModelSelectValue placeholder="Model..." />
              </PromptInputModelSelectTrigger>
              <PromptInputModelSelectContent>
                {availableModels.map((model) => (
                  <PromptInputModelSelectItem key={model.id} value={model.model}>
                    {model.display_name}
                  </PromptInputModelSelectItem>
                ))}
              </PromptInputModelSelectContent>
            </PromptInputModelSelect>

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
