import { useState } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Button } from '@/components/ui/button';
import { FolderTree, Package, Settings, History, Copy, CheckCircle } from 'lucide-react';
import { ContextBuilder } from './context/ContextBuilder';
import { ContextTemplates } from './context/ContextTemplates';
import { ContextHistory } from './context/ContextHistory';
import { useToast } from '@/hooks/use-toast';

interface ContextTabProps {
  projectId: string;
  projectPath: string;
}

export function ContextTab({ projectId, projectPath }: ContextTabProps) {
  const [generatedContext, setGeneratedContext] = useState<string>('');
  const [tokenCount, setTokenCount] = useState(0);
  const [copied, setCopied] = useState(false);
  const { toast } = useToast();

  const handleCopyToClipboard = async () => {
    if (!generatedContext) {
      toast({
        title: 'No context to copy',
        description: 'Generate context first before copying',
        variant: 'destructive',
      });
      return;
    }

    try {
      await navigator.clipboard.writeText(generatedContext);
      setCopied(true);
      toast({
        title: 'Copied to clipboard',
        description: `${tokenCount.toLocaleString()} tokens copied successfully`,
      });
      setTimeout(() => setCopied(false), 2000);
    } catch {
      toast({
        title: 'Failed to copy',
        description: 'Could not copy context to clipboard',
        variant: 'destructive',
      });
    }
  };

  return (
    <div className="space-y-4">
      {/* Header with quick actions */}
      <Card>
        <CardHeader>
          <CardTitle>Context Generation</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex gap-2">
            <Button 
              variant="outline"
              onClick={handleCopyToClipboard}
              disabled={!generatedContext}
            >
              {copied ? (
                <>
                  <CheckCircle className="mr-2 h-4 w-4" />
                  Copied!
                </>
              ) : (
                <>
                  <Copy className="mr-2 h-4 w-4" />
                  Copy to Clipboard
                </>
              )}
            </Button>
            {tokenCount > 0 && (
              <div className="flex items-center gap-2 px-3 py-2 bg-muted rounded-md">
                <Package className="h-4 w-4" />
                <span className="text-sm font-medium">
                  {tokenCount.toLocaleString()} tokens
                </span>
              </div>
            )}
          </div>
        </CardContent>
      </Card>

      {/* Main tabbed interface */}
      <Tabs defaultValue="builder" className="space-y-4">
        <TabsList>
          <TabsTrigger value="builder">
            <FolderTree className="h-4 w-4" />
            Builder
          </TabsTrigger>
          <TabsTrigger value="templates">
            <Settings className="mr-2 h-4 w-4" />
            Templates
          </TabsTrigger>
          <TabsTrigger value="history">
            <History className="mr-2 h-4 w-4" />
            History
          </TabsTrigger>
        </TabsList>

        <TabsContent value="builder">
          <ContextBuilder
            projectId={projectId}
            projectPath={projectPath}
            onContextGenerated={(content, tokens) => {
              setGeneratedContext(content);
              setTokenCount(tokens);
            }}
          />
        </TabsContent>

        <TabsContent value="templates">
          <ContextTemplates projectId={projectId} />
        </TabsContent>

        <TabsContent value="history">
          <ContextHistory projectId={projectId} />
        </TabsContent>
      </Tabs>
    </div>
  );
}
