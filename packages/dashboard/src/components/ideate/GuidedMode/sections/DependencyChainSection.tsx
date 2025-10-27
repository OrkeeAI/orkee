// ABOUTME: Dependency chain section for feature dependencies and build phases
// ABOUTME: Stub with basic text fields for feature lists
import { useEffect } from 'react';
import { useForm } from 'react-hook-form';
import { Button } from '@/components/ui/button';
import { Textarea } from '@/components/ui/textarea';
import { Label } from '@/components/ui/label';
import { Loader2 } from 'lucide-react';
import { useIdeateDependencies, useSaveDependencies } from '@/hooks/useIdeate';
import { toast } from 'sonner';

export function DependencyChainSection({ sessionId }: { sessionId: string }) {
  const { data, isLoading } = useIdeateDependencies(sessionId);
  const saveMutation = useSaveDependencies(sessionId);
  const { register, handleSubmit, reset, formState: { isDirty } } = useForm();

  useEffect(() => {
    if (data) {
      reset({
        foundation: data.foundation_features?.join('\n') || '',
        visible: data.visible_features?.join('\n') || '',
        enhancement: data.enhancement_features?.join('\n') || '',
      });
    }
  }, [data, reset]);

  const onSubmit = async (formData: any) => {
    try {
      await saveMutation.mutateAsync({
        session_id: sessionId,
        foundation_features: formData.foundation?.split('\n').filter((s: string) => s.trim()) || null,
        visible_features: formData.visible?.split('\n').filter((s: string) => s.trim()) || null,
        enhancement_features: formData.enhancement?.split('\n').filter((s: string) => s.trim()) || null,
        dependency_graph: null,
        ai_generated: false,
      });
      toast.success('Dependencies saved!');
    } catch (error) {
      toast.error('Failed to save');
    }
  };

  if (isLoading) return <div className="flex py-12"><Loader2 className="w-8 h-8 animate-spin mx-auto" /></div>;

  return (
    <form onSubmit={handleSubmit(onSubmit)} className="space-y-6 max-w-3xl">
      <div className="space-y-2">
        <Label>Foundation Features (Phase 1 - one per line)</Label>
        <Textarea {...register('foundation')} placeholder="Core backend\nDatabase schema\nAuthentication" rows={4} />
      </div>
      <div className="space-y-2">
        <Label>Visible Features (Phase 2 - one per line)</Label>
        <Textarea {...register('visible')} placeholder="User dashboard\nData visualization\nSearch" rows={4} />
      </div>
      <div className="space-y-2">
        <Label>Enhancement Features (Phase 3 - one per line)</Label>
        <Textarea {...register('enhancement')} placeholder="Advanced analytics\nExport tools\nIntegrations" rows={4} />
      </div>
      <Button type="submit" disabled={!isDirty || saveMutation.isPending}>{saveMutation.isPending ? 'Saving...' : 'Save Section'}</Button>
    </form>
  );
}
