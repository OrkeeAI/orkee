// ABOUTME: Detailed epic view with markdown rendering, architecture decisions, and metadata
// ABOUTME: Displays technical approach, dependencies, success criteria, and task progress

import { useState } from 'react';
import { FileText, Edit, ExternalLink, Target, Package, CheckCircle2, AlertCircle, Calendar, TrendingUp } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Progress } from '@/components/ui/progress';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { MarkdownRenderer } from '@/components/MarkdownRenderer';
import type { Epic, EpicStatus, EpicComplexity, DecompositionResult, WorkAnalysis } from '@/services/epics';
import { TaskBreakdown } from './TaskBreakdown';
import { DependencyView } from './DependencyView';
import { WorkStreamAnalysis } from './WorkStreamAnalysis';
import { GitHubSyncStatus } from './GitHubSyncStatus';

interface EpicDetailProps {
  epic: Epic;
  onEdit?: () => void;
  onGenerateTasks?: () => void;
  onSyncSuccess?: () => void;
  decompositionResult?: DecompositionResult | null;
  workAnalysis?: WorkAnalysis | null;
}

export function EpicDetail({ epic, onEdit, onGenerateTasks, onSyncSuccess, decompositionResult, workAnalysis }: EpicDetailProps) {
  const [activeTab, setActiveTab] = useState('overview');

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'long',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  const getStatusBadge = (status: EpicStatus) => {
    const variants: Record<EpicStatus, { variant: 'default' | 'secondary' | 'outline' | 'destructive'; label: string }> = {
      draft: { variant: 'outline', label: 'Draft' },
      ready: { variant: 'secondary', label: 'Ready' },
      in_progress: { variant: 'default', label: 'In Progress' },
      blocked: { variant: 'destructive', label: 'Blocked' },
      completed: { variant: 'default', label: 'Completed' },
      cancelled: { variant: 'outline', label: 'Cancelled' },
    };
    const { variant, label} = variants[status];
    return <Badge variant={variant}>{label}</Badge>;
  };

  const getComplexityColor = (complexity?: EpicComplexity) => {
    if (!complexity) return 'text-muted-foreground';
    const colors: Record<EpicComplexity, string> = {
      low: 'text-green-600',
      medium: 'text-yellow-600',
      high: 'text-orange-600',
      very_high: 'text-red-600',
    };
    return colors[complexity];
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-start justify-between">
        <div className="flex-1">
          <div className="flex items-center gap-3 mb-2">
            <h1 className="text-3xl font-bold">{epic.name}</h1>
            {getStatusBadge(epic.status)}
          </div>
          <div className="flex flex-wrap gap-4 text-sm text-muted-foreground">
            <div className="flex items-center gap-1">
              <Calendar className="h-4 w-4" />
              <span>Created {formatDate(epic.createdAt)}</span>
            </div>
            {epic.updatedAt !== epic.createdAt && (
              <div className="flex items-center gap-1">
                <Edit className="h-4 w-4" />
                <span>Updated {formatDate(epic.updatedAt)}</span>
              </div>
            )}
            {epic.githubIssueUrl && (
              <a
                href={epic.githubIssueUrl}
                target="_blank"
                rel="noopener noreferrer"
                className="flex items-center gap-1 hover:text-primary"
              >
                <ExternalLink className="h-4 w-4" />
                <span>GitHub #{epic.githubIssueNumber}</span>
              </a>
            )}
          </div>
        </div>
        <div className="flex gap-2">
          {onEdit && (
            <Button variant="outline" onClick={onEdit}>
              <Edit className="h-4 w-4 mr-2" />
              Edit
            </Button>
          )}
          {onGenerateTasks && (
            <Button onClick={onGenerateTasks}>
              <FileText className="h-4 w-4 mr-2" />
              Generate Tasks
            </Button>
          )}
        </div>
      </div>

      {/* Progress Card */}
      <Card>
        <CardHeader>
          <CardTitle className="text-base">Progress</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <span className="text-sm font-medium">Completion</span>
              <span className="text-2xl font-bold">{epic.progressPercentage}%</span>
            </div>
            <Progress value={epic.progressPercentage} className="h-3" />
          </div>
          <div className="grid grid-cols-2 gap-4 pt-3">
            {epic.complexity && (
              <div className="space-y-1">
                <p className="text-sm text-muted-foreground">Complexity</p>
                <p className={`text-lg font-semibold capitalize ${getComplexityColor(epic.complexity)}`}>
                  {epic.complexity.replace('_', ' ')}
                </p>
              </div>
            )}
            {epic.estimatedEffort && (
              <div className="space-y-1">
                <p className="text-sm text-muted-foreground">Estimated Effort</p>
                <p className="text-lg font-semibold capitalize">{epic.estimatedEffort}</p>
              </div>
            )}
          </div>
        </CardContent>
      </Card>

      {/* Main Content Tabs */}
      <Tabs value={activeTab} onValueChange={setActiveTab}>
        <TabsList className="grid w-full grid-cols-8">
          <TabsTrigger value="overview">Overview</TabsTrigger>
          <TabsTrigger value="architecture">Architecture</TabsTrigger>
          <TabsTrigger value="dependencies">Dependencies</TabsTrigger>
          <TabsTrigger value="success">Success Criteria</TabsTrigger>
          <TabsTrigger value="tasks">Task Breakdown</TabsTrigger>
          <TabsTrigger value="deps">Dependencies</TabsTrigger>
          <TabsTrigger value="streams">Work Streams</TabsTrigger>
          <TabsTrigger value="github">GitHub Sync</TabsTrigger>
        </TabsList>

        <TabsContent value="overview" className="space-y-6">
          <Card>
            <CardHeader>
              <CardTitle>Overview</CardTitle>
            </CardHeader>
            <CardContent>
              <MarkdownRenderer content={epic.overviewMarkdown} />
            </CardContent>
          </Card>

          {epic.technicalApproach && (
            <Card>
              <CardHeader>
                <CardTitle>Technical Approach</CardTitle>
              </CardHeader>
              <CardContent>
                <MarkdownRenderer content={epic.technicalApproach} />
              </CardContent>
            </Card>
          )}

          {epic.implementationStrategy && (
            <Card>
              <CardHeader>
                <CardTitle>Implementation Strategy</CardTitle>
              </CardHeader>
              <CardContent>
                <MarkdownRenderer content={epic.implementationStrategy} />
              </CardContent>
            </Card>
          )}

          {epic.taskCategories && epic.taskCategories.length > 0 && (
            <Card>
              <CardHeader>
                <CardTitle>Task Categories</CardTitle>
                <CardDescription>
                  Organizational structure for task decomposition
                </CardDescription>
              </CardHeader>
              <CardContent>
                <div className="flex flex-wrap gap-2">
                  {epic.taskCategories.map((category, idx) => (
                    <Badge key={idx} variant="secondary">
                      {category}
                    </Badge>
                  ))}
                </div>
              </CardContent>
            </Card>
          )}
        </TabsContent>

        <TabsContent value="architecture" className="space-y-6">
          {epic.architectureDecisions && epic.architectureDecisions.length > 0 ? (
            <div className="space-y-4">
              {epic.architectureDecisions.map((decision, idx) => (
                <Card key={idx}>
                  <CardHeader>
                    <CardTitle className="text-lg">{decision.decision}</CardTitle>
                  </CardHeader>
                  <CardContent className="space-y-4">
                    <div>
                      <h4 className="text-sm font-semibold mb-2 flex items-center gap-2">
                        <CheckCircle2 className="h-4 w-4" />
                        Rationale
                      </h4>
                      <p className="text-sm text-muted-foreground">{decision.rationale}</p>
                    </div>
                    {decision.alternatives && decision.alternatives.length > 0 && (
                      <div>
                        <h4 className="text-sm font-semibold mb-2 flex items-center gap-2">
                          <TrendingUp className="h-4 w-4" />
                          Alternatives Considered
                        </h4>
                        <ul className="list-disc list-inside space-y-1 text-sm text-muted-foreground">
                          {decision.alternatives.map((alt, altIdx) => (
                            <li key={altIdx}>{alt}</li>
                          ))}
                        </ul>
                      </div>
                    )}
                    {decision.tradeoffs && (
                      <div>
                        <h4 className="text-sm font-semibold mb-2 flex items-center gap-2">
                          <AlertCircle className="h-4 w-4" />
                          Trade-offs
                        </h4>
                        <p className="text-sm text-muted-foreground">{decision.tradeoffs}</p>
                      </div>
                    )}
                  </CardContent>
                </Card>
              ))}
            </div>
          ) : (
            <Card>
              <CardContent className="flex flex-col items-center justify-center py-12">
                <FileText className="h-12 w-12 text-muted-foreground mb-4" />
                <p className="text-lg font-medium text-muted-foreground">
                  No architecture decisions documented
                </p>
              </CardContent>
            </Card>
          )}
        </TabsContent>

        <TabsContent value="dependencies" className="space-y-6">
          {epic.dependencies && epic.dependencies.length > 0 ? (
            <Card>
              <CardHeader>
                <CardTitle>External Dependencies</CardTitle>
                <CardDescription>Libraries, services, and APIs required</CardDescription>
              </CardHeader>
              <CardContent>
                <div className="space-y-4">
                  {epic.dependencies.map((dep, idx) => (
                    <div key={idx} className="flex items-start gap-3 pb-4 last:pb-0 border-b last:border-0">
                      <Package className="h-5 w-5 text-muted-foreground mt-0.5" />
                      <div className="flex-1">
                        <div className="flex items-center gap-2 mb-1">
                          <h4 className="font-semibold">{dep.name}</h4>
                          <Badge variant="outline" className="text-xs">
                            {dep.type}
                          </Badge>
                          {dep.version && (
                            <Badge variant="secondary" className="text-xs">
                              v{dep.version}
                            </Badge>
                          )}
                        </div>
                        <p className="text-sm text-muted-foreground">{dep.reason}</p>
                      </div>
                    </div>
                  ))}
                </div>
              </CardContent>
            </Card>
          ) : (
            <Card>
              <CardContent className="flex flex-col items-center justify-center py-12">
                <Package className="h-12 w-12 text-muted-foreground mb-4" />
                <p className="text-lg font-medium text-muted-foreground">
                  No external dependencies defined
                </p>
              </CardContent>
            </Card>
          )}
        </TabsContent>

        <TabsContent value="success" className="space-y-6">
          {epic.successCriteria && epic.successCriteria.length > 0 ? (
            <Card>
              <CardHeader>
                <CardTitle>Success Criteria</CardTitle>
                <CardDescription>Measurable outcomes for epic completion</CardDescription>
              </CardHeader>
              <CardContent>
                <div className="space-y-3">
                  {epic.successCriteria.map((criterion, idx) => (
                    <div key={idx} className="flex items-start gap-3 pb-3 last:pb-0 border-b last:border-0">
                      <Target className="h-5 w-5 text-muted-foreground mt-0.5" />
                      <div className="flex-1">
                        <p className="text-sm font-medium mb-1">{criterion.criterion}</p>
                        <div className="flex items-center gap-2">
                          <Badge variant={criterion.measurable ? 'default' : 'outline'} className="text-xs">
                            {criterion.measurable ? 'Measurable' : 'Qualitative'}
                          </Badge>
                          {criterion.target && (
                            <span className="text-xs text-muted-foreground">Target: {criterion.target}</span>
                          )}
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              </CardContent>
            </Card>
          ) : (
            <Card>
              <CardContent className="flex flex-col items-center justify-center py-12">
                <Target className="h-12 w-12 text-muted-foreground mb-4" />
                <p className="text-lg font-medium text-muted-foreground">
                  No success criteria defined
                </p>
              </CardContent>
            </Card>
          )}
        </TabsContent>

        {/* Task Breakdown Tab */}
        <TabsContent value="tasks" className="space-y-6">
          <TaskBreakdown
            epicId={epic.id}
            decompositionResult={decompositionResult || null}
          />
        </TabsContent>

        {/* Dependency Graph Tab */}
        <TabsContent value="deps" className="space-y-6">
          <DependencyView decompositionResult={decompositionResult || null} />
        </TabsContent>

        {/* Work Streams Tab */}
        <TabsContent value="streams" className="space-y-6">
          <WorkStreamAnalysis workAnalysis={workAnalysis || null} />
        </TabsContent>

        {/* GitHub Sync Tab */}
        <TabsContent value="github" className="space-y-6">
          <GitHubSyncStatus epic={epic} onSyncSuccess={onSyncSuccess} />
        </TabsContent>
      </Tabs>
    </div>
  );
}
