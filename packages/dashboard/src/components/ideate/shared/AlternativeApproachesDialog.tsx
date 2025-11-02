// ABOUTME: Dialog for displaying and selecting alternative technical approaches
// ABOUTME: Shows comparison table with pros/cons, estimated days, and complexity for each approach

import { useState } from 'react';
import { CheckCircle, TrendingUp, Clock, AlertTriangle } from 'lucide-react';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { RadioGroup, RadioGroupItem } from '@/components/ui/radio-group';
import { Label } from '@/components/ui/label';

export interface TechnicalApproach {
  name: string;
  description: string;
  pros: string[];
  cons: string[];
  estimated_days: number;
  complexity: 'Low' | 'Medium' | 'High';
  recommended: boolean;
  reasoning: string;
}

interface AlternativeApproachesDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  approaches: TechnicalApproach[];
  onSelect: (approachName: string) => void;
  isLoading?: boolean;
}

export function AlternativeApproachesDialog({
  open,
  onOpenChange,
  approaches,
  onSelect,
  isLoading = false,
}: AlternativeApproachesDialogProps) {
  const [selectedApproach, setSelectedApproach] = useState<string>(
    approaches.find(a => a.recommended)?.name || approaches[0]?.name || ''
  );

  const getComplexityColor = (complexity: string) => {
    switch (complexity.toLowerCase()) {
      case 'low':
        return 'bg-green-100 text-green-800 border-green-200';
      case 'medium':
        return 'bg-yellow-100 text-yellow-800 border-yellow-200';
      case 'high':
        return 'bg-red-100 text-red-800 border-red-200';
      default:
        return 'bg-gray-100 text-gray-800 border-gray-200';
    }
  };

  const handleConfirm = () => {
    onSelect(selectedApproach);
    onOpenChange(false);
  };

  if (isLoading) {
    return (
      <Dialog open={open} onOpenChange={onOpenChange}>
        <DialogContent className="max-w-4xl">
          <DialogHeader>
            <DialogTitle>Generating Alternative Approaches...</DialogTitle>
            <DialogDescription>
              Analyzing your requirements to suggest different technical approaches
            </DialogDescription>
          </DialogHeader>
          <div className="py-12 text-center text-muted-foreground">
            <div className="animate-spin mx-auto h-8 w-8 border-4 border-primary border-t-transparent rounded-full mb-4" />
            <p>Generating approaches...</p>
          </div>
        </DialogContent>
      </Dialog>
    );
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-6xl max-h-[90vh] flex flex-col">
        <DialogHeader>
          <DialogTitle>Choose Your Technical Approach</DialogTitle>
          <DialogDescription>
            Compare different approaches and select the one that best fits your needs
          </DialogDescription>
        </DialogHeader>

        <div className="flex-1 overflow-y-auto">
          <RadioGroup value={selectedApproach} onValueChange={setSelectedApproach}>
            <div className="space-y-4">
              {approaches.map((approach) => (
                <Card
                  key={approach.name}
                  className={`cursor-pointer transition-all ${
                    selectedApproach === approach.name
                      ? 'ring-2 ring-primary border-primary'
                      : 'hover:border-gray-400'
                  }`}
                  onClick={() => setSelectedApproach(approach.name)}
                >
                  <CardHeader>
                    <div className="flex items-start justify-between">
                      <div className="flex items-start gap-3 flex-1">
                        <RadioGroupItem value={approach.name} id={approach.name} className="mt-1" />
                        <div className="flex-1">
                          <div className="flex items-center gap-2 mb-2">
                            <Label htmlFor={approach.name} className="text-lg font-semibold cursor-pointer">
                              {approach.name}
                            </Label>
                            {approach.recommended && (
                              <Badge variant="default" className="gap-1">
                                <CheckCircle className="h-3 w-3" />
                                Recommended
                              </Badge>
                            )}
                          </div>
                          <CardDescription>{approach.description}</CardDescription>
                        </div>
                      </div>

                      <div className="flex flex-col gap-2">
                        <Badge className={getComplexityColor(approach.complexity)}>
                          {approach.complexity} Complexity
                        </Badge>
                      </div>
                    </div>
                  </CardHeader>

                  <CardContent className="space-y-4">
                    {/* Metrics Row */}
                    <div className="flex gap-6 text-sm">
                      <div className="flex items-center gap-2">
                        <Clock className="h-4 w-4 text-muted-foreground" />
                        <span className="font-medium">{approach.estimated_days} days</span>
                      </div>
                      <div className="flex items-center gap-2">
                        <TrendingUp className="h-4 w-4 text-muted-foreground" />
                        <span className="font-medium">{approach.complexity} complexity</span>
                      </div>
                    </div>

                    {/* Pros and Cons Grid */}
                    <div className="grid md:grid-cols-2 gap-4">
                      {/* Pros */}
                      <div className="space-y-2">
                        <div className="font-semibold text-sm text-green-700 flex items-center gap-1">
                          <CheckCircle className="h-4 w-4" />
                          Pros
                        </div>
                        <ul className="space-y-1">
                          {approach.pros.map((pro, index) => (
                            <li key={index} className="text-sm flex items-start gap-2">
                              <span className="text-green-600 mt-1">•</span>
                              <span>{pro}</span>
                            </li>
                          ))}
                        </ul>
                      </div>

                      {/* Cons */}
                      <div className="space-y-2">
                        <div className="font-semibold text-sm text-red-700 flex items-center gap-1">
                          <AlertTriangle className="h-4 w-4" />
                          Cons
                        </div>
                        <ul className="space-y-1">
                          {approach.cons.map((con, index) => (
                            <li key={index} className="text-sm flex items-start gap-2">
                              <span className="text-red-600 mt-1">•</span>
                              <span>{con}</span>
                            </li>
                          ))}
                        </ul>
                      </div>
                    </div>

                    {/* Reasoning */}
                    {approach.reasoning && (
                      <div className="pt-3 border-t">
                        <p className="text-sm text-muted-foreground italic">
                          <strong>Why this approach:</strong> {approach.reasoning}
                        </p>
                      </div>
                    )}
                  </CardContent>
                </Card>
              ))}
            </div>
          </RadioGroup>
        </div>

        <DialogFooter className="flex items-center justify-between">
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button onClick={handleConfirm} disabled={!selectedApproach}>
            Select Approach
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
