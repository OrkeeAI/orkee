// ABOUTME: Main chat view component with message display and input
// ABOUTME: Handles message rendering, auto-scroll, and user input submission

import React, { useRef, useEffect } from 'react';
import { ScrollArea } from '@/components/ui/scroll-area';
import { ArrowUpIcon, Loader2 } from 'lucide-react';
import { useModels } from '@/hooks/useModels';
import { useCurrentUser } from '@/hooks/useUsers';
import { useTaskModel } from '@/contexts/ModelPreferencesContext';
import {
  InputGroup,
  InputGroupTextarea,
  InputGroupAddon,
  InputGroupButton,
} from '@/components/ui/input-group';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { MessageBubble } from './MessageBubble';
import type { ChatMessage } from '@/services/chat';
import type { StreamingMessage } from '../hooks/useStreamingResponse';
import { UI_TEXT } from '../constants';

export interface ChatViewProps {
  messages: ChatMessage[];
  streamingMessage: StreamingMessage | null;
  onSendMessage: (content: string, model?: string, provider?: string) => void | Promise<void>;
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

  const { data: currentUser, isLoading: isLoadingUser } = useCurrentUser();
  const { data: models, isLoading: isLoadingModels } = useModels();
  const chatModelPreference = useTaskModel('chat');

  // Get available providers (matching ModelSelectionDialog pattern)
  const availableProviders = React.useMemo(() => {
    if (!currentUser) {
      console.log('[ChatView] No currentUser data yet');
      return [];
    }

    console.log('[ChatView] currentUser:', currentUser);

    const providers = [
      {
        value: 'anthropic',
        label: 'Anthropic',
        hasKey: !!currentUser.has_anthropic_api_key,
      },
      {
        value: 'openai',
        label: 'OpenAI',
        hasKey: !!currentUser.has_openai_api_key,
      },
      {
        value: 'google',
        label: 'Google',
        hasKey: !!currentUser.has_google_api_key,
      },
      {
        value: 'xai',
        label: 'xAI',
        hasKey: !!currentUser.has_xai_api_key,
      },
    ];

    const filtered = providers.filter((p) => p.hasKey);
    console.log('[ChatView] availableProviders:', filtered);
    return filtered;
  }, [currentUser]);

  // Get models for selected provider - Initialize with user's chat preferences
  const [selectedProvider, setSelectedProvider] = React.useState(chatModelPreference.provider);
  const [selectedModel, setSelectedModel] = React.useState(chatModelPreference.model);
  const [inputValue, setInputValue] = React.useState('');

  const availableModels = React.useMemo(() => {
    console.log('[ChatView] Computing availableModels:', { models, selectedProvider });
    if (!models || !selectedProvider) {
      console.log('[ChatView] No models or provider:', { hasModels: !!models, selectedProvider });
      return [];
    }

    const filtered = models.filter((model) => model.provider === selectedProvider);
    console.log('[ChatView] Filtered models for provider:', { provider: selectedProvider, count: filtered.length, models: filtered });
    return filtered;
  }, [models, selectedProvider]);

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  };

  useEffect(() => {
    scrollToBottom();
  }, [messages, streamingMessage]);

  // Update selected values when user's preferences change
  useEffect(() => {
    setSelectedProvider(chatModelPreference.provider);
    setSelectedModel(chatModelPreference.model);
  }, [chatModelPreference.provider, chatModelPreference.model]);

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

  const handleSubmit = async (e?: React.FormEvent) => {
    e?.preventDefault();
    if (inputValue.trim() && !isSending) {
      console.log('[ChatView.handleSubmit] Submitting message with:', {
        provider: selectedProvider,
        model: selectedModel,
        text: inputValue.substring(0, 50) + '...'
      });
      await onSendMessage(inputValue, selectedModel, selectedProvider);
      setInputValue('');
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSubmit();
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

      <div className="px-4 pb-4">
        <InputGroup className="min-h-[120px] p-3">
          <InputGroupTextarea
            placeholder={UI_TEXT.INPUT_PLACEHOLDER}
            value={inputValue}
            onChange={(e) => setInputValue(e.target.value)}
            onKeyDown={handleKeyDown}
            disabled={isSending}
            rows={2}
            className="min-h-[50px] max-h-[200px] py-0"
          />
          <InputGroupAddon align="block-end">
            {/* Provider Selector */}
            <Select value={selectedProvider} onValueChange={setSelectedProvider} disabled={isSending}>
              <SelectTrigger className="h-6 w-auto min-w-[140px] border-0 bg-transparent shadow-none text-xs focus:ring-0">
                <SelectValue>
                  {availableProviders.find(p => p.value === selectedProvider)?.label || 'Provider...'}
                </SelectValue>
              </SelectTrigger>
              <SelectContent>
                {isLoadingUser ? (
                  <SelectItem value="loading" disabled>Loading providers...</SelectItem>
                ) : availableProviders.length === 0 ? (
                  <SelectItem value="none" disabled>No API keys configured</SelectItem>
                ) : (
                  availableProviders.map((provider) => (
                    <SelectItem key={provider.value} value={provider.value}>
                      {provider.label}
                    </SelectItem>
                  ))
                )}
              </SelectContent>
            </Select>

            {/* Model Selector */}
            <Select value={selectedModel} onValueChange={setSelectedModel} disabled={isSending}>
              <SelectTrigger className="h-6 w-auto min-w-[140px] border-0 bg-transparent shadow-none text-xs focus:ring-0">
                <SelectValue>
                  {availableModels.find(m => m.model === selectedModel)?.display_name ||
                   availableModels.find(m => m.model === selectedModel)?.model ||
                   'Model...'}
                </SelectValue>
              </SelectTrigger>
              <SelectContent>
                {isLoadingModels ? (
                  <SelectItem value="loading" disabled>Loading models...</SelectItem>
                ) : availableModels.length === 0 ? (
                  <SelectItem value="none" disabled>No models for this provider</SelectItem>
                ) : (
                  availableModels.map((model) => (
                    <SelectItem key={model.id} value={model.model}>
                      {model.display_name || model.model}
                    </SelectItem>
                  ))
                )}
              </SelectContent>
            </Select>

            <InputGroupButton
              variant="default"
              className="rounded-full ml-auto"
              size="icon-xs"
              disabled={!inputValue.trim() || isSending}
              onClick={handleSubmit}
            >
              {isSending ? (
                <Loader2 className="h-4 w-4 animate-spin" />
              ) : (
                <ArrowUpIcon className="h-4 w-4" />
              )}
              <span className="sr-only">Send</span>
            </InputGroupButton>
          </InputGroupAddon>
        </InputGroup>
      </div>
    </div>
  );
}
