// ABOUTME: Main orchestrator component for expert roundtable workflow
// ABOUTME: Manages multi-step flow from setup through discussion to insights extraction

import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Loader2, Users, MessageSquare, Lightbulb, Play, CheckCircle, AlertCircle } from 'lucide-react';
import { ExpertSelector } from './ExpertSelector';
import { LiveDiscussionView } from './LiveDiscussionView';
import { UserInterjectionInput } from './UserInterjectionInput';
import { RoundtableStatus } from './RoundtableStatus';
import { InsightsExtractor } from './InsightsExtractor';
import {
  useCreateRoundtable,
  useAddParticipants,
  useStartDiscussion,
  useGetRoundtable,
  useListRoundtables,
} from '@/hooks/useIdeate';
import { toast } from 'sonner';
import type { ExpertPersona } from '@/services/ideate';

interface ExpertRoundtableFlowProps {
  sessionId: string;
  defaultTopic?: string;
}

type FlowStep = 'setup' | 'discussion' | 'insights';

export function ExpertRoundtableFlow({ sessionId, defaultTopic }: ExpertRoundtableFlowProps) {
  const [currentStep, setCurrentStep] = useState<FlowStep>('setup');
  const [topic, setTopic] = useState(defaultTopic || '');
  const [numExperts, setNumExperts] = useState(5);
  const [selectedExperts, setSelectedExperts] = useState<ExpertPersona[]>([]);
  const [activeRoundtableId, setActiveRoundtableId] = useState<string | null>(null);

  const createRoundtableMutation = useCreateRoundtable(sessionId);
  const addParticipantsMutation = useAddParticipants(activeRoundtableId || '');
  const startDiscussionMutation = useStartDiscussion(activeRoundtableId || '');

  const { data: roundtable } = useGetRoundtable(activeRoundtableId || '', {
    enabled: !!activeRoundtableId,
  });
  const { data: existingRoundtables } = useListRoundtables(sessionId);

  const handleCreateRoundtable = async () => {
    if (!topic.trim()) {
      toast.error('Please enter a discussion topic');
      return;
    }

    if (selectedExperts.length < 2 || selectedExperts.length > 10) {
      toast.error('Please select 2-10 experts');
      return;
    }

    try {
      toast.info('Creating roundtable...', { duration: 2000 });

      // Create roundtable session
      const newRoundtable = await createRoundtableMutation.mutateAsync({
        topic: topic.trim(),
        numExperts: selectedExperts.length,
      });

      setActiveRoundtableId(newRoundtable.id);

      // Add selected participants
      await addParticipantsMutation.mutateAsync({
        expertIds: selectedExperts.map((e) => e.id),
      });

      toast.success('Roundtable created successfully!');
      setCurrentStep('discussion');
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error';
      toast.error('Failed to create roundtable', { description: errorMessage });
    }
  };

  const handleStartDiscussion = async () => {
    if (!activeRoundtableId) return;

    try {
      toast.info('Starting discussion...', { duration: 2000 });

      await startDiscussionMutation.mutateAsync({
        topic: topic.trim(),
      });

      toast.success('Discussion started! Experts are now deliberating...');
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error';
      toast.error('Failed to start discussion', { description: errorMessage });
    }
  };

  const isSetupComplete = topic.trim().length > 0 && selectedExperts.length >= 2 && selectedExperts.length <= 10;
  const isDiscussionActive = roundtable?.status === 'discussing';
  const isDiscussionComplete = roundtable?.status === 'completed';

  const renderSetupStep = () => (
    <div className="space-y-6">
      <Card>
        <CardHeader>
          <CardTitle>Roundtable Setup</CardTitle>
          <CardDescription>
            Configure your expert roundtable discussion topic and participants
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div>
            <Label htmlFor="topic">Discussion Topic *</Label>
            <Input
              id="topic"
              placeholder="e.g., Building a scalable payment platform for emerging markets"
              value={topic}
              onChange={(e) => setTopic(e.target.value)}
              className="mt-1.5"
            />
            <p className="text-xs text-muted-foreground mt-1.5">
              Be specific about the challenge or decision you want expert input on
            </p>
          </div>

          <div>
            <Label htmlFor="num-experts">Number of Experts</Label>
            <Input
              id="num-experts"
              type="number"
              min={2}
              max={10}
              value={numExperts}
              onChange={(e) => setNumExperts(parseInt(e.target.value) || 5)}
              className="mt-1.5 w-32"
            />
          </div>
        </CardContent>
      </Card>

      <ExpertSelector
        sessionId={sessionId}
        selectedExperts={selectedExperts}
        onSelectionsChange={setSelectedExperts}
        minExperts={2}
        maxExperts={numExperts}
        topic={topic}
      />

      <div className="flex justify-end gap-2">
        <Button
          onClick={handleCreateRoundtable}
          disabled={!isSetupComplete || createRoundtableMutation.isPending || addParticipantsMutation.isPending}
          size="lg"
        >
          {(createRoundtableMutation.isPending || addParticipantsMutation.isPending) && (
            <Loader2 className="h-4 w-4 mr-2 animate-spin" />
          )}
          <Play className="h-4 w-4 mr-2" />
          Create Roundtable
        </Button>
      </div>
    </div>
  );

  const renderDiscussionStep = () => (
    <div className="space-y-6">
      {!isDiscussionActive && !isDiscussionComplete && (
        <Alert>
          <AlertCircle className="h-4 w-4" />
          <AlertDescription className="flex items-center justify-between">
            <span>Ready to start the discussion?</span>
            <Button
              onClick={handleStartDiscussion}
              disabled={startDiscussionMutation.isPending}
              size="sm"
            >
              {startDiscussionMutation.isPending ? (
                <>
                  <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                  Starting...
                </>
              ) : (
                <>
                  <Play className="h-4 w-4 mr-2" />
                  Start Discussion
                </>
              )}
            </Button>
          </AlertDescription>
        </Alert>
      )}

      {isDiscussionComplete && (
        <Alert>
          <CheckCircle className="h-4 w-4" />
          <AlertDescription>
            Discussion completed. View insights or start a new roundtable.
          </AlertDescription>
        </Alert>
      )}

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <div className="lg:col-span-2 space-y-6">
          <LiveDiscussionView
            roundtableId={activeRoundtableId!}
            participants={selectedExperts}
            isActive={isDiscussionActive}
          />

          {isDiscussionActive && (
            <UserInterjectionInput roundtableId={activeRoundtableId!} />
          )}
        </div>

        <div className="space-y-6">
          <RoundtableStatus roundtableId={activeRoundtableId!} />
        </div>
      </div>

      <div className="flex justify-between">
        <Button variant="outline" onClick={() => setCurrentStep('setup')}>
          Back to Setup
        </Button>
        <Button
          onClick={() => setCurrentStep('insights')}
          disabled={!isDiscussionComplete}
        >
          View Insights
          <Lightbulb className="h-4 w-4 ml-2" />
        </Button>
      </div>
    </div>
  );

  const renderInsightsStep = () => (
    <div className="space-y-6">
      <InsightsExtractor roundtableId={activeRoundtableId!} />

      <div className="flex justify-between">
        <Button variant="outline" onClick={() => setCurrentStep('discussion')}>
          Back to Discussion
        </Button>
        <Button onClick={() => {
          setActiveRoundtableId(null);
          setCurrentStep('setup');
          setSelectedExperts([]);
        }}>
          New Roundtable
        </Button>
      </div>
    </div>
  );

  return (
    <div className="space-y-6">
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle className="flex items-center gap-2">
                <Users className="h-5 w-5" />
                Expert Roundtable
              </CardTitle>
              <CardDescription>
                AI-powered expert discussion to refine and validate your PRD ideas
              </CardDescription>
            </div>

            <Tabs value={currentStep} onValueChange={(v) => setCurrentStep(v as FlowStep)}>
              <TabsList>
                <TabsTrigger value="setup" disabled={!activeRoundtableId && currentStep !== 'setup'}>
                  <Users className="h-4 w-4 mr-2" />
                  Setup
                </TabsTrigger>
                <TabsTrigger value="discussion" disabled={!activeRoundtableId}>
                  <MessageSquare className="h-4 w-4 mr-2" />
                  Discussion
                </TabsTrigger>
                <TabsTrigger value="insights" disabled={!activeRoundtableId || !isDiscussionComplete}>
                  <Lightbulb className="h-4 w-4 mr-2" />
                  Insights
                </TabsTrigger>
              </TabsList>
            </Tabs>
          </div>
        </CardHeader>
      </Card>

      {existingRoundtables && existingRoundtables.length > 0 && !activeRoundtableId && (
        <Alert>
          <AlertCircle className="h-4 w-4" />
          <AlertDescription>
            You have {existingRoundtables.length} existing roundtable(s) for this session.
          </AlertDescription>
        </Alert>
      )}

      {currentStep === 'setup' && renderSetupStep()}
      {currentStep === 'discussion' && activeRoundtableId && renderDiscussionStep()}
      {currentStep === 'insights' && activeRoundtableId && renderInsightsStep()}
    </div>
  );
}
