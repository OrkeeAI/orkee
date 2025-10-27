// ABOUTME: Enhanced dependency chain section with Phase 4 dependency intelligence
// ABOUTME: Interactive graph, build order optimization, and smart feature selection

import { useState, useEffect } from 'react';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Button } from '@/components/ui/button';
import { Loader2 } from 'lucide-react';
import { useIdeateDependencies, useSaveDependencies } from '@/hooks/useIdeate';
import { DependencyMapper } from './DependencyMapper';
import { BuildOrderVisualizer } from './BuildOrderVisualizer';
import { FeaturePicker } from './FeaturePicker';
import { toast } from 'sonner';

export function DependencyChainSection({ sessionId }: { sessionId: string }) {
  const { data, isLoading } = useIdeateDependencies(sessionId);
  const saveMutation = useSaveDependencies(sessionId);

  const [foundationFeatures, setFoundationFeatures] = useState<string[]>([]);
  const [visibleFeatures, setVisibleFeatures] = useState<string[]>([]);
  const [enhancementFeatures, setEnhancementFeatures] = useState<string[]>([]);

  useEffect(() => {
    if (data) {
      setFoundationFeatures(data.foundation_features || []);
      setVisibleFeatures(data.visible_features || []);
      setEnhancementFeatures(data.enhancement_features || []);
    }
  }, [data]);

  const handleSave = async () => {
    try {
      await saveMutation.mutateAsync({
        session_id: sessionId,
        foundation_features: foundationFeatures.length > 0 ? foundationFeatures : null,
        visible_features: visibleFeatures.length > 0 ? visibleFeatures : null,
        enhancement_features: enhancementFeatures.length > 0 ? enhancementFeatures : null,
        dependency_graph: data?.dependency_graph || null,
        ai_generated: false,
      });
      toast.success('Dependencies saved!');
    } catch (error) {
      toast.error('Failed to save');
    }
  };

  if (isLoading) {
    return (
      <div className="flex py-12">
        <Loader2 className="w-8 h-8 animate-spin mx-auto" />
      </div>
    );
  }

  const allFeatures = [
    ...foundationFeatures,
    ...visibleFeatures,
    ...enhancementFeatures,
  ].filter((f, i, arr) => arr.indexOf(f) === i); // Unique features

  return (
    <div className="space-y-6 max-w-6xl">
      <div className="space-y-2">
        <h2 className="text-2xl font-bold">Dependency Chain & Build Order</h2>
        <p className="text-muted-foreground">
          Define feature dependencies, visualize relationships, and optimize your build order using
          AI-powered analysis.
        </p>
      </div>

      <Tabs defaultValue="picker" className="w-full">
        <TabsList className="grid w-full grid-cols-3">
          <TabsTrigger value="picker">Feature Selection</TabsTrigger>
          <TabsTrigger value="graph">Dependency Graph</TabsTrigger>
          <TabsTrigger value="timeline">Build Timeline</TabsTrigger>
        </TabsList>

        <TabsContent value="picker" className="space-y-4 mt-6">
          <FeaturePicker
            sessionId={sessionId}
            allFeatures={allFeatures}
            foundationFeatures={foundationFeatures}
            visibleFeatures={visibleFeatures}
            onUpdateFoundation={setFoundationFeatures}
            onUpdateVisible={setVisibleFeatures}
          />
        </TabsContent>

        <TabsContent value="graph" className="space-y-4 mt-6">
          {allFeatures.length > 0 ? (
            <DependencyMapper sessionId={sessionId} features={allFeatures} />
          ) : (
            <div className="text-center py-12 border rounded-lg">
              <p className="text-muted-foreground">
                Add features in the Feature Selection tab to visualize dependencies
              </p>
            </div>
          )}
        </TabsContent>

        <TabsContent value="timeline" className="space-y-4 mt-6">
          <BuildOrderVisualizer sessionId={sessionId} />
        </TabsContent>
      </Tabs>

      <div className="flex justify-end pt-4">
        <Button onClick={handleSave} disabled={saveMutation.isPending}>
          {saveMutation.isPending ? 'Saving...' : 'Save Section'}
        </Button>
      </div>
    </div>
  );
}
