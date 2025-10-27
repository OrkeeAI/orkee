// ABOUTME: Technical architecture section stub
// ABOUTME: Simplified form - to be expanded with component/data model editors
import { useEffect } from 'react';
import { useForm } from 'react-hook-form';
import { Button } from '@/components/ui/button';
import { Textarea } from '@/components/ui/textarea';
import { Label } from '@/components/ui/label';
import { Loader2 } from 'lucide-react';
import { useIdeateTechnical, useSaveTechnical } from '@/hooks/useIdeate';
import { toast } from 'sonner';

export function TechnicalSection({ sessionId }: { sessionId: string }) {
  const { data, isLoading } = useIdeateTechnical(sessionId);
  const saveMutation = useSaveTechnical(sessionId);
  const { register, handleSubmit, reset, formState: { isDirty } } = useForm();

  useEffect(() => {
    if (data) reset({ tech_stack_quick: data.tech_stack_quick || '' });
  }, [data, reset]);

  const onSubmit = async (formData: unknown) => {
    try {
      await saveMutation.mutateAsync({ session_id: sessionId, ...(formData as Record<string, unknown>), components: null, data_models: null, apis: null, infrastructure: null, ai_generated: false });
      toast.success('Technical section saved!');
    } catch {
      toast.error('Failed to save');
    }
  };

  if (isLoading) return <div className="flex py-12"><Loader2 className="w-8 h-8 animate-spin mx-auto" /></div>;

  return (
    <form onSubmit={handleSubmit(onSubmit)} className="space-y-6 max-w-3xl">
      <div className="space-y-2">
        <Label>Tech Stack Overview</Label>
        <Textarea {...register('tech_stack_quick')} placeholder="Key technologies, frameworks, and tools..." rows={6} />
      </div>
      <Button type="submit" disabled={!isDirty || saveMutation.isPending}>{saveMutation.isPending ? 'Saving...' : 'Save Section'}</Button>
    </form>
  );
}
