import { useState, useEffect, useMemo, memo } from 'react';
import { useNavigate } from 'react-router-dom';
import { toast } from 'sonner';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Input } from '@/components/ui/input';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { FolderOpen, Plus, Edit, Trash2, Search, LayoutGrid, List, GripVertical, GitBranch, Sparkles, Play, Square, Loader2 } from 'lucide-react';
import { AITestDialog } from '@/components/AITestDialog';
import {
  DndContext,
  closestCenter,
  KeyboardSensor,
  PointerSensor,
  useSensor,
  useSensors,
  DragEndEvent,
} from '@dnd-kit/core';
import {
  arrayMove,
  SortableContext,
  sortableKeyboardCoordinates,
  verticalListSortingStrategy,
} from '@dnd-kit/sortable';
import {
  useSortable,
} from '@dnd-kit/sortable';
import { CSS } from '@dnd-kit/utilities';
import { ProjectCreateDialog } from '@/components/ProjectCreateDialog';
import { ProjectEditDialog } from '@/components/ProjectEditDialog';
import { ProjectDeleteDialog } from '@/components/ProjectDeleteDialog';
import { GlobalSyncStatus } from '@/components/cloud/GlobalSyncStatus';
import { ProjectSyncBadge } from '@/components/cloud/ProjectSyncBadge';
import { useProjects, useUpdateProject, useSearchProjects } from '@/hooks/useProjects';
import { Project } from '@/services/projects';
import { previewService } from '@/services/preview';

type ViewType = 'card' | 'list';
type SortType = 'rank' | 'priority' | 'alpha';
type StatusFilter = 'planning' | 'building' | 'review' | 'launched' | 'on-hold' | 'archived';

// Helper function to get git repository info
const getRepositoryInfo = (project: Project): { owner: string; repo: string } | null => {
  // Use real git repository data from the backend
  if (project.gitRepository) {
    return {
      owner: project.gitRepository.owner,
      repo: project.gitRepository.repo,
    };
  }
  
  return null;
};

// Sortable Row Component
interface SortableRowProps {
  project: Project;
  onEdit: (project: Project) => void;
  onDelete: (project: Project) => void;
  onView: (project: Project) => void;
  formatDate: (dateString: string) => string;
  getPriorityColor: (priority: string) => string;
  isDevServerRunning: (project: Project) => boolean;
  onStartServer: (projectId: string) => void;
  onStopServer: (projectId: string) => void;
  isServerLoading: (projectId: string) => boolean;
}

const SortableRow = memo(function SortableRow({ project, onEdit, onDelete, onView, formatDate, getPriorityColor, isDevServerRunning, onStartServer, onStopServer, isServerLoading }: SortableRowProps) {
  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({ id: project.id });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.5 : 1,
  };

  return (
    <tr ref={setNodeRef} style={style} className="border-b hover:bg-muted/50">
      <td className="py-3 px-2 sm:px-4">
        <div className="flex items-center gap-1 sm:gap-2">
          <button
            className="cursor-grab active:cursor-grabbing p-1 hover:bg-muted rounded"
            {...attributes}
            {...listeners}
          >
            <GripVertical className="h-4 w-4 text-muted-foreground" />
          </button>
          <FolderOpen className="h-4 w-4 text-primary" />
          <button
            onClick={() => onView(project)}
            className="font-medium text-sm sm:text-base truncate hover:text-primary transition-colors text-left"
          >
            {project.name}
          </button>
        </div>
      </td>
      <td className="py-3 px-2 sm:px-4 hidden md:table-cell">
        <div className="flex items-center gap-2 text-sm text-muted-foreground">
          {(() => {
            const repoInfo = getRepositoryInfo(project);
            if (repoInfo) {
              return (
                <>
                  <GitBranch className="h-4 w-4" />
                  <span className="truncate max-w-xs">{repoInfo.owner}/{repoInfo.repo}</span>
                </>
              );
            } else {
              return (
                <>
                  <FolderOpen className="h-4 w-4" />
                  <span className="text-muted-foreground">No remote repository</span>
                </>
              );
            }
          })()}
        </div>
      </td>
      <td className="py-3 px-2 sm:px-4">
        <div className="flex items-center justify-center gap-2">
          <div className={`w-2 h-2 rounded-full ${
            isDevServerRunning(project) ? 'bg-green-500 dark:bg-green-400' : 'bg-muted-foreground/40'
          }`} />
          {isServerLoading(project.id) ? (
            <Loader2 className="h-4 w-4 animate-spin text-muted-foreground" aria-label="Server starting" />
          ) : isDevServerRunning(project) ? (
            <Button
              size="sm"
              variant="ghost"
              onClick={(e) => {
                e.stopPropagation();
                onStopServer(project.id);
              }}
              className="h-7 w-7 p-0"
              title="Stop dev server"
              aria-label="Stop dev server"
              disabled={isServerLoading(project.id)}
            >
              <Square className="h-3.5 w-3.5" />
            </Button>
          ) : (
            <Button
              size="sm"
              variant="ghost"
              onClick={(e) => {
                e.stopPropagation();
                onStartServer(project.id);
              }}
              className="h-7 w-7 p-0"
              title="Start dev server"
              aria-label="Start dev server"
              disabled={isServerLoading(project.id)}
            >
              <Play className="h-3.5 w-3.5" />
            </Button>
          )}
        </div>
      </td>
      <td className="py-3 px-2 sm:px-4">
        <Badge className={`${getPriorityColor(project.priority)} text-xs`} variant="secondary">
          {project.priority}
        </Badge>
      </td>
      <td className="py-3 px-2 sm:px-4 hidden sm:table-cell">
        <ProjectSyncBadge projectId={project.id} variant="compact" />
      </td>
      <td className="py-3 px-2 sm:px-4 hidden lg:table-cell">
        {project.tags && project.tags.length > 0 ? (
          <div className="flex gap-1">
            {project.tags.slice(0, 2).map((tag, index) => (
              <Badge key={index} variant="outline" className="text-xs">
                {tag}
              </Badge>
            ))}
            {project.tags.length > 2 && (
              <span className="text-xs text-muted-foreground">+{project.tags.length - 2}</span>
            )}
          </div>
        ) : (
          <span className="text-muted-foreground text-sm">—</span>
        )}
      </td>
      <td className="py-3 px-2 sm:px-4 text-sm text-muted-foreground hidden xl:table-cell">
        {formatDate(project.createdAt)}
      </td>
      <td className="py-3 px-2 sm:px-4">
        <div className="flex gap-1">
          <Button
            size="sm"
            variant="ghost"
            onClick={() => onEdit(project)}
            className="h-8 w-8 p-0"
          >
            <Edit className="h-3 w-3" />
          </Button>
          <Button
            size="sm"
            variant="ghost"
            onClick={() => onDelete(project)}
            className="h-8 w-8 p-0 text-red-600 hover:text-red-700"
          >
            <Trash2 className="h-3 w-3" />
          </Button>
        </div>
      </td>
    </tr>
  );
});

export function Projects() {
  const navigate = useNavigate();
  const [activeServers, setActiveServers] = useState<Set<string>>(new Set());
  const [loadingServers, setLoadingServers] = useState<Set<string>>(new Set());

  // Dialog states
  const [createDialogOpen, setCreateDialogOpen] = useState(false);
  const [editDialogOpen, setEditDialogOpen] = useState(false);
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const [aiTestDialogOpen, setAITestDialogOpen] = useState(false);
  const [selectedProject, setSelectedProject] = useState<Project | null>(null);
  
  // Filter and view states
  const [searchTerm, setSearchTerm] = useState('');
  const [sortBy, setSortBy] = useState<SortType>('rank');
  const [viewType, setViewType] = useState<ViewType>('list');
  const [statusFilter, setStatusFilter] = useState<StatusFilter>('planning');

  // React Query hooks
  const { data: allProjects = [], isLoading, error, isError } = useProjects();
  const { data: searchResults = [], isLoading: isSearching } = useSearchProjects(
    searchTerm, 
    searchTerm.length >= 2
  );
  const updateProjectMutation = useUpdateProject();

  // Drag and drop sensors
  const sensors = useSensors(
    useSensor(PointerSensor),
    useSensor(KeyboardSensor, {
      coordinateGetter: sortableKeyboardCoordinates,
    })
  );

  // Use search results if searching, otherwise use all projects
  const projects = searchTerm.length >= 2 ? searchResults : allProjects;
  const loading = isLoading || (searchTerm.length >= 2 && isSearching);

  const loadActiveServers = async () => {
    try {
      const activeServerIds = await previewService.getActiveServers();
      setActiveServers(new Set(activeServerIds));
    } catch (err) {
      console.error('Failed to load active servers:', err);
      setActiveServers(new Set());
    }
  };

  // Calculate project counts by status
  const projectCounts = useMemo(() => {
    const planning = allProjects.filter(project => project.status === 'planning').length;
    const building = allProjects.filter(project => project.status === 'building').length;
    const review = allProjects.filter(project => project.status === 'review').length;
    const launched = allProjects.filter(project => project.status === 'launched').length;
    const onHold = allProjects.filter(project => project.status === 'on-hold').length;
    const archived = allProjects.filter(project => project.status === 'archived').length;
    return { planning, building, review, launched, onHold, archived };
  }, [allProjects]);

  // Filter and sort projects
  const filteredAndSortedProjects = useMemo(() => {
    const filtered = projects.filter(project => project.status === statusFilter);

    const priorityOrder = { high: 3, medium: 2, low: 1 };
    
    filtered.sort((a, b) => {
      switch (sortBy) {
        case 'alpha':
          return a.name.localeCompare(b.name);
        case 'priority':
          return (priorityOrder[b.priority as keyof typeof priorityOrder] || 0) - 
                 (priorityOrder[a.priority as keyof typeof priorityOrder] || 0);
        case 'rank':
        default: {
          // Use rank if available, otherwise sort by priority then date
          if (a.rank !== undefined && b.rank !== undefined) {
            return a.rank - b.rank;
          }
          const priorityDiff = (priorityOrder[b.priority as keyof typeof priorityOrder] || 0) - 
                              (priorityOrder[a.priority as keyof typeof priorityOrder] || 0);
          if (priorityDiff !== 0) return priorityDiff;
          return new Date(b.updatedAt).getTime() - new Date(a.updatedAt).getTime();
        }
      }
    });

    return filtered;
  }, [projects, sortBy, statusFilter]);

  useEffect(() => {
    loadActiveServers();
    
    // Set up periodic refresh for active servers every 20 seconds
    const interval = setInterval(loadActiveServers, 20000);
    
    return () => clearInterval(interval);
  }, []);

  const handleViewProject = (project: Project) => {
    navigate(`/projects/${project.id}`);
  };

  const handleEditProject = (project: Project) => {
    setSelectedProject(project);
    setEditDialogOpen(true);
  };

  const handleDeleteProject = (project: Project) => {
    setSelectedProject(project);
    setDeleteDialogOpen(true);
  };

  // No need for manual reload callbacks - React Query handles cache updates automatically
  const handleProjectCreated = () => {};
  const handleProjectUpdated = () => {};
  const handleProjectDeleted = () => {};

  const handleDragEnd = async (event: DragEndEvent) => {
    const { active, over } = event;

    if (over && active.id !== over.id) {
      const oldIndex = filteredAndSortedProjects.findIndex(p => p.id === active.id);
      const newIndex = filteredAndSortedProjects.findIndex(p => p.id === over.id);
      
      const newProjects = arrayMove(filteredAndSortedProjects, oldIndex, newIndex);
      
      // Update ranks based on new order
      const updatedProjects = newProjects.map((project, index) => ({
        ...project,
        rank: index
      }));

      // React Query will handle optimistic updates through the mutation

      // Update rankings on server
      try {
        for (const project of updatedProjects) {
          await updateProjectMutation.mutateAsync({
            id: project.id,
            input: { rank: project.rank }
          });
        }
      } catch (err) {
        console.error('Failed to update project rankings:', err);
        // React Query will handle rollback through its error handling
      }
    }
  };

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString();
  };

  const getPriorityColor = (priority: string) => {
    switch (priority) {
      case 'high': return 'bg-red-500/10 text-red-600 dark:text-red-400';
      case 'medium': return 'bg-yellow-500/10 text-yellow-700 dark:text-yellow-400';
      case 'low': return 'bg-green-500/10 text-green-700 dark:text-green-400';
      default: return 'bg-muted text-muted-foreground';
    }
  };

  const isDevServerRunning = (project: Project) => {
    return activeServers.has(project.id);
  };

  const isServerLoading = (projectId: string) => {
    return loadingServers.has(projectId);
  };

  const addToSet = <T,>(setter: React.Dispatch<React.SetStateAction<Set<T>>>, value: T) => {
    setter(prev => {
      if (prev.has(value)) return prev;
      const newSet = new Set(prev);
      newSet.add(value);
      return newSet;
    });
  };

  const removeFromSet = <T,>(setter: React.Dispatch<React.SetStateAction<Set<T>>>, value: T) => {
    setter(prev => {
      if (!prev.has(value)) return prev;
      const newSet = new Set(prev);
      newSet.delete(value);
      return newSet;
    });
  };

  const handleStartServer = async (projectId: string) => {
    if (loadingServers.has(projectId)) return;

    addToSet(setLoadingServers, projectId);
    addToSet(setActiveServers, projectId);

    try {
      await previewService.startServer(projectId);
    } catch (err) {
      console.error('Failed to start server:', err);
      removeFromSet(setActiveServers, projectId);
      await loadActiveServers();
      toast.error('Failed to start dev server', {
        description: err instanceof Error ? err.message : 'An unexpected error occurred',
      });
    } finally {
      removeFromSet(setLoadingServers, projectId);
    }
  };

  const handleStopServer = async (projectId: string) => {
    if (loadingServers.has(projectId)) return;

    addToSet(setLoadingServers, projectId);
    removeFromSet(setActiveServers, projectId);

    try {
      await previewService.stopServer(projectId);
    } catch (err) {
      console.error('Failed to stop server:', err);
      addToSet(setActiveServers, projectId);
      await loadActiveServers();
      toast.error('Failed to stop dev server', {
        description: err instanceof Error ? err.message : 'An unexpected error occurred',
      });
    } finally {
      removeFromSet(setLoadingServers, projectId);
    }
  };

  if (loading) {
    return (
      <div className="space-y-6">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-3xl font-bold tracking-tight">Projects</h1>
            <p className="text-muted-foreground">
              Manage your development projects and their configurations.
            </p>
          </div>
        </div>
        <div className="flex items-center justify-center h-32">
          <div className="h-6 w-6 animate-spin rounded-full border-2 border-muted border-t-primary"></div>
          <span className="ml-2">Loading projects...</span>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
        <div>
          <h1 className="text-2xl sm:text-3xl font-bold tracking-tight">Projects</h1>
          <p className="text-sm sm:text-base text-muted-foreground">
            Manage your development projects and their configurations.
          </p>
        </div>
        
        <div className="flex flex-col gap-4 sm:flex-row sm:items-center lg:flex-col lg:items-end xl:flex-row">
          <div className="relative">
            <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-muted-foreground h-4 w-4" />
            <Input
              placeholder="Search..."
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              className="pl-10 w-full sm:w-64"
            />
          </div>
          
          <div className="flex items-center justify-between sm:gap-4">
            <div className="flex border rounded-md">
              <Button
                variant={sortBy === 'rank' ? 'default' : 'ghost'}
                size="sm"
                onClick={() => setSortBy('rank')}
                className="rounded-r-none text-xs px-2 sm:px-3 sm:text-sm"
              >
                Rank
              </Button>
              <Button
                variant={sortBy === 'priority' ? 'default' : 'ghost'}
                size="sm"
                onClick={() => setSortBy('priority')}
                className="rounded-none border-x text-xs px-2 sm:px-3 sm:text-sm"
              >
                Priority
              </Button>
              <Button
                variant={sortBy === 'alpha' ? 'default' : 'ghost'}
                size="sm"
                onClick={() => setSortBy('alpha')}
                className="rounded-l-none text-xs px-2 sm:px-3 sm:text-sm"
              >
                Alpha
              </Button>
            </div>
            
            <div className="flex border rounded-md">
              <Button
                variant={viewType === 'card' ? 'default' : 'ghost'}
                size="sm"
                onClick={() => setViewType('card')}
                className="rounded-r-none"
              >
                <LayoutGrid className="h-4 w-4" />
              </Button>
              <Button
                variant={viewType === 'list' ? 'default' : 'ghost'}
                size="sm"
                onClick={() => setViewType('list')}
                className="rounded-l-none"
              >
                <List className="h-4 w-4" />
              </Button>
            </div>

            <Button
              onClick={() => setAITestDialogOpen(true)}
              variant="outline"
              className="shrink-0"
            >
              <Sparkles className="mr-0 sm:mr-2 h-4 w-4" />
              <span className="hidden sm:inline">Test AI</span>
            </Button>

            <Button onClick={() => setCreateDialogOpen(true)} className="shrink-0">
              <Plus className="mr-0 sm:mr-2 h-4 w-4" />
              <span className="hidden sm:inline">New</span>
            </Button>
          </div>
        </div>
      </div>

      {isError && (
        <div className="bg-destructive/10 border border-destructive/20 text-destructive px-4 py-3 rounded-md">
          <p className="font-medium">Error loading projects</p>
          <p className="text-sm">{error?.message || 'Failed to load projects'}</p>
        </div>
      )}

      {/* Cloud Sync Status */}
      <GlobalSyncStatus className="mb-6" />

      <Tabs value={statusFilter} onValueChange={(value) => setStatusFilter(value as StatusFilter)}>
        <TabsList className="grid w-full grid-cols-6 gap-1">
          <TabsTrigger value="planning" className="text-xs sm:text-sm">Planning ({projectCounts.planning})</TabsTrigger>
          <TabsTrigger value="building" className="text-xs sm:text-sm">Building ({projectCounts.building})</TabsTrigger>
          <TabsTrigger value="review" className="text-xs sm:text-sm">Review ({projectCounts.review})</TabsTrigger>
          <TabsTrigger value="launched" className="text-xs sm:text-sm">Launched ({projectCounts.launched})</TabsTrigger>
          <TabsTrigger value="on-hold" className="text-xs sm:text-sm">On-Hold ({projectCounts.onHold})</TabsTrigger>
          <TabsTrigger value="archived" className="text-xs sm:text-sm">Archived ({projectCounts.archived})</TabsTrigger>
        </TabsList>

        <TabsContent value={statusFilter} className="mt-6">
          {filteredAndSortedProjects.length === 0 ? (
            allProjects.length === 0 ? (
              <div className="text-center py-12">
                <FolderOpen className="h-12 w-12 text-muted-foreground mx-auto mb-4" />
                <h3 className="text-lg font-medium text-muted-foreground mb-2">No projects found</h3>
                <p className="text-sm text-muted-foreground mb-4">
                  Get started by creating your first project.
                </p>
                <Button onClick={() => setCreateDialogOpen(true)}>
                  <Plus className="mr-2 h-4 w-4" />
                  Create Project
                </Button>
              </div>
            ) : (
              <div className="text-center py-12">
                <Search className="h-12 w-12 text-muted-foreground mx-auto mb-4" />
                <h3 className="text-lg font-medium text-muted-foreground mb-2">
                  No {statusFilter} projects match your search
                </h3>
                <p className="text-sm text-muted-foreground mb-4">
                  Try adjusting your search terms or switch to the {statusFilter === 'active' ? 'archived' : 'active'} tab.
                </p>
              </div>
            )
          ) : (
            viewType === 'list' ? (
              <DndContext
                sensors={sensors}
                collisionDetection={closestCenter}
                onDragEnd={handleDragEnd}
              >
                <div className="bg-card rounded-lg border overflow-x-auto">
                  <table className="w-full min-w-full">
                    <thead className="border-b bg-muted/50">
                      <tr>
                        <th className="py-3 px-2 sm:px-4 text-left font-medium">Name</th>
                        <th className="py-3 px-2 sm:px-4 text-left font-medium hidden md:table-cell">Repository</th>
                        <th className="py-3 px-2 sm:px-4 text-left font-medium">Dev</th>
                        <th className="py-3 px-2 sm:px-4 text-left font-medium">Priority</th>
                        <th className="py-3 px-2 sm:px-4 text-left font-medium hidden sm:table-cell">Sync</th>
                        <th className="py-3 px-2 sm:px-4 text-left font-medium hidden lg:table-cell">Tags</th>
                        <th className="py-3 px-2 sm:px-4 text-left font-medium hidden xl:table-cell">Created</th>
                        <th className="py-3 px-2 sm:px-4 text-left font-medium">Actions</th>
                      </tr>
                    </thead>
                    <tbody>
                      <SortableContext
                        items={filteredAndSortedProjects.map(p => p.id)}
                        strategy={verticalListSortingStrategy}
                      >
                        {filteredAndSortedProjects.map((project) => (
                          <SortableRow
                            key={project.id}
                            project={project}
                            onEdit={handleEditProject}
                            onDelete={handleDeleteProject}
                            onView={handleViewProject}
                            formatDate={formatDate}
                            getPriorityColor={getPriorityColor}
                            isDevServerRunning={isDevServerRunning}
                            onStartServer={handleStartServer}
                            onStopServer={handleStopServer}
                            isServerLoading={isServerLoading}
                          />
                        ))}
                      </SortableContext>
                    </tbody>
                  </table>
                </div>
              </DndContext>
            ) : (
              <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
                {filteredAndSortedProjects.map((project) => (
                  <div key={project.id} className="rounded-lg border p-4 sm:p-6 hover:shadow-md transition-shadow">
                    <div className="flex items-start justify-between mb-3">
                      <div className="flex items-center gap-2 flex-1">
                        <FolderOpen className="h-5 w-5 text-primary flex-shrink-0" />
                        <h3 className="font-semibold truncate">{project.name}</h3>
                      </div>
                      <div className="flex gap-1 ml-2">
                        <Button
                          size="sm"
                          variant="ghost"
                          onClick={() => handleEditProject(project)}
                          className="h-8 w-8 p-0"
                        >
                          <Edit className="h-3 w-3" />
                        </Button>
                        <Button
                          size="sm"
                          variant="ghost"
                          onClick={() => handleDeleteProject(project)}
                          className="h-8 w-8 p-0 text-red-600 hover:text-red-700"
                        >
                          <Trash2 className="h-3 w-3" />
                        </Button>
                      </div>
                    </div>
                    
                    {project.description && (
                      <p className="text-sm text-muted-foreground mb-4 line-clamp-2">
                        {project.description}
                      </p>
                    )}
                    
                    <div className="space-y-3">
                      <div className="flex items-center justify-between">
                        <div className="flex items-center gap-2">
                          <div className={`w-2 h-2 rounded-full ${
                            isDevServerRunning(project) ? 'bg-green-500 dark:bg-green-400' : 'bg-muted-foreground/40'
                          }`} />
                          {isServerLoading(project.id) ? (
                            <Loader2 className="h-3.5 w-3.5 animate-spin text-muted-foreground" aria-label="Server starting" />
                          ) : isDevServerRunning(project) ? (
                            <Button
                              size="sm"
                              variant="ghost"
                              onClick={(e) => {
                                e.stopPropagation();
                                handleStopServer(project.id);
                              }}
                              className="h-6 w-6 p-0"
                              title="Stop dev server"
                              aria-label="Stop dev server"
                              disabled={isServerLoading(project.id)}
                            >
                              <Square className="h-3 w-3" />
                            </Button>
                          ) : (
                            <Button
                              size="sm"
                              variant="ghost"
                              onClick={(e) => {
                                e.stopPropagation();
                                handleStartServer(project.id);
                              }}
                              className="h-6 w-6 p-0"
                              title="Start dev server"
                              aria-label="Start dev server"
                              disabled={isServerLoading(project.id)}
                            >
                              <Play className="h-3 w-3" />
                            </Button>
                          )}
                          <span className="text-sm">Dev Server</span>
                        </div>
                        <div className="flex items-center gap-2">
                          <Badge className={getPriorityColor(project.priority)}>
                            {project.priority}
                          </Badge>
                          <ProjectSyncBadge projectId={project.id} variant="compact" />
                        </div>
                      </div>
                      
                      {project.tags && project.tags.length > 0 && (
                        <div className="flex flex-wrap gap-1">
                          {project.tags.slice(0, 3).map((tag, index) => (
                            <Badge key={index} variant="outline" className="text-xs">
                              {tag}
                            </Badge>
                          ))}
                          {project.tags.length > 3 && (
                            <Badge variant="outline" className="text-xs">
                              +{project.tags.length - 3}
                            </Badge>
                          )}
                        </div>
                      )}
                      
                      <div className="text-xs text-muted-foreground pt-2 border-t">
                        <div>Created: {formatDate(project.createdAt)}</div>
                        {(() => {
                          const repoInfo = getRepositoryInfo(project);
                          if (repoInfo) {
                            return <div className="flex items-center gap-1">
                              <GitBranch className="h-3 w-3" />
                              <span>{repoInfo.owner}/{repoInfo.repo}</span>
                            </div>;
                          }
                          return null;
                        })()}
                        <div className="flex items-center gap-1 truncate">
                          <FolderOpen className="h-3 w-3 flex-shrink-0" />
                          <span className="truncate">{project.projectRoot}</span>
                        </div>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            )
          )}
        </TabsContent>
      </Tabs>

      <ProjectCreateDialog
        open={createDialogOpen}
        onOpenChange={setCreateDialogOpen}
        onProjectCreated={handleProjectCreated}
      />

      <ProjectEditDialog
        project={selectedProject}
        open={editDialogOpen}
        onOpenChange={setEditDialogOpen}
        onProjectUpdated={handleProjectUpdated}
      />

      <ProjectDeleteDialog
        project={selectedProject}
        open={deleteDialogOpen}
        onOpenChange={setDeleteDialogOpen}
        onProjectDeleted={handleProjectDeleted}
      />

      <AITestDialog
        open={aiTestDialogOpen}
        onOpenChange={setAITestDialogOpen}
      />
    </div>
  );
}