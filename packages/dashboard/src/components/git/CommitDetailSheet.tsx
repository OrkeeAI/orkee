import { useState } from 'react';
import {
  GitCommit,
  User,
  Calendar,
  FileText,
  Plus,
  Minus,
  BarChart3,
  Info,
  File,
  AlertCircle,
} from 'lucide-react';
import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetHeader,
  SheetTitle,
} from '@/components/ui/sheet';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Separator } from '@/components/ui/separator';
import { formatCommitMessage, formatAuthor, formatFileStatus, useCommitDetails } from '@/services/git';
import { DiffViewer } from './DiffViewer';

interface CommitDetailSheetProps {
  projectId: string;
  commitId: string;
  open: boolean;
  onClose: () => void;
}

export function CommitDetailSheet({ projectId, commitId, open, onClose }: CommitDetailSheetProps) {
  const [selectedFile, setSelectedFile] = useState<string | null>(null);

  const {
    data: commitDetail,
    isLoading,
    error,
    isError,
  } = useCommitDetails(projectId, commitId, { enabled: open });

  const formatDate = (timestamp: number) => {
    const date = new Date(timestamp * 1000);
    return date.toLocaleString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
      hour12: false,
    });
  };

  const handleFileClick = (filePath: string) => {
    setSelectedFile(filePath);
  };

  const handleCloseFileView = () => {
    setSelectedFile(null);
  };

  if (!open) return null;

  return (
    <Sheet open={open} onOpenChange={onClose}>
      <SheetContent 
        className="w-full sm:max-w-[90vw] lg:max-w-[80vw] xl:max-w-[70vw] p-0"
        aria-describedby={commitDetail ? "commit-description" : undefined}
      >
        {isLoading && (
          <div className="flex items-center justify-center h-full">
            <div className="text-center">
              <div className="h-8 w-8 animate-spin rounded-full border-2 border-primary border-t-transparent mx-auto mb-4" />
              <p className="text-muted-foreground">Loading commit details...</p>
            </div>
          </div>
        )}

        {isError && (
          <div className="flex items-center justify-center h-full">
            <div className="text-center max-w-md">
              <AlertCircle className="h-12 w-12 mx-auto mb-4 text-destructive" />
              <h3 className="text-lg font-semibold mb-2">Unable to Load Commit</h3>
              <p className="text-muted-foreground mb-4">
                {error?.message || 'Failed to load commit details'}
              </p>
              <Button variant="outline" onClick={onClose}>
                Close
              </Button>
            </div>
          </div>
        )}

        {commitDetail && !selectedFile && (
          <div className="flex flex-col h-full">
            <SheetHeader className="p-6 border-b">
              <SheetTitle className="flex items-center gap-2">
                <GitCommit className="h-5 w-5" />
                {formatCommitMessage(commitDetail.commit.message, 60)}
              </SheetTitle>
              <SheetDescription id="commit-description" className="flex items-center gap-4 text-sm">
                <code className="bg-muted px-2 py-1 rounded">
                  {commitDetail.commit.short_id}
                </code>
                <span className="flex items-center gap-1">
                  <User className="h-3 w-3" />
                  {formatAuthor(commitDetail.commit.author, commitDetail.commit.email)}
                </span>
                <span className="flex items-center gap-1">
                  <Calendar className="h-3 w-3" />
                  {formatDate(commitDetail.commit.timestamp)}
                </span>
              </SheetDescription>
            </SheetHeader>

            <div className="flex-1 overflow-hidden">
              <Tabs defaultValue="files" className="h-full flex flex-col">
                <TabsList className="grid w-full grid-cols-3 mx-6 mt-6">
                  <TabsTrigger value="files" className="flex items-center gap-2">
                    <File className="h-4 w-4" />
                    Files Changed ({commitDetail.files.length})
                  </TabsTrigger>
                  <TabsTrigger value="stats" className="flex items-center gap-2">
                    <BarChart3 className="h-4 w-4" />
                    Statistics
                  </TabsTrigger>
                  <TabsTrigger value="info" className="flex items-center gap-2">
                    <Info className="h-4 w-4" />
                    Commit Info
                  </TabsTrigger>
                </TabsList>

                <div className="flex-1 overflow-hidden p-6">
                  <TabsContent value="files" className="h-full overflow-auto">
                    <Card>
                      <CardHeader>
                        <CardTitle className="text-lg flex items-center gap-2">
                          <FileText className="h-5 w-5" />
                          Changed Files
                        </CardTitle>
                        <CardDescription>
                          Click on a file to view its diff
                        </CardDescription>
                      </CardHeader>
                      <CardContent className="p-0">
                        <div className="divide-y">
                          {commitDetail.files.map((file) => {
                            const fileStatus = formatFileStatus(file.status);
                            return (
                              <div
                                key={file.path}
                                className="p-4 hover:bg-muted/50 cursor-pointer transition-colors"
                                onClick={() => handleFileClick(file.path)}
                              >
                                <div className="flex items-center justify-between">
                                  <div className="flex items-center gap-3 flex-1 min-w-0">
                                    <div className={`w-6 h-6 rounded-full flex items-center justify-center text-xs font-bold bg-current/10 ${fileStatus.color}`}>
                                      {fileStatus.icon}
                                    </div>
                                    <div className="flex-1 min-w-0">
                                      <div className="font-medium truncate">{file.path}</div>
                                      {file.old_path && file.old_path !== file.path && (
                                        <div className="text-xs text-muted-foreground">
                                          from {file.old_path}
                                        </div>
                                      )}
                                    </div>
                                    <Badge variant="outline" className={fileStatus.color}>
                                      {fileStatus.label}
                                    </Badge>
                                  </div>
                                  <div className="flex items-center gap-3 text-xs text-muted-foreground ml-4">
                                    {file.insertions > 0 && (
                                      <span className="flex items-center gap-1 text-green-600">
                                        <Plus className="h-3 w-3" />
                                        {file.insertions}
                                      </span>
                                    )}
                                    {file.deletions > 0 && (
                                      <span className="flex items-center gap-1 text-red-600">
                                        <Minus className="h-3 w-3" />
                                        {file.deletions}
                                      </span>
                                    )}
                                  </div>
                                </div>
                              </div>
                            );
                          })}
                        </div>
                      </CardContent>
                    </Card>
                  </TabsContent>

                  <TabsContent value="stats" className="h-full overflow-auto">
                    <div className="space-y-4">
                      <Card>
                        <CardHeader>
                          <CardTitle className="text-lg">Commit Statistics</CardTitle>
                          <CardDescription>
                            Summary of changes in this commit
                          </CardDescription>
                        </CardHeader>
                        <CardContent>
                          <div className="grid grid-cols-3 gap-4">
                            <div className="text-center">
                              <div className="text-2xl font-bold">{commitDetail.stats.files_changed}</div>
                              <div className="text-sm text-muted-foreground">Files Changed</div>
                            </div>
                            <div className="text-center">
                              <div className="text-2xl font-bold text-green-600">+{commitDetail.stats.total_insertions}</div>
                              <div className="text-sm text-muted-foreground">Insertions</div>
                            </div>
                            <div className="text-center">
                              <div className="text-2xl font-bold text-red-600">-{commitDetail.stats.total_deletions}</div>
                              <div className="text-sm text-muted-foreground">Deletions</div>
                            </div>
                          </div>
                          
                          <Separator className="my-4" />
                          
                          <div className="space-y-3">
                            <h4 className="font-medium">File Type Breakdown</h4>
                            {(() => {
                              const fileTypes: Record<string, { count: number; insertions: number; deletions: number }> = {};
                              commitDetail.files.forEach((file) => {
                                const ext = file.path.split('.').pop() || 'no extension';
                                if (!fileTypes[ext]) {
                                  fileTypes[ext] = { count: 0, insertions: 0, deletions: 0 };
                                }
                                fileTypes[ext].count += 1;
                                fileTypes[ext].insertions += file.insertions;
                                fileTypes[ext].deletions += file.deletions;
                              });
                              
                              return Object.entries(fileTypes)
                                .sort(([,a], [,b]) => b.count - a.count)
                                .slice(0, 5)
                                .map(([ext, stats]) => (
                                  <div key={ext} className="flex items-center justify-between text-sm">
                                    <span className="font-mono">.{ext}</span>
                                    <div className="flex items-center gap-3">
                                      <span>{stats.count} file{stats.count !== 1 ? 's' : ''}</span>
                                      <span className="text-green-600">+{stats.insertions}</span>
                                      <span className="text-red-600">-{stats.deletions}</span>
                                    </div>
                                  </div>
                                ));
                            })()}
                          </div>
                        </CardContent>
                      </Card>
                    </div>
                  </TabsContent>

                  <TabsContent value="info" className="h-full overflow-auto">
                    <div className="space-y-4">
                      <Card>
                        <CardHeader>
                          <CardTitle className="text-lg">Commit Information</CardTitle>
                          <CardDescription>
                            Detailed information about this commit
                          </CardDescription>
                        </CardHeader>
                        <CardContent className="space-y-4">
                          <div className="grid gap-4">
                            <div>
                              <label className="text-sm font-medium text-muted-foreground">Commit Hash</label>
                              <div className="font-mono text-sm bg-muted p-2 rounded mt-1">
                                {commitDetail.commit.id}
                              </div>
                            </div>
                            
                            <div>
                              <label className="text-sm font-medium text-muted-foreground">Full Message</label>
                              <div className="text-sm bg-muted p-3 rounded mt-1 whitespace-pre-wrap">
                                {commitDetail.commit.message}
                              </div>
                            </div>
                            
                            <div className="grid grid-cols-2 gap-4">
                              <div>
                                <label className="text-sm font-medium text-muted-foreground">Author</label>
                                <div className="text-sm mt-1">
                                  {formatAuthor(commitDetail.commit.author, commitDetail.commit.email)}
                                </div>
                              </div>
                              <div>
                                <label className="text-sm font-medium text-muted-foreground">Date</label>
                                <div className="text-sm mt-1">
                                  {formatDate(commitDetail.commit.timestamp)}
                                </div>
                              </div>
                            </div>
                            
                            {commitDetail.parent_ids.length > 0 && (
                              <div>
                                <label className="text-sm font-medium text-muted-foreground">
                                  Parent Commit{commitDetail.parent_ids.length > 1 ? 's' : ''}
                                </label>
                                <div className="space-y-1 mt-1">
                                  {commitDetail.parent_ids.map((parentId) => (
                                    <div key={parentId} className="font-mono text-sm bg-muted p-2 rounded">
                                      {parentId}
                                    </div>
                                  ))}
                                </div>
                              </div>
                            )}
                          </div>
                        </CardContent>
                      </Card>
                    </div>
                  </TabsContent>
                </div>
              </Tabs>
            </div>
          </div>
        )}

        {selectedFile && commitDetail && (
          <DiffViewer
            projectId={projectId}
            commitId={commitId}
            filePath={selectedFile}
            onBack={handleCloseFileView}
            commit={commitDetail.commit}
          />
        )}
      </SheetContent>
    </Sheet>
  );
}