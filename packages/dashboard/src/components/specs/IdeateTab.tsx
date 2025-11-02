// ABOUTME: Ideation session management view for creating and managing PRD generation sessions
// ABOUTME: Supports quick, guided, and conversational ideation modes with session tracking
import { useState } from 'react';
import { Lightbulb } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { CreatePRDFlow } from '@/components/ideate/CreatePRDFlow';
import { SessionsList } from '@/components/ideate/SessionsList';
import { QuickModeFlow } from '@/components/ideate/QuickMode';
import { GuidedModeFlow } from '@/components/ideate/GuidedMode';
import { ConversationalModeFlow } from '@/components/ideate/ConversationalMode/ConversationalModeFlow';
import type { IdeateSession, IdeateMode } from '@/services/ideate';

interface IdeateTabProps {
  projectId: string;
}

export function IdeateTab({ projectId }: IdeateTabProps) {
  const [showIdeateFlow, setShowIdeateFlow] = useState(false);
  const [showQuickModeFlow, setShowQuickModeFlow] = useState(false);
  const [showGuidedModeFlow, setShowGuidedModeFlow] = useState(false);
  const [showConversationalModeFlow, setShowConversationalModeFlow] = useState(false);
  const [activeSessionId, setActiveSessionId] = useState<string | null>(null);
  const [skipToGenerateQuick, setSkipToGenerateQuick] = useState(false);

  const handleResumeSession = (session: IdeateSession) => {
    setActiveSessionId(session.id);
    if (session.mode === 'quick') {
      setShowQuickModeFlow(true);
    } else if (session.mode === 'guided') {
      setShowGuidedModeFlow(true);
    } else if (session.mode === 'conversational') {
      setShowConversationalModeFlow(true);
    }
  };

  const handleSessionCreated = (sessionId: string, mode: IdeateMode) => {
    setActiveSessionId(sessionId);
    if (mode === 'quick') {
      setSkipToGenerateQuick(true);
      setShowQuickModeFlow(true);
    } else if (mode === 'guided') {
      setShowGuidedModeFlow(true);
    } else if (mode === 'conversational') {
      setShowConversationalModeFlow(true);
    }
  };

  const handleQuickModeComplete = () => {
    setShowQuickModeFlow(false);
    setActiveSessionId(null);
    setSkipToGenerateQuick(false);
  };

  const handleGuidedModeComplete = () => {
    setShowGuidedModeFlow(false);
    setActiveSessionId(null);
  };

  const handleConversationalModeComplete = (prdId: string) => {
    setShowConversationalModeFlow(false);
    setActiveSessionId(null);
    console.log('PRD generated:', prdId);
  };

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-lg font-semibold">Ideate Sessions</h3>
          <p className="text-sm text-muted-foreground">
            Create and manage PRD generation sessions
          </p>
        </div>
        <Button onClick={() => setShowIdeateFlow(true)} size="sm">
          <Lightbulb className="mr-2 h-4 w-4" />
          New Session
        </Button>
      </div>

      <SessionsList projectId={projectId} onResumeSession={handleResumeSession} />

      <CreatePRDFlow
        projectId={projectId}
        open={showIdeateFlow}
        onOpenChange={setShowIdeateFlow}
        onSessionCreated={handleSessionCreated}
      />

      {activeSessionId && (
        <>
          <QuickModeFlow
            projectId={projectId}
            sessionId={activeSessionId}
            open={showQuickModeFlow}
            onOpenChange={setShowQuickModeFlow}
            onComplete={handleQuickModeComplete}
            skipToGenerate={skipToGenerateQuick}
          />

          <GuidedModeFlow
            projectId={projectId}
            sessionId={activeSessionId}
            open={showGuidedModeFlow}
            onOpenChange={setShowGuidedModeFlow}
            onComplete={handleGuidedModeComplete}
          />

          {showConversationalModeFlow && (
            <div className="fixed inset-0 z-50 bg-background">
              <div className="h-full flex flex-col">
                <div className="border-b p-4">
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => handleConversationalModeComplete('')}
                  >
                    ← Back to Sessions
                  </Button>
                </div>
                <div className="flex-1 overflow-hidden p-4">
                  <ConversationalModeFlow
                    projectId={projectId}
                    sessionId={activeSessionId}
                    onPRDGenerated={handleConversationalModeComplete}
                  />
                </div>
              </div>
            </div>
          )}
        </>
      )}
    </div>
  );
}
