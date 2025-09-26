import { useTasks, KanbanBoard, Task } from '@orkee/tasks';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Loader2, AlertCircle } from 'lucide-react';

interface TasksTabProps {
  projectId: string;
  projectPath: string;
  taskSource: string;
}

export function TasksTab({ projectId, projectPath, taskSource }: TasksTabProps) {
  const {
    tasks,
    isLoading,
    error,
    createTask,
    updateTask,
    deleteTask
  } = useTasks({
    projectId,
    projectPath,
    providerType: taskSource as any,
    enabled: true,
    apiBaseUrl: 'http://localhost:4001'
  });

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-96">
        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    );
  }

  if (error) {
    return (
      <Alert variant="destructive">
        <AlertCircle className="h-4 w-4" />
        <AlertDescription>
          {error instanceof Error ? error.message : 'Failed to load tasks. Please try again.'}
        </AlertDescription>
      </Alert>
    );
  }

  if (tasks.length === 0 && taskSource === 'taskmaster') {
    return (
      <Card>
        <CardHeader>
          <CardTitle>No Tasks Found</CardTitle>
          <CardDescription>
            No tasks found in the .taskmaster/tasks/tasks.json file. Make sure you have initialized
            Task Master in this project by running <code>task-master init</code> in the project root.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <Alert>
            <AlertTitle>Getting Started with Task Master</AlertTitle>
            <AlertDescription className="space-y-2">
              <p>To start using Task Master:</p>
              <ol className="list-decimal list-inside space-y-1">
                <li>Run <code>npm install -g task-master-ai</code> to install Task Master</li>
                <li>Navigate to your project root: <code>cd {projectPath}</code></li>
                <li>Initialize Task Master: <code>task-master init</code></li>
                <li>Parse your PRD: <code>task-master parse-prd</code></li>
                <li>Refresh this page to see your tasks</li>
              </ol>
            </AlertDescription>
          </Alert>
        </CardContent>
      </Card>
    );
  }

  const handleTaskUpdate = (taskId: string, updates: Partial<Task>) => {
    updateTask(taskId, updates);
  };

  const handleTaskCreate = (task: Partial<Task>) => {
    createTask(task);
  };

  const handleTaskDelete = (taskId: string) => {
    if (confirm('Are you sure you want to delete this task?')) {
      deleteTask(taskId);
    }
  };

  return (
    <div className="space-y-4">
      <KanbanBoard
        tasks={tasks}
        onTaskUpdate={handleTaskUpdate}
        onTaskCreate={handleTaskCreate}
        onTaskDelete={handleTaskDelete}
      />
    </div>
  );
}