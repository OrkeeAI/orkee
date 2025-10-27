// ABOUTME: Risks and mitigations section stub
// ABOUTME: Simple text fields for capturing risk areas
import { useEffect } from 'react';
import { useForm } from 'react-hook-form';
import { Button } from '@/components/ui/button';
import { Textarea } from '@/components/ui/textarea';
import { Label } from '@/components/ui/label';
import { Loader2 } from 'lucide-react';
import { useIdeateRisks, useSaveRisks } from '@/hooks/useIdeate';
import { toast } from 'sonner';

export function RisksSection({ sessionId }: { sessionId: string }) {
  const { data, isLoading } = useIdeateRisks(sessionId);
  const saveMutation = useSaveRisks(sessionId);
  const { register, handleSubmit, reset, formState: { isDirty } } = useForm();

  useEffect(() => {
    if (data) {
      reset({
        technical: data.technical_risks?.map(r => r.description).join('\n') || '',
        scoping: data.mvp_scoping_risks?.map(r => r.description).join('\n') || '',
        resource: data.resource_risks?.map(r => r.description).join('\n') || '',
      });
    }
  }, [data, reset]);

  const onSubmit = async (formData: unknown) => {
    const data = formData as { technical?: string; scoping?: string; resource?: string };
    try {
      await saveMutation.mutateAsync({
        session_id: sessionId,
        technical_risks: data.technical?.split('\n').filter((s: string) => s.trim()).map((desc: string) => ({ description: desc, severity: 'medium', probability: 'medium' })) || null,
        mvp_scoping_risks: data.scoping?.split('\n').filter((s: string) => s.trim()).map((desc: string) => ({ description: desc, severity: 'medium', probability: 'medium' })) || null,
        resource_risks: data.resource?.split('\n').filter((s: string) => s.trim()).map((desc: string) => ({ description: desc, severity: 'medium', probability: 'medium' })) || null,
        mitigations: null,
        ai_generated: false,
      });
      toast.success('Risks saved!');
    } catch {
      toast.error('Failed to save');
    }
  };

  if (isLoading) return <div className="flex py-12"><Loader2 className="w-8 h-8 animate-spin mx-auto" /></div>;

  return (
    <form onSubmit={handleSubmit(onSubmit)} className="space-y-6 max-w-3xl">
      <div className="space-y-2">
        <Label>Technical Risks (one per line)</Label>
        <Textarea {...register('technical')} placeholder="Complex integration\nPerformance concerns\nScaling challenges" rows={4} />
      </div>
      <div className="space-y-2">
        <Label>MVP Scoping Risks (one per line)</Label>
        <Textarea {...register('scoping')} placeholder="Feature creep\nUnclear requirements\nTimeline pressure" rows={4} />
      </div>
      <div className="space-y-2">
        <Label>Resource Risks (one per line)</Label>
        <Textarea {...register('resource')} placeholder="Budget constraints\nTeam availability\nSkill gaps" rows={4} />
      </div>
      <Button type="submit" disabled={!isDirty || saveMutation.isPending}>{saveMutation.isPending ? 'Saving...' : 'Save Section'}</Button>
    </form>
  );
}
