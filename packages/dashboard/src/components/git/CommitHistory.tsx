import { useState } from 'react';
import { MoreHorizontal, GitCommit, User, Calendar, FileText, Plus, Minus } from 'lucide-react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { formatCommitMessage, formatAuthor, type CommitInfo } from '@/services/git';
import { CommitDetailSheet } from './CommitDetailSheet';

interface CommitHistoryProps {
  commits: CommitInfo[];
  projectId: string;
}

export function CommitHistory({ commits, projectId }: CommitHistoryProps) {
  const [selectedCommit, setSelectedCommit] = useState<string | null>(null);

  // Ensure commits is always an array
  const safeCommits = Array.isArray(commits) ? commits : [];

  const formatDate = (timestamp: number) => {
    const date = new Date(timestamp * 1000);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

    if (diffDays === 0) {
      return 'Today';
    } else if (diffDays === 1) {
      return 'Yesterday';
    } else if (diffDays < 7) {
      return `${diffDays} days ago`;
    } else {
      return date.toLocaleDateString('en-US', {
        year: 'numeric',
        month: 'short',
        day: 'numeric'
      });
    }
  };

  const formatTime = (timestamp: number) => {
    const date = new Date(timestamp * 1000);
    return date.toLocaleTimeString('en-US', {
      hour: '2-digit',
      minute: '2-digit',
      hour12: false
    });
  };

  const handleCommitClick = (commitId: string) => {
    setSelectedCommit(commitId);
  };

  const handleCloseDetails = () => {
    setSelectedCommit(null);
  };

  return (
    <>
      <Card>
        <CardHeader>
          <CardTitle className="text-lg flex items-center gap-2">
            <GitCommit className="h-5 w-5" />
            Commit History
          </CardTitle>
          <CardDescription>
            Recent commits to this repository
          </CardDescription>
        </CardHeader>
        <CardContent className="p-0">
          <div className="divide-y">
            {safeCommits.map((commit, index) => (
              <div
                key={commit.id}
                className="p-4 hover:bg-muted/50 cursor-pointer transition-colors"
                onClick={() => handleCommitClick(commit.id)}
              >
                <div className="space-y-2">
                  {/* Line 1: Commit message and more button */}
                  <div className="flex items-start justify-between gap-3">
                    <div className="flex-1 min-w-0">
                      <h4 className="font-medium truncate">
                        {formatCommitMessage(commit.message, 80)}
                      </h4>
                    </div>
                    {index === 0 && (
                      <Badge variant="outline" className="text-xs shrink-0 mr-2">
                        Latest
                      </Badge>
                    )}
                    <Button
                      variant="ghost"
                      size="sm"
                      className="shrink-0 h-6 w-6 p-0"
                      onClick={(e) => {
                        e.stopPropagation();
                        handleCommitClick(commit.id);
                      }}
                    >
                      <MoreHorizontal className="h-3 w-3" />
                    </Button>
                  </div>

                  {/* Line 2: Hash, author, stats, and date */}
                  <div className="flex items-center justify-between text-xs text-muted-foreground">
                    <div className="flex items-center gap-3">
                      <code className="bg-muted px-1.5 py-0.5 rounded text-xs">
                        {commit.short_id}
                      </code>
                      <span className="flex items-center gap-1">
                        <User className="h-3 w-3" />
                        {formatAuthor(commit.author, commit.email)}
                      </span>
                      <span className="flex items-center gap-1">
                        <FileText className="h-3 w-3" />
                        {commit.files_changed} file{commit.files_changed !== 1 ? 's' : ''}
                      </span>
                      {commit.insertions > 0 && (
                        <span className="flex items-center gap-1 text-green-600">
                          <Plus className="h-3 w-3" />
                          {commit.insertions}
                        </span>
                      )}
                      {commit.deletions > 0 && (
                        <span className="flex items-center gap-1 text-red-600">
                          <Minus className="h-3 w-3" />
                          {commit.deletions}
                        </span>
                      )}
                    </div>
                    <div className="flex items-center gap-1 shrink-0">
                      <Calendar className="h-3 w-3" />
                      <span>{formatDate(commit.timestamp)}</span>
                      <span className="text-muted-foreground/70 ml-1">
                        {formatTime(commit.timestamp)}
                      </span>
                    </div>
                  </div>
                </div>
              </div>
            ))}
          </div>

          {/* Load more button if needed */}
          {safeCommits.length >= 50 && (
            <div className="p-4 border-t">
              <Button variant="outline" className="w-full" disabled>
                Load More Commits (Coming Soon)
              </Button>
            </div>
          )}
        </CardContent>
      </Card>

      {/* Commit detail sheet */}
      {selectedCommit && (
        <CommitDetailSheet
          projectId={projectId}
          commitId={selectedCommit}
          open={!!selectedCommit}
          onClose={handleCloseDetails}
        />
      )}
    </>
  );
}