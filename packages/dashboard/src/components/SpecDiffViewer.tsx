// ABOUTME: Component for viewing differences between spec versions
// ABOUTME: Shows side-by-side diff with added/modified/removed requirements highlighted

import { useMemo } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Plus, Minus, Edit } from 'lucide-react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';

interface SpecVersion {
  id: string;
  name: string;
  purpose: string;
  specMarkdown: string;
  requirements: Array<{
    name: string;
    content: string;
  }>;
  version: number;
  updatedAt: string;
}

interface SpecDiffViewerProps {
  oldVersion: SpecVersion;
  newVersion: SpecVersion;
}

interface DiffResult {
  added: Array<{ name: string; content: string }>;
  removed: Array<{ name: string; content: string }>;
  modified: Array<{ name: string; oldContent: string; newContent: string }>;
  unchanged: Array<{ name: string; content: string }>;
}

function computeDiff(oldReqs: SpecVersion['requirements'], newReqs: SpecVersion['requirements']): DiffResult {
  const oldMap = new Map(oldReqs.map(r => [r.name, r.content]));
  const newMap = new Map(newReqs.map(r => [r.name, r.content]));

  const added: DiffResult['added'] = [];
  const removed: DiffResult['removed'] = [];
  const modified: DiffResult['modified'] = [];
  const unchanged: DiffResult['unchanged'] = [];

  // Find added and modified
  for (const [name, newContent] of newMap) {
    const oldContent = oldMap.get(name);
    if (!oldContent) {
      added.push({ name, content: newContent });
    } else if (oldContent !== newContent) {
      modified.push({ name, oldContent, newContent });
    } else {
      unchanged.push({ name, content: newContent });
    }
  }

  // Find removed
  for (const [name, content] of oldMap) {
    if (!newMap.has(name)) {
      removed.push({ name, content });
    }
  }

  return { added, removed, modified, unchanged };
}

export function SpecDiffViewer({ oldVersion, newVersion }: SpecDiffViewerProps) {
  const diff = useMemo(
    () => computeDiff(oldVersion.requirements, newVersion.requirements),
    [oldVersion.requirements, newVersion.requirements]
  );

  const totalChanges = diff.added.length + diff.removed.length + diff.modified.length;

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <div>
            <CardTitle>Spec Diff: {newVersion.name}</CardTitle>
            <CardDescription>
              Comparing v{oldVersion.version} (
              {new Date(oldVersion.updatedAt).toLocaleDateString()}) with v{newVersion.version} (
              {new Date(newVersion.updatedAt).toLocaleDateString()})
            </CardDescription>
          </div>
          <div className="flex gap-2">
            {diff.added.length > 0 && (
              <Badge variant="default">
                <Plus className="mr-1 h-3 w-3" />
                {diff.added.length} Added
              </Badge>
            )}
            {diff.modified.length > 0 && (
              <Badge variant="secondary">
                <Edit className="mr-1 h-3 w-3" />
                {diff.modified.length} Modified
              </Badge>
            )}
            {diff.removed.length > 0 && (
              <Badge variant="destructive">
                <Minus className="mr-1 h-3 w-3" />
                {diff.removed.length} Removed
              </Badge>
            )}
          </div>
        </div>
      </CardHeader>

      <CardContent>
        <Tabs defaultValue="changes">
          <TabsList>
            <TabsTrigger value="changes">
              Changes ({totalChanges})
            </TabsTrigger>
            <TabsTrigger value="side-by-side">Side by Side</TabsTrigger>
            <TabsTrigger value="all">All Requirements</TabsTrigger>
          </TabsList>

          <TabsContent value="changes" className="space-y-4 mt-4">
            {totalChanges === 0 ? (
              <p className="text-muted-foreground text-center py-8">No changes detected</p>
            ) : (
              <>
                {diff.added.map((req, idx) => (
                  <div key={`added-${idx}`} className="rounded-lg border border-green-500 bg-green-50 dark:bg-green-950 p-4">
                    <div className="flex items-center gap-2 mb-2">
                      <Badge variant="default">
                        <Plus className="mr-1 h-3 w-3" />
                        Added
                      </Badge>
                      <h4 className="font-medium">{req.name}</h4>
                    </div>
                    <div className="prose prose-sm dark:prose-invert max-w-none">
                      <ReactMarkdown remarkPlugins={[remarkGfm]}>{req.content}</ReactMarkdown>
                    </div>
                  </div>
                ))}

                {diff.modified.map((req, idx) => (
                  <div key={`modified-${idx}`} className="rounded-lg border border-blue-500 bg-blue-50 dark:bg-blue-950 p-4 space-y-3">
                    <div className="flex items-center gap-2">
                      <Badge variant="secondary">
                        <Edit className="mr-1 h-3 w-3" />
                        Modified
                      </Badge>
                      <h4 className="font-medium">{req.name}</h4>
                    </div>
                    <div className="grid grid-cols-2 gap-4">
                      <div>
                        <p className="text-xs font-medium text-muted-foreground mb-2">Old Version</p>
                        <div className="prose prose-sm dark:prose-invert max-w-none bg-red-100 dark:bg-red-900/30 p-2 rounded">
                          <ReactMarkdown remarkPlugins={[remarkGfm]}>{req.oldContent}</ReactMarkdown>
                        </div>
                      </div>
                      <div>
                        <p className="text-xs font-medium text-muted-foreground mb-2">New Version</p>
                        <div className="prose prose-sm dark:prose-invert max-w-none bg-green-100 dark:bg-green-900/30 p-2 rounded">
                          <ReactMarkdown remarkPlugins={[remarkGfm]}>{req.newContent}</ReactMarkdown>
                        </div>
                      </div>
                    </div>
                  </div>
                ))}

                {diff.removed.map((req, idx) => (
                  <div key={`removed-${idx}`} className="rounded-lg border border-red-500 bg-red-50 dark:bg-red-950 p-4">
                    <div className="flex items-center gap-2 mb-2">
                      <Badge variant="destructive">
                        <Minus className="mr-1 h-3 w-3" />
                        Removed
                      </Badge>
                      <h4 className="font-medium">{req.name}</h4>
                    </div>
                    <div className="prose prose-sm dark:prose-invert max-w-none opacity-70">
                      <ReactMarkdown remarkPlugins={[remarkGfm]}>{req.content}</ReactMarkdown>
                    </div>
                  </div>
                ))}
              </>
            )}
          </TabsContent>

          <TabsContent value="side-by-side" className="space-y-4 mt-4">
            <div className="grid grid-cols-2 gap-4">
              <div>
                <h3 className="font-medium mb-2">v{oldVersion.version}</h3>
                <div className="prose prose-sm dark:prose-invert max-w-none border rounded-lg p-4">
                  <ReactMarkdown remarkPlugins={[remarkGfm]}>{oldVersion.specMarkdown}</ReactMarkdown>
                </div>
              </div>
              <div>
                <h3 className="font-medium mb-2">v{newVersion.version}</h3>
                <div className="prose prose-sm dark:prose-invert max-w-none border rounded-lg p-4">
                  <ReactMarkdown remarkPlugins={[remarkGfm]}>{newVersion.specMarkdown}</ReactMarkdown>
                </div>
              </div>
            </div>
          </TabsContent>

          <TabsContent value="all" className="space-y-4 mt-4">
            {[...diff.added, ...diff.modified.map(m => ({ name: m.name, content: m.newContent })), ...diff.unchanged].map((req, idx) => (
              <div key={idx} className="rounded-lg border p-4">
                <h4 className="font-medium mb-2">{req.name}</h4>
                <div className="prose prose-sm dark:prose-invert max-w-none">
                  <ReactMarkdown remarkPlugins={[remarkGfm]}>{req.content}</ReactMarkdown>
                </div>
              </div>
            ))}
          </TabsContent>
        </Tabs>
      </CardContent>
    </Card>
  );
}
