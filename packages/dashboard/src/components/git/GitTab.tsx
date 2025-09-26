import { GitBranch, AlertCircle } from 'lucide-react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { useCommitHistory } from '@/services/git';
import { CommitHistory } from './CommitHistory';

interface GitTabProps {
  projectId: string;
  gitRepository?: {
    owner: string;
    repo: string;
    url: string;
    branch?: string;
  } | null;
}

export function GitTab({ projectId, gitRepository }: GitTabProps) {
  const {
    data: commits,
    isLoading,
    error,
    isError
  } = useCommitHistory(projectId, {}, { 
    enabled: !!gitRepository 
  });

  // If no git repository, show empty state
  if (!gitRepository) {
    return (
      <div className="space-y-4">
        <Card>
          <CardHeader className="text-center">
            <div className="mx-auto w-fit">
              <GitBranch className="h-12 w-12 text-muted-foreground mb-4" />
            </div>
            <CardTitle className="text-lg">No Git Repository</CardTitle>
            <CardDescription>
              This project doesn't have a Git repository initialized. Initialize Git to view commit history and changes.
            </CardDescription>
          </CardHeader>
          <CardContent className="text-center">
            <div className="p-4 bg-muted rounded-lg">
              <p className="text-sm text-muted-foreground mb-3">
                To initialize a Git repository in this project:
              </p>
              <div className="space-y-1 text-xs font-mono bg-background p-3 rounded border">
                <div>cd {projectId}</div>
                <div>git init</div>
                <div>git add .</div>
                <div>git commit -m "Initial commit"</div>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>
    );
  }

  // If there's an error loading commits
  if (isError) {
    return (
      <div className="space-y-4">
        <Card>
          <CardHeader>
            <div className="flex items-center gap-3">
              <GitBranch className="h-5 w-5" />
              <CardTitle className="text-lg">Git Repository</CardTitle>
              <Badge variant="outline" className="text-green-600 border-green-200">
                Connected
              </Badge>
            </div>
            <CardDescription>
              Repository: {gitRepository.owner}/{gitRepository.repo}
              {gitRepository.branch && (
                <span className="ml-2">
                  • Branch: <code className="text-xs">{gitRepository.branch}</code>
                </span>
              )}
            </CardDescription>
          </CardHeader>
          <CardContent className="text-center py-8">
            <AlertCircle className="mx-auto h-12 w-12 text-destructive mb-4" />
            <h3 className="text-lg font-semibold mb-2">Unable to Load Commits</h3>
            <p className="text-muted-foreground mb-4">
              {error?.message || 'Failed to load git commit history'}
            </p>
            <div className="p-3 bg-muted rounded-lg text-sm text-muted-foreground">
              This could happen if:
              <ul className="list-disc list-inside mt-2 space-y-1 text-left max-w-md mx-auto">
                <li>The repository has no commits yet</li>
                <li>The current branch doesn't exist</li>
                <li>There are permission issues accessing the repository</li>
                <li>The repository is in an invalid state</li>
              </ul>
            </div>
          </CardContent>
        </Card>
      </div>
    );
  }

  // Loading state
  if (isLoading) {
    return (
      <div className="space-y-4">
        <Card>
          <CardHeader>
            <div className="flex items-center gap-3">
              <GitBranch className="h-5 w-5" />
              <CardTitle className="text-lg">Git Repository</CardTitle>
              <Badge variant="outline" className="text-green-600 border-green-200">
                Connected
              </Badge>
            </div>
            <CardDescription>
              Repository: {gitRepository.owner}/{gitRepository.repo}
              {gitRepository.branch && (
                <span className="ml-2">
                  • Branch: <code className="text-xs">{gitRepository.branch}</code>
                </span>
              )}
            </CardDescription>
          </CardHeader>
          <CardContent className="text-center py-8">
            <div className="h-8 w-8 animate-spin rounded-full border-2 border-primary border-t-transparent mx-auto mb-4" />
            <p className="text-muted-foreground">Loading commit history...</p>
          </CardContent>
        </Card>
      </div>
    );
  }

  // Empty repository state
  if (!commits || commits.length === 0) {
    return (
      <div className="space-y-4">
        <Card>
          <CardHeader>
            <div className="flex items-center gap-3">
              <GitBranch className="h-5 w-5" />
              <CardTitle className="text-lg">Git Repository</CardTitle>
              <Badge variant="outline" className="text-green-600 border-green-200">
                Connected
              </Badge>
            </div>
            <CardDescription>
              Repository: {gitRepository.owner}/{gitRepository.repo}
              {gitRepository.branch && (
                <span className="ml-2">
                  • Branch: <code className="text-xs">{gitRepository.branch}</code>
                </span>
              )}
            </CardDescription>
          </CardHeader>
          <CardContent className="text-center py-8">
            <GitBranch className="mx-auto h-12 w-12 text-muted-foreground mb-4" />
            <h3 className="text-lg font-semibold mb-2">No Commits Found</h3>
            <p className="text-muted-foreground mb-4">
              This repository doesn't have any commits yet.
            </p>
            <div className="p-3 bg-muted rounded-lg text-sm text-muted-foreground">
              Make your first commit to start tracking changes:
              <div className="text-xs font-mono bg-background p-2 mt-2 rounded border">
                git add . && git commit -m "Initial commit"
              </div>
            </div>
          </CardContent>
        </Card>
      </div>
    );
  }

  // Success state with commits
  return (
    <div className="space-y-4">
      <Card>
        <CardHeader>
          <div className="flex items-center gap-3">
            <GitBranch className="h-5 w-5" />
            <CardTitle className="text-lg">Git Repository</CardTitle>
            <Badge variant="outline" className="text-green-600 border-green-200">
              Connected
            </Badge>
          </div>
          <CardDescription>
            Repository: {gitRepository.owner}/{gitRepository.repo}
            {gitRepository.branch && (
              <span className="ml-2">
                • Branch: <code className="text-xs">{gitRepository.branch}</code>
              </span>
            )}
          </CardDescription>
        </CardHeader>
      </Card>

      <CommitHistory 
        commits={commits || []} 
        projectId={projectId}
      />
    </div>
  );
}