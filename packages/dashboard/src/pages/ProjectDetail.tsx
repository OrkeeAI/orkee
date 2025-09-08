import { useState } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { 
  ArrowLeft, 
  Folder, 
  Calendar, 
  Tag, 
  Settings,
  Edit,
  GitBranch,
  Clock,
  CheckCircle,
  AlertCircle,
  Circle,
  X,
  Info,
  AlertTriangle,
  Trash2,
  Github,
  Play
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Separator } from '@/components/ui/separator';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { ProjectEditDialog } from '@/components/ProjectEditDialog';
import { ProjectDeleteDialog } from '@/components/ProjectDeleteDialog';
import { PreviewPanel } from '@/components/preview';
import { useProject } from '@/hooks/useProjects';

export function ProjectDetail() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  
  const [showEditDialog, setShowEditDialog] = useState(false);
  const [showDeleteDialog, setShowDeleteDialog] = useState(false);

  // Use React Query to fetch project data
  const { data: project, isLoading, error, isError } = useProject(id!);

  const handleProjectUpdated = () => {
    setShowEditDialog(false);
    // React Query will automatically update the cache
  };

  const handleProjectDeleted = () => {
    navigate('/projects');
  };

  const getPriorityColor = (priority: string | undefined) => {
    switch (priority) {
      case 'high':
        return 'destructive';
      case 'medium':
        return 'default';
      case 'low':
        return 'secondary';
      default:
        return 'outline';
    }
  };

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric'
    });
  };

  if (isLoading) {
    return (
      <div className="flex flex-1 items-center justify-center">
        <div className="text-center">
          <div className="h-8 w-8 animate-spin rounded-full border-2 border-primary border-t-transparent mx-auto mb-4" />
          <p className="text-muted-foreground">Loading project details...</p>
        </div>
      </div>
    );
  }

  if (isError || !project) {
    return (
      <div className="flex flex-1 flex-col gap-4 p-4">
        <div className="flex flex-1 items-center justify-center">
          <div className="text-center max-w-md">
            <AlertCircle className="h-12 w-12 mx-auto mb-4 text-destructive" />
            <h2 className="text-lg font-semibold mb-2">Unable to Load Project</h2>
            <p className="text-muted-foreground mb-4">{error?.message || 'Project not found'}</p>
            <Button variant="outline" onClick={() => navigate('/projects')}>
              <ArrowLeft className="mr-2 h-4 w-4" />
              Back to Projects
            </Button>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="flex flex-1 flex-col gap-4 p-4">
      {/* Project Header */}
      <div className="flex flex-col lg:flex-row lg:items-start lg:justify-between gap-4">
        <div className="space-y-2">
          <div className="flex flex-col sm:flex-row sm:items-center gap-2 sm:gap-3">
            <h1 className="text-2xl sm:text-3xl font-bold tracking-tight break-words">{project.name}</h1>
            <div className="flex items-center gap-2">
              <Badge variant={project.status === 'active' ? 'default' : 'secondary'}>
                {project.status}
              </Badge>
              {project.priority && (
                <Badge variant={getPriorityColor(project.priority)}>
                  {project.priority}
                </Badge>
              )}
            </div>
          </div>
          {project.description && (
            <p className="text-muted-foreground">{project.description}</p>
          )}
          <div className="flex flex-col sm:flex-row sm:items-center gap-2 sm:gap-4 text-sm text-muted-foreground">
            {project.gitRepository && (
              <div className="flex items-center gap-1">
                <Github className="h-4 w-4 shrink-0" />
                <span className="text-xs truncate">{project.gitRepository.owner}/{project.gitRepository.repo}</span>
              </div>
            )}
            <div className="flex items-center gap-1 min-w-0">
              <Folder className="h-4 w-4 shrink-0" />
              <code className="text-xs truncate">{project.projectRoot}</code>
            </div>
            <div className="flex items-center gap-1">
              <Calendar className="h-4 w-4 shrink-0" />
              <span className="text-xs whitespace-nowrap">Created {formatDate(project.createdAt)}</span>
            </div>
          </div>
        </div>
        
        <div className="flex flex-col sm:flex-row sm:flex-wrap gap-2 w-full sm:w-auto">
          <Button variant="outline" onClick={() => setShowEditDialog(true)} className="sm:w-auto w-full">
            <Edit className="mr-2 h-4 w-4" />
            <span className="sm:inline">Edit</span>
          </Button>
          <Button variant="outline" onClick={() => navigate('/projects')} className="sm:w-auto w-full">
            <ArrowLeft className="mr-2 h-4 w-4" />
            <span className="sm:inline">Back</span>
          </Button>
        </div>
      </div>

      <Separator />

      {/* Tabbed Content */}
      <Tabs defaultValue="overview" className="space-y-4">
        <TabsList className="grid w-full grid-cols-3">
          <TabsTrigger value="overview" className="flex items-center gap-2">
            <Info className="h-4 w-4" />
            Overview
          </TabsTrigger>
          <TabsTrigger value="preview" className="flex items-center gap-2">
            <Play className="h-4 w-4" />
            Preview
          </TabsTrigger>
          <TabsTrigger value="settings" className="flex items-center gap-2">
            <Settings className="h-4 w-4" />
            Settings
          </TabsTrigger>
        </TabsList>

        <TabsContent value="overview" className="space-y-4">
          {/* Project Stats */}
          <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
            {/* Status Card */}
            <Card>
              <CardHeader className="pb-2">
                <CardTitle className="text-sm font-medium">Status</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold capitalize">{project.status}</div>
                <p className="text-xs text-muted-foreground mt-1">
                  {project.status === 'active' ? 'Currently active' : 'Archived project'}
                </p>
              </CardContent>
            </Card>

            {/* Priority Card */}
            <Card>
              <CardHeader className="pb-2">
                <CardTitle className="text-sm font-medium">Priority</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="flex items-center gap-2">
                  <span className="text-2xl font-bold capitalize">{project.priority || 'Medium'}</span>
                  <Badge variant={getPriorityColor(project.priority)}>
                    {project.priority ? project.priority[0].toUpperCase() : 'M'}
                  </Badge>
                </div>
                <p className="text-xs text-muted-foreground mt-1">
                  Project priority level
                </p>
              </CardContent>
            </Card>

            {/* Task Source Card */}
            <Card>
              <CardHeader className="pb-2">
                <CardTitle className="text-sm font-medium">Task Source</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold capitalize">{project.taskSource || 'Manual'}</div>
                <p className="text-xs text-muted-foreground mt-1">
                  {project.taskSource === 'taskmaster' ? 'Using Taskmaster' : 'Manual tasks'}
                </p>
              </CardContent>
            </Card>

            {/* Last Updated Card */}
            <Card>
              <CardHeader className="pb-2">
                <CardTitle className="text-sm font-medium">Last Updated</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold">
                  {new Date(project.updatedAt).toLocaleDateString('en-US', { month: 'short', day: 'numeric' })}
                </div>
                <p className="text-xs text-muted-foreground mt-1">
                  {new Date(project.updatedAt).toLocaleTimeString('en-US', { hour: '2-digit', minute: '2-digit' })}
                </p>
              </CardContent>
            </Card>
          </div>

          {/* Tags Card */}
          {project.tags && project.tags.length > 0 && (
            <Card>
              <CardHeader>
                <CardTitle className="text-sm font-medium">Tags</CardTitle>
                <CardDescription>Labels associated with this project</CardDescription>
              </CardHeader>
              <CardContent>
                <div className="flex flex-wrap gap-2">
                  {project.tags.map((tag) => (
                    <Badge key={tag} variant="secondary">
                      <Tag className="mr-1 h-3 w-3" />
                      {tag}
                    </Badge>
                  ))}
                </div>
              </CardContent>
            </Card>
          )}

          {/* Manual Tasks Summary */}
          {project.manualTasks && project.manualTasks.length > 0 && (
            <Card>
              <CardHeader>
                <CardTitle className="text-sm font-medium">Task Summary</CardTitle>
                <CardDescription>Manual tasks created in this project</CardDescription>
              </CardHeader>
              <CardContent>
                <div className="space-y-3">
                  <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-4">
                    {(() => {
                      const stats = {
                        pending: project.manualTasks?.filter(t => t.status === 'pending').length || 0,
                        inProgress: project.manualTasks?.filter(t => t.status === 'in-progress').length || 0,
                        review: project.manualTasks?.filter(t => t.status === 'review').length || 0,
                        done: project.manualTasks?.filter(t => t.status === 'done').length || 0,
                        deferred: project.manualTasks?.filter(t => t.status === 'deferred').length || 0,
                        cancelled: project.manualTasks?.filter(t => t.status === 'cancelled').length || 0,
                        total: project.manualTasks?.length || 0
                      };
                      
                      return (
                        <>
                          <div className="space-y-1">
                            <div className="flex items-center gap-1">
                              <Circle className="h-3 w-3 text-gray-600" />
                              <p className="text-xs text-muted-foreground">Pending</p>
                            </div>
                            <p className="text-xl font-bold">{stats.pending}</p>
                          </div>
                          <div className="space-y-1">
                            <div className="flex items-center gap-1">
                              <Clock className="h-3 w-3 text-blue-600" />
                              <p className="text-xs text-muted-foreground">In Progress</p>
                            </div>
                            <p className="text-xl font-bold text-blue-600">{stats.inProgress}</p>
                          </div>
                          <div className="space-y-1">
                            <div className="flex items-center gap-1">
                              <AlertCircle className="h-3 w-3 text-purple-600" />
                              <p className="text-xs text-muted-foreground">Review</p>
                            </div>
                            <p className="text-xl font-bold text-purple-600">{stats.review}</p>
                          </div>
                          <div className="space-y-1">
                            <div className="flex items-center gap-1">
                              <CheckCircle className="h-3 w-3 text-green-600" />
                              <p className="text-xs text-muted-foreground">Completed</p>
                            </div>
                            <p className="text-xl font-bold text-green-600">{stats.done}</p>
                          </div>
                          <div className="space-y-1">
                            <div className="flex items-center gap-1">
                              <Clock className="h-3 w-3 text-yellow-600" />
                              <p className="text-xs text-muted-foreground">Deferred</p>
                            </div>
                            <p className="text-xl font-bold text-yellow-600">{stats.deferred}</p>
                          </div>
                          <div className="space-y-1">
                            <div className="flex items-center gap-1">
                              <X className="h-3 w-3 text-red-600" />
                              <p className="text-xs text-muted-foreground">Cancelled</p>
                            </div>
                            <p className="text-xl font-bold text-red-600">{stats.cancelled}</p>
                          </div>
                          {stats.total > 0 && (
                            <div className="col-span-full">
                              <Separator className="my-3" />
                              <div className="space-y-2">
                                <div className="flex justify-between text-xs text-muted-foreground">
                                  <span>Progress</span>
                                  <span>{Math.round((stats.done / stats.total) * 100)}% Complete</span>
                                </div>
                                <div className="h-2 bg-secondary rounded-full overflow-hidden">
                                  <div 
                                    className="h-full bg-green-600 transition-all duration-300"
                                    style={{ width: `${(stats.done / stats.total) * 100}%` }}
                                  />
                                </div>
                              </div>
                            </div>
                          )}
                        </>
                      );
                    })()}
                  </div>
                </div>
              </CardContent>
            </Card>
          )}

          {/* Recent Activity */}
          <Card>
            <CardHeader>
              <CardTitle className="text-sm font-medium">Recent Activity</CardTitle>
              <CardDescription>Last activity for this project</CardDescription>
            </CardHeader>
            <CardContent>
              <div className="text-center py-4 text-sm text-muted-foreground">
                No recent activity
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="preview" className="space-y-4">
          <PreviewPanel projectId={project.id} projectName={project.name} />
        </TabsContent>

        <TabsContent value="settings" className="space-y-4">
          {/* Task Management Settings */}
          <Card>
            <CardHeader>
              <CardTitle className="text-sm font-medium">Task Management</CardTitle>
              <CardDescription>Task configuration for this project</CardDescription>
            </CardHeader>
            <CardContent className="space-y-3">
              <div>
                <div className="text-sm text-muted-foreground mb-1">Task Source</div>
                <div className="flex items-center gap-2">
                  <Badge variant="outline">
                    {project.taskSource === 'taskmaster' ? 'Taskmaster' : 'Manual Tasks'}
                  </Badge>
                  <span className="text-sm text-muted-foreground">
                    {project.taskSource === 'taskmaster' 
                      ? '.taskmaster folder detected'
                      : 'Manual task management'}
                  </span>
                </div>
              </div>
              {project.taskSource === 'manual' && (
                <div>
                  <div className="text-sm text-muted-foreground mb-1">Manual Tasks</div>
                  <p className="text-sm">{project.manualTasks?.length || 0} tasks created</p>
                </div>
              )}
            </CardContent>
          </Card>

          {/* Git Repository */}
          <Card>
            <CardHeader>
              <CardTitle className="text-sm font-medium">Git Repository</CardTitle>
              <CardDescription>Source control integration and status</CardDescription>
            </CardHeader>
            <CardContent className="space-y-3">
              {project.gitRepository ? (
                <>
                  <div>
                    <div className="text-sm text-muted-foreground mb-1">Repository</div>
                    <div className="flex items-center gap-2">
                      <Badge variant="outline" className="text-green-600 border-green-200">
                        <GitBranch className="mr-1 h-3 w-3" />
                        Git Repository
                      </Badge>
                      <code className="text-sm bg-muted px-2 py-1 rounded">
                        {project.gitRepository.owner}/{project.gitRepository.repo}
                      </code>
                    </div>
                  </div>
                  
                  {project.gitRepository.branch && (
                    <div>
                      <div className="text-sm text-muted-foreground mb-1">Branch</div>
                      <code className="text-sm bg-muted px-2 py-1 rounded">{project.gitRepository.branch}</code>
                    </div>
                  )}
                  
                  <div>
                    <div className="text-sm text-muted-foreground mb-1">Remote URL</div>
                    <code className="text-sm bg-muted px-2 py-1 rounded break-all">{project.gitRepository.url}</code>
                  </div>
                </>
              ) : (
                <>
                  <div>
                    <div className="text-sm text-muted-foreground mb-1">Repository Status</div>
                    <Badge variant="outline" className="text-yellow-600 border-yellow-200">
                      No Git Repository
                    </Badge>
                  </div>
                  
                  <div className="p-3 bg-muted rounded-md">
                    <p className="text-xs text-muted-foreground">
                      💡 Initialize a Git repository to enable version control features.
                    </p>
                  </div>
                </>
              )}
            </CardContent>
          </Card>

          {/* Project Location */}
          <Card>
            <CardHeader>
              <CardTitle className="text-sm font-medium">Project Location</CardTitle>
              <CardDescription>File system path and configuration</CardDescription>
            </CardHeader>
            <CardContent className="space-y-3">
              <div>
                <div className="text-sm text-muted-foreground mb-1">Project Root</div>
                <code className="text-sm bg-muted px-2 py-1 rounded">{project.projectRoot}</code>
              </div>
            </CardContent>
          </Card>

          {/* Project Settings */}
          <Card>
            <CardHeader>
              <CardTitle>Project Details</CardTitle>
              <CardDescription>Project information and metadata</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="grid gap-2">
                <div className="flex justify-between items-center py-2">
                  <span className="text-sm">Project ID</span>
                  <code className="text-xs bg-muted px-2 py-1 rounded">{project.id}</code>
                </div>
                <div className="flex justify-between items-center py-2">
                  <span className="text-sm">Created</span>
                  <span className="text-sm text-muted-foreground">
                    {new Date(project.createdAt).toLocaleString()}
                  </span>
                </div>
                <div className="flex justify-between items-center py-2">
                  <span className="text-sm">Last Modified</span>
                  <span className="text-sm text-muted-foreground">
                    {new Date(project.updatedAt).toLocaleString()}
                  </span>
                </div>
              </div>
            </CardContent>
          </Card>

          {/* Danger Zone */}
          <Card className="border-destructive/20">
            <CardHeader>
              <CardTitle className="text-destructive flex items-center gap-2">
                <AlertTriangle className="h-5 w-5" />
                Danger Zone
              </CardTitle>
              <CardDescription>
                Irreversible and destructive actions for this project
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="p-4 bg-destructive/5 border border-destructive/20 rounded-md">
                <div className="space-y-2">
                  <h4 className="font-medium text-destructive">Delete Project</h4>
                  <p className="text-sm text-muted-foreground">
                    Permanently delete this project and all of its data. This action cannot be undone.
                  </p>
                  <div className="pt-2">
                    <Button 
                      variant="destructive" 
                      onClick={() => setShowDeleteDialog(true)}
                      className="w-full sm:w-auto"
                    >
                      <Trash2 className="mr-2 h-4 w-4" />
                      Delete Project
                    </Button>
                  </div>
                </div>
              </div>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>

      {/* Dialogs */}
      <ProjectEditDialog
        project={project}
        open={showEditDialog}
        onOpenChange={setShowEditDialog}
        onProjectUpdated={handleProjectUpdated}
      />

      <ProjectDeleteDialog
        project={project}
        open={showDeleteDialog}
        onOpenChange={setShowDeleteDialog}
        onProjectDeleted={handleProjectDeleted}
      />
    </div>
  );
}