// ABOUTME: Smart feature picker with AI-suggested quick wins and foundation features
// ABOUTME: Helps identify which features should be foundation vs visible based on dependencies

import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Checkbox } from '@/components/ui/checkbox';
import { Sparkles, CheckCircle2, AlertCircle } from 'lucide-react';
import { useQuickWins, useCircularDependencies } from '@/hooks/useIdeate';
import { toast } from 'sonner';

interface FeaturePickerProps {
  sessionId: string;
  allFeatures: string[];
  foundationFeatures: string[];
  visibleFeatures: string[];
  onUpdateFoundation: (features: string[]) => void;
  onUpdateVisible: (features: string[]) => void;
}

export function FeaturePicker({
  sessionId,
  allFeatures,
  foundationFeatures,
  visibleFeatures,
  onUpdateFoundation,
  onUpdateVisible,
}: FeaturePickerProps) {
  const { data: quickWins = [], isLoading: loadingQuickWins } = useQuickWins(sessionId);
  const { data: circularDeps = [] } = useCircularDependencies(sessionId);

  const [selectedFoundation, setSelectedFoundation] = useState<Set<string>>(
    new Set(foundationFeatures)
  );
  const [selectedVisible, setSelectedVisible] = useState<Set<string>>(
    new Set(visibleFeatures)
  );

  const isInCircularDep = (feature: string) => {
    return circularDeps.some((circle) => circle.cycle.includes(feature));
  };

  const isQuickWin = (feature: string) => quickWins.includes(feature);

  const toggleFoundation = (feature: string) => {
    const newSet = new Set(selectedFoundation);
    if (newSet.has(feature)) {
      newSet.delete(feature);
    } else {
      newSet.add(feature);
      // Remove from visible if added to foundation
      const newVisible = new Set(selectedVisible);
      newVisible.delete(feature);
      setSelectedVisible(newVisible);
    }
    setSelectedFoundation(newSet);
  };

  const toggleVisible = (feature: string) => {
    const newSet = new Set(selectedVisible);
    if (newSet.has(feature)) {
      newSet.delete(feature);
    } else {
      newSet.add(feature);
      // Remove from foundation if added to visible
      const newFoundation = new Set(selectedFoundation);
      newFoundation.delete(feature);
      setSelectedFoundation(newFoundation);
    }
    setSelectedVisible(newSet);
  };

  const handleApply = () => {
    onUpdateFoundation(Array.from(selectedFoundation));
    onUpdateVisible(Array.from(selectedVisible));
    toast.success('Feature selection updated');
  };

  const applyQuickWins = () => {
    const newVisible = new Set(selectedVisible);
    quickWins.forEach((feature) => {
      if (allFeatures.includes(feature) && !selectedFoundation.has(feature)) {
        newVisible.add(feature);
      }
    });
    setSelectedVisible(newVisible);
    toast.success(`Added ${quickWins.length} quick-win features to visible`);
  };

  const unassignedFeatures = allFeatures.filter(
    (f) => !selectedFoundation.has(f) && !selectedVisible.has(f)
  );

  return (
    <div className="space-y-6">
      {/* Quick Wins Suggestion */}
      {!loadingQuickWins && quickWins.length > 0 && (
        <Card className="border-green-500/50 bg-green-500/5">
          <CardHeader className="pb-3">
            <div className="flex items-center justify-between">
              <CardTitle className="text-base flex items-center gap-2">
                <Sparkles className="w-4 h-4 text-green-500" />
                Quick Win Recommendations
              </CardTitle>
              <Button size="sm" variant="outline" onClick={applyQuickWins}>
                Apply All
              </Button>
            </div>
          </CardHeader>
          <CardContent>
            <p className="text-sm text-muted-foreground mb-3">
              These features have minimal dependencies and can deliver value quickly.
            </p>
            <div className="flex flex-wrap gap-2">
              {quickWins.map((feature) => (
                <Badge key={feature} variant="outline" className="bg-green-500/10">
                  {feature}
                </Badge>
              ))}
            </div>
          </CardContent>
        </Card>
      )}

      {/* Circular Dependency Warning */}
      {circularDeps.length > 0 && (
        <Card className="border-red-500/50 bg-red-500/5">
          <CardHeader className="pb-3">
            <CardTitle className="text-base flex items-center gap-2">
              <AlertCircle className="w-4 h-4 text-red-500" />
              Circular Dependencies Detected
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-sm text-muted-foreground mb-3">
              These features have circular dependencies. Consider breaking them up or removing
              dependencies.
            </p>
            {circularDeps.map((circle, index) => (
              <div key={index} className="text-sm mb-2">
                <Badge variant="destructive" className="mb-1">
                  {circle.severity}
                </Badge>
                <div className="text-muted-foreground">
                  {circle.cycle.join(' → ')} → {circle.cycle[0]}
                </div>
                <div className="text-xs mt-1">{circle.suggestion}</div>
              </div>
            ))}
          </CardContent>
        </Card>
      )}

      {/* Feature Lists */}
      <div className="grid gap-6 md:grid-cols-2">
        {/* Foundation Features */}
        <Card>
          <CardHeader className="pb-3">
            <CardTitle className="text-base">Foundation Features (Phase 1)</CardTitle>
            <p className="text-sm text-muted-foreground">
              Core infrastructure, APIs, data models. No user-facing features.
            </p>
          </CardHeader>
          <CardContent>
            <div className="space-y-2">
              {allFeatures.map((feature) => (
                <div
                  key={feature}
                  className="flex items-center space-x-2 p-2 rounded hover:bg-muted/50"
                >
                  <Checkbox
                    checked={selectedFoundation.has(feature)}
                    onCheckedChange={() => toggleFoundation(feature)}
                    id={`foundation-${feature}`}
                  />
                  <label
                    htmlFor={`foundation-${feature}`}
                    className="flex-1 text-sm cursor-pointer flex items-center gap-2"
                  >
                    {feature}
                    {isQuickWin(feature) && (
                      <Badge variant="outline" className="text-xs bg-green-500/10">
                        quick win
                      </Badge>
                    )}
                    {isInCircularDep(feature) && (
                      <Badge variant="destructive" className="text-xs">
                        circular
                      </Badge>
                    )}
                  </label>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>

        {/* Visible Features */}
        <Card>
          <CardHeader className="pb-3">
            <CardTitle className="text-base">Visible Features (Phase 2)</CardTitle>
            <p className="text-sm text-muted-foreground">
              User-facing features that depend on foundation. Build these for MVP.
            </p>
          </CardHeader>
          <CardContent>
            <div className="space-y-2">
              {allFeatures.map((feature) => (
                <div
                  key={feature}
                  className="flex items-center space-x-2 p-2 rounded hover:bg-muted/50"
                >
                  <Checkbox
                    checked={selectedVisible.has(feature)}
                    onCheckedChange={() => toggleVisible(feature)}
                    id={`visible-${feature}`}
                  />
                  <label
                    htmlFor={`visible-${feature}`}
                    className="flex-1 text-sm cursor-pointer flex items-center gap-2"
                  >
                    {feature}
                    {isQuickWin(feature) && (
                      <Badge variant="outline" className="text-xs bg-green-500/10">
                        quick win
                      </Badge>
                    )}
                    {isInCircularDep(feature) && (
                      <Badge variant="destructive" className="text-xs">
                        circular
                      </Badge>
                    )}
                  </label>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Unassigned Features */}
      {unassignedFeatures.length > 0 && (
        <Card className="border-yellow-500/50 bg-yellow-500/5">
          <CardHeader className="pb-3">
            <CardTitle className="text-base">Unassigned Features</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="flex flex-wrap gap-2">
              {unassignedFeatures.map((feature) => (
                <Badge key={feature} variant="outline">
                  {feature}
                </Badge>
              ))}
            </div>
          </CardContent>
        </Card>
      )}

      {/* Apply Button */}
      <div className="flex justify-end">
        <Button onClick={handleApply} className="gap-2">
          <CheckCircle2 className="w-4 h-4" />
          Apply Selection
        </Button>
      </div>
    </div>
  );
}
