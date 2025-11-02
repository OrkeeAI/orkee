// ABOUTME: Similar projects section for Comprehensive Mode research
// ABOUTME: Track similar projects, extract lessons, and identify patterns to adopt

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
  Plus,
  BookOpen,
  ThumbsUp,
  ThumbsDown,
  Lightbulb,
  AlertCircle,
  Loader2,
  ExternalLink,
  Target,
} from 'lucide-react';
import {
  useSimilarProjects,
  useAddSimilarProject,
  useExtractLessons,
  useSynthesizeResearch,
} from '@/hooks/useIdeate';
import { toast } from 'sonner';
import type { SimilarProject, Lesson, ResearchSynthesis } from '@/services/ideate';

interface SimilarProjectsSectionProps {
  sessionId: string;
}

export function SimilarProjectsSection({ sessionId }: SimilarProjectsSectionProps) {
  const [showAddForm, setShowAddForm] = useState(false);
  const [projectName, setProjectName] = useState('');
  const [projectUrl, setProjectUrl] = useState('');
  const [positiveAspects, setPositiveAspects] = useState('');
  const [negativeAspects, setNegativeAspects] = useState('');
  const [patternsToAdopt, setPatternsToAdopt] = useState('');
  const [selectedProjectForLessons, setSelectedProjectForLessons] = useState<string | null>(null);
  const [lessonsResult, setLessonsResult] = useState<Lesson[] | null>(null);
  const [synthesisResult, setSynthesisResult] = useState<ResearchSynthesis | null>(null);

  const { data: similarProjects, isLoading: projectsLoading } = useSimilarProjects(sessionId);
  const addProjectMutation = useAddSimilarProject(sessionId);
  const extractLessonsMutation = useExtractLessons(sessionId);
  const synthesizeMutation = useSynthesizeResearch(sessionId);

  const handleAddProject = async () => {
    if (!projectName.trim() || !projectUrl.trim()) {
      toast.error('Please enter project name and URL');
      return;
    }

    const project: SimilarProject = {
      name: projectName.trim(),
      url: projectUrl.trim(),
      positive_aspects: positiveAspects
        .split('\n')
        .map(a => a.trim())
        .filter(a => a.length > 0),
      negative_aspects: negativeAspects
        .split('\n')
        .map(a => a.trim())
        .filter(a => a.length > 0),
      patterns_to_adopt: patternsToAdopt
        .split('\n')
        .map(p => p.trim())
        .filter(p => p.length > 0),
    };

    try {
      await addProjectMutation.mutateAsync(project);
      toast.success('Similar project added successfully!');
      resetForm();
      setShowAddForm(false);
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error';
      toast.error('Failed to add project', { description: errorMessage });
    }
  };

  const handleExtractLessons = async (projectName: string) => {
    try {
      toast.info('Extracting lessons...', { duration: 2000 });
      setSelectedProjectForLessons(projectName);
      const lessons = await extractLessonsMutation.mutateAsync({ projectName });
      setLessonsResult(lessons);
      toast.success(`Extracted ${lessons.length} lessons!`);
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error';
      toast.error('Failed to extract lessons', { description: errorMessage });
      setSelectedProjectForLessons(null);
    }
  };

  const handleSynthesizeResearch = async () => {
    try {
      toast.info('Synthesizing research...', { duration: 2000 });
      const synthesis = await synthesizeMutation.mutateAsync();
      setSynthesisResult(synthesis);
      toast.success('Research synthesis complete!');
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error';
      toast.error('Failed to synthesize research', { description: errorMessage });
    }
  };

  const resetForm = () => {
    setProjectName('');
    setProjectUrl('');
    setPositiveAspects('');
    setNegativeAspects('');
    setPatternsToAdopt('');
  };

  const getPriorityColor = (priority: string) => {
    switch (priority) {
      case 'high':
        return 'bg-red-100 text-red-800 border-red-200';
      case 'medium':
        return 'bg-yellow-100 text-yellow-800 border-yellow-200';
      case 'low':
        return 'bg-green-100 text-green-800 border-green-200';
      default:
        return 'bg-gray-100 text-gray-800 border-gray-200';
    }
  };

  return (
    <div className="space-y-6">
      {/* Similar Projects List */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle className="flex items-center gap-2">
                <BookOpen className="h-5 w-5" />
                Similar Projects
              </CardTitle>
              <CardDescription>
                Track projects that inspire or inform your product design
              </CardDescription>
            </div>
            <Button
              onClick={() => setShowAddForm(!showAddForm)}
              variant={showAddForm ? 'outline' : 'default'}
            >
              <Plus className="mr-2 h-4 w-4" />
              {showAddForm ? 'Cancel' : 'Add Project'}
            </Button>
          </div>
        </CardHeader>
        <CardContent className="space-y-4">
          {showAddForm && (
            <Card className="border-2 border-dashed">
              <CardContent className="pt-4 space-y-4">
                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <Label htmlFor="project-name">Project Name</Label>
                    <Input
                      id="project-name"
                      placeholder="Example: Notion"
                      value={projectName}
                      onChange={e => setProjectName(e.target.value)}
                    />
                  </div>
                  <div>
                    <Label htmlFor="project-url">Project URL</Label>
                    <Input
                      id="project-url"
                      placeholder="https://notion.so"
                      value={projectUrl}
                      onChange={e => setProjectUrl(e.target.value)}
                    />
                  </div>
                </div>

                <div>
                  <Label htmlFor="positive-aspects">Positive Aspects (one per line)</Label>
                  <Textarea
                    id="positive-aspects"
                    placeholder="Great onboarding flow&#10;Intuitive drag-and-drop&#10;..."
                    value={positiveAspects}
                    onChange={e => setPositiveAspects(e.target.value)}
                    rows={3}
                  />
                </div>

                <div>
                  <Label htmlFor="negative-aspects">Negative Aspects (one per line)</Label>
                  <Textarea
                    id="negative-aspects"
                    placeholder="Steep learning curve&#10;Performance issues&#10;..."
                    value={negativeAspects}
                    onChange={e => setNegativeAspects(e.target.value)}
                    rows={3}
                  />
                </div>

                <div>
                  <Label htmlFor="patterns-to-adopt">Patterns to Adopt (one per line)</Label>
                  <Textarea
                    id="patterns-to-adopt"
                    placeholder="Block-based editor&#10;Workspace organization&#10;..."
                    value={patternsToAdopt}
                    onChange={e => setPatternsToAdopt(e.target.value)}
                    rows={3}
                  />
                </div>

                <div className="flex gap-2">
                  <Button
                    onClick={handleAddProject}
                    disabled={addProjectMutation.isPending}
                    className="flex-1"
                  >
                    {addProjectMutation.isPending ? (
                      <>
                        <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                        Adding...
                      </>
                    ) : (
                      'Add Project'
                    )}
                  </Button>
                  <Button onClick={() => setShowAddForm(false)} variant="outline">
                    Cancel
                  </Button>
                </div>
              </CardContent>
            </Card>
          )}

          {projectsLoading && (
            <div className="flex items-center gap-2 text-sm text-muted-foreground">
              <Loader2 className="h-4 w-4 animate-spin" />
              Loading similar projects...
            </div>
          )}

          {similarProjects && similarProjects.length > 0 && (
            <div className="space-y-3">
              {similarProjects.map((project, idx) => (
                <SimilarProjectCard
                  key={idx}
                  project={project}
                  onExtractLessons={handleExtractLessons}
                  isExtracting={
                    extractLessonsMutation.isPending &&
                    selectedProjectForLessons === project.name
                  }
                />
              ))}
            </div>
          )}

          {!projectsLoading && (!similarProjects || similarProjects.length === 0) && (
            <Alert>
              <AlertCircle className="h-4 w-4" />
              <AlertDescription>
                No similar projects added yet. Click "Add Project" to get started.
              </AlertDescription>
            </Alert>
          )}
        </CardContent>
      </Card>

      {/* Extracted Lessons */}
      {lessonsResult && lessonsResult.length > 0 && (
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Lightbulb className="h-5 w-5" />
              Extracted Lessons ({lessonsResult.length})
            </CardTitle>
            <CardDescription>Key insights from {selectedProjectForLessons}</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-2">
              {lessonsResult.map((lesson, idx) => (
                <Card key={idx}>
                  <CardContent className="pt-4">
                    <div className="flex items-start gap-2">
                      <Target className="h-4 w-4 mt-0.5 flex-shrink-0 text-blue-500" />
                      <div className="flex-1">
                        <div className="flex items-center gap-2 mb-1">
                          <Badge variant="outline">{lesson.category}</Badge>
                          <Badge className={getPriorityColor(lesson.priority)}>
                            {lesson.priority}
                          </Badge>
                        </div>
                        <p className="text-sm font-medium mb-1">{lesson.insight}</p>
                        <p className="text-sm text-muted-foreground">
                          <strong>Application:</strong> {lesson.application}
                        </p>
                      </div>
                    </div>
                  </CardContent>
                </Card>
              ))}
            </div>
          </CardContent>
        </Card>
      )}

      {/* Research Synthesis */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Target className="h-5 w-5" />
            Research Synthesis
          </CardTitle>
          <CardDescription>
            Synthesize all competitor analysis and similar project research
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <Button
            onClick={handleSynthesizeResearch}
            disabled={synthesizeMutation.isPending}
            className="w-full"
          >
            {synthesizeMutation.isPending ? (
              <>
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                Synthesizing...
              </>
            ) : (
              <>
                <Target className="mr-2 h-4 w-4" />
                Synthesize Research
              </>
            )}
          </Button>

          {synthesisResult && (
            <div className="space-y-4">
              <Separator />

              <div>
                <h4 className="text-sm font-medium mb-2">Key Findings</h4>
                <ul className="space-y-1">
                  {synthesisResult.key_findings.map((finding, idx) => (
                    <li key={idx} className="text-sm text-muted-foreground flex items-start gap-2">
                      <Lightbulb className="h-3 w-3 mt-0.5 flex-shrink-0" />
                      <span>{finding}</span>
                    </li>
                  ))}
                </ul>
              </div>

              <div>
                <h4 className="text-sm font-medium mb-2">Market Position</h4>
                <p className="text-sm text-muted-foreground">{synthesisResult.market_position}</p>
              </div>

              <div>
                <h4 className="text-sm font-medium mb-2">Differentiators</h4>
                <div className="flex flex-wrap gap-2">
                  {synthesisResult.differentiators.map((diff, idx) => (
                    <Badge key={idx} variant="secondary">
                      {diff}
                    </Badge>
                  ))}
                </div>
              </div>

              <div>
                <h4 className="text-sm font-medium mb-2">Risks</h4>
                <ul className="space-y-1">
                  {synthesisResult.risks.map((risk, idx) => (
                    <li key={idx} className="text-sm text-muted-foreground flex items-start gap-2">
                      <AlertCircle className="h-3 w-3 mt-0.5 flex-shrink-0 text-orange-500" />
                      <span>{risk}</span>
                    </li>
                  ))}
                </ul>
              </div>

              <div>
                <h4 className="text-sm font-medium mb-2">Recommendations</h4>
                <ul className="space-y-1">
                  {synthesisResult.recommendations.map((rec, idx) => (
                    <li key={idx} className="text-sm text-muted-foreground flex items-start gap-2">
                      <Target className="h-3 w-3 mt-0.5 flex-shrink-0 text-blue-500" />
                      <span>{rec}</span>
                    </li>
                  ))}
                </ul>
              </div>
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}

function SimilarProjectCard({
  project,
  onExtractLessons,
  isExtracting,
}: {
  project: SimilarProject;
  onExtractLessons: (projectName: string) => void;
  isExtracting: boolean;
}) {
  return (
    <Card>
      <CardContent className="pt-4">
        <div className="space-y-3">
          <div className="flex items-start justify-between">
            <div>
              <h5 className="text-sm font-medium">{project.name}</h5>
              <a
                href={project.url}
                target="_blank"
                rel="noopener noreferrer"
                className="text-xs text-blue-500 hover:underline flex items-center gap-1"
              >
                {project.url}
                <ExternalLink className="h-3 w-3" />
              </a>
            </div>
            <Button
              size="sm"
              variant="outline"
              onClick={() => onExtractLessons(project.name)}
              disabled={isExtracting}
            >
              {isExtracting ? (
                <>
                  <Loader2 className="mr-2 h-3 w-3 animate-spin" />
                  Extracting...
                </>
              ) : (
                <>
                  <Lightbulb className="mr-2 h-3 w-3" />
                  Extract Lessons
                </>
              )}
            </Button>
          </div>

          {project.positive_aspects.length > 0 && (
            <div>
              <div className="flex items-center gap-2 mb-1">
                <ThumbsUp className="h-3 w-3 text-green-500" />
                <span className="text-xs font-medium">Positive Aspects</span>
              </div>
              <ul className="space-y-1">
                {project.positive_aspects.map((aspect, idx) => (
                  <li key={idx} className="text-xs text-muted-foreground flex items-start gap-1">
                    <span className="text-green-500">•</span>
                    <span>{aspect}</span>
                  </li>
                ))}
              </ul>
            </div>
          )}

          {project.negative_aspects.length > 0 && (
            <div>
              <div className="flex items-center gap-2 mb-1">
                <ThumbsDown className="h-3 w-3 text-orange-500" />
                <span className="text-xs font-medium">Negative Aspects</span>
              </div>
              <ul className="space-y-1">
                {project.negative_aspects.map((aspect, idx) => (
                  <li key={idx} className="text-xs text-muted-foreground flex items-start gap-1">
                    <span className="text-orange-500">•</span>
                    <span>{aspect}</span>
                  </li>
                ))}
              </ul>
            </div>
          )}

          {project.patterns_to_adopt.length > 0 && (
            <div>
              <span className="text-xs font-medium">Patterns to Adopt</span>
              <div className="flex flex-wrap gap-1 mt-1">
                {project.patterns_to_adopt.map((pattern, idx) => (
                  <Badge key={idx} variant="secondary" className="text-xs">
                    {pattern}
                  </Badge>
                ))}
              </div>
            </div>
          )}
        </div>
      </CardContent>
    </Card>
  );
}
