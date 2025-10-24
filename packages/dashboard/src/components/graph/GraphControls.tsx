// ABOUTME: Graph visualization controls component
// ABOUTME: Provides layout selection, zoom controls, search, and export functionality

import { Card, CardContent } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { ZoomIn, ZoomOut, Maximize, Download, Search } from 'lucide-react';
import type { GraphFilters } from './DependencyGraph';

interface GraphControlsProps {
  layout: string;
  onLayoutChange: (layout: string) => void;
  filters: GraphFilters;
  onFiltersChange: (filters: GraphFilters) => void;
  onZoomIn?: () => void;
  onZoomOut?: () => void;
  onFitView?: () => void;
  onExport?: () => void;
  cytoscapeRef?: React.RefObject<cytoscape.Core | null>;
}

export function GraphControls({
  layout,
  onLayoutChange,
  filters,
  onFiltersChange,
  onZoomIn,
  onZoomOut,
  onFitView,
  onExport,
  cytoscapeRef,
}: GraphControlsProps) {
  const handleZoomIn = () => {
    if (cytoscapeRef?.current) {
      const cy = cytoscapeRef.current;
      cy.zoom(cy.zoom() * 1.2);
      cy.center();
    } else if (onZoomIn) {
      onZoomIn();
    }
  };

  const handleZoomOut = () => {
    if (cytoscapeRef?.current) {
      const cy = cytoscapeRef.current;
      cy.zoom(cy.zoom() * 0.8);
      cy.center();
    } else if (onZoomOut) {
      onZoomOut();
    }
  };

  const handleFitView = () => {
    if (cytoscapeRef?.current) {
      const cy = cytoscapeRef.current;
      cy.fit(undefined, 50);
    } else if (onFitView) {
      onFitView();
    }
  };

  const handleExport = () => {
    if (cytoscapeRef?.current) {
      const cy = cytoscapeRef.current;
      const png = cy.png({ scale: 2, full: true });

      // Create download link
      const link = document.createElement('a');
      link.download = `graph-${Date.now()}.png`;
      link.href = png;
      link.click();
    } else if (onExport) {
      onExport();
    }
  };

  return (
    <Card>
      <CardContent className="p-4">
        <div className="flex items-center justify-between gap-4 flex-wrap">
          <div className="flex items-center gap-2">
            <Select value={layout} onValueChange={onLayoutChange}>
              <SelectTrigger className="w-[180px]">
                <SelectValue placeholder="Select layout" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="hierarchical">Hierarchical</SelectItem>
                <SelectItem value="force">Force Directed</SelectItem>
                <SelectItem value="circular">Circular</SelectItem>
                <SelectItem value="grid">Grid</SelectItem>
              </SelectContent>
            </Select>

            <div className="flex items-center gap-1">
              <Button
                variant="outline"
                size="icon"
                onClick={handleZoomIn}
                title="Zoom In"
              >
                <ZoomIn className="h-4 w-4" />
              </Button>
              <Button
                variant="outline"
                size="icon"
                onClick={handleZoomOut}
                title="Zoom Out"
              >
                <ZoomOut className="h-4 w-4" />
              </Button>
              <Button
                variant="outline"
                size="icon"
                onClick={handleFitView}
                title="Fit to View"
              >
                <Maximize className="h-4 w-4" />
              </Button>
            </div>
          </div>

          <div className="flex items-center gap-2">
            <div className="relative">
              <Search className="absolute left-2 top-2.5 h-4 w-4 text-muted-foreground" />
              <Input
                placeholder="Search nodes..."
                className="pl-8 w-[200px]"
                value={filters.search || ''}
                onChange={(e) =>
                  onFiltersChange({ ...filters, search: e.target.value })
                }
              />
            </div>

            <Button
              variant="outline"
              size="icon"
              onClick={handleExport}
              title="Export as PNG"
            >
              <Download className="h-4 w-4" />
            </Button>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
