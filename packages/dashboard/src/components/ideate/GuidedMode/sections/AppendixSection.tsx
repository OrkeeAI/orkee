// ABOUTME: Research and references section (appendix)
// ABOUTME: Stub for competitors, similar projects, and references
import { useEffect } from 'react';
import { useForm } from 'react-hook-form';
import { Button } from '@/components/ui/button';
import { Textarea } from '@/components/ui/textarea';
import { Label } from '@/components/ui/label';
import { Loader2 } from 'lucide-react';
import { useIdeateResearch, useSaveResearch } from '@/hooks/useIdeate';
import { toast } from 'sonner';

export function AppendixSection({ sessionId }: { sessionId: string }) {
  const { data, isLoading } = useIdeateResearch(sessionId);
  const saveMutation = useSaveResearch(sessionId);
  const { register, handleSubmit, reset, formState: { isDirty } } = useForm();

  useEffect(() => {
    if (data) {
      reset({
        research_findings: data.research_findings || '',
        technical_specs: data.technical_specs || '',
      });
    }
  }, [data, reset]);

  const onSubmit = async (formData: unknown) => {
    try {
      await saveMutation.mutateAsync({
        session_id: sessionId,
        ...(formData as Record<string, unknown>),
        competitors: null,
        similar_projects: null,
        reference_links: null,
        ai_generated: false,
      });
      toast.success('Research saved!');
    } catch {
      toast.error('Failed to save');
    }
  };

  if (isLoading) return <div className="flex py-12"><Loader2 className="w-8 h-8 animate-spin mx-auto" /></div>;

  return (
    <form onSubmit={handleSubmit(onSubmit)} className="space-y-6 max-w-3xl">
      <div className="space-y-2">
        <Label>Research Findings</Label>
        <Textarea {...register('research_findings')} placeholder="Key insights from competitor analysis, user research, market trends..." rows={6} />
      </div>
      <div className="space-y-2">
        <Label>Technical Specifications</Label>
        <Textarea {...register('technical_specs')} placeholder="Relevant technical standards, APIs, protocols, best practices..." rows={6} />
      </div>
      <Button type="submit" disabled={!isDirty || saveMutation.isPending}>{saveMutation.isPending ? 'Saving...' : 'Save Section'}</Button>
    </form>
  );
}
