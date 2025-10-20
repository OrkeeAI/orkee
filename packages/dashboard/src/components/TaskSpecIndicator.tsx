import { useState } from 'react';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/components/ui/tooltip';
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from '@/components/ui/popover';
import { CheckCircle2, XCircle, AlertTriangle, FileText, Loader2 } from 'lucide-react';
import { useTaskSpecLinks } from '@/hooks/useTaskSpecLinks';
import { TaskSpecLinker } from './TaskSpecLinker';

interface TaskSpecIndicatorProps {
  projectId: string;
  taskId: string;
  taskTitle: string;
  compact?: boolean;
}

export function TaskSpecIndicator({
  projectId,
  taskId,
  taskTitle,
  compact = false,
}: TaskSpecIndicatorProps) {
  const [linkerOpen, setLinkerOpen] = useState(false);
  const { data: linkedRequirements = [], isLoading } = useTaskSpecLinks(taskId);

  const hasLinks = linkedRequirements.length > 0;
  const linkCount = linkedRequirements.length;

  if (isLoading) {
    return (
      <Badge variant="outline" className="gap-1">
        <Loader2 className="h-3 w-3 animate-spin" />
        {!compact && <span>Loading...</span>}
      </Badge>
    );
  }

  if (!hasLinks) {
    return (
      <>
        <TooltipProvider>
          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                variant="ghost"
                size="sm"
                onClick={() => setLinkerOpen(true)}
                className="h-7 px-2 text-xs"
              >
                <AlertTriangle className="h-3 w-3 mr-1 text-yellow-600" />
                {!compact && <span>No Spec</span>}
              </Button>
            </TooltipTrigger>
            <TooltipContent>
              <p>This task is not linked to any spec requirements</p>
              <p className="text-xs text-muted-foreground">Click to link</p>
            </TooltipContent>
          </Tooltip>
        </TooltipProvider>

        <TaskSpecLinker
          projectId={projectId}
          taskId={taskId}
          taskTitle={taskTitle}
          open={linkerOpen}
          onOpenChange={setLinkerOpen}
        />
      </>
    );
  }

  return (
    <>
      <Popover>
        <PopoverTrigger asChild>
          <Button variant="ghost" size="sm" className="h-7 px-2 text-xs">
            <CheckCircle2 className="h-3 w-3 mr-1 text-green-600" />
            {!compact && (
              <span>
                {linkCount} Spec{linkCount !== 1 ? 's' : ''}
              </span>
            )}
          </Button>
        </PopoverTrigger>
        <PopoverContent className="w-80" align="start">
          <div className="space-y-3">
            <div className="flex items-center justify-between">
              <h4 className="font-medium text-sm">Linked Requirements</h4>
              <Badge variant="secondary">{linkCount}</Badge>
            </div>

            <div className="space-y-2">
              {linkedRequirements.map((req) => (
                <div
                  key={req.id}
                  className="rounded-md border p-2 space-y-1"
                >
                  <div className="flex items-start justify-between gap-2">
                    <div className="flex-1">
                      <div className="font-medium text-sm">{req.name}</div>
                      <div className="text-xs text-muted-foreground line-clamp-2">
                        {req.content}
                      </div>
                    </div>
                    <FileText className="h-4 w-4 text-muted-foreground shrink-0" />
                  </div>

                  {req.scenarios && req.scenarios.length > 0 && (
                    <div className="text-xs text-muted-foreground">
                      {req.scenarios.length} scenario{req.scenarios.length !== 1 ? 's' : ''}
                    </div>
                  )}
                </div>
              ))}
            </div>

            <Button
              variant="outline"
              size="sm"
              onClick={() => setLinkerOpen(true)}
              className="w-full"
            >
              Manage Links
            </Button>
          </div>
        </PopoverContent>
      </Popover>

      <TaskSpecLinker
        projectId={projectId}
        taskId={taskId}
        taskTitle={taskTitle}
        open={linkerOpen}
        onOpenChange={setLinkerOpen}
      />
    </>
  );
}
