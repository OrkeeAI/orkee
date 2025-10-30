// ABOUTME: Main orchestrator for Comprehensive Mode with advanced research tools
// ABOUTME: Extends Guided Mode with competitor analysis and similar project research

import { useState, useEffect } from 'react';
import { ArrowLeft, FileText } from 'lucide-react';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { SectionNavigator } from '../GuidedMode/SectionNavigator';
import { SectionProgress } from '../GuidedMode/SectionProgress';
import { SkipDialog } from '../GuidedMode/SkipDialog';
import { OverviewSection } from '../GuidedMode/sections/OverviewSection';
import { UXSection } from '../GuidedMode/sections/UXSection';
import { TechnicalSection } from '../GuidedMode/sections/TechnicalSection';
import { RoadmapSection } from '../GuidedMode/sections/RoadmapSection';
import { DependencyChainSection } from '../GuidedMode/sections/DependencyChainSection';
import { RisksSection } from '../GuidedMode/sections/RisksSection';
import { CompetitorAnalysisSection } from './sections/CompetitorAnalysisSection';
import { SimilarProjectsSection } from './sections/SimilarProjectsSection';
import { ExpertRoundtableFlow } from './ExpertRoundtable';
import { PRDGeneratorFlow } from '../PRDGenerator/PRDGeneratorFlow';
import {
  useIdeateSession,
  useIdeateStatus,
  useNavigateToSection,
} from '@/hooks/useIdeate';
import { toast } from 'sonner';

export type SectionName =
  | 'overview'
  | 'ux'
  | 'technical'
  | 'roadmap'
  | 'dependencies'
  | 'risks'
  | 'research';

// eslint-disable-next-line react-refresh/only-export-components
export const SECTIONS: Array<{ id: SectionName; label: string; description: string }> = [
  { id: 'overview', label: 'Overview', description: 'Problem, audience, and value proposition' },
  { id: 'ux', label: 'User Experience', description: 'Personas, user flows, and UI principles' },
  { id: 'technical', label: 'Technical Architecture', description: 'Components, data models, and infrastructure' },
  { id: 'roadmap', label: 'Roadmap', description: 'MVP scope and future phases' },
  { id: 'dependencies', label: 'Dependency Chain', description: 'Feature dependencies and build phases' },
  { id: 'risks', label: 'Risks & Mitigations', description: 'Technical, scoping, and resource risks' },
  { id: 'research', label: 'Research & Analysis', description: 'Competitor and similar project research' },
];

interface ComprehensiveModeFlowProps {
  projectId: string;
  sessionId: string;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onComplete?: (prdId: string) => void;
}

export function ComprehensiveModeFlow({
  projectId: _projectId, // eslint-disable-line @typescript-eslint/no-unused-vars
  sessionId,
  open,
  onOpenChange,
  onComplete: _onComplete, // eslint-disable-line @typescript-eslint/no-unused-vars
}: ComprehensiveModeFlowProps) {
  const [skipDialogOpen, setSkipDialogOpen] = useState(false);
  const [currentSection, setCurrentSection] = useState<SectionName>('overview');
  const [researchTab, setResearchTab] = useState<'competitors' | 'similar-projects' | 'expert-roundtable'>('competitors');
  const [showPRDGenerator, setShowPRDGenerator] = useState(false);

  const { data: session } = useIdeateSession(sessionId);
  const { data: status } = useIdeateStatus(sessionId);
  const navigateMutation = useNavigateToSection(sessionId);

  // Initialize current section from session, or show PRDGeneratorFlow for completed sessions
  useEffect(() => {
    if (session?.current_section && SECTIONS.some(s => s.id === session.current_section)) {
      setCurrentSection(session.current_section as SectionName);
    }
    // For completed sessions, automatically show the PRD generator
    if (session?.status === 'completed') {
      setShowPRDGenerator(true);
    }
  }, [session]);

  const handleSectionSelect = async (sectionId: SectionName) => {
    try {
      await navigateMutation.mutateAsync(sectionId);
      setCurrentSection(sectionId);
    } catch (error) {
      toast.error('Failed to navigate to section');
      console.error(error);
    }
  };

  const handleNext = async () => {
    const currentIndex = SECTIONS.findIndex(s => s.id === currentSection);
    if (currentIndex < SECTIONS.length - 1) {
      const nextSection = SECTIONS[currentIndex + 1].id;
      await handleSectionSelect(nextSection);
    }
  };

  const handlePrevious = async () => {
    const currentIndex = SECTIONS.findIndex(s => s.id === currentSection);
    if (currentIndex > 0) {
      const prevSection = SECTIONS[currentIndex - 1].id;
      await handleSectionSelect(prevSection);
    }
  };

  const handleSkip = () => {
    setSkipDialogOpen(true);
  };

  const handleSkipConfirm = async () => {
    await handleNext();
    setSkipDialogOpen(false);
  };

  const handleGeneratePRD = () => {
    setShowPRDGenerator(true);
  };

  const handleBackToSections = () => {
    setShowPRDGenerator(false);
  };

  const renderSection = () => {
    switch (currentSection) {
      case 'overview':
        return <OverviewSection sessionId={sessionId} />;
      case 'ux':
        return <UXSection sessionId={sessionId} />;
      case 'technical':
        return <TechnicalSection sessionId={sessionId} />;
      case 'roadmap':
        return <RoadmapSection sessionId={sessionId} />;
      case 'dependencies':
        return <DependencyChainSection sessionId={sessionId} />;
      case 'risks':
        return <RisksSection sessionId={sessionId} />;
      case 'research':
        return (
          <Tabs value={researchTab} onValueChange={(v) => setResearchTab(v as 'competitors' | 'similar-projects' | 'expert-roundtable')}>
            <TabsList className="grid w-full grid-cols-3">
              <TabsTrigger value="competitors">Competitor Analysis</TabsTrigger>
              <TabsTrigger value="similar-projects">Similar Projects</TabsTrigger>
              <TabsTrigger value="expert-roundtable">Expert Roundtable</TabsTrigger>
            </TabsList>
            <TabsContent value="competitors" className="mt-4">
              <CompetitorAnalysisSection sessionId={sessionId} />
            </TabsContent>
            <TabsContent value="similar-projects" className="mt-4">
              <SimilarProjectsSection sessionId={sessionId} />
            </TabsContent>
            <TabsContent value="expert-roundtable" className="mt-4">
              <ExpertRoundtableFlow sessionId={sessionId} defaultTopic={session?.initial_description} />
            </TabsContent>
          </Tabs>
        );
      default:
        return null;
    }
  };

  const currentIndex = SECTIONS.findIndex(s => s.id === currentSection);
  const isFirstSection = currentIndex === 0;
  const isLastSection = currentIndex === SECTIONS.length - 1;

  // If showing PRD generator, render it in a simple layout
  if (showPRDGenerator && session) {
    return (
      <Dialog open={open} onOpenChange={onOpenChange}>
        <DialogContent className="max-w-7xl h-[90vh] flex flex-col">
          <DialogHeader>
            <Button
              variant="ghost"
              size="sm"
              onClick={handleBackToSections}
              className="gap-2 w-fit"
            >
              <ArrowLeft className="h-4 w-4" />
              Back to Sections
            </Button>
          </DialogHeader>

          <div className="flex-1 overflow-y-auto">
            <PRDGeneratorFlow session={session} />
          </div>
        </DialogContent>
      </Dialog>
    );
  }

  return (
    <>
      <Dialog open={open} onOpenChange={onOpenChange}>
        <DialogContent
          className="max-w-7xl h-[90vh] flex flex-col"
          aria-describedby="comprehensive-mode-description"
        >
          <DialogHeader>
            <DialogTitle>Comprehensive Mode - {SECTIONS[currentIndex].label}</DialogTitle>
            <DialogDescription id="comprehensive-mode-description">
              {SECTIONS[currentIndex].description}
            </DialogDescription>
          </DialogHeader>

          <div className="flex-1 flex gap-4 overflow-hidden">
            {/* Left sidebar - Section navigator */}
            <div className="w-64 flex-shrink-0 space-y-4 overflow-y-auto">
              <SectionProgress
                sections={SECTIONS}
                currentSection={currentSection}
                completedSections={status?.completed_sections ?? 0}
                totalSections={status?.total_sections ?? SECTIONS.length}
              />
              <SectionNavigator
                sections={SECTIONS}
                currentSection={currentSection}
                onSectionSelect={handleSectionSelect}
                completionStatus={status}
              />
            </div>

            {/* Main content area */}
            <div className="flex-1 overflow-y-auto pr-2">{renderSection()}</div>
          </div>

          {/* Footer navigation */}
          <div className="flex items-center justify-between pt-4 border-t">
            <Button
              variant="outline"
              onClick={handlePrevious}
              disabled={isFirstSection || navigateMutation.isPending}
            >
              <ArrowLeft className="mr-2 h-4 w-4" />
              Previous
            </Button>

            <div className="flex gap-2">
              <Button variant="outline" onClick={handleSkip}>
                Skip Section
              </Button>

              {!isLastSection && (
                <Button onClick={handleNext} disabled={navigateMutation.isPending}>
                  Next Section
                </Button>
              )}

              {isLastSection && status?.is_ready_for_prd && (
                <Button onClick={handleGeneratePRD}>
                  <FileText className="mr-2 h-4 w-4" />
                  Generate PRD
                </Button>
              )}
            </div>
          </div>
        </DialogContent>
      </Dialog>

      <SkipDialog
        open={skipDialogOpen}
        onOpenChange={setSkipDialogOpen}
        onConfirm={handleSkipConfirm}
        sectionName={SECTIONS[currentIndex].label}
      />
    </>
  );
}
