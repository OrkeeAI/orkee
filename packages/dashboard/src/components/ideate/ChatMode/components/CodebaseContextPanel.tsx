// ABOUTME: Codebase context display panel for showing analysis results
// ABOUTME: Shows patterns, similar features, reusable components, and architecture

import React, { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Loader2, Code2, FileCode, Layers, Sparkles, ChevronDown, ChevronUp } from 'lucide-react';
import { CodebaseContext } from '@/services/ideate';

export interface CodebaseContextPanelProps {
  sessionId: string;
  projectPath: string | null;
  context: CodebaseContext | null;
  isAnalyzing: boolean;
  onAnalyze: () => void;
  className?: string;
}

export function CodebaseContextPanel({
  projectPath,
  context,
  isAnalyzing,
  onAnalyze,
  className = '',
}: CodebaseContextPanelProps) {
  const [expandedSections, setExpandedSections] = useState<Record<string, boolean>>({
    patterns: true,
    features: true,
    components: true,
  });

  const toggleSection = (section: string) => {
    setExpandedSections((prev) => ({
      ...prev,
      [section]: !prev[section],
    }));
  };

  if (!projectPath) {
    return (
      <Card className={className}>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Code2 className="h-5 w-5" />
            Codebase Context
          </CardTitle>
          <CardDescription>
            No project path specified
          </CardDescription>
        </CardHeader>
      </Card>
    );
  }

  if (!context && !isAnalyzing) {
    return (
      <Card className={className}>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Code2 className="h-5 w-5" />
            Codebase Context
          </CardTitle>
          <CardDescription>
            Analyze your codebase to find patterns, similar features, and reusable components
          </CardDescription>
        </CardHeader>
        <CardContent>
          <Button onClick={onAnalyze} className="w-full" variant="outline">
            <Sparkles className="h-4 w-4 mr-2" />
            Analyze Codebase
          </Button>
        </CardContent>
      </Card>
    );
  }

  if (isAnalyzing) {
    return (
      <Card className={className}>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Loader2 className="h-5 w-5 animate-spin" />
            Analyzing Codebase...
          </CardTitle>
          <CardDescription>
            Scanning for patterns, features, and components
          </CardDescription>
        </CardHeader>
      </Card>
    );
  }

  if (!context) {
    return null;
  }

  return (
    <Card className={className}>
      <CardHeader>
        <div className="flex items-center justify-between">
          <CardTitle className="flex items-center gap-2">
            <Code2 className="h-5 w-5" />
            Codebase Context
          </CardTitle>
          <Button onClick={onAnalyze} size="sm" variant="ghost">
            <Sparkles className="h-3 w-3 mr-1" />
            Re-analyze
          </Button>
        </div>
        <CardDescription>
          Architecture: <Badge variant="outline">{context.architecture_style}</Badge>
        </CardDescription>
      </CardHeader>
      <CardContent>
        <ScrollArea className="h-[400px]">
          <div className="space-y-4">
            {/* Patterns Section */}
            {context.patterns && context.patterns.length > 0 && (
              <div>
                <button
                  onClick={() => toggleSection('patterns')}
                  className="flex items-center justify-between w-full text-left mb-2 hover:text-primary"
                >
                  <div className="flex items-center gap-2 font-medium">
                    <Layers className="h-4 w-4" />
                    Patterns ({context.patterns.length})
                  </div>
                  {expandedSections.patterns ? (
                    <ChevronUp className="h-4 w-4" />
                  ) : (
                    <ChevronDown className="h-4 w-4" />
                  )}
                </button>
                {expandedSections.patterns && (
                  <div className="space-y-2 pl-6">
                    {context.patterns.map((pattern, idx) => (
                      <div key={idx} className="text-sm border-l-2 pl-2 border-muted">
                        <div className="font-medium">{pattern.name}</div>
                        <div className="text-muted-foreground">{pattern.description}</div>
                        <div className="text-xs text-muted-foreground mt-1">
                          Type: {pattern.pattern_type}
                        </div>
                      </div>
                    ))}
                  </div>
                )}
              </div>
            )}

            {/* Similar Features Section */}
            {context.similar_features && context.similar_features.length > 0 && (
              <div>
                <button
                  onClick={() => toggleSection('features')}
                  className="flex items-center justify-between w-full text-left mb-2 hover:text-primary"
                >
                  <div className="flex items-center gap-2 font-medium">
                    <FileCode className="h-4 w-4" />
                    Similar Features ({context.similar_features.length})
                  </div>
                  {expandedSections.features ? (
                    <ChevronUp className="h-4 w-4" />
                  ) : (
                    <ChevronDown className="h-4 w-4" />
                  )}
                </button>
                {expandedSections.features && (
                  <div className="space-y-2 pl-6">
                    {context.similar_features.map((feature, idx) => (
                      <div key={idx} className="text-sm border-l-2 pl-2 border-muted">
                        <div className="font-medium">{feature.name}</div>
                        <div className="text-muted-foreground text-xs">{feature.description}</div>
                        <div className="text-xs text-muted-foreground mt-1">
                          {feature.file_path}
                        </div>
                        <Badge variant="secondary" className="text-xs mt-1">
                          {Math.round(feature.similarity_score * 100)}% similar
                        </Badge>
                      </div>
                    ))}
                  </div>
                )}
              </div>
            )}

            {/* Reusable Components Section */}
            {context.reusable_components && context.reusable_components.length > 0 && (
              <div>
                <button
                  onClick={() => toggleSection('components')}
                  className="flex items-center justify-between w-full text-left mb-2 hover:text-primary"
                >
                  <div className="flex items-center gap-2 font-medium">
                    <Layers className="h-4 w-4" />
                    Reusable Components ({context.reusable_components.length})
                  </div>
                  {expandedSections.components ? (
                    <ChevronUp className="h-4 w-4" />
                  ) : (
                    <ChevronDown className="h-4 w-4" />
                  )}
                </button>
                {expandedSections.components && (
                  <div className="space-y-1 pl-6">
                    {context.reusable_components.map((component, idx) => (
                      <div key={idx} className="text-sm text-muted-foreground">
                        • {component}
                      </div>
                    ))}
                  </div>
                )}
              </div>
            )}
          </div>
        </ScrollArea>
      </CardContent>
    </Card>
  );
}
