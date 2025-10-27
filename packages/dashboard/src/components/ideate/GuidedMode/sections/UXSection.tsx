// ABOUTME: UX section stub for personas, user flows, and UI considerations
// ABOUTME: Placeholder form - to be expanded with dynamic persona/flow editors
import { useEffect } from 'react';
import { useForm } from 'react-hook-form';
import { Button } from '@/components/ui/button';
import { Textarea } from '@/components/ui/textarea';
import { Label } from '@/components/ui/label';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Loader2, Save } from 'lucide-react';
import { useIdeateUX, useSaveUX } from '@/hooks/useIdeate';
import { toast } from 'sonner';

interface UXFormData {
  ui_considerations: string;
  ux_principles: string;
}

interface UXSectionProps {
  sessionId: string;
}

export function UXSection({ sessionId }: UXSectionProps) {
  const { data: ux, isLoading, error } = useIdeateUX(sessionId);
  const saveMutation = useSaveUX(sessionId);

  const { register, handleSubmit, reset, formState: { isDirty } } = useForm<UXFormData>();

  useEffect(() => {
    if (ux) {
      reset({
        ui_considerations: ux.ui_considerations || '',
        ux_principles: ux.ux_principles || '',
      });
    }
  }, [ux, reset]);

  const onSubmit = async (data: UXFormData) => {
    try {
      await saveMutation.mutateAsync({
        session_id: sessionId,
        ...data,
        personas: null,
        user_flows: null,
        ai_generated: false,
      });
      toast.success('UX section saved successfully!');
    } catch {
      toast.error('Failed to save UX section');
    }
  };

  if (isLoading) return <div className="flex items-center justify-center py-12"><Loader2 className="w-8 h-8 animate-spin" /></div>;
  if (error) return <Alert variant="destructive"><AlertDescription>Failed to load UX section</AlertDescription></Alert>;

  return (
    <form onSubmit={handleSubmit(onSubmit)} className="space-y-6 max-w-3xl">
      <div className="space-y-2">
        <Label htmlFor="ui_considerations">UI Considerations</Label>
        <Textarea id="ui_considerations" {...register('ui_considerations')} placeholder="Key UI design considerations..." rows={4} />
        <p className="text-sm text-muted-foreground">Describe important UI elements, layouts, and design patterns</p>
      </div>

      <div className="space-y-2">
        <Label htmlFor="ux_principles">UX Principles</Label>
        <Textarea id="ux_principles" {...register('ux_principles')} placeholder="Core UX principles..." rows={4} />
        <p className="text-sm text-muted-foreground">Define the user experience philosophy and principles</p>
      </div>

      <div className="flex justify-end pt-4">
        <Button type="submit" disabled={!isDirty || saveMutation.isPending}>
          {saveMutation.isPending ? <><Loader2 className="w-4 h-4 mr-2 animate-spin" />Saving...</> : <><Save className="w-4 h-4 mr-2" />Save Section</>}
        </Button>
      </div>
    </form>
  );
}
