// ABOUTME: Main chat view component with message display and input
// ABOUTME: Handles message rendering, auto-scroll, and user input submission

import { useRef, useEffect, useState } from 'react';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Loader2 } from 'lucide-react';
import { modelsService, type Model } from '@/services/models';
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

  // Model selection state
  const [allModels, setAllModels] = useState<Model[]>([]);
  const [selectedProvider, setSelectedProvider] = useState<string>('');
  const [selectedModel, setSelectedModel] = useState<string>('');

  // Get unique providers from models
  const providers = Array.from(new Set(allModels.map(m => m.provider)));

  // Filter models by selected provider
  const availableModels = selectedProvider
    ? allModels.filter(m => m.provider === selectedProvider)
    : [];

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  };

  useEffect(() => {
    scrollToBottom();
  }, [messages, streamingMessage]);

  // Load available models
  useEffect(() => {
    const loadModels = async () => {
      try {
        const response = await modelsService.listModels();
        const available = response.items.filter(m => m.is_available);
        setAllModels(available);

        // Set default provider and model
        if (available.length > 0 && !selectedProvider) {
          const firstProvider = available[0].provider;
          setSelectedProvider(firstProvider);

          const firstModel = available.find(m => m.provider === firstProvider);
          if (firstModel) {
            setSelectedModel(firstModel.id);
          }
        }
      } catch (error) {
        console.error('Failed to load models:', error);
      }
    };

    loadModels();
  }, [selectedProvider]);

  // Update selected model when provider changes
  useEffect(() => {
    if (selectedProvider && availableModels.length > 0) {
      // If current model doesn't belong to new provider, select first model from new provider
      const currentModelProvider = allModels.find(m => m.id === selectedModel)?.provider;
      if (currentModelProvider !== selectedProvider) {
        setSelectedModel(availableModels[0].id);
      }
    }
  }, [selectedProvider, availableModels, allModels, selectedModel]);

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
                {providers.map((provider) => (
                  <PromptInputModelSelectItem key={provider} value={provider}>
                    {provider}
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
                  <PromptInputModelSelectItem key={model.id} value={model.id}>
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
