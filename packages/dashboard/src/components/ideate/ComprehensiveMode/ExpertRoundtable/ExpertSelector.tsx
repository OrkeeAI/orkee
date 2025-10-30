// ABOUTME: Expert selection interface with grid display, AI suggestions, and custom expert creation
// ABOUTME: Manages multi-select expert persona selection with visual feedback and validation

import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';
import { Badge } from '@/components/ui/badge';
import { Alert, AlertDescription } from '@/components/ui/alert';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Loader2, Plus, Sparkles, AlertCircle, Users } from 'lucide-react';
import { ExpertCard } from './ExpertCard';
import {
  useListExperts,
  useCreateExpert,
  useSuggestExperts,
} from '@/hooks/useIdeate';
import { toast } from 'sonner';
import type { ExpertPersona, CreateExpertPersonaInput } from '@/services/ideate';

interface ExpertSelectorProps {
  sessionId: string;
  selectedExperts: ExpertPersona[];
  onSelectionsChange: (experts: ExpertPersona[]) => void;
  minExperts?: number;
  maxExperts?: number;
  topic?: string;
}

export function ExpertSelector({
  sessionId,
  selectedExperts,
  onSelectionsChange,
  minExperts = 2,
  maxExperts = 10,
  topic,
}: ExpertSelectorProps) {
  const [createDialogOpen, setCreateDialogOpen] = useState(false);
  const [suggestDialogOpen, setSuggestDialogOpen] = useState(false);
  const [customExpert, setCustomExpert] = useState<CreateExpertPersonaInput>({
    name: '',
    role: '',
    expertise: [],
    system_prompt: '',
    bio: '',
  });
  const [expertiseInput, setExpertiseInput] = useState('');
  const [suggestionTopic, setSuggestionTopic] = useState(topic || '');

  const { data: experts, isLoading: expertsLoading } = useListExperts(sessionId);
  const createExpertMutation = useCreateExpert(sessionId);
  const suggestExpertsMutation = useSuggestExperts(sessionId);

  const handleSelectExpert = (expert: ExpertPersona) => {
    if (selectedExperts.length >= maxExperts) {
      toast.error(`Maximum ${maxExperts} experts allowed`);
      return;
    }

    if (!selectedExperts.find((e) => e.id === expert.id)) {
      onSelectionsChange([...selectedExperts, expert]);
    }
  };

  const handleDeselectExpert = (expert: ExpertPersona) => {
    onSelectionsChange(selectedExperts.filter((e) => e.id !== expert.id));
  };

  const handleCreateExpert = async () => {
    if (!customExpert.name.trim() || !customExpert.role.trim()) {
      toast.error('Please fill in name and role');
      return;
    }

    if (customExpert.expertise.length === 0) {
      toast.error('Please add at least one area of expertise');
      return;
    }

    try {
      const newExpert = await createExpertMutation.mutateAsync(customExpert);
      toast.success('Custom expert created successfully!');
      setCreateDialogOpen(false);
      setCustomExpert({
        name: '',
        role: '',
        expertise: [],
        system_prompt: '',
        bio: '',
      });
      setExpertiseInput('');

      // Auto-select the newly created expert
      if (selectedExperts.length < maxExperts) {
        handleSelectExpert(newExpert);
      }
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error';
      toast.error('Failed to create expert', { description: errorMessage });
    }
  };

  const handleAddExpertise = () => {
    const skill = expertiseInput.trim();
    if (skill && !customExpert.expertise.includes(skill)) {
      setCustomExpert({
        ...customExpert,
        expertise: [...customExpert.expertise, skill],
      });
      setExpertiseInput('');
    }
  };

  const handleRemoveExpertise = (skill: string) => {
    setCustomExpert({
      ...customExpert,
      expertise: customExpert.expertise.filter((s) => s !== skill),
    });
  };

  const handleSuggestExperts = async () => {
    if (!suggestionTopic.trim()) {
      toast.error('Please enter a topic for AI suggestions');
      return;
    }

    try {
      toast.info('Getting AI suggestions...', { duration: 2000 });
      const suggestions = await suggestExpertsMutation.mutateAsync({
        topic: suggestionTopic,
        num_suggestions: maxExperts - selectedExperts.length,
      });

      if (suggestions.length === 0) {
        toast.info('No additional experts suggested');
        return;
      }

      toast.success(`Suggested ${suggestions.length} experts`);

      // Auto-select suggested experts that aren't already selected
      const expertsToAdd: ExpertPersona[] = [];
      for (const suggestion of suggestions) {
        const expert = experts?.find((e) => e.name === suggestion.expert_name);
        if (expert && !selectedExperts.find((e) => e.id === expert.id)) {
          expertsToAdd.push(expert);
          if (selectedExperts.length + expertsToAdd.length >= maxExperts) break;
        }
      }

      if (expertsToAdd.length > 0) {
        onSelectionsChange([...selectedExperts, ...expertsToAdd]);
      }

      setSuggestDialogOpen(false);
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error';
      toast.error('Failed to get suggestions', { description: errorMessage });
    }
  };

  const isExpertSelected = (expertId: string) => {
    return selectedExperts.some((e) => e.id === expertId);
  };

  const canProceed = selectedExperts.length >= minExperts && selectedExperts.length <= maxExperts;

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-lg font-semibold flex items-center gap-2">
            <Users className="h-5 w-5" />
            Select Experts
          </h3>
          <p className="text-sm text-muted-foreground">
            Choose {minExperts}-{maxExperts} experts for the roundtable discussion
          </p>
        </div>
        <div className="flex items-center gap-2">
          <Badge variant={canProceed ? 'default' : 'secondary'}>
            {selectedExperts.length}/{maxExperts} selected
          </Badge>
          <Dialog open={suggestDialogOpen} onOpenChange={setSuggestDialogOpen}>
            <DialogTrigger asChild>
              <Button variant="outline" size="sm" disabled={selectedExperts.length >= maxExperts}>
                <Sparkles className="h-4 w-4 mr-2" />
                AI Suggest
              </Button>
            </DialogTrigger>
            <DialogContent>
              <DialogHeader>
                <DialogTitle>AI-Suggested Experts</DialogTitle>
                <DialogDescription>
                  Get AI recommendations for which experts would be most valuable for your topic
                </DialogDescription>
              </DialogHeader>
              <div className="space-y-4">
                <div>
                  <Label htmlFor="suggestion-topic">Topic</Label>
                  <Input
                    id="suggestion-topic"
                    placeholder="e.g., Building a mobile payment app"
                    value={suggestionTopic}
                    onChange={(e) => setSuggestionTopic(e.target.value)}
                  />
                </div>
              </div>
              <DialogFooter>
                <Button
                  onClick={handleSuggestExperts}
                  disabled={suggestExpertsMutation.isPending || !suggestionTopic.trim()}
                >
                  {suggestExpertsMutation.isPending && (
                    <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                  )}
                  Get Suggestions
                </Button>
              </DialogFooter>
            </DialogContent>
          </Dialog>
          <Dialog open={createDialogOpen} onOpenChange={setCreateDialogOpen}>
            <DialogTrigger asChild>
              <Button variant="outline" size="sm">
                <Plus className="h-4 w-4 mr-2" />
                Custom Expert
              </Button>
            </DialogTrigger>
            <DialogContent className="max-w-2xl">
              <DialogHeader>
                <DialogTitle>Create Custom Expert</DialogTitle>
                <DialogDescription>
                  Define a custom expert persona with specific knowledge and perspective
                </DialogDescription>
              </DialogHeader>
              <ScrollArea className="max-h-[60vh]">
                <div className="space-y-4 pr-4">
                  <div className="grid grid-cols-2 gap-4">
                    <div>
                      <Label htmlFor="expert-name">Name *</Label>
                      <Input
                        id="expert-name"
                        placeholder="e.g., Dr. Sarah Chen"
                        value={customExpert.name}
                        onChange={(e) => setCustomExpert({ ...customExpert, name: e.target.value })}
                      />
                    </div>
                    <div>
                      <Label htmlFor="expert-role">Role *</Label>
                      <Input
                        id="expert-role"
                        placeholder="e.g., AI Ethics Researcher"
                        value={customExpert.role}
                        onChange={(e) => setCustomExpert({ ...customExpert, role: e.target.value })}
                      />
                    </div>
                  </div>

                  <div>
                    <Label htmlFor="expertise-input">Areas of Expertise *</Label>
                    <div className="flex gap-2">
                      <Input
                        id="expertise-input"
                        placeholder="Add expertise area"
                        value={expertiseInput}
                        onChange={(e) => setExpertiseInput(e.target.value)}
                        onKeyDown={(e) => e.key === 'Enter' && (e.preventDefault(), handleAddExpertise())}
                      />
                      <Button type="button" onClick={handleAddExpertise} size="sm">
                        Add
                      </Button>
                    </div>
                    <div className="flex flex-wrap gap-1.5 mt-2">
                      {customExpert.expertise.map((skill) => (
                        <Badge
                          key={skill}
                          variant="secondary"
                          className="cursor-pointer"
                          onClick={() => handleRemoveExpertise(skill)}
                        >
                          {skill} ×
                        </Badge>
                      ))}
                    </div>
                  </div>

                  <div>
                    <Label htmlFor="expert-bio">Bio</Label>
                    <Textarea
                      id="expert-bio"
                      placeholder="Brief description of the expert's background and perspective"
                      rows={3}
                      value={customExpert.bio}
                      onChange={(e) => setCustomExpert({ ...customExpert, bio: e.target.value })}
                    />
                  </div>

                  <div>
                    <Label htmlFor="expert-prompt">System Prompt (Optional)</Label>
                    <Textarea
                      id="expert-prompt"
                      placeholder="Custom instructions for how this expert should behave in discussions"
                      rows={4}
                      value={customExpert.system_prompt}
                      onChange={(e) => setCustomExpert({ ...customExpert, system_prompt: e.target.value })}
                    />
                  </div>
                </div>
              </ScrollArea>
              <DialogFooter>
                <Button
                  onClick={handleCreateExpert}
                  disabled={createExpertMutation.isPending}
                >
                  {createExpertMutation.isPending && (
                    <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                  )}
                  Create Expert
                </Button>
              </DialogFooter>
            </DialogContent>
          </Dialog>
        </div>
      </div>

      {!canProceed && (
        <Alert>
          <AlertCircle className="h-4 w-4" />
          <AlertDescription>
            {selectedExperts.length < minExperts
              ? `Select at least ${minExperts} experts to continue`
              : `Maximum ${maxExperts} experts allowed`}
          </AlertDescription>
        </Alert>
      )}

      {expertsLoading ? (
        <div className="flex items-center justify-center p-12">
          <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {experts?.map((expert) => (
            <ExpertCard
              key={expert.id}
              expert={expert}
              isSelected={isExpertSelected(expert.id)}
              onSelect={handleSelectExpert}
              onDeselect={handleDeselectExpert}
              showActions
            />
          ))}
        </div>
      )}
    </div>
  );
}
