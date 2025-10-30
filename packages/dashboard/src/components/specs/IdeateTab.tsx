// ABOUTME: Ideation session management view for creating and managing PRD generation sessions
// ABOUTME: Supports quick, guided, and comprehensive ideation modes with session tracking
import { useState } from 'react';
import { Lightbulb } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { CreatePRDFlow } from '@/components/ideate/CreatePRDFlow';
import { SessionsList } from '@/components/ideate/SessionsList';
import { QuickModeFlow } from '@/components/ideate/QuickMode';
import { GuidedModeFlow } from '@/components/ideate/GuidedMode';
import { ComprehensiveModeFlow } from '@/components/ideate/ComprehensiveMode';
import type { IdeateSession } from '@/services/ideate';

interface IdeateTabProps {
  projectId: string;
}

export function IdeateTab({ projectId }: IdeateTabProps) {
  const [showIdeateFlow, setShowIdeateFlow] = useState(false);
  const [showQuickModeFlow, setShowQuickModeFlow] = useState(false);
  const [showGuidedModeFlow, setShowGuidedModeFlow] = useState(false);
  const [showComprehensiveModeFlow, setShowComprehensiveModeFlow] = useState(false);
  const [activeSessionId, setActiveSessionId] = useState<string | null>(null);
  const [skipToGenerateQuick, setSkipToGenerateQuick] = useState(false);

  const handleResumeSession = (session: IdeateSession) => {
    setActiveSessionId(session.id);
    if (session.mode === 'quick') {
      setShowQuickModeFlow(true);
    } else if (session.mode === 'guided') {
      setShowGuidedModeFlow(true);
    } else if (session.mode === 'comprehensive') {
      setShowComprehensiveModeFlow(true);
    }
  };

  const handleSessionCreated = (sessionId: string, mode: 'quick' | 'guided' | 'comprehensive') => {
    setActiveSessionId(sessionId);
    if (mode === 'quick') {
      setSkipToGenerateQuick(true);
      setShowQuickModeFlow(true);
    } else if (mode === 'guided') {
      setShowGuidedModeFlow(true);
    } else if (mode === 'comprehensive') {
      setShowComprehensiveModeFlow(true);
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

  const handleComprehensiveModeComplete = () => {
    setShowComprehensiveModeFlow(false);
    setActiveSessionId(null);
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

          <ComprehensiveModeFlow
            projectId={projectId}
            sessionId={activeSessionId}
            open={showComprehensiveModeFlow}
            onOpenChange={setShowComprehensiveModeFlow}
            onComplete={handleComprehensiveModeComplete}
          />
        </>
      )}
    </div>
  );
}
