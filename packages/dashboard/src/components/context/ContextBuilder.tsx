import { useState, useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Checkbox } from '@/components/ui/checkbox';
import { Button } from '@/components/ui/button';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Badge } from '@/components/ui/badge';
import { Loader2, FileText, Folder, ChevronRight, ChevronDown } from 'lucide-react';
import { useToast } from '@/hooks/use-toast';
import { getApiBaseUrl } from '@/services/api';

interface ContextBuilderProps {
  projectId: string;
  projectPath: string;
  onContextGenerated: (content: string, tokens: number) => void;
}

interface FileInfo {
  path: string;
  size: number;
  extension?: string;
  is_directory: boolean;
}

interface FileTreeNode {
  name: string;
  path: string;
  isDirectory: boolean;
  children?: FileTreeNode[];
  size?: number;
}

export function ContextBuilder({ projectId, onContextGenerated }: ContextBuilderProps) {
  const [fileTree, setFileTree] = useState<FileTreeNode[]>([]);
  const [selectedFiles, setSelectedFiles] = useState<Set<string>>(new Set());
  const [expandedDirs, setExpandedDirs] = useState<Set<string>>(new Set());
  const [isLoading, setIsLoading] = useState(false);
  const [isGenerating, setIsGenerating] = useState(false);
  const [excludePatterns, setExcludePatterns] = useState<string[]>([
    'node_modules/**',
    '*.test.ts',
    '*.spec.ts',
    '.git/**',
    'dist/**',
    'build/**',
    'target/**',
    '*.lock',
  ]);
  const [newPattern, setNewPattern] = useState('');
  const [apiBaseUrl, setApiBaseUrl] = useState<string>('');
  const { toast } = useToast();

  useEffect(() => {
    getApiBaseUrl().then(setApiBaseUrl);
  }, []);

  useEffect(() => {
    if (apiBaseUrl && projectId) {
      loadFiles();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [apiBaseUrl, projectId]);

  const loadFiles = async () => {
    setIsLoading(true);
    try {
      const response = await fetch(`${apiBaseUrl}/api/projects/${projectId}/files`, {
        method: 'GET',
        headers: { 'Content-Type': 'application/json' },
      });

      if (!response.ok) {
        const errorData = await response.json().catch(() => ({}));
        throw new Error(errorData.error || `HTTP ${response.status}: Failed to load files`);
      }

      const data = await response.json();
      console.log('Loaded files:', data.files?.length || 0, 'files');
      
      // Build file tree from flat list
      const tree = buildFileTree(data.files || []);
      setFileTree(tree);
    } catch (error) {
      console.error('Error loading files:', error);
      toast({
        title: 'Error loading files',
        description: error instanceof Error ? error.message : 'Failed to load project files',
        variant: 'destructive',
      });
    } finally {
      setIsLoading(false);
    }
  };

  const buildFileTree = (files: FileInfo[]): FileTreeNode[] => {
    const root: FileTreeNode[] = [];
    const map = new Map<string, FileTreeNode>();

    // Sort files by path
    const sortedFiles = [...files].sort((a, b) => a.path.localeCompare(b.path));

    sortedFiles.forEach(file => {
      const parts = file.path.split('/');
      const name = parts[parts.length - 1];
      
      const node: FileTreeNode = {
        name,
        path: file.path,
        isDirectory: file.is_directory,
        size: file.size,
        children: file.is_directory ? [] : undefined,
      };

      map.set(file.path, node);

      if (parts.length === 1) {
        root.push(node);
      } else {
        const parentPath = parts.slice(0, -1).join('/');
        const parent = map.get(parentPath);
        if (parent && parent.children) {
          parent.children.push(node);
        }
      }
    });

    return root;
  };

  const toggleFile = (filePath: string) => {
    const newSelected = new Set(selectedFiles);
    if (newSelected.has(filePath)) {
      newSelected.delete(filePath);
    } else {
      newSelected.add(filePath);
    }
    setSelectedFiles(newSelected);
  };

  const toggleDirectory = (dirPath: string) => {
    const newExpanded = new Set(expandedDirs);
    if (newExpanded.has(dirPath)) {
      newExpanded.delete(dirPath);
    } else {
      newExpanded.add(dirPath);
    }
    setExpandedDirs(newExpanded);
  };

  const addExcludePattern = () => {
    if (newPattern.trim() && !excludePatterns.includes(newPattern.trim())) {
      setExcludePatterns([...excludePatterns, newPattern.trim()]);
      setNewPattern('');
    }
  };

  const removeExcludePattern = (pattern: string) => {
    setExcludePatterns(excludePatterns.filter(p => p !== pattern));
  };

  const generateContext = async () => {
    if (selectedFiles.size === 0) {
      toast({
        title: 'No files selected',
        description: 'Please select at least one file to generate context',
        variant: 'destructive',
      });
      return;
    }

    setIsGenerating(true);
    try {
      const response = await fetch(`${apiBaseUrl}/api/projects/${projectId}/context/generate`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          project_id: projectId,
          include_patterns: Array.from(selectedFiles),
          exclude_patterns: excludePatterns,
          max_tokens: 100000,
          save_configuration: false,
        }),
      });

      if (!response.ok) {
        throw new Error('Failed to generate context');
      }

      const result = await response.json();
      onContextGenerated(result.content, result.total_tokens);
      
      toast({
        title: 'Context generated',
        description: `Generated ${result.file_count} files with ${result.total_tokens.toLocaleString()} tokens`,
      });
    } catch (error) {
      toast({
        title: 'Error generating context',
        description: error instanceof Error ? error.message : 'Failed to generate context',
        variant: 'destructive',
      });
    } finally {
      setIsGenerating(false);
    }
  };

  const renderFileTree = (nodes: FileTreeNode[], depth: number = 0): JSX.Element[] => {
    return nodes.map(node => (
      <div key={node.path} style={{ marginLeft: `${depth * 16}px` }}>
        {node.isDirectory ? (
          <div>
            <div 
              className="flex items-center gap-2 py-1 hover:bg-muted rounded cursor-pointer"
              onClick={() => toggleDirectory(node.path)}
            >
              {expandedDirs.has(node.path) ? (
                <ChevronDown className="h-4 w-4" />
              ) : (
                <ChevronRight className="h-4 w-4" />
              )}
              <Folder className="h-4 w-4 text-blue-500" />
              <span className="text-sm">{node.name}</span>
            </div>
            {expandedDirs.has(node.path) && node.children && (
              <div>
                {renderFileTree(node.children, depth + 1)}
              </div>
            )}
          </div>
        ) : (
          <div className="flex items-center gap-2 py-1 hover:bg-muted rounded">
            <Checkbox
              checked={selectedFiles.has(node.path)}
              onCheckedChange={() => toggleFile(node.path)}
            />
            <FileText className="h-4 w-4 text-muted-foreground" />
            <span className="text-sm">{node.name}</span>
            <span className="text-xs text-muted-foreground ml-auto">
              {node.size ? `${(node.size / 1024).toFixed(1)} KB` : ''}
            </span>
          </div>
        )}
      </div>
    ));
  };

  if (isLoading) {
    return (
      <Card>
        <CardContent className="flex items-center justify-center h-96">
          <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
        </CardContent>
      </Card>
    );
  }

  return (
    <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
      {/* File selector */}
      <Card>
        <CardHeader>
          <CardTitle>Select Files</CardTitle>
          <div className="text-sm text-muted-foreground">
            {selectedFiles.size} file{selectedFiles.size !== 1 ? 's' : ''} selected
          </div>
        </CardHeader>
        <CardContent>
          <ScrollArea className="h-[500px] pr-4">
            {fileTree.length > 0 ? (
              renderFileTree(fileTree)
            ) : (
              <div className="text-sm text-muted-foreground text-center py-8">
                No files found in project
              </div>
            )}
          </ScrollArea>
        </CardContent>
      </Card>

      {/* Configuration and actions */}
      <Card>
        <CardHeader>
          <CardTitle>Configuration</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          {/* Exclude patterns */}
          <div className="space-y-2">
            <Label>Exclude Patterns</Label>
            <div className="flex gap-2">
              <Input
                placeholder="e.g., *.test.ts"
                value={newPattern}
                onChange={(e) => setNewPattern(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === 'Enter') {
                    addExcludePattern();
                  }
                }}
              />
              <Button onClick={addExcludePattern} variant="outline">
                Add
              </Button>
            </div>
            <div className="flex flex-wrap gap-2 mt-2">
              {excludePatterns.map(pattern => (
                <Badge 
                  key={pattern} 
                  variant="secondary"
                  className="cursor-pointer"
                  onClick={() => removeExcludePattern(pattern)}
                >
                  {pattern} ×
                </Badge>
              ))}
            </div>
          </div>

          {/* Generate button */}
          <Button
            className="w-full"
            onClick={generateContext}
            disabled={selectedFiles.size === 0 || isGenerating}
          >
            {isGenerating ? (
              <>
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                Generating...
              </>
            ) : (
              'Generate Context'
            )}
          </Button>

          {/* Info */}
          <div className="text-xs text-muted-foreground space-y-1">
            <p>• Select files to include in the context</p>
            <p>• Add patterns to exclude files (supports glob syntax)</p>
            <p>• Generated context will be ready to copy</p>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
