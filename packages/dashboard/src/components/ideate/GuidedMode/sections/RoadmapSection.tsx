// ABOUTME: Roadmap section for MVP scope and future phases
// ABOUTME: Stub implementation with basic text fields
import { useEffect } from 'react';
import { useForm } from 'react-hook-form';
import { Button } from '@/components/ui/button';
import { Textarea } from '@/components/ui/textarea';
import { Label } from '@/components/ui/label';
import { Loader2 } from 'lucide-react';
import { useIdeateRoadmap, useSaveRoadmap } from '@/hooks/useIdeate';
import { toast } from 'sonner';

export function RoadmapSection({ sessionId }: { sessionId: string }) {
  const { data, isLoading } = useIdeateRoadmap(sessionId);
  const saveMutation = useSaveRoadmap(sessionId);
  const { register, handleSubmit, reset, formState: { isDirty } } = useForm();

  useEffect(() => {
    if (data) reset({ mvp_scope_text: data.mvp_scope?.join('\n') || '' });
  }, [data, reset]);

  const onSubmit = async (formData: any) => {
    const mvp_scope = formData.mvp_scope_text ? formData.mvp_scope_text.split('\n').filter((s: string) => s.trim()) : null;
    try {
      await saveMutation.mutateAsync({ session_id: sessionId, mvp_scope, future_phases: null, ai_generated: false });
      toast.success('Roadmap saved!');
    } catch (error) {
      toast.error('Failed to save');
    }
  };

  if (isLoading) return <div className="flex py-12"><Loader2 className="w-8 h-8 animate-spin mx-auto" /></div>;

  return (
    <form onSubmit={handleSubmit(onSubmit)} className="space-y-6 max-w-3xl">
      <div className="space-y-2">
        <Label>MVP Scope (one per line)</Label>
        <Textarea {...register('mvp_scope_text')} placeholder="Feature 1\nFeature 2\nFeature 3" rows={8} />
      </div>
      <Button type="submit" disabled={!isDirty || saveMutation.isPending}>{saveMutation.isPending ? 'Saving...' : 'Save Section'}</Button>
    </form>
  );
}
