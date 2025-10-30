// ABOUTME: Real-time markdown PRD generation status with live preview
// ABOUTME: Shows streaming markdown content as it's generated from template

import { CheckCircle2, Loader2, FileText, Eye } from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Progress } from '@/components/ui/progress';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { ScrollArea } from '@/components/ui/scroll-area';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import rehypeHighlight from 'rehype-highlight';
import rehypeSanitize from 'rehype-sanitize';

interface MarkdownGenerationStatusProps {
  markdown: string;
  isComplete: boolean;
  templateName?: string;
}

export function MarkdownGenerationStatus({ 
  markdown, 
  isComplete,
  templateName 
}: MarkdownGenerationStatusProps) {
  // Estimate progress based on markdown length (rough heuristic)
  // Typical PRD is ~5000-10000 characters
  const estimatedTotalChars = 8000;
  const currentChars = markdown.length;
  const progressPercentage = Math.min((currentChars / estimatedTotalChars) * 100, 95);
  const displayProgress = isComplete ? 100 : progressPercentage;

  // Count sections by counting markdown headers
  const sectionMatches = markdown.match(/^#{1,2}\s+.+$/gm) || [];
  const sectionCount = sectionMatches.length;

  return (
    <div className="space-y-6">
      {/* Header with Progress */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <CardTitle className="flex items-center gap-2">
              <FileText className="h-5 w-5" />
              Generating PRD with {templateName || 'Template'}
            </CardTitle>
            <Badge variant={isComplete ? 'default' : 'secondary'} className="gap-1">
              {isComplete ? (
                <>
                  <CheckCircle2 className="h-3 w-3" />
                  Complete
                </>
              ) : (
                <>
                  <Loader2 className="h-3 w-3 animate-spin" />
                  Generating
                </>
              )}
            </Badge>
          </div>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <div className="flex justify-between text-sm">
              <span className="text-muted-foreground">Overall Progress</span>
              <span className="font-medium">
                {currentChars.toLocaleString()} characters • {sectionCount} sections
              </span>
            </div>
            <Progress value={displayProgress} className="h-2" />
          </div>
          {!isComplete && (
            <div className="flex items-center gap-2 text-sm text-muted-foreground">
              <Loader2 className="h-4 w-4 animate-spin" />
              <span>Streaming markdown content in real-time...</span>
            </div>
          )}
        </CardContent>
      </Card>

      {/* Content Tabs: Raw Markdown vs Rendered Preview */}
      <Card className="flex-1">
        <Tabs defaultValue="preview" className="w-full">
          <CardHeader className="pb-3">
            <TabsList className="grid w-full grid-cols-2">
              <TabsTrigger value="preview" className="gap-2">
                <Eye className="h-4 w-4" />
                Preview
              </TabsTrigger>
              <TabsTrigger value="markdown" className="gap-2">
                <FileText className="h-4 w-4" />
                Markdown
              </TabsTrigger>
            </TabsList>
          </CardHeader>
          
          <CardContent>
            <TabsContent value="preview" className="mt-0">
              <ScrollArea className="h-[500px] w-full rounded-md border p-4">
                {markdown ? (
                  <div className="prose prose-sm dark:prose-invert max-w-none">
                    <ReactMarkdown
                      remarkPlugins={[remarkGfm]}
                      rehypePlugins={[rehypeHighlight, rehypeSanitize]}
                    >
                      {markdown}
                    </ReactMarkdown>
                  </div>
                ) : (
                  <div className="flex items-center justify-center h-full text-muted-foreground">
                    <div className="flex flex-col items-center gap-2">
                      <Loader2 className="h-8 w-8 animate-spin" />
                      <p>Waiting for content...</p>
                    </div>
                  </div>
                )}
              </ScrollArea>
            </TabsContent>
            
            <TabsContent value="markdown" className="mt-0">
              <ScrollArea className="h-[500px] w-full rounded-md border">
                {markdown ? (
                  <pre className="p-4 text-sm font-mono whitespace-pre-wrap break-words">
                    {markdown}
                  </pre>
                ) : (
                  <div className="flex items-center justify-center h-full text-muted-foreground">
                    <div className="flex flex-col items-center gap-2">
                      <Loader2 className="h-8 w-8 animate-spin" />
                      <p>Waiting for content...</p>
                    </div>
                  </div>
                )}
              </ScrollArea>
            </TabsContent>
          </CardContent>
        </Tabs>
      </Card>

      {/* Status hint */}
      <div className="text-center text-sm text-muted-foreground">
        {isComplete ? (
          <p className="text-green-600 dark:text-green-400 font-medium">
            ✓ PRD generation complete! Review the content above.
          </p>
        ) : (
          <p>Content is being generated and will appear in real-time as it streams...</p>
        )}
      </div>
    </div>
  );
}
