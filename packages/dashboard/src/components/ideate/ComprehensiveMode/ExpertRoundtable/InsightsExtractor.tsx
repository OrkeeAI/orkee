// ABOUTME: Insight extraction and display component for roundtable discussions
// ABOUTME: Extracts key insights using AI and displays them grouped by category with priority levels

import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Separator } from '@/components/ui/separator';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { ScrollArea } from '@/components/ui/scroll-area';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog';
import {
  Lightbulb,
  Sparkles,
  AlertCircle,
  CheckCircle2,
  AlertTriangle,
  Info,
  Loader2,
  Plus,
  X,
} from 'lucide-react';
import { useExtractInsights, useGetInsights } from '@/hooks/useIdeate';
import { toast } from 'sonner';
import type { InsightsByCategory, InsightPriority } from '@/services/ideate';

interface InsightsExtractorProps {
  roundtableId: string;
}

export function InsightsExtractor({ roundtableId }: InsightsExtractorProps) {
  const [extractDialogOpen, setExtractDialogOpen] = useState(false);
  const [categories, setCategories] = useState<string[]>([]);
  const [categoryInput, setCategoryInput] = useState('');

  const { data: insights, isLoading: insightsLoading } = useGetInsights(roundtableId);
  const extractInsightsMutation = useExtractInsights(roundtableId);

  const getPriorityIcon = (priority: InsightPriority) => {
    switch (priority) {
      case 'critical':
        return AlertCircle;
      case 'high':
        return AlertTriangle;
      case 'medium':
        return Info;
      case 'low':
        return CheckCircle2;
      default:
        return Info;
    }
  };

  const getPriorityBadgeVariant = (
    priority: InsightPriority
  ): 'default' | 'secondary' | 'destructive' | 'outline' => {
    switch (priority) {
      case 'critical':
        return 'destructive';
      case 'high':
        return 'default';
      case 'medium':
        return 'secondary';
      case 'low':
        return 'outline';
      default:
        return 'secondary';
    }
  };

  const handleAddCategory = () => {
    const category = categoryInput.trim();
    if (category && !categories.includes(category)) {
      setCategories([...categories, category]);
      setCategoryInput('');
    }
  };

  const handleRemoveCategory = (category: string) => {
    setCategories(categories.filter((c) => c !== category));
  };

  const handleExtractInsights = async () => {
    try {
      toast.info('Extracting insights...', { duration: 2000 });
      await extractInsightsMutation.mutateAsync({
        categories: categories.length > 0 ? categories : undefined,
      });
      toast.success('Insights extracted successfully!');
      setExtractDialogOpen(false);
      setCategories([]);
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error';
      toast.error('Failed to extract insights', { description: errorMessage });
    }
  };

  const renderInsightCard = (insight: InsightsByCategory['insights'][0]) => {
    const PriorityIcon = getPriorityIcon(insight.priority);

    return (
      <div key={insight.id} className="group p-4 rounded-lg border bg-card hover:shadow-sm transition-shadow">
        <div className="flex items-start gap-3">
          <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-primary/10">
            <PriorityIcon className="h-4 w-4 text-primary" />
          </div>
          <div className="flex-1 space-y-2">
            <div className="flex items-center gap-2">
              <Badge variant={getPriorityBadgeVariant(insight.priority)} className="text-xs">
                {insight.priority}
              </Badge>
              {insight.source_experts && insight.source_experts.length > 0 && (
                <span className="text-xs text-muted-foreground">
                  from {insight.source_experts.join(', ')}
                </span>
              )}
            </div>
            <p className="text-sm leading-relaxed">{insight.insight_text}</p>
          </div>
        </div>
      </div>
    );
  };

  const renderCategorySection = (categoryData: InsightsByCategory) => {
    return (
      <div key={categoryData.category} className="space-y-3">
        <div className="flex items-center gap-2">
          <h4 className="font-semibold text-base">{categoryData.category}</h4>
          <Badge variant="secondary">{categoryData.count} insights</Badge>
        </div>
        <div className="space-y-2">
          {categoryData.insights.map((insight) => renderInsightCard(insight))}
        </div>
      </div>
    );
  };

  return (
    <Card>
      <CardHeader className="pb-3">
        <div className="flex items-center justify-between">
          <div>
            <CardTitle className="flex items-center gap-2">
              <Lightbulb className="h-5 w-5" />
              Discussion Insights
            </CardTitle>
            <CardDescription>
              Key takeaways and conclusions from the roundtable
            </CardDescription>
          </div>
          <Dialog open={extractDialogOpen} onOpenChange={setExtractDialogOpen}>
            <DialogTrigger asChild>
              <Button variant="outline" size="sm">
                <Sparkles className="h-4 w-4 mr-2" />
                Extract Insights
              </Button>
            </DialogTrigger>
            <DialogContent>
              <DialogHeader>
                <DialogTitle>Extract Insights</DialogTitle>
                <DialogDescription>
                  Use AI to automatically extract and categorize key insights from the discussion
                </DialogDescription>
              </DialogHeader>
              <div className="space-y-4">
                <div>
                  <Label htmlFor="category-input">
                    Categories (optional)
                  </Label>
                  <p className="text-xs text-muted-foreground mb-2">
                    Specify categories to organize insights. Leave empty for automatic categorization.
                  </p>
                  <div className="flex gap-2">
                    <Input
                      id="category-input"
                      placeholder="e.g., Technical, UX, Business"
                      value={categoryInput}
                      onChange={(e) => setCategoryInput(e.target.value)}
                      onKeyDown={(e) => e.key === 'Enter' && (e.preventDefault(), handleAddCategory())}
                    />
                    <Button type="button" onClick={handleAddCategory} size="sm">
                      <Plus className="h-4 w-4" />
                    </Button>
                  </div>
                  <div className="flex flex-wrap gap-1.5 mt-2">
                    {categories.map((category) => (
                      <Badge
                        key={category}
                        variant="secondary"
                        className="cursor-pointer"
                        onClick={() => handleRemoveCategory(category)}
                      >
                        {category} <X className="h-3 w-3 ml-1" />
                      </Badge>
                    ))}
                  </div>
                </div>
              </div>
              <DialogFooter>
                <Button
                  onClick={handleExtractInsights}
                  disabled={extractInsightsMutation.isPending}
                >
                  {extractInsightsMutation.isPending && (
                    <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                  )}
                  Extract Insights
                </Button>
              </DialogFooter>
            </DialogContent>
          </Dialog>
        </div>
      </CardHeader>
      <CardContent>
        {insightsLoading ? (
          <div className="flex items-center justify-center p-8">
            <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
          </div>
        ) : !insights || insights.length === 0 ? (
          <Alert>
            <AlertCircle className="h-4 w-4" />
            <AlertDescription>
              No insights extracted yet. Click "Extract Insights" to analyze the discussion.
            </AlertDescription>
          </Alert>
        ) : (
          <ScrollArea className="h-[600px] pr-4">
            <div className="space-y-6">
              {insights.map((categoryData) => renderCategorySection(categoryData))}
            </div>
          </ScrollArea>
        )}
      </CardContent>
    </Card>
  );
}
