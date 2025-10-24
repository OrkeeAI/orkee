// ABOUTME: Main Graph tab component for code visualization
// ABOUTME: Provides tabbed interface for dependency, symbol, module, and spec-mapping graphs

import { useState } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Network, GitBranch, Layers, FileCode, Info } from 'lucide-react';

interface GraphTabProps {
  projectId: string;
  projectPath: string;
}

export function GraphTab({ projectId, projectPath }: GraphTabProps) {
  const [selectedTab, setSelectedTab] = useState('dependencies');

  return (
    <div className="flex flex-col gap-4">
      {/* Header */}
      <Card>
        <CardHeader>
          <div className="flex items-center gap-2">
            <Network className="h-5 w-5" />
            <CardTitle>Code Graph Visualization</CardTitle>
          </div>
          <CardDescription>
            Explore your codebase structure through interactive dependency, symbol, module, and spec-mapping graphs
          </CardDescription>
        </CardHeader>
      </Card>

      {/* Graph Tabs */}
      <Tabs value={selectedTab} onValueChange={setSelectedTab} className="flex-1">
        <TabsList className="grid w-full grid-cols-4">
          <TabsTrigger value="dependencies" className="flex items-center gap-2">
            <GitBranch className="h-4 w-4" />
            Dependencies
          </TabsTrigger>
          <TabsTrigger value="symbols" className="flex items-center gap-2">
            <FileCode className="h-4 w-4" />
            Symbols
          </TabsTrigger>
          <TabsTrigger value="modules" className="flex items-center gap-2">
            <Layers className="h-4 w-4" />
            Modules
          </TabsTrigger>
          <TabsTrigger value="spec-mapping" className="flex items-center gap-2">
            <Network className="h-4 w-4" />
            Spec Mapping
          </TabsTrigger>
        </TabsList>

        <TabsContent value="dependencies" className="mt-4">
          <Card>
            <CardHeader>
              <CardTitle>File Dependencies</CardTitle>
              <CardDescription>
                Visualize how files import and depend on each other
              </CardDescription>
            </CardHeader>
            <CardContent>
              <Alert>
                <Info className="h-4 w-4" />
                <AlertDescription>
                  Dependency graph visualization coming soon. This will show import/export relationships between files.
                </AlertDescription>
              </Alert>
              <div className="mt-4 rounded-lg border border-dashed border-muted-foreground/25 p-12 text-center">
                <GitBranch className="mx-auto h-12 w-12 text-muted-foreground/50 mb-4" />
                <p className="text-sm text-muted-foreground">
                  Project: {projectPath}
                </p>
                <p className="text-sm text-muted-foreground mt-1">
                  Graph visualization will appear here
                </p>
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="symbols" className="mt-4">
          <Card>
            <CardHeader>
              <CardTitle>Symbol Graph</CardTitle>
              <CardDescription>
                Explore functions, classes, and other code symbols
              </CardDescription>
            </CardHeader>
            <CardContent>
              <Alert>
                <Info className="h-4 w-4" />
                <AlertDescription>
                  Symbol graph visualization coming soon. This will show relationships between functions, classes, and other code symbols.
                </AlertDescription>
              </Alert>
              <div className="mt-4 rounded-lg border border-dashed border-muted-foreground/25 p-12 text-center">
                <FileCode className="mx-auto h-12 w-12 text-muted-foreground/50 mb-4" />
                <p className="text-sm text-muted-foreground">
                  Symbol graph for project ID: {projectId}
                </p>
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="modules" className="mt-4">
          <Card>
            <CardHeader>
              <CardTitle>Module Architecture</CardTitle>
              <CardDescription>
                View your project's directory and module structure
              </CardDescription>
            </CardHeader>
            <CardContent>
              <Alert>
                <Info className="h-4 w-4" />
                <AlertDescription>
                  Module graph visualization coming soon. This will show your directory hierarchy and module organization.
                </AlertDescription>
              </Alert>
              <div className="mt-4 rounded-lg border border-dashed border-muted-foreground/25 p-12 text-center">
                <Layers className="mx-auto h-12 w-12 text-muted-foreground/50 mb-4" />
                <p className="text-sm text-muted-foreground">
                  Module hierarchy visualization will appear here
                </p>
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="spec-mapping" className="mt-4">
          <Card>
            <CardHeader>
              <CardTitle>Spec Mapping</CardTitle>
              <CardDescription>
                Map specifications to code implementation
              </CardDescription>
            </CardHeader>
            <CardContent>
              <Alert>
                <Info className="h-4 w-4" />
                <AlertDescription>
                  Spec mapping graph coming soon. This will show how specifications relate to your code implementation.
                </AlertDescription>
              </Alert>
              <div className="mt-4 rounded-lg border border-dashed border-muted-foreground/25 p-12 text-center">
                <Network className="mx-auto h-12 w-12 text-muted-foreground/50 mb-4" />
                <p className="text-sm text-muted-foreground">
                  Spec-to-code mapping visualization will appear here
                </p>
              </div>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  );
}
