import { useState, useEffect, useMemo } from 'react';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Input } from '@/components/ui/input';
import { FolderOpen, Plus, Edit, Trash2, Search, LayoutGrid, List, GripVertical, GitBranch } from 'lucide-react';
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
import { projectsService, Project } from '@/services/projects';

type ViewType = 'card' | 'list';
type SortType = 'rank' | 'priority' | 'alpha';

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
  formatDate: (dateString: string) => string;
  getPriorityColor: (priority: string) => string;
}

function SortableRow({ project, onEdit, onDelete, formatDate, getPriorityColor }: SortableRowProps) {
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
          <span className="font-medium text-sm sm:text-base truncate">{project.name}</span>
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
        <div className="flex items-center gap-1 sm:gap-2">
          <div className={`w-2 h-2 rounded-full ${
            project.status === 'active' ? 'bg-green-500' : 'bg-gray-500'
          }`} />
          <Badge className={`${getPriorityColor(project.priority)} text-xs`} variant="secondary">
            {project.priority}
          </Badge>
        </div>
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
}

export function Projects() {
  const [projects, setProjects] = useState<Project[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  
  // Dialog states
  const [createDialogOpen, setCreateDialogOpen] = useState(false);
  const [editDialogOpen, setEditDialogOpen] = useState(false);
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const [selectedProject, setSelectedProject] = useState<Project | null>(null);
  
  // Filter and view states
  const [searchTerm, setSearchTerm] = useState('');
  const [sortBy, setSortBy] = useState<SortType>('rank');
  const [viewType, setViewType] = useState<ViewType>('list');

  // Drag and drop sensors
  const sensors = useSensors(
    useSensor(PointerSensor),
    useSensor(KeyboardSensor, {
      coordinateGetter: sortableKeyboardCoordinates,
    })
  );

  const loadProjects = async () => {
    try {
      setLoading(true);
      setError(null);
      const projectsList = await projectsService.getAllProjects();
      setProjects(projectsList);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load projects');
    } finally {
      setLoading(false);
    }
  };

  // Filter and sort projects
  const filteredAndSortedProjects = useMemo(() => {
    let filtered = projects.filter(project => 
      project.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
      project.description?.toLowerCase().includes(searchTerm.toLowerCase()) ||
      project.tags?.some(tag => tag.toLowerCase().includes(searchTerm.toLowerCase())) ||
      project.projectRoot.toLowerCase().includes(searchTerm.toLowerCase())
    );

    const priorityOrder = { high: 3, medium: 2, low: 1 };
    
    filtered.sort((a, b) => {
      switch (sortBy) {
        case 'alpha':
          return a.name.localeCompare(b.name);
        case 'priority':
          return (priorityOrder[b.priority as keyof typeof priorityOrder] || 0) - 
                 (priorityOrder[a.priority as keyof typeof priorityOrder] || 0);
        case 'rank':
        default:
          // Use rank if available, otherwise sort by priority then date
          if (a.rank !== undefined && b.rank !== undefined) {
            return a.rank - b.rank;
          }
          const priorityDiff = (priorityOrder[b.priority as keyof typeof priorityOrder] || 0) - 
                              (priorityOrder[a.priority as keyof typeof priorityOrder] || 0);
          if (priorityDiff !== 0) return priorityDiff;
          return new Date(b.updatedAt).getTime() - new Date(a.updatedAt).getTime();
      }
    });

    return filtered;
  }, [projects, searchTerm, sortBy]);

  useEffect(() => {
    loadProjects();
  }, []);

  const handleEditProject = (project: Project) => {
    setSelectedProject(project);
    setEditDialogOpen(true);
  };

  const handleDeleteProject = (project: Project) => {
    setSelectedProject(project);
    setDeleteDialogOpen(true);
  };

  const handleProjectCreated = () => {
    loadProjects();
  };

  const handleProjectUpdated = () => {
    loadProjects();
  };

  const handleProjectDeleted = () => {
    loadProjects();
  };

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

      // Optimistically update the UI
      setProjects(prev => {
        const updated = [...prev];
        updatedProjects.forEach(updatedProject => {
          const index = updated.findIndex(p => p.id === updatedProject.id);
          if (index !== -1) {
            updated[index] = updatedProject;
          }
        });
        return updated;
      });

      // Update rankings on server
      try {
        for (const project of updatedProjects) {
          await projectsService.updateProject(project.id, { rank: project.rank });
        }
      } catch (err) {
        console.error('Failed to update project rankings:', err);
        // Reload projects to get correct state
        loadProjects();
      }
    }
  };

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString();
  };

  const getPriorityColor = (priority: string) => {
    switch (priority) {
      case 'high': return 'bg-red-100 text-red-800';
      case 'medium': return 'bg-yellow-100 text-yellow-800';
      case 'low': return 'bg-green-100 text-green-800';
      default: return 'bg-gray-100 text-gray-800';
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
          <div className="h-6 w-6 animate-spin rounded-full border-2 border-gray-300 border-t-blue-600"></div>
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

            <Button onClick={() => setCreateDialogOpen(true)} className="shrink-0">
              <Plus className="mr-0 sm:mr-2 h-4 w-4" />
              <span className="hidden sm:inline">New</span>
            </Button>
          </div>
        </div>
      </div>

      {error && (
        <div className="bg-red-50 border border-red-200 text-red-800 px-4 py-3 rounded-md">
          <p className="font-medium">Error loading projects</p>
          <p className="text-sm">{error}</p>
        </div>
      )}

      {filteredAndSortedProjects.length === 0 ? (
        projects.length === 0 ? (
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
            <h3 className="text-lg font-medium text-muted-foreground mb-2">No projects match your search</h3>
            <p className="text-sm text-muted-foreground mb-4">
              Try adjusting your search terms or filters.
            </p>
          </div>
        )
      ) : (
        viewType === 'list' ? (
          <div className="bg-white rounded-lg border overflow-x-auto">
            <table className="w-full min-w-full">
              <thead className="border-b bg-muted/30">
                <tr>
                  <th className="py-3 px-2 sm:px-4 text-left font-medium">Name</th>
                  <th className="py-3 px-2 sm:px-4 text-left font-medium hidden md:table-cell">Repository</th>
                  <th className="py-3 px-2 sm:px-4 text-left font-medium">Status</th>
                  <th className="py-3 px-2 sm:px-4 text-left font-medium hidden lg:table-cell">Tags</th>
                  <th className="py-3 px-2 sm:px-4 text-left font-medium hidden xl:table-cell">Created</th>
                  <th className="py-3 px-2 sm:px-4 text-left font-medium">Actions</th>
                </tr>
              </thead>
              <tbody>
                <DndContext
                  sensors={sensors}
                  collisionDetection={closestCenter}
                  onDragEnd={handleDragEnd}
                >
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
                        formatDate={formatDate}
                        getPriorityColor={getPriorityColor}
                      />
                    ))}
                  </SortableContext>
                </DndContext>
              </tbody>
            </table>
          </div>
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
                        project.status === 'active' ? 'bg-green-500' : 'bg-gray-500'
                      }`} />
                      <span className="text-sm capitalize">{project.status}</span>
                    </div>
                    <Badge className={getPriorityColor(project.priority)}>
                      {project.priority}
                    </Badge>
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
    </div>
  );
}