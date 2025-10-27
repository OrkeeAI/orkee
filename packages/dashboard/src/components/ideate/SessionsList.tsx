// ABOUTME: Displays list of ideate sessions with filtering, search, and management actions
// ABOUTME: Shows session mode, status, timestamps, and provides resume/delete functionality
import { useState } from 'react';
import { formatDistanceToNow } from 'date-fns';
import {
  Search,
  Trash2,
  Play,
  FileText,
  Zap,
  MapPin,
  Sparkles,
  Filter,
} from 'lucide-react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Badge } from '@/components/ui/badge';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog';
import { Skeleton } from '@/components/ui/skeleton';
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip';
import { useIdeateSessions, useDeleteIdeateSession } from '@/hooks/useIdeate';
import type { IdeateSession, IdeateMode, IdeateStatus } from '@/services/ideate';
import { cn } from '@/lib/utils';
import { toast } from 'sonner';

interface SessionsListProps {
  projectId: string;
  onResumeSession?: (session: IdeateSession) => void;
}

const MODE_ICONS = {
  quick: <Zap className="h-4 w-4" />,
  guided: <MapPin className="h-4 w-4" />,
  comprehensive: <Sparkles className="h-4 w-4" />,
};

const MODE_LABELS = {
  quick: 'Quick',
  guided: 'Guided',
  comprehensive: 'Comprehensive',
};

export function SessionsList({ projectId, onResumeSession }: SessionsListProps) {
  const [searchQuery, setSearchQuery] = useState('');
  const [modeFilter, setModeFilter] = useState<IdeateMode | 'all'>('all');
  const [statusFilter, setStatusFilter] = useState<IdeateStatus | 'all'>('all');
  const [deleteSessionId, setDeleteSessionId] = useState<string | null>(null);

  const { data: sessions, isLoading, error } = useIdeateSessions(projectId);
  const deleteSessionMutation = useDeleteIdeateSession(projectId);

  const getModeVariant = (mode: IdeateMode) => {
    switch (mode) {
      case 'quick':
        return 'default';
      case 'guided':
        return 'secondary';
      case 'comprehensive':
        return 'outline';
    }
  };

  const getStatusVariant = (status: IdeateStatus) => {
    switch (status) {
      case 'completed':
        return 'default';
      case 'ready_for_prd':
        return 'default';
      case 'in_progress':
        return 'secondary';
      default:
        return 'outline';
    }
  };

  const getStatusLabel = (status: IdeateStatus) => {
    switch (status) {
      case 'draft':
        return 'Draft';
      case 'in_progress':
        return 'In Progress';
      case 'ready_for_prd':
        return 'Ready';
      case 'completed':
        return 'Completed';
    }
  };

  const handleDeleteClick = (sessionId: string) => {
    setDeleteSessionId(sessionId);
  };

  const handleDeleteConfirm = async () => {
    if (!deleteSessionId) return;

    try {
      await deleteSessionMutation.mutateAsync(deleteSessionId);
      toast.success('Session deleted successfully');
      setDeleteSessionId(null);
    } catch (error) {
      toast.error('Failed to delete session', {
        description: error instanceof Error ? error.message : 'Unknown error',
      });
    }
  };

  const filteredSessions = sessions?.filter((session) => {
    const matchesSearch = session.initial_description
      .toLowerCase()
      .includes(searchQuery.toLowerCase());
    const matchesMode = modeFilter === 'all' || session.mode === modeFilter;
    const matchesStatus = statusFilter === 'all' || session.status === statusFilter;
    return matchesSearch && matchesMode && matchesStatus;
  });

  if (isLoading) {
    return (
      <div className="space-y-4">
        <Skeleton className="h-10 w-full" />
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {[...Array(6)].map((_, i) => (
            <Skeleton key={i} className="h-48" />
          ))}
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="text-center py-12">
        <p className="text-destructive">Failed to load sessions: {error.message}</p>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Search and Filters */}
      <div className="flex flex-col sm:flex-row gap-4">
        <div className="relative flex-1">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
          <Input
            type="text"
            placeholder="Search sessions..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="pl-9"
          />
        </div>

        <div className="flex gap-2">
          <Select value={modeFilter} onValueChange={(value) => setModeFilter(value as IdeateMode | 'all')}>
            <SelectTrigger className="w-[140px]">
              <Filter className="h-4 w-4 mr-2" />
              <SelectValue placeholder="Mode" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">All Modes</SelectItem>
              <SelectItem value="quick">Quick</SelectItem>
              <SelectItem value="guided">Guided</SelectItem>
              <SelectItem value="comprehensive">Comprehensive</SelectItem>
            </SelectContent>
          </Select>

          <Select value={statusFilter} onValueChange={(value) => setStatusFilter(value as IdeateStatus | 'all')}>
            <SelectTrigger className="w-[140px]">
              <Filter className="h-4 w-4 mr-2" />
              <SelectValue placeholder="Status" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">All Status</SelectItem>
              <SelectItem value="draft">Draft</SelectItem>
              <SelectItem value="in_progress">In Progress</SelectItem>
              <SelectItem value="ready_for_prd">Ready</SelectItem>
              <SelectItem value="completed">Completed</SelectItem>
            </SelectContent>
          </Select>
        </div>
      </div>

      {/* Sessions Grid */}
      {filteredSessions && filteredSessions.length > 0 ? (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {filteredSessions.map((session) => (
            <Card key={session.id} className="hover:shadow-md transition-shadow">
              <CardHeader>
                <div className="flex items-start justify-between gap-2">
                  <div className="flex-1 min-w-0">
                    <CardTitle className="text-base line-clamp-2">
                      {session.initial_description}
                    </CardTitle>
                    <CardDescription className="mt-2">
                      <TooltipProvider>
                        <Tooltip>
                          <TooltipTrigger asChild>
                            <span className="text-xs">
                              {formatDistanceToNow(new Date(session.updated_at), { addSuffix: true })}
                            </span>
                          </TooltipTrigger>
                          <TooltipContent>
                            <p>Created: {new Date(session.created_at).toLocaleString()}</p>
                            <p>Updated: {new Date(session.updated_at).toLocaleString()}</p>
                          </TooltipContent>
                        </Tooltip>
                      </TooltipProvider>
                    </CardDescription>
                  </div>
                </div>

                <div className="flex flex-wrap gap-2 mt-3">
                  <Badge variant={getModeVariant(session.mode)} className="gap-1">
                    {MODE_ICONS[session.mode]}
                    {MODE_LABELS[session.mode]}
                  </Badge>
                  <Badge variant={getStatusVariant(session.status)}>
                    {getStatusLabel(session.status)}
                  </Badge>
                  {session.status === 'completed' && (
                    <Badge variant="outline" className="gap-1">
                      <FileText className="h-3 w-3" />
                      PRD Saved
                    </Badge>
                  )}
                </div>
              </CardHeader>

              <CardContent>
                <div className="flex gap-2">
                  <Button
                    variant="outline"
                    size="sm"
                    className="flex-1 gap-2"
                    onClick={() => onResumeSession?.(session)}
                    disabled={session.status === 'completed'}
                  >
                    <Play className="h-4 w-4" />
                    {session.status === 'completed' ? 'Completed' : 'Resume'}
                  </Button>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => handleDeleteClick(session.id)}
                  >
                    <Trash2 className="h-4 w-4" />
                  </Button>
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      ) : (
        <div className="text-center py-12 border-2 border-dashed rounded-lg">
          <p className="text-muted-foreground">
            {searchQuery || modeFilter !== 'all' || statusFilter !== 'all'
              ? 'No sessions match your filters'
              : 'No ideate sessions yet. Create your first PRD to get started!'}
          </p>
        </div>
      )}

      {/* Delete Confirmation Dialog */}
      <AlertDialog open={!!deleteSessionId} onOpenChange={() => setDeleteSessionId(null)}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Delete Session</AlertDialogTitle>
            <AlertDialogDescription>
              Are you sure you want to delete this ideate session? This action cannot be undone.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction
              onClick={handleDeleteConfirm}
              className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
            >
              Delete
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
}
