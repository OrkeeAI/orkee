// ABOUTME: Multi-step wizard for building spec capabilities from PRDs, manual input, or tasks
// ABOUTME: Guides users through capability definition, requirements, scenarios, and validation
import React, { useState } from 'react';
import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';
import { Progress } from '@/components/ui/progress';
import { Badge } from '@/components/ui/badge';
import { useCreateSpec, useValidateSpec } from '@/hooks/useSpecs';
import { usePRDs } from '@/hooks/usePRDs';
import type { SpecRequirement, SpecScenario, SpecCapabilityCreateInput } from '@/services/specs';
import type { PRD } from '@/services/prds';
import {
  FileText,
  PenTool,
  ListTodo,
  ChevronRight,
  ChevronLeft,
  Check,
  Plus,
  Trash2,
  AlertCircle,
} from 'lucide-react';
import { cn } from '@/lib/utils';

type WizardMode = 'prd-driven' | 'manual' | 'task-driven';

interface SpecBuilderWizardProps {
  projectId: string;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onComplete: (specId: string) => void;
  mode?: WizardMode;
  sourcePRD?: PRD;
}

type Step = 'mode' | 'capability' | 'requirements' | 'validation';

const STEPS: Step[] = ['mode', 'capability', 'requirements', 'validation'];

export function SpecBuilderWizard({
  projectId,
  open,
  onOpenChange,
  onComplete,
  mode: initialMode,
  sourcePRD,
}: SpecBuilderWizardProps) {
  const [currentStep, setCurrentStep] = useState<Step>('mode');
  const [mode, setMode] = useState<WizardMode>(initialMode || 'manual');
  const [selectedPRD, setSelectedPRD] = useState<PRD | null>(sourcePRD || null);

  // Capability data
  const [capabilityName, setCapabilityName] = useState('');
  const [purpose, setPurpose] = useState('');
  const [requirements, setRequirements] = useState<SpecRequirement[]>([]);

  const { data: prds } = usePRDs(projectId);
  const createSpecMutation = useCreateSpec(projectId);
  const validateSpecMutation = useValidateSpec(projectId);
  const { isPending: isSaving, error: saveError } = createSpecMutation;

  const stepIndex = STEPS.indexOf(currentStep);
  const progress = ((stepIndex + 1) / STEPS.length) * 100;

  const handleNext = () => {
    const currentIndex = STEPS.indexOf(currentStep);
    if (currentIndex < STEPS.length - 1) {
      setCurrentStep(STEPS[currentIndex + 1]);
    }
  };

  const handleBack = () => {
    const currentIndex = STEPS.indexOf(currentStep);
    if (currentIndex > 0) {
      setCurrentStep(STEPS[currentIndex - 1]);
    }
  };

  const handleSave = async () => {
    if (!capabilityName.trim() || !purpose.trim() || requirements.length === 0) {
      return;
    }

    const specData: SpecCapabilityCreateInput = {
      prdId: selectedPRD?.id,
      name: capabilityName,
      purpose,
      requirements,
      status: 'active',
    };

    try {
      const spec = await createSpecMutation.mutateAsync(specData);
      onComplete(spec.id);
      onOpenChange(false);
      resetForm();
    } catch {
      // Error handled by React Query mutation
    }
  };

  const resetForm = () => {
    setCurrentStep('mode');
    setMode(initialMode || 'manual');
    setSelectedPRD(sourcePRD || null);
    setCapabilityName('');
    setPurpose('');
    setRequirements([]);
    createSpecMutation.reset();
  };

  const addRequirement = () => {
    const newReq: SpecRequirement = {
      name: '',
      content: '',
      scenarios: [],
      position: requirements.length,
    };
    setRequirements([...requirements, newReq]);
  };

  const updateRequirement = (index: number, updates: Partial<SpecRequirement>) => {
    const updated = [...requirements];
    updated[index] = { ...updated[index], ...updates };
    setRequirements(updated);
  };

  const deleteRequirement = (index: number) => {
    setRequirements(requirements.filter((_, i) => i !== index));
  };

  const addScenario = (reqIndex: number) => {
    const updated = [...requirements];
    const newScenario: SpecScenario = {
      name: '',
      whenClause: '',
      thenClause: '',
      andClauses: [],
      position: updated[reqIndex].scenarios.length,
    };
    updated[reqIndex].scenarios = [...updated[reqIndex].scenarios, newScenario];
    setRequirements(updated);
  };

  const updateScenario = (
    reqIndex: number,
    scenarioIndex: number,
    updates: Partial<SpecScenario>
  ) => {
    const updated = [...requirements];
    updated[reqIndex].scenarios[scenarioIndex] = {
      ...updated[reqIndex].scenarios[scenarioIndex],
      ...updates,
    };
    setRequirements(updated);
  };

  const deleteScenario = (reqIndex: number, scenarioIndex: number) => {
    const updated = [...requirements];
    updated[reqIndex].scenarios = updated[reqIndex].scenarios.filter(
      (_, i) => i !== scenarioIndex
    );
    setRequirements(updated);
  };

  const canProceed = () => {
    switch (currentStep) {
      case 'mode':
        return mode === 'manual' || (mode === 'prd-driven' && selectedPRD !== null);
      case 'capability':
        return capabilityName.trim().length > 0 && purpose.trim().length > 0;
      case 'requirements':
        return (
          requirements.length > 0 &&
          requirements.every(
            (r) => r.name.trim() && r.content.trim() && r.scenarios.length > 0
          )
        );
      case 'validation':
        return true;
      default:
        return false;
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent
        className="sm:max-w-[800px] max-h-[90vh]"
        aria-describedby="spec-builder-description"
      >
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <PenTool className="h-5 w-5" />
            Build Spec Capability
          </DialogTitle>
          <DialogDescription id="spec-builder-description">
            Create a new spec capability with requirements and scenarios
          </DialogDescription>
          <Progress value={progress} className="mt-4" />
          <div className="flex justify-between text-xs text-muted-foreground mt-2">
            {STEPS.map((step, idx) => (
              <span
                key={step}
                className={cn(
                  'capitalize',
                  idx <= stepIndex ? 'text-primary font-medium' : ''
                )}
              >
                {step}
              </span>
            ))}
          </div>
        </DialogHeader>

        <div className="py-6 overflow-y-auto max-h-[60vh]">
          {currentStep === 'mode' && (
            <ModeSelectionStep
              mode={mode}
              setMode={setMode}
              selectedPRD={selectedPRD}
              setSelectedPRD={setSelectedPRD}
              prds={prds || []}
            />
          )}

          {currentStep === 'capability' && (
            <CapabilityDefinitionStep
              name={capabilityName}
              setName={setCapabilityName}
              purpose={purpose}
              setPurpose={setPurpose}
            />
          )}

          {currentStep === 'requirements' && (
            <RequirementsEditorStep
              requirements={requirements}
              onAddRequirement={addRequirement}
              onUpdateRequirement={updateRequirement}
              onDeleteRequirement={deleteRequirement}
              onAddScenario={addScenario}
              onUpdateScenario={updateScenario}
              onDeleteScenario={deleteScenario}
            />
          )}

          {currentStep === 'validation' && (
            <ValidationStep
              capabilityName={capabilityName}
              purpose={purpose}
              requirements={requirements}
            />
          )}
        </div>

        {saveError && (
          <div className="text-sm text-red-600 bg-red-50 dark:bg-red-950 p-3 rounded-md">
            {saveError.message || 'Failed to save spec'}
          </div>
        )}

        <DialogFooter className="flex justify-between items-center">
          <div>
            {stepIndex > 0 && (
              <Button variant="outline" onClick={handleBack} disabled={isSaving}>
                <ChevronLeft className="h-4 w-4 mr-1" />
                Back
              </Button>
            )}
          </div>
          <div className="flex gap-2">
            <Button variant="outline" onClick={() => onOpenChange(false)} disabled={isSaving}>
              Cancel
            </Button>
            {stepIndex < STEPS.length - 1 ? (
              <Button onClick={handleNext} disabled={!canProceed()}>
                Next
                <ChevronRight className="h-4 w-4 ml-1" />
              </Button>
            ) : (
              <Button onClick={handleSave} disabled={!canProceed() || isSaving}>
                {isSaving ? 'Saving...' : 'Create Spec'}
                <Check className="h-4 w-4 ml-1" />
              </Button>
            )}
          </div>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

function ModeSelectionStep({
  mode,
  setMode,
  selectedPRD,
  setSelectedPRD,
  prds,
}: {
  mode: WizardMode;
  setMode: (mode: WizardMode) => void;
  selectedPRD: PRD | null;
  setSelectedPRD: (prd: PRD | null) => void;
  prds: PRD[];
}) {
  const modes: Array<{ id: WizardMode; label: string; description: string; icon: any }> = [
    {
      id: 'prd-driven',
      label: 'From PRD',
      description: 'Extract capabilities from an existing PRD',
      icon: FileText,
    },
    {
      id: 'manual',
      label: 'Manual',
      description: 'Create spec from scratch',
      icon: PenTool,
    },
    {
      id: 'task-driven',
      label: 'From Tasks',
      description: 'Build spec from orphan tasks',
      icon: ListTodo,
    },
  ];

  return (
    <div className="space-y-6">
      <div>
        <h3 className="text-sm font-medium mb-3">How do you want to create this spec?</h3>
        <div className="grid gap-3">
          {modes.map((m) => (
            <button
              key={m.id}
              onClick={() => setMode(m.id)}
              className={cn(
                'flex items-start gap-3 p-4 rounded-lg border-2 text-left transition-colors',
                mode === m.id
                  ? 'border-primary bg-primary/5'
                  : 'border-border hover:border-primary/50'
              )}
            >
              <m.icon
                className={cn('h-5 w-5 mt-0.5', mode === m.id ? 'text-primary' : 'text-muted-foreground')}
              />
              <div className="flex-1">
                <div className="font-medium">{m.label}</div>
                <div className="text-sm text-muted-foreground">{m.description}</div>
              </div>
              {mode === m.id && <Check className="h-5 w-5 text-primary" />}
            </button>
          ))}
        </div>
      </div>

      {mode === 'prd-driven' && (
        <div className="space-y-2">
          <Label>Select PRD</Label>
          {prds.length === 0 ? (
            <div className="text-sm text-muted-foreground p-4 border rounded-md">
              No PRDs available. Upload a PRD first.
            </div>
          ) : (
            <div className="grid gap-2 max-h-48 overflow-y-auto">
              {prds.map((prd) => (
                <button
                  key={prd.id}
                  onClick={() => setSelectedPRD(prd)}
                  className={cn(
                    'flex items-start justify-between p-3 rounded-md border text-left transition-colors',
                    selectedPRD?.id === prd.id
                      ? 'border-primary bg-primary/5'
                      : 'border-border hover:border-primary/50'
                  )}
                >
                  <div>
                    <div className="font-medium text-sm">{prd.title}</div>
                    <div className="text-xs text-muted-foreground">Version {prd.version}</div>
                  </div>
                  {selectedPRD?.id === prd.id && <Check className="h-4 w-4 text-primary" />}
                </button>
              ))}
            </div>
          )}
        </div>
      )}

      {mode === 'task-driven' && (
        <div className="p-4 border rounded-md bg-muted/30">
          <div className="flex items-start gap-2">
            <AlertCircle className="h-5 w-5 text-muted-foreground mt-0.5" />
            <div className="text-sm text-muted-foreground">
              Task-driven mode will analyze your orphan tasks and suggest spec requirements.
              This feature requires the task-spec integration to be complete.
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

function CapabilityDefinitionStep({
  name,
  setName,
  purpose,
  setPurpose,
}: {
  name: string;
  setName: (name: string) => void;
  purpose: string;
  setPurpose: (purpose: string) => void;
}) {
  return (
    <div className="space-y-4">
      <div className="space-y-2">
        <Label htmlFor="capability-name">Capability Name *</Label>
        <Input
          id="capability-name"
          placeholder="e.g., User Authentication"
          value={name}
          onChange={(e) => setName(e.target.value)}
        />
        <p className="text-xs text-muted-foreground">
          A clear, concise name for this capability
        </p>
      </div>

      <div className="space-y-2">
        <Label htmlFor="capability-purpose">Purpose *</Label>
        <Textarea
          id="capability-purpose"
          placeholder="Describe the purpose and context of this capability..."
          value={purpose}
          onChange={(e) => setPurpose(e.target.value)}
          rows={6}
        />
        <p className="text-xs text-muted-foreground">
          Explain why this capability exists and what problem it solves
        </p>
      </div>
    </div>
  );
}

function RequirementsEditorStep({
  requirements,
  onAddRequirement,
  onUpdateRequirement,
  onDeleteRequirement,
  onAddScenario,
  onUpdateScenario,
  onDeleteScenario,
}: {
  requirements: SpecRequirement[];
  onAddRequirement: () => void;
  onUpdateRequirement: (index: number, updates: Partial<SpecRequirement>) => void;
  onDeleteRequirement: (index: number) => void;
  onAddScenario: (reqIndex: number) => void;
  onUpdateScenario: (reqIndex: number, scenarioIndex: number, updates: Partial<SpecScenario>) => void;
  onDeleteScenario: (reqIndex: number, scenarioIndex: number) => void;
}) {
  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h3 className="text-sm font-medium">Requirements</h3>
        <Button size="sm" onClick={onAddRequirement} variant="outline">
          <Plus className="h-4 w-4 mr-1" />
          Add Requirement
        </Button>
      </div>

      {requirements.length === 0 ? (
        <div className="text-center p-8 border-2 border-dashed rounded-lg">
          <p className="text-sm text-muted-foreground">
            No requirements yet. Click "Add Requirement" to get started.
          </p>
        </div>
      ) : (
        <div className="space-y-4">
          {requirements.map((req, reqIdx) => (
            <RequirementEditor
              key={reqIdx}
              requirement={req}
              index={reqIdx}
              onUpdate={(updates) => onUpdateRequirement(reqIdx, updates)}
              onDelete={() => onDeleteRequirement(reqIdx)}
              onAddScenario={() => onAddScenario(reqIdx)}
              onUpdateScenario={(scenarioIdx, updates) =>
                onUpdateScenario(reqIdx, scenarioIdx, updates)
              }
              onDeleteScenario={(scenarioIdx) => onDeleteScenario(reqIdx, scenarioIdx)}
            />
          ))}
        </div>
      )}
    </div>
  );
}

function RequirementEditor({
  requirement,
  index,
  onUpdate,
  onDelete,
  onAddScenario,
  onUpdateScenario,
  onDeleteScenario,
}: {
  requirement: SpecRequirement;
  index: number;
  onUpdate: (updates: Partial<SpecRequirement>) => void;
  onDelete: () => void;
  onAddScenario: () => void;
  onUpdateScenario: (scenarioIndex: number, updates: Partial<SpecScenario>) => void;
  onDeleteScenario: (scenarioIndex: number) => void;
}) {
  return (
    <div className="border rounded-lg p-4 space-y-3">
      <div className="flex items-start justify-between gap-2">
        <div className="flex-1 space-y-3">
          <div>
            <Label>Requirement Name *</Label>
            <Input
              placeholder="e.g., Password Authentication"
              value={requirement.name}
              onChange={(e) => onUpdate({ name: e.target.value })}
              className="mt-1"
            />
          </div>
          <div>
            <Label>Description *</Label>
            <Textarea
              placeholder="Describe this requirement..."
              value={requirement.content}
              onChange={(e) => onUpdate({ content: e.target.value })}
              rows={3}
              className="mt-1"
            />
          </div>
        </div>
        <Button size="sm" variant="ghost" onClick={onDelete}>
          <Trash2 className="h-4 w-4 text-destructive" />
        </Button>
      </div>

      <div className="space-y-2">
        <div className="flex items-center justify-between">
          <Label className="text-xs">Scenarios (WHEN/THEN)</Label>
          <Button size="sm" variant="outline" onClick={onAddScenario}>
            <Plus className="h-3 w-3 mr-1" />
            Add Scenario
          </Button>
        </div>

        {requirement.scenarios.length === 0 ? (
          <div className="text-xs text-muted-foreground p-2 border-2 border-dashed rounded">
            No scenarios. Add at least one scenario.
          </div>
        ) : (
          <div className="space-y-2">
            {requirement.scenarios.map((scenario, sIdx) => (
              <div key={sIdx} className="border rounded p-3 space-y-2 bg-muted/20">
                <div className="flex items-start gap-2">
                  <div className="flex-1 space-y-2">
                    <Input
                      placeholder="Scenario name"
                      value={scenario.name}
                      onChange={(e) => onUpdateScenario(sIdx, { name: e.target.value })}
                      className="h-8 text-sm"
                    />
                    <div className="grid grid-cols-2 gap-2">
                      <Input
                        placeholder="WHEN..."
                        value={scenario.whenClause}
                        onChange={(e) => onUpdateScenario(sIdx, { whenClause: e.target.value })}
                        className="h-8 text-sm"
                      />
                      <Input
                        placeholder="THEN..."
                        value={scenario.thenClause}
                        onChange={(e) => onUpdateScenario(sIdx, { thenClause: e.target.value })}
                        className="h-8 text-sm"
                      />
                    </div>
                  </div>
                  <Button size="sm" variant="ghost" onClick={() => onDeleteScenario(sIdx)}>
                    <Trash2 className="h-3 w-3 text-destructive" />
                  </Button>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

function ValidationStep({
  capabilityName,
  purpose,
  requirements,
}: {
  capabilityName: string;
  purpose: string;
  requirements: SpecRequirement[];
}) {
  const totalScenarios = requirements.reduce((sum, r) => sum + r.scenarios.length, 0);

  return (
    <div className="space-y-6">
      <div className="flex items-start gap-3 p-4 bg-green-50 dark:bg-green-950 border border-green-200 dark:border-green-900 rounded-lg">
        <Check className="h-5 w-5 text-green-600 dark:text-green-400 mt-0.5" />
        <div>
          <h3 className="font-medium text-green-900 dark:text-green-100">Ready to Create</h3>
          <p className="text-sm text-green-700 dark:text-green-300 mt-1">
            Your spec capability is ready to be created
          </p>
        </div>
      </div>

      <div className="space-y-4">
        <div>
          <Label className="text-xs text-muted-foreground">Capability</Label>
          <div className="mt-1 font-medium">{capabilityName}</div>
        </div>

        <div>
          <Label className="text-xs text-muted-foreground">Purpose</Label>
          <div className="mt-1 text-sm">{purpose}</div>
        </div>

        <div className="grid grid-cols-3 gap-4">
          <div className="p-3 border rounded-lg">
            <div className="text-2xl font-bold">{requirements.length}</div>
            <div className="text-xs text-muted-foreground">Requirements</div>
          </div>
          <div className="p-3 border rounded-lg">
            <div className="text-2xl font-bold">{totalScenarios}</div>
            <div className="text-xs text-muted-foreground">Scenarios</div>
          </div>
          <div className="p-3 border rounded-lg">
            <div className="text-2xl font-bold">
              {requirements.every((r) => r.scenarios.length > 0) ? '✓' : '!'}
            </div>
            <div className="text-xs text-muted-foreground">Validation</div>
          </div>
        </div>
      </div>
    </div>
  );
}
