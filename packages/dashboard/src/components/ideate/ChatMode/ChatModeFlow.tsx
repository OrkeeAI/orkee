// ABOUTME: Main container for chat PRD discovery mode
// ABOUTME: Orchestrates chat, insights, quality tracking, and PRD generation

import React, { useState, useCallback, useEffect } from 'react';
import { Button } from '@/components/ui/button';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Loader2, FileText, AlertCircle } from 'lucide-react';
import { ChatView } from './components/ChatView';
import { QualityIndicator } from './components/QualityIndicator';
import { InsightsSidebar } from './components/InsightsSidebar';
import { DiscoveryProgress } from './components/DiscoveryProgress';
import { CodebaseContextPanel } from './components/CodebaseContextPanel';
import { ValidationCheckpoint, CheckpointSection } from './components/ValidationCheckpoint';
import { useChat } from './hooks/useChat';
import { useStreamingResponse } from './hooks/useStreamingResponse';
import { chatService, ChatInsight } from '@/services/chat';
import { extractInsights } from '@/services/chat-ai';
import { ideateService, DiscoveryProgress as DiscoveryProgressType, CodebaseContext } from '@/services/ideate';
import { useCurrentUser } from '@/hooks/useUsers';
import { useModelPreferences, getModelForTask } from '@/services/model-preferences';
import { UI_TEXT } from './constants';

export interface ChatModeFlowProps {
  sessionId: string;
  projectId: string;
  onPRDGenerated: (prdId: string) => void;
}

export function ChatModeFlow({
  sessionId,
  projectId,
  onPRDGenerated,
}: ChatModeFlowProps) {
  const [insights, setInsights] = useState<ChatInsight[]>([]);
  const [isGeneratingPRD, setIsGeneratingPRD] = useState(false);
  const [prdError, setPrdError] = useState<Error | null>(null);
  const [isReanalyzing, setIsReanalyzing] = useState(false);

  // Phase 6C: Discovery Progress state
  const [discoveryProgress, setDiscoveryProgress] = useState<DiscoveryProgressType | null>(null);

  // Phase 6C: Codebase Context state
  const [codebaseContext, setCodebaseContext] = useState<CodebaseContext | null>(null);
  const [isAnalyzingCodebase, setIsAnalyzingCodebase] = useState(false);
  const [projectPath] = useState<string | null>(null);

  // Phase 6C: Validation Checkpoint state
  const [showCheckpoint, setShowCheckpoint] = useState(false);
  const [checkpointSections, setCheckpointSections] = useState<CheckpointSection[]>([]);
  const [messagesSinceLastCheckpoint, setMessagesSinceLastCheckpoint] = useState(0);

  // Phase 6: Get user and model preferences for insight extraction
  const { data: currentUser } = useCurrentUser();
  const { data: modelPreferences } = useModelPreferences(currentUser?.id);

  const {
    messages,
    qualityMetrics,
    isLoading,
    isSending,
    error: chatError,
    sendMessage,
    refresh,
  } = useChat({
    sessionId,
    autoLoadHistory: true,
  });

  const { streamingMessage, isStreaming, startStreaming, stopStreaming } = useStreamingResponse({
    sessionId,
    chatHistory: messages,
    projectId,
    onMessageComplete: async (content: string) => {
      try {
        // Save assistant message to backend
        await chatService.sendMessage(sessionId, {
          content,
          message_type: 'discovery',
          role: 'assistant',
        });

        // Phase 6: Extract insights using user's preferred model
        // This happens in the frontend now to respect user's model preferences
        try {
          const insightPreferences = getModelForTask(modelPreferences, 'insight_extraction');
          console.log('[ChatModeFlow] Extracting insights with model:', insightPreferences);

          await extractInsights(sessionId, messages, insightPreferences, projectId);
          console.log('[ChatModeFlow] Insights extracted successfully');
        } catch (insightError) {
          // Don't block on insight extraction failure - log and continue
          console.warn('[ChatModeFlow] Failed to extract insights:', insightError);
        }
      } catch (err) {
        console.error('Failed to save assistant message:', err);
      } finally {
        // Always clear streaming state to prevent race conditions with new messages
        stopStreaming();
        await refresh();
        await loadInsights();
      }
    },
    onError: (error) => {
      console.error('Streaming error:', error);
    },
  });

  // Load insights from backend (extracted by frontend after streaming completes)
  const loadInsights = useCallback(async () => {
    try {
      const data = await chatService.getInsights(sessionId);
      setInsights(data);
    } catch (err) {
      console.error('Failed to load insights:', err);
    }
  }, [sessionId]);

  // Load insights initially and after each new message
  React.useEffect(() => {
    loadInsights();
  }, [loadInsights, messages.length]); // Reload when messages change

  // Re-analyze all messages to extract insights
  const handleReanalyzeInsights = useCallback(async () => {
    try {
      setIsReanalyzing(true);
      const result = await chatService.reanalyzeInsights(sessionId);
      console.log(`Re-analysis complete: ${result.extracted_count} insights extracted from ${result.total_messages_processed} messages`);
      await loadInsights();
    } catch (err) {
      console.error('Failed to re-analyze insights:', err);
    } finally {
      setIsReanalyzing(false);
    }
  }, [sessionId, loadInsights]);

  // Phase 6C: Load discovery progress
  const loadDiscoveryProgress = useCallback(async () => {
    try {
      const progress = await ideateService.getDiscoveryProgress(sessionId);
      setDiscoveryProgress(progress);
    } catch (err) {
      console.error('Failed to load discovery progress:', err);
    }
  }, [sessionId]);

  // Load discovery progress when messages change
  React.useEffect(() => {
    loadDiscoveryProgress();
  }, [loadDiscoveryProgress, messages.length]); // Reload when messages change

  // Phase 6C: Load codebase context
  const loadCodebaseContext = useCallback(async () => {
    try {
      const context = await ideateService.getCodebaseContext(sessionId);
      setCodebaseContext(context);
    } catch (err) {
      // It's okay if context doesn't exist yet
      console.debug('No codebase context yet:', err);
    }
  }, [sessionId]);

  // Phase 6C: Trigger codebase analysis
  const handleAnalyzeCodebase = useCallback(async () => {
    if (!projectPath) {
      console.warn('No project path available for analysis');
      return;
    }

    try {
      setIsAnalyzingCodebase(true);
      await ideateService.analyzeCodebase(sessionId, projectPath);
      await loadCodebaseContext();
    } catch (err) {
      console.error('Failed to analyze codebase:', err);
    } finally {
      setIsAnalyzingCodebase(false);
    }
  }, [sessionId, projectPath, loadCodebaseContext]);

  // Phase 6C: Handle checkpoint approval
  const handleCheckpointApprove = useCallback(async () => {
    setShowCheckpoint(false);
    setMessagesSinceLastCheckpoint(0);

    // Store validation feedback
    try {
      for (const section of checkpointSections) {
        await ideateService.storeValidationFeedback(sessionId, {
          section_name: section.name,
          validation_status: 'approved',
          quality_score: section.quality_score,
        });
      }
    } catch (err) {
      console.error('Failed to store validation feedback:', err);
    }
  }, [sessionId, checkpointSections]);

  // Phase 6C: Handle checkpoint rejection
  const handleCheckpointReject = useCallback(async () => {
    setShowCheckpoint(false);
    setMessagesSinceLastCheckpoint(0);

    // Store rejection feedback
    try {
      for (const section of checkpointSections) {
        await ideateService.storeValidationFeedback(sessionId, {
          section_name: section.name,
          validation_status: 'rejected',
          quality_score: section.quality_score,
        });
      }
    } catch (err) {
      console.error('Failed to store validation feedback:', err);
    }
  }, [sessionId, checkpointSections]);

  // Phase 6C: Handle inline editing of checkpoint sections
  const handleCheckpointEdit = useCallback(async (sectionName: string, newContent: string) => {
    // Update the section content locally
    setCheckpointSections((prev) =>
      prev.map((section) =>
        section.name === sectionName ? { ...section, content: newContent } : section
      )
    );

    // TODO: Persist the edited content to the backend
    console.log('Section edited:', sectionName, newContent);
  }, []);


  // Phase 6C: Load codebase context on mount
  useEffect(() => {
    loadCodebaseContext();
  }, [loadCodebaseContext]);

  // Phase 6C: Track messages silently in background (checkpoint modal disabled - only show when explicitly requested)
  // Note: Auto-triggering removed - validation tracking happens in background via insights/metrics
  // To re-enable auto checkpoints, uncomment the showCheckpoint logic below
  useEffect(() => {
    const newMessageCount = messagesSinceLastCheckpoint + 1;

    if (newMessageCount >= 5 && messages.length > 0) {
      // Background validation tracking - prepare sections but DON'T show modal
      const sections: CheckpointSection[] = [
        {
          name: 'problem',
          content: insights.filter((i) => i.insight_type === 'requirement').map((i) => i.insight_text).join('\n') || 'Not captured yet',
          quality_score: qualityMetrics?.coverage.problem ? 80 : 40,
        },
        {
          name: 'users',
          content: insights.filter((i) => i.insight_type === 'assumption').map((i) => i.insight_text).join('\n') || 'Not captured yet',
          quality_score: qualityMetrics?.coverage.users ? 80 : 40,
        },
        {
          name: 'features',
          content: insights.filter((i) => i.insight_type === 'decision').map((i) => i.insight_text).join('\n') || 'Not captured yet',
          quality_score: qualityMetrics?.coverage.features ? 80 : 40,
        },
      ];

      setCheckpointSections(sections);
      // setShowCheckpoint(true); // DISABLED - only show checkpoint when user explicitly requests
      setMessagesSinceLastCheckpoint(0); // Reset counter after validation
    } else {
      setMessagesSinceLastCheckpoint(newMessageCount);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [messages.length, insights, qualityMetrics]); // Note: Intentionally not including messagesSinceLastCheckpoint to avoid loops

  const handleSendMessage = useCallback(
    async (content: string, model?: string, provider?: string) => {
      try {
        console.log('[ChatModeFlow.handleSendMessage] Received:', { model, provider, content: content.substring(0, 50) });
        // Save user message to backend
        await sendMessage(content, 'discovery', model);
        // Start AI streaming response with selected model
        console.log('[ChatModeFlow.handleSendMessage] Calling startStreaming with:', { provider, model });
        await startStreaming(content, provider, model);
      } catch (err) {
        console.error('Failed to send message:', err);
      }
    },
    [sendMessage, startStreaming]
  );

  const handleGeneratePRD = useCallback(async () => {
    try {
      setIsGeneratingPRD(true);
      setPrdError(null);

      const validation = await chatService.validateForPRD(sessionId);

      if (!validation.is_valid) {
        setPrdError(
          new Error(
            `Cannot generate PRD yet. Missing: ${validation.missing_required.join(', ')}`
          )
        );
        return;
      }

      const result = await chatService.generatePRD(sessionId, {
        title: `PRD from Chat - ${new Date().toLocaleDateString()}`,
      });

      onPRDGenerated(result.prd_id);
    } catch (err) {
      setPrdError(err instanceof Error ? err : new Error('Failed to generate PRD'));
    } finally {
      setIsGeneratingPRD(false);
    }
  }, [sessionId, onPRDGenerated]);

  return (
    <div className="h-full flex flex-col">
      <div className="flex-1 grid grid-cols-12 gap-4 min-h-0">
        <div className="col-span-8 flex flex-col border rounded-lg bg-card min-h-0">
          <div className="border-b p-4 space-y-3 flex-shrink-0">
            <div className="flex items-start justify-between">
              <div>
                <h2 className="text-lg font-semibold">Chat</h2>
                <p className="text-sm text-muted-foreground">
                  Discuss your project idea and I'll help discover the requirements
                </p>
              </div>

              <Button
                onClick={handleGeneratePRD}
                disabled={
                  !qualityMetrics?.is_ready_for_prd || isGeneratingPRD || isSending || isStreaming
                }
                size="lg"
                className="gap-2"
              >
                {isGeneratingPRD ? (
                  <>
                    <Loader2 className="h-4 w-4 animate-spin" />
                    {UI_TEXT.GENERATING_PRD}
                  </>
                ) : (
                  <>
                    <FileText className="h-4 w-4" />
                    {UI_TEXT.GENERATE_PRD}
                  </>
                )}
              </Button>
            </div>

            {/* Phase 6C: Discovery Progress */}
            <DiscoveryProgress progress={discoveryProgress} />

            {/* Error Alerts */}
            {chatError && (
              <Alert variant="destructive">
                <AlertCircle className="h-4 w-4" />
                <AlertDescription>{chatError.message}</AlertDescription>
              </Alert>
            )}

            {prdError && (
              <Alert variant="destructive">
                <AlertCircle className="h-4 w-4" />
                <AlertDescription>{prdError.message}</AlertDescription>
              </Alert>
            )}
          </div>

          <div className="flex-1 overflow-hidden min-h-0">
            <ChatView
              messages={messages}
              streamingMessage={streamingMessage}
              onSendMessage={handleSendMessage}
              isLoading={isLoading}
              isSending={isSending || isStreaming}
            />
          </div>
        </div>

        <div className="col-span-4 flex flex-col gap-4 min-h-0 overflow-y-auto">
          <QualityIndicator metrics={qualityMetrics} className="flex-shrink-0" />

          {/* Phase 6C: Codebase Context Panel */}
          <CodebaseContextPanel
            sessionId={sessionId}
            projectPath={projectPath}
            context={codebaseContext}
            isAnalyzing={isAnalyzingCodebase}
            onAnalyze={handleAnalyzeCodebase}
            className="flex-shrink-0"
          />

          <InsightsSidebar
            insights={insights}
            onReanalyze={handleReanalyzeInsights}
            isReanalyzing={isReanalyzing}
            className="flex-shrink-0"
          />
        </div>
      </div>

      {/* Phase 6C: Validation Checkpoint Modal */}
      <ValidationCheckpoint
        open={showCheckpoint}
        onOpenChange={setShowCheckpoint}
        sections={checkpointSections}
        onApprove={handleCheckpointApprove}
        onEdit={handleCheckpointEdit}
        onReject={handleCheckpointReject}
      />
    </div>
  );
}
