// ABOUTME: Competitor analysis section for Comprehensive Mode research
// ABOUTME: Scrapes URLs, analyzes competitors, performs gap analysis, and extracts UI patterns

import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Separator } from '@/components/ui/separator';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Textarea } from '@/components/ui/textarea';
import {
  Search,
  Globe,
  TrendingUp,
  TrendingDown,
  Target,
  AlertCircle,
  Loader2,
  CheckCircle,
  Lightbulb,
  Layout,
} from 'lucide-react';
import {
  useCompetitors,
  useAnalyzeCompetitor,
  useAnalyzeGaps,
  useExtractPatterns,
} from '@/hooks/useIdeate';
import { toast } from 'sonner';
import type { Competitor, GapAnalysis, UIPattern } from '@/services/ideate';

interface CompetitorAnalysisSectionProps {
  sessionId: string;
}

export function CompetitorAnalysisSection({ sessionId }: CompetitorAnalysisSectionProps) {
  const [competitorUrl, setCompetitorUrl] = useState('');
  const [patternUrl, setPatternUrl] = useState('');
  const [yourFeatures, setYourFeatures] = useState('');
  const [gapAnalysisResult, setGapAnalysisResult] = useState<GapAnalysis | null>(null);
  const [patternsResult, setPatternsResult] = useState<UIPattern[] | null>(null);

  const { data: competitors, isLoading: competitorsLoading } = useCompetitors(sessionId);
  const analyzeCompetitorMutation = useAnalyzeCompetitor(sessionId);
  const analyzeGapsMutation = useAnalyzeGaps(sessionId);
  const extractPatternsMutation = useExtractPatterns(sessionId);

  const handleAnalyzeCompetitor = async () => {
    if (!competitorUrl.trim()) {
      toast.error('Please enter a competitor URL');
      return;
    }

    try {
      toast.info('Analyzing competitor...', { duration: 2000 });
      await analyzeCompetitorMutation.mutateAsync({ url: competitorUrl.trim() });
      toast.success('Competitor analyzed successfully!');
      setCompetitorUrl('');
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error';
      toast.error('Failed to analyze competitor', { description: errorMessage });
    }
  };

  const handleAnalyzeGaps = async () => {
    if (!yourFeatures.trim()) {
      toast.error('Please enter your features (one per line)');
      return;
    }

    if (!competitors || competitors.length === 0) {
      toast.error('Please analyze at least one competitor first');
      return;
    }

    try {
      toast.info('Analyzing gaps...', { duration: 2000 });
      const features = yourFeatures
        .split('\n')
        .map(f => f.trim())
        .filter(f => f.length > 0);

      const result = await analyzeGapsMutation.mutateAsync(features);
      setGapAnalysisResult(result);
      toast.success('Gap analysis complete!');
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error';
      toast.error('Failed to analyze gaps', { description: errorMessage });
    }
  };

  const handleExtractPatterns = async () => {
    if (!patternUrl.trim()) {
      toast.error('Please enter a URL to extract patterns from');
      return;
    }

    try {
      toast.info('Extracting UI patterns...', { duration: 2000 });
      const patterns = await extractPatternsMutation.mutateAsync({ url: patternUrl.trim() });
      setPatternsResult(patterns);
      toast.success(`Extracted ${patterns.length} UI/UX patterns!`);
      setPatternUrl('');
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error';
      toast.error('Failed to extract patterns', { description: errorMessage });
    }
  };

  const getOpportunityIcon = (type: string) => {
    switch (type) {
      case 'differentiation':
        return <Target className="h-4 w-4" />;
      case 'improvement':
        return <TrendingUp className="h-4 w-4" />;
      case 'gap':
        return <AlertCircle className="h-4 w-4" />;
      default:
        return <Lightbulb className="h-4 w-4" />;
    }
  };

  const getPatternIcon = (type: string) => {
    switch (type) {
      case 'layout':
      case 'navigation':
      case 'interaction':
      case 'visual':
      case 'content':
        return <Layout className="h-4 w-4" />;
      default:
        return <Layout className="h-4 w-4" />;
    }
  };

  return (
    <div className="space-y-6">
      {/* Competitor Scanner */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Search className="h-5 w-5" />
            Competitor Scanner
          </CardTitle>
          <CardDescription>
            Analyze competitor websites to understand their features, strengths, and gaps
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex gap-2">
            <div className="flex-1">
              <Label htmlFor="competitor-url">Competitor URL</Label>
              <Input
                id="competitor-url"
                placeholder="https://competitor.com"
                value={competitorUrl}
                onChange={e => setCompetitorUrl(e.target.value)}
                onKeyDown={e => e.key === 'Enter' && handleAnalyzeCompetitor()}
              />
            </div>
            <Button
              onClick={handleAnalyzeCompetitor}
              disabled={analyzeCompetitorMutation.isPending}
              className="mt-auto"
            >
              {analyzeCompetitorMutation.isPending ? (
                <>
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  Analyzing...
                </>
              ) : (
                <>
                  <Globe className="mr-2 h-4 w-4" />
                  Analyze
                </>
              )}
            </Button>
          </div>

          {competitorsLoading && (
            <div className="flex items-center gap-2 text-sm text-muted-foreground">
              <Loader2 className="h-4 w-4 animate-spin" />
              Loading competitors...
            </div>
          )}

          {competitors && competitors.length > 0 && (
            <div className="space-y-3">
              <Separator />
              <h4 className="text-sm font-medium">Analyzed Competitors ({competitors.length})</h4>
              {competitors.map((competitor, idx) => (
                <CompetitorCard key={idx} competitor={competitor} />
              ))}
            </div>
          )}

          {!competitorsLoading && (!competitors || competitors.length === 0) && (
            <Alert>
              <AlertCircle className="h-4 w-4" />
              <AlertDescription>
                No competitors analyzed yet. Enter a competitor URL above to get started.
              </AlertDescription>
            </Alert>
          )}
        </CardContent>
      </Card>

      {/* Gap Analysis */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Target className="h-5 w-5" />
            Gap Analysis
          </CardTitle>
          <CardDescription>
            Compare your planned features against competitors to find opportunities
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div>
            <Label htmlFor="your-features">Your Planned Features (one per line)</Label>
            <Textarea
              id="your-features"
              placeholder="User authentication&#10;Real-time notifications&#10;Dashboard analytics&#10;..."
              value={yourFeatures}
              onChange={e => setYourFeatures(e.target.value)}
              rows={6}
            />
          </div>

          <Button
            onClick={handleAnalyzeGaps}
            disabled={analyzeGapsMutation.isPending || !competitors || competitors.length === 0}
            className="w-full"
          >
            {analyzeGapsMutation.isPending ? (
              <>
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                Analyzing Gaps...
              </>
            ) : (
              <>
                <Target className="mr-2 h-4 w-4" />
                Analyze Gaps
              </>
            )}
          </Button>

          {gapAnalysisResult && (
            <div className="space-y-3">
              <Separator />
              <div>
                <h4 className="text-sm font-medium mb-2">Analysis Summary</h4>
                <p className="text-sm text-muted-foreground">{gapAnalysisResult.summary}</p>
              </div>

              {gapAnalysisResult.opportunities.length > 0 && (
                <div>
                  <h4 className="text-sm font-medium mb-2">
                    Opportunities ({gapAnalysisResult.opportunities.length})
                  </h4>
                  <div className="space-y-2">
                    {gapAnalysisResult.opportunities.map((opp, idx) => (
                      <Card key={idx}>
                        <CardContent className="pt-4">
                          <div className="flex items-start gap-2">
                            {getOpportunityIcon(opp.opportunity_type)}
                            <div className="flex-1">
                              <div className="flex items-center gap-2 mb-1">
                                <h5 className="text-sm font-medium">{opp.title}</h5>
                                <Badge variant="outline">{opp.opportunity_type}</Badge>
                              </div>
                              <p className="text-sm text-muted-foreground mb-2">
                                {opp.description}
                              </p>
                              <p className="text-xs text-muted-foreground mb-1">
                                <strong>Context:</strong> {opp.competitor_context}
                              </p>
                              <p className="text-xs text-muted-foreground">
                                <strong>Recommendation:</strong> {opp.recommendation}
                              </p>
                            </div>
                          </div>
                        </CardContent>
                      </Card>
                    ))}
                  </div>
                </div>
              )}
            </div>
          )}
        </CardContent>
      </Card>

      {/* UI Pattern Extraction */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Layout className="h-5 w-5" />
            UI Pattern Extraction
          </CardTitle>
          <CardDescription>
            Extract and analyze UI/UX patterns from successful products
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex gap-2">
            <div className="flex-1">
              <Label htmlFor="pattern-url">Website URL</Label>
              <Input
                id="pattern-url"
                placeholder="https://example.com"
                value={patternUrl}
                onChange={e => setPatternUrl(e.target.value)}
                onKeyDown={e => e.key === 'Enter' && handleExtractPatterns()}
              />
            </div>
            <Button
              onClick={handleExtractPatterns}
              disabled={extractPatternsMutation.isPending}
              className="mt-auto"
            >
              {extractPatternsMutation.isPending ? (
                <>
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  Extracting...
                </>
              ) : (
                <>
                  <Layout className="mr-2 h-4 w-4" />
                  Extract
                </>
              )}
            </Button>
          </div>

          {patternsResult && patternsResult.length > 0 && (
            <div className="space-y-3">
              <Separator />
              <h4 className="text-sm font-medium">
                Extracted Patterns ({patternsResult.length})
              </h4>
              <div className="space-y-2">
                {patternsResult.map((pattern, idx) => (
                  <Card key={idx}>
                    <CardContent className="pt-4">
                      <div className="flex items-start gap-2">
                        {getPatternIcon(pattern.pattern_type)}
                        <div className="flex-1">
                          <div className="flex items-center gap-2 mb-1">
                            <h5 className="text-sm font-medium">{pattern.name}</h5>
                            <Badge variant="outline">{pattern.pattern_type}</Badge>
                          </div>
                          <p className="text-sm text-muted-foreground mb-2">
                            {pattern.description}
                          </p>
                          <p className="text-xs text-muted-foreground mb-1">
                            <strong>Benefits:</strong> {pattern.benefits}
                          </p>
                          <p className="text-xs text-muted-foreground">
                            <strong>Adoption Notes:</strong> {pattern.adoption_notes}
                          </p>
                        </div>
                      </div>
                    </CardContent>
                  </Card>
                ))}
              </div>
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}

function CompetitorCard({ competitor }: { competitor: Competitor }) {
  return (
    <Card>
      <CardContent className="pt-4">
        <div className="space-y-3">
          <div>
            <div className="flex items-center gap-2 mb-1">
              <Globe className="h-4 w-4 text-muted-foreground" />
              <h5 className="text-sm font-medium">{competitor.name}</h5>
            </div>
            <a
              href={competitor.url}
              target="_blank"
              rel="noopener noreferrer"
              className="text-xs text-blue-500 hover:underline"
            >
              {competitor.url}
            </a>
          </div>

          <div>
            <div className="flex items-center gap-2 mb-1">
              <TrendingUp className="h-3 w-3 text-green-500" />
              <span className="text-xs font-medium">Strengths</span>
            </div>
            <ul className="space-y-1">
              {competitor.strengths.map((strength, idx) => (
                <li key={idx} className="text-xs text-muted-foreground flex items-start gap-1">
                  <CheckCircle className="h-3 w-3 mt-0.5 flex-shrink-0 text-green-500" />
                  <span>{strength}</span>
                </li>
              ))}
            </ul>
          </div>

          <div>
            <div className="flex items-center gap-2 mb-1">
              <TrendingDown className="h-3 w-3 text-orange-500" />
              <span className="text-xs font-medium">Gaps</span>
            </div>
            <ul className="space-y-1">
              {competitor.gaps.map((gap, idx) => (
                <li key={idx} className="text-xs text-muted-foreground flex items-start gap-1">
                  <AlertCircle className="h-3 w-3 mt-0.5 flex-shrink-0 text-orange-500" />
                  <span>{gap}</span>
                </li>
              ))}
            </ul>
          </div>

          <div>
            <span className="text-xs font-medium">Features</span>
            <div className="flex flex-wrap gap-1 mt-1">
              {competitor.features.map((feature, idx) => (
                <Badge key={idx} variant="secondary" className="text-xs">
                  {feature}
                </Badge>
              ))}
            </div>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
