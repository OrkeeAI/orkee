import { useState, useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Checkbox } from '@/components/ui/checkbox';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Function, Box, FileCode, Variable, Package, FolderTree, Loader2 } from 'lucide-react';
import { useToast } from '@/hooks/use-toast';

interface ASTExplorerProps {
  projectId: string;
  filePath: string;
  onSymbolsSelected: (symbols: SymbolSelection[]) => void;
}

interface Symbol {
  name: string;
  kind: 'function' | 'class' | 'interface' | 'variable' | 'method' | 'struct' | 'enum' | 'trait' | 'module' | 'import' | 'export';
  line_start: number;
  line_end: number;
  children: Symbol[];
  doc_comment?: string;
}

export interface SymbolSelection {
  name: string;
  kind: string;
  lineStart: number;
  lineEnd: number;
}

export function ASTExplorer({ projectId, filePath, onSymbolsSelected }: ASTExplorerProps) {
  const [symbols, setSymbols] = useState<Symbol[]>([]);
  const [selectedSymbols, setSelectedSymbols] = useState<Set<string>>(new Set());
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const { toast } = useToast();

  useEffect(() => {
    if (filePath) {
      loadSymbols();
    }
  }, [filePath, projectId]);

  const loadSymbols = async () => {
    setIsLoading(true);
    setError(null);
    
    try {
      const response = await fetch(
        `/api/projects/${projectId}/ast/symbols?file=${encodeURIComponent(filePath)}`
      );

      if (!response.ok) {
        throw new Error(`Failed to load symbols: ${response.statusText}`);
      }

      const data = await response.json();
      setSymbols(data.symbols || []);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to load symbols';
      setError(errorMessage);
      toast({
        title: 'Error loading AST',
        description: errorMessage,
        variant: 'destructive',
      });
    } finally {
      setIsLoading(false);
    }
  };

  const getSymbolIcon = (kind: string) => {
    switch (kind) {
      case 'function':
        return <Function className="h-4 w-4 text-blue-500" />;
      case 'class':
        return <Box className="h-4 w-4 text-purple-500" />;
      case 'interface':
        return <FileCode className="h-4 w-4 text-green-500" />;
      case 'variable':
        return <Variable className="h-4 w-4 text-yellow-500" />;
      case 'method':
        return <Function className="h-4 w-4 text-cyan-500" />;
      case 'struct':
        return <Box className="h-4 w-4 text-orange-500" />;
      case 'enum':
        return <Package className="h-4 w-4 text-pink-500" />;
      case 'trait':
        return <FileCode className="h-4 w-4 text-indigo-500" />;
      case 'module':
        return <FolderTree className="h-4 w-4 text-gray-500" />;
      default:
        return <FileCode className="h-4 w-4 text-gray-400" />;
    }
  };

  const getSymbolKindColor = (kind: string) => {
    switch (kind) {
      case 'function':
        return 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200';
      case 'class':
        return 'bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-200';
      case 'interface':
        return 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200';
      case 'variable':
        return 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200';
      case 'method':
        return 'bg-cyan-100 text-cyan-800 dark:bg-cyan-900 dark:text-cyan-200';
      case 'struct':
        return 'bg-orange-100 text-orange-800 dark:bg-orange-900 dark:text-orange-200';
      case 'enum':
        return 'bg-pink-100 text-pink-800 dark:bg-pink-900 dark:text-pink-200';
      default:
        return 'bg-gray-100 text-gray-800 dark:bg-gray-900 dark:text-gray-200';
    }
  };

  const toggleSymbol = (symbol: Symbol, path: string) => {
    const newSelected = new Set(selectedSymbols);
    if (newSelected.has(path)) {
      newSelected.delete(path);
    } else {
      newSelected.add(path);
    }
    setSelectedSymbols(newSelected);

    // Notify parent component
    const selections: SymbolSelection[] = [];
    symbols.forEach(s => {
      const symbolPath = s.name;
      if (newSelected.has(symbolPath)) {
        selections.push({
          name: s.name,
          kind: s.kind,
          lineStart: s.line_start,
          lineEnd: s.line_end,
        });
      }
      // Handle nested symbols
      s.children.forEach(child => {
        const childPath = `${s.name}.${child.name}`;
        if (newSelected.has(childPath)) {
          selections.push({
            name: child.name,
            kind: child.kind,
            lineStart: child.line_start,
            lineEnd: child.line_end,
          });
        }
      });
    });
    onSymbolsSelected(selections);
  };

  const selectAll = () => {
    const allPaths = new Set<string>();
    symbols.forEach(s => {
      allPaths.add(s.name);
      s.children.forEach(child => {
        allPaths.add(`${s.name}.${child.name}`);
      });
    });
    setSelectedSymbols(allPaths);
  };

  const deselectAll = () => {
    setSelectedSymbols(new Set());
    onSymbolsSelected([]);
  };

  const renderSymbol = (symbol: Symbol, path: string = '', level: number = 0) => {
    const fullPath = path ? `${path}.${symbol.name}` : symbol.name;
    const isSelected = selectedSymbols.has(fullPath);
    const indent = level * 20;

    return (
      <div key={fullPath}>
        <div 
          className="flex items-center gap-2 py-2 px-2 hover:bg-muted rounded-md transition-colors cursor-pointer"
          style={{ paddingLeft: `${indent + 8}px` }}
          onClick={() => toggleSymbol(symbol, fullPath)}
        >
          <Checkbox
            checked={isSelected}
            onCheckedChange={() => toggleSymbol(symbol, fullPath)}
            onClick={(e) => e.stopPropagation()}
          />
          {getSymbolIcon(symbol.kind)}
          <span className="text-sm font-mono flex-1">{symbol.name}</span>
          <Badge variant="secondary" className={`text-xs ${getSymbolKindColor(symbol.kind)}`}>
            {symbol.kind}
          </Badge>
          <Badge variant="outline" className="text-xs">
            L{symbol.line_start}-{symbol.line_end}
          </Badge>
        </div>
        {symbol.doc_comment && isSelected && (
          <div className="ml-8 mt-1 mb-2 text-xs text-muted-foreground italic border-l-2 border-muted pl-3">
            {symbol.doc_comment}
          </div>
        )}
        {symbol.children.length > 0 && (
          <div className="ml-2">
            {symbol.children.map(child => renderSymbol(child, fullPath, level + 1))}
          </div>
        )}
      </div>
    );
  };

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <CardTitle className="text-lg">Code Structure</CardTitle>
          <div className="flex gap-2">
            <Button
              variant="outline"
              size="sm"
              onClick={selectAll}
              disabled={symbols.length === 0 || isLoading}
            >
              Select All
            </Button>
            <Button
              variant="outline"
              size="sm"
              onClick={deselectAll}
              disabled={selectedSymbols.size === 0 || isLoading}
            >
              Clear
            </Button>
            <Button
              variant="outline"
              size="sm"
              onClick={loadSymbols}
              disabled={isLoading}
            >
              {isLoading ? (
                <>
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  Loading...
                </>
              ) : (
                'Refresh'
              )}
            </Button>
          </div>
        </div>
        {filePath && (
          <p className="text-xs text-muted-foreground mt-2">
            Analyzing: <code className="bg-muted px-1 py-0.5 rounded">{filePath}</code>
          </p>
        )}
      </CardHeader>
      <CardContent>
        {isLoading ? (
          <div className="flex items-center justify-center py-8">
            <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
          </div>
        ) : error ? (
          <div className="text-center py-8">
            <p className="text-sm text-destructive mb-2">{error}</p>
            <Button variant="outline" size="sm" onClick={loadSymbols}>
              Try Again
            </Button>
          </div>
        ) : symbols.length === 0 ? (
          <div className="text-center py-8 text-muted-foreground">
            <FileCode className="h-12 w-12 mx-auto mb-2 opacity-50" />
            <p className="text-sm">No symbols found in this file</p>
            <p className="text-xs mt-1">Select a source file to analyze its structure</p>
          </div>
        ) : (
          <>
            <div className="mb-3 text-xs text-muted-foreground">
              Found {symbols.length} top-level symbol(s)
              {selectedSymbols.size > 0 && ` · ${selectedSymbols.size} selected`}
            </div>
            <ScrollArea className="h-[500px] pr-4">
              <div className="space-y-1">
                {symbols.map(symbol => renderSymbol(symbol))}
              </div>
            </ScrollArea>
          </>
        )}
      </CardContent>
    </Card>
  );
}
