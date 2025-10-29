// ABOUTME: Main orchestrator for Guided Mode step-by-step PRD creation
// ABOUTME: Manages navigation through 7 sections with progress tracking and skip functionality
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
import { SectionNavigator } from './SectionNavigator';
import { SectionProgress } from './SectionProgress';
import { SkipDialog } from './SkipDialog';
import { OverviewSection } from './sections/OverviewSection';
import { UXSection } from './sections/UXSection';
import { TechnicalSection } from './sections/TechnicalSection';
import { RoadmapSection } from './sections/RoadmapSection';
import { DependencyChainSection } from './sections/DependencyChainSection';
import { RisksSection } from './sections/RisksSection';
import { AppendixSection } from './sections/AppendixSection';
import { PRDGeneratorFlow } from '../PRDGenerator/PRDGeneratorFlow';
import {
  useIdeateSession,
  useIdeateStatus,
  useNavigateToSection,
} from '@/hooks/useIdeate';
import { toast } from 'sonner';

// Note: Features section not yet implemented in backend
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
  { id: 'research', label: 'Research & References', description: 'Competitors, similar projects, and resources' },
];

interface GuidedModeFlowProps {
  projectId: string;
  sessionId: string;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onComplete?: (prdId: string) => void;
}

export function GuidedModeFlow({
  projectId: _projectId, // eslint-disable-line @typescript-eslint/no-unused-vars
  sessionId,
  open,
  onOpenChange,
  onComplete: _onComplete, // eslint-disable-line @typescript-eslint/no-unused-vars
}: GuidedModeFlowProps) {
  const [skipDialogOpen, setSkipDialogOpen] = useState(false);
  const [currentSection, setCurrentSection] = useState<SectionName>('overview');
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
    // Skip logic will be implemented in SkipDialog component
    // For now, just move to next section
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
        return <AppendixSection sessionId={sessionId} />;
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
        <DialogContent className="max-w-7xl h-[90vh] p-0">
          <div className="flex flex-col h-full">
            {/* Header with back button */}
            <div className="border-b p-4">
              <Button
                variant="ghost"
                size="sm"
                onClick={handleBackToSections}
                className="gap-2"
              >
                <ArrowLeft className="h-4 w-4" />
                Back to Sections
              </Button>
            </div>

            {/* PRD Generator */}
            <div className="flex-1 overflow-y-auto p-6">
              <PRDGeneratorFlow session={session} />
            </div>
          </div>
        </DialogContent>
      </Dialog>
    );
  }

  return (
    <>
      <Dialog open={open} onOpenChange={onOpenChange}>
        <DialogContent className="max-w-7xl h-[90vh] p-0">
          <div className="flex h-full">
            {/* Sidebar Navigation */}
            <div className="w-80 border-r bg-muted/10 p-6 flex flex-col">
              <DialogHeader className="mb-6">
                <DialogTitle className="text-2xl">Guided Mode</DialogTitle>
                <DialogDescription>
                  Build your PRD step-by-step
                </DialogDescription>
              </DialogHeader>

              <SectionProgress
                sections={SECTIONS}
                currentSection={currentSection}
                completedSections={status?.completed_sections || 0}
                totalSections={status?.total_sections || 7}
              />

              <SectionNavigator
                sections={SECTIONS}
                currentSection={currentSection}
                onSectionSelect={handleSectionSelect}
                completionStatus={status}
              />

              <div className="mt-auto pt-6 space-y-2">
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => onOpenChange(false)}
                  className="w-full"
                >
                  <ArrowLeft className="w-4 h-4 mr-2" />
                  Save & Exit
                </Button>

                {status?.is_ready_for_prd && (
                  <Button
                    onClick={handleGeneratePRD}
                    className="w-full"
                  >
                    <FileText className="w-4 h-4 mr-2" />
                    Generate PRD
                  </Button>
                )}
              </div>
            </div>

            {/* Main Content */}
            <div className="flex-1 flex flex-col">
              {/* Section Header */}
              <div className="border-b p-6">
                <h2 className="text-2xl font-semibold">
                  {SECTIONS[currentIndex].label}
                </h2>
                <p className="text-muted-foreground mt-1">
                  {SECTIONS[currentIndex].description}
                </p>
              </div>

              {/* Section Content */}
              <div className="flex-1 overflow-y-auto p-6">
                {renderSection()}
              </div>

              {/* Navigation Footer */}
              <div className="border-t p-6 flex justify-between items-center">
                <Button
                  variant="outline"
                  onClick={handlePrevious}
                  disabled={isFirstSection || navigateMutation.isPending}
                >
                  <ArrowLeft className="w-4 h-4 mr-2" />
                  Previous
                </Button>

                <div className="flex gap-2">
                  <Button
                    variant="ghost"
                    onClick={handleSkip}
                    disabled={navigateMutation.isPending}
                  >
                    Skip Section
                  </Button>

                  <Button
                    onClick={handleNext}
                    disabled={isLastSection || navigateMutation.isPending}
                  >
                    Next
                    <ArrowLeft className="w-4 h-4 ml-2 rotate-180" />
                  </Button>
                </div>
              </div>
            </div>
          </div>
        </DialogContent>
      </Dialog>

      <SkipDialog
        open={skipDialogOpen}
        onOpenChange={setSkipDialogOpen}
        sectionName={SECTIONS[currentIndex].label}
        onConfirm={handleSkipConfirm}
      />
    </>
  );
}
