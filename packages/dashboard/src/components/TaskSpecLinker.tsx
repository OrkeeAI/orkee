import { useState, useMemo } from 'react';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Input } from '@/components/ui/input';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Separator } from '@/components/ui/separator';
import { Alert, AlertDescription } from '@/components/ui/alert';
import {
  Search,
  Link as LinkIcon,
  Unlink,
  CheckCircle2,
  XCircle,
  AlertCircle,
  Loader2,
  ChevronDown,
  ChevronUp,
} from 'lucide-react';
import { useSpecs } from '@/hooks/useSpecs';
import { useTaskSpecLinks, useLinkTaskToRequirement, useValidateTask } from '@/hooks/useTaskSpecLinks';
import type { SpecCapability, SpecRequirement, SpecScenario } from '@/services/specs';

interface TaskSpecLinkerProps {
  projectId: string;
  taskId: string;
  taskTitle: string;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function TaskSpecLinker({
  projectId,
  taskId,
  taskTitle,
  open,
  onOpenChange,
}: TaskSpecLinkerProps) {
  const [searchTerm, setSearchTerm] = useState('');
  const [expandedRequirements, setExpandedRequirements] = useState<Set<string>>(new Set());

  const { data: specs = [], isLoading: specsLoading } = useSpecs(projectId);
  const { data: linkedRequirements = [], isLoading: linksLoading } = useTaskSpecLinks(taskId);
  const linkMutation = useLinkTaskToRequirement(taskId);
  const validateMutation = useValidateTask(taskId);

  const allRequirements = useMemo(() => {
    const reqs: Array<SpecRequirement & { capabilityName: string; capabilityId: string }> = [];
    specs.forEach((spec) => {
      spec.requirements.forEach((req) => {
        reqs.push({
          ...req,
          capabilityName: spec.name,
          capabilityId: spec.id,
        });
      });
    });
    return reqs;
  }, [specs]);

  const filteredRequirements = useMemo(() => {
    if (!searchTerm) return allRequirements;
    const term = searchTerm.toLowerCase();
    return allRequirements.filter(
      (req) =>
        req.name.toLowerCase().includes(term) ||
        req.content.toLowerCase().includes(term) ||
        req.capabilityName.toLowerCase().includes(term)
    );
  }, [allRequirements, searchTerm]);

  const linkedRequirementIds = useMemo(
    () => new Set(linkedRequirements.map((r) => r.id)),
    [linkedRequirements]
  );

  const toggleRequirementExpansion = (requirementId: string) => {
    setExpandedRequirements((prev) => {
      const next = new Set(prev);
      if (next.has(requirementId)) {
        next.delete(requirementId);
      } else {
        next.add(requirementId);
      }
      return next;
    });
  };

  const handleLinkRequirement = async (requirementId: string) => {
    try {
      await linkMutation.mutateAsync({ requirementId });
    } catch (error) {
      console.error('Failed to link requirement:', error);
    }
  };

  const handleValidateTask = async () => {
    try {
      await validateMutation.mutateAsync();
    } catch (error) {
      console.error('Failed to validate task:', error);
    }
  };

  const isLoading = specsLoading || linksLoading;

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-4xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>Link Task to Spec Requirements</DialogTitle>
          <DialogDescription>
            Connect "{taskTitle}" to spec requirements for validation and tracking
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-6">
          {/* Current Links Section */}
          {linkedRequirements.length > 0 && (
            <div>
              <h3 className="text-sm font-medium mb-3">Linked Requirements ({linkedRequirements.length})</h3>
              <div className="space-y-2">
                {linkedRequirements.map((req) => (
                  <Card key={req.id} className="bg-muted/30">
                    <CardHeader className="pb-3">
                      <div className="flex items-start justify-between">
                        <div className="space-y-1">
                          <CardTitle className="text-sm">{req.name}</CardTitle>
                          <CardDescription className="text-xs">
                            {req.scenarios.length} scenario{req.scenarios.length !== 1 ? 's' : ''}
                          </CardDescription>
                        </div>
                        <Badge variant="secondary" className="flex items-center gap-1">
                          <CheckCircle2 className="h-3 w-3" />
                          Linked
                        </Badge>
                      </div>
                    </CardHeader>
                  </Card>
                ))}
              </div>

              {linkedRequirements.length > 0 && (
                <div className="mt-4">
                  <Button
                    onClick={handleValidateTask}
                    disabled={validateMutation.isPending}
                    variant="outline"
                    size="sm"
                  >
                    {validateMutation.isPending ? (
                      <>
                        <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                        Validating...
                      </>
                    ) : (
                      <>
                        <CheckCircle2 className="mr-2 h-4 w-4" />
                        Validate Task
                      </>
                    )}
                  </Button>

                  {validateMutation.data && (
                    <Alert className="mt-3" variant={validateMutation.data.valid ? 'default' : 'destructive'}>
                      <AlertDescription>
                        <div className="space-y-2">
                          <div className="font-medium">
                            {validateMutation.data.valid ? (
                              <span className="text-green-600 flex items-center gap-2">
                                <CheckCircle2 className="h-4 w-4" />
                                All scenarios passed ({validateMutation.data.passedScenarios}/
                                {validateMutation.data.totalScenarios})
                              </span>
                            ) : (
                              <span className="flex items-center gap-2">
                                <XCircle className="h-4 w-4" />
                                Validation failed ({validateMutation.data.passedScenarios}/
                                {validateMutation.data.totalScenarios} scenarios passed)
                              </span>
                            )}
                          </div>
                          {validateMutation.data.errors.length > 0 && (
                            <ul className="list-disc list-inside text-sm space-y-1">
                              {validateMutation.data.errors.map((error, idx) => (
                                <li key={idx}>{error}</li>
                              ))}
                            </ul>
                          )}
                        </div>
                      </AlertDescription>
                    </Alert>
                  )}
                </div>
              )}
            </div>
          )}

          {linkedRequirements.length > 0 && <Separator />}

          {/* Search and Filter */}
          <div>
            <div className="relative">
              <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
              <Input
                placeholder="Search requirements by name, content, or capability..."
                value={searchTerm}
                onChange={(e) => setSearchTerm(e.target.value)}
                className="pl-10"
              />
            </div>
          </div>

          {/* Available Requirements */}
          <div>
            <h3 className="text-sm font-medium mb-3">
              Available Requirements ({filteredRequirements.length})
            </h3>

            {isLoading ? (
              <div className="flex items-center justify-center py-8">
                <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
              </div>
            ) : filteredRequirements.length === 0 ? (
              <Alert>
                <AlertCircle className="h-4 w-4" />
                <AlertDescription>
                  {searchTerm
                    ? 'No requirements match your search.'
                    : 'No spec requirements available. Create a spec capability first.'}
                </AlertDescription>
              </Alert>
            ) : (
              <div className="space-y-3 max-h-96 overflow-y-auto">
                {filteredRequirements.map((req) => {
                  const isLinked = linkedRequirementIds.has(req.id || '');
                  const isExpanded = expandedRequirements.has(req.id || '');

                  return (
                    <Card key={req.id} className={isLinked ? 'bg-muted/30' : ''}>
                      <CardHeader className="pb-3">
                        <div className="flex items-start justify-between gap-3">
                          <div className="flex-1 space-y-2">
                            <div className="flex items-center gap-2">
                              <CardTitle className="text-sm">{req.name}</CardTitle>
                              <Badge variant="outline" className="text-xs">
                                {req.capabilityName}
                              </Badge>
                            </div>
                            <CardDescription className="text-xs line-clamp-2">
                              {req.content}
                            </CardDescription>
                            <div className="flex items-center gap-3 text-xs text-muted-foreground">
                              <span>{req.scenarios.length} scenario{req.scenarios.length !== 1 ? 's' : ''}</span>
                            </div>
                          </div>

                          <div className="flex items-center gap-2 shrink-0">
                            <Button
                              variant="ghost"
                              size="sm"
                              onClick={() => toggleRequirementExpansion(req.id || '')}
                            >
                              {isExpanded ? (
                                <ChevronUp className="h-4 w-4" />
                              ) : (
                                <ChevronDown className="h-4 w-4" />
                              )}
                            </Button>
                            <Button
                              onClick={() => handleLinkRequirement(req.id || '')}
                              disabled={isLinked || linkMutation.isPending}
                              variant={isLinked ? 'secondary' : 'default'}
                              size="sm"
                            >
                              {isLinked ? (
                                <>
                                  <CheckCircle2 className="mr-1 h-3 w-3" />
                                  Linked
                                </>
                              ) : (
                                <>
                                  <LinkIcon className="mr-1 h-3 w-3" />
                                  Link
                                </>
                              )}
                            </Button>
                          </div>
                        </div>
                      </CardHeader>

                      {isExpanded && req.scenarios.length > 0 && (
                        <CardContent className="pt-0">
                          <div className="space-y-2">
                            <div className="text-xs font-medium text-muted-foreground">Scenarios:</div>
                            {req.scenarios.map((scenario, idx) => (
                              <div key={idx} className="pl-4 border-l-2 border-muted space-y-1 text-xs">
                                <div className="font-medium">{scenario.name}</div>
                                <div className="text-muted-foreground">
                                  <strong>WHEN:</strong> {scenario.whenClause}
                                </div>
                                <div className="text-muted-foreground">
                                  <strong>THEN:</strong> {scenario.thenClause}
                                </div>
                                {scenario.andClauses && scenario.andClauses.length > 0 && (
                                  <div className="text-muted-foreground">
                                    <strong>AND:</strong>
                                    <ul className="list-disc list-inside ml-4">
                                      {scenario.andClauses.map((clause, andIdx) => (
                                        <li key={andIdx}>{clause}</li>
                                      ))}
                                    </ul>
                                  </div>
                                )}
                              </div>
                            ))}
                          </div>
                        </CardContent>
                      )}
                    </Card>
                  );
                })}
              </div>
            )}
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}
