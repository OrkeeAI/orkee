// ABOUTME: Overview section form for problem statement, target audience, and value proposition
// ABOUTME: Captures the core "why" and "who" of the project idea
import { useEffect } from 'react';
import { useForm } from 'react-hook-form';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Textarea } from '@/components/ui/textarea';
import { Label } from '@/components/ui/label';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Loader2, Save } from 'lucide-react';
import { useIdeateOverview, useSaveOverview } from '@/hooks/useIdeate';
import { toast } from 'sonner';

interface OverviewFormData {
  problem_statement: string;
  target_audience: string;
  value_proposition: string;
  one_line_pitch: string;
}

interface OverviewSectionProps {
  sessionId: string;
}

export function OverviewSection({ sessionId }: OverviewSectionProps) {
  const { data: overview, isLoading, error } = useIdeateOverview(sessionId);
  const saveMutation = useSaveOverview(sessionId);

  const {
    register,
    handleSubmit,
    reset,
    formState: { errors, isDirty },
  } = useForm<OverviewFormData>();

  // Load existing data
  useEffect(() => {
    if (overview) {
      reset({
        problem_statement: overview.problem_statement || '',
        target_audience: overview.target_audience || '',
        value_proposition: overview.value_proposition || '',
        one_line_pitch: overview.one_line_pitch || '',
      });
    }
  }, [overview, reset]);

  const onSubmit = async (data: OverviewFormData) => {
    try {
      await saveMutation.mutateAsync({
        session_id: sessionId,
        ...data,
        ai_generated: false,
      });
      toast.success('Overview saved successfully!');
    } catch (error) {
      toast.error('Failed to save overview');
      console.error(error);
    }
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-12">
        <Loader2 className="w-8 h-8 animate-spin text-muted-foreground" />
      </div>
    );
  }

  if (error) {
    return (
      <Alert variant="destructive">
        <AlertDescription>Failed to load overview section</AlertDescription>
      </Alert>
    );
  }

  return (
    <form onSubmit={handleSubmit(onSubmit)} className="space-y-6 max-w-3xl">
      <div className="space-y-2">
        <Label htmlFor="problem_statement">Problem Statement *</Label>
        <Textarea
          id="problem_statement"
          {...register('problem_statement', { required: 'Problem statement is required' })}
          placeholder="What problem does this project solve?"
          rows={4}
          className="resize-none"
        />
        {errors.problem_statement && (
          <p className="text-sm text-destructive">{errors.problem_statement.message}</p>
        )}
        <p className="text-sm text-muted-foreground">
          Describe the core problem or pain point your project addresses
        </p>
      </div>

      <div className="space-y-2">
        <Label htmlFor="target_audience">Target Audience *</Label>
        <Textarea
          id="target_audience"
          {...register('target_audience', { required: 'Target audience is required' })}
          placeholder="Who will use this product?"
          rows={3}
          className="resize-none"
        />
        {errors.target_audience && (
          <p className="text-sm text-destructive">{errors.target_audience.message}</p>
        )}
        <p className="text-sm text-muted-foreground">
          Define your primary users or customers
        </p>
      </div>

      <div className="space-y-2">
        <Label htmlFor="value_proposition">Value Proposition *</Label>
        <Textarea
          id="value_proposition"
          {...register('value_proposition', { required: 'Value proposition is required' })}
          placeholder="What value does this project deliver?"
          rows={4}
          className="resize-none"
        />
        {errors.value_proposition && (
          <p className="text-sm text-destructive">{errors.value_proposition.message}</p>
        )}
        <p className="text-sm text-muted-foreground">
          Explain the unique value and benefits your solution provides
        </p>
      </div>

      <div className="space-y-2">
        <Label htmlFor="one_line_pitch">One-Line Pitch</Label>
        <Input
          id="one_line_pitch"
          {...register('one_line_pitch')}
          placeholder="A concise tagline for your project"
          maxLength={120}
        />
        <p className="text-sm text-muted-foreground">
          Sum up your project in one compelling sentence
        </p>
      </div>

      <div className="flex justify-end pt-4">
        <Button
          type="submit"
          disabled={!isDirty || saveMutation.isPending}
        >
          {saveMutation.isPending ? (
            <>
              <Loader2 className="w-4 h-4 mr-2 animate-spin" />
              Saving...
            </>
          ) : (
            <>
              <Save className="w-4 h-4 mr-2" />
              Save Section
            </>
          )}
        </Button>
      </div>
    </form>
  );
}
