import { ArrowLeft, File, AlertCircle, FileText } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { formatFileStatus, useFileDiff, type CommitInfo } from '@/services/git';
import ReactDiffViewer, { DiffMethod } from 'react-diff-viewer-continued';

interface DiffViewerProps {
  projectId: string;
  commitId: string;
  filePath: string;
  onBack: () => void;
  commit: CommitInfo;
}

export function DiffViewer({ projectId, commitId, filePath, onBack, commit }: DiffViewerProps) {
  const {
    data: fileDiff,
    isLoading,
    error,
    isError,
  } = useFileDiff(projectId, commitId, filePath);

  const fileStatus = formatFileStatus(fileDiff?.status || 'modified');

  // Parse the unified diff to extract old and new content
  const parseDiff = (diffContent: string) => {
    const lines = diffContent.split('\n');
    let oldContent: string[] = [];
    let newContent: string[] = [];
    let isInHunk = false;
    
    for (const line of lines) {
      if (line.startsWith('@@')) {
        isInHunk = true;
        continue;
      }
      
      if (!isInHunk) continue;
      
      if (line.startsWith('-') && !line.startsWith('---')) {
        oldContent.push(line.substring(1));
      } else if (line.startsWith('+') && !line.startsWith('+++')) {
        newContent.push(line.substring(1));
      } else if (line.startsWith(' ')) {
        // Context line - add to both
        const contextLine = line.substring(1);
        oldContent.push(contextLine);
        newContent.push(contextLine);
      }
    }
    
    return {
      oldValue: oldContent.join('\n'),
      newValue: newContent.join('\n'),
    };
  };

  const customStyles = {
    variables: {
      light: {
        codeFoldGutterBackground: '#f8f9fa',
        codeFoldBackground: '#f1f3f4',
        gutterBackground: '#f8f9fa',
        gutterBackgroundDark: '#f1f3f4',
        highlightBackground: '#fff3cd',
        highlightGutterBackground: '#ffecb3',
        lineNumberColor: '#6b7280',
        addedBackground: '#dcfce7',
        addedGutterBackground: '#bbf7d0',
        removedBackground: '#fef2f2',
        removedGutterBackground: '#fecaca',
        wordAddedBackground: '#86efac',
        wordRemovedBackground: '#fca5a5',
        addedGutterColor: '#16a34a',
        removedGutterColor: '#dc2626',
        neutralGutterColor: '#6b7280',
        emptyLineBackground: '#fafbfc',
      },
    },
    lineHeight: '1.1',
    fontFamily: 'ui-monospace, SFMono-Regular, "SF Mono", Monaco, Inconsolata, "Roboto Mono", "Segoe UI Mono", "Courier New", monospace',
    fontSize: '10px',
    // Minimal styling - no layout properties
    lineNumber: {
      fontSize: '9px',
    },
    line: {
      padding: '1px 2px',
    },
    gutter: {
      padding: '1px 4px',
    },
  };

  if (isLoading) {
    return (
      <div className="flex flex-col h-full">
        <div className="p-6 border-b">
          <div className="flex items-center gap-4">
            <Button variant="ghost" size="sm" onClick={onBack}>
              <ArrowLeft className="h-4 w-4" />
              Back
            </Button>
            <div className="flex items-center gap-2">
              <File className="h-4 w-4" />
              <span className="font-medium">{filePath}</span>
            </div>
          </div>
        </div>
        <div className="flex-1 flex items-center justify-center">
          <div className="text-center">
            <div className="h-8 w-8 animate-spin rounded-full border-2 border-primary border-t-transparent mx-auto mb-4" />
            <p className="text-muted-foreground">Loading file diff...</p>
          </div>
        </div>
      </div>
    );
  }

  if (isError || !fileDiff) {
    return (
      <div className="flex flex-col h-full">
        <div className="p-6 border-b">
          <div className="flex items-center gap-4">
            <Button variant="ghost" size="sm" onClick={onBack}>
              <ArrowLeft className="h-4 w-4" />
              Back
            </Button>
            <div className="flex items-center gap-2">
              <File className="h-4 w-4" />
              <span className="font-medium">{filePath}</span>
            </div>
          </div>
        </div>
        <div className="flex-1 flex items-center justify-center">
          <div className="text-center max-w-md">
            <AlertCircle className="h-12 w-12 mx-auto mb-4 text-destructive" />
            <h3 className="text-lg font-semibold mb-2">Unable to Load Diff</h3>
            <p className="text-muted-foreground mb-4">
              {error?.message || 'Failed to load file diff'}
            </p>
            <Button variant="outline" onClick={onBack}>
              Go Back
            </Button>
          </div>
        </div>
      </div>
    );
  }

  if (fileDiff.is_binary) {
    return (
      <div className="flex flex-col h-full">
        <div className="p-6 border-b">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-4">
              <Button variant="ghost" size="sm" onClick={onBack}>
                <ArrowLeft className="h-4 w-4" />
                Back
              </Button>
              <div className="flex items-center gap-2">
                <div className={`w-6 h-6 rounded-full flex items-center justify-center text-xs font-bold bg-current/10 ${fileStatus.color}`}>
                  {fileStatus.icon}
                </div>
                <span className="font-medium">{filePath}</span>
                <Badge variant="outline" className={fileStatus.color}>
                  {fileStatus.label}
                </Badge>
              </div>
            </div>
          </div>
        </div>
        <div className="flex-1 flex items-center justify-center">
          <Card className="max-w-md">
            <CardHeader className="text-center">
              <FileText className="h-12 w-12 mx-auto mb-4 text-muted-foreground" />
              <CardTitle>Binary File</CardTitle>
            </CardHeader>
            <CardContent className="text-center">
              <p className="text-muted-foreground mb-4">
                This file contains binary data and cannot be displayed as text.
              </p>
              <Button variant="outline" onClick={onBack}>
                Go Back
              </Button>
            </CardContent>
          </Card>
        </div>
      </div>
    );
  }

  // Parse the diff content
  const { oldValue, newValue } = parseDiff(fileDiff.content);

  return (
    <div className="flex flex-col h-full">
      <div className="p-6 border-b bg-background">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-4">
            <Button variant="ghost" size="sm" onClick={onBack}>
              <ArrowLeft className="h-4 w-4" />
              Back
            </Button>
            <div className="flex items-center gap-2">
              <div className={`w-6 h-6 rounded-full flex items-center justify-center text-xs font-bold bg-current/10 ${fileStatus.color}`}>
                {fileStatus.icon}
              </div>
              <span className="font-medium">{fileDiff.old_path && fileDiff.old_path !== filePath ? fileDiff.old_path : filePath}</span>
              {fileDiff.old_path && fileDiff.old_path !== filePath && (
                <>
                  <span className="text-muted-foreground">→</span>
                  <span className="font-medium">{filePath}</span>
                </>
              )}
              <Badge variant="outline" className={fileStatus.color}>
                {fileStatus.label}
              </Badge>
            </div>
          </div>
          <div className="flex items-center gap-2 text-sm text-muted-foreground">
            <span>from</span>
            <code className="bg-muted px-2 py-1 rounded text-xs">
              {commit.short_id}
            </code>
          </div>
        </div>
      </div>

      <div className="flex-1 overflow-auto bg-background">
        <div className="min-h-full">
          {fileDiff.content.trim() === '' ? (
            <div className="flex items-center justify-center h-64">
              <div className="text-center">
                <FileText className="h-12 w-12 mx-auto mb-4 text-muted-foreground" />
                <p className="text-muted-foreground">No differences to display</p>
              </div>
            </div>
          ) : (
            <ReactDiffViewer
              oldValue={oldValue}
              newValue={newValue}
              splitView={true}
              compareMethod={DiffMethod.WORDS}
              leftTitle={fileDiff.old_path || filePath}
              rightTitle={filePath}
              styles={customStyles}
              hideLineNumbers={false}
              renderContent={(str) => (
                <span style={{ whiteSpace: 'pre-wrap', wordBreak: 'break-word', fontSize: '10px', lineHeight: '1.1' }}>{str}</span>
              )}
            />
          )}
        </div>
      </div>
    </div>
  );
}