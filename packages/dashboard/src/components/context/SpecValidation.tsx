import { useState } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Progress } from '@/components/ui/progress';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Badge } from '@/components/ui/badge';
import { CheckCircle2, XCircle, AlertCircle, Loader2, Code2, FileCode } from 'lucide-react';

interface SpecValidationProps {
  projectId: string;
  specId: string;
  onValidationComplete?: (results: ValidationResult[]) => void;
}

interface ValidationResult {
  requirement_id: string;
  requirement_name: string;
  status: 'passed' | 'failed' | 'warning' | 'unknown';
  details: string[];
  code_references: CodeReference[];
}

interface CodeReference {
  file_path: string;
  line_number: number;
  symbol_name: string;
  snippet?: string;
}

interface ValidationSummary {
  total_requirements: number;
  passed: number;
  failed: number;
  warnings: number;
  unknown: number;
  completion_percentage: number;
}

export function SpecValidation({ projectId, specId, onValidationComplete }: SpecValidationProps) {
  const [validationResults, setValidationResults] = useState<ValidationResult[]>([]);
  const [summary, setSummary] = useState<ValidationSummary | null>(null);
  const [isValidating, setIsValidating] = useState(false);
  const [error, setError] = useState<string>();

  const runValidation = async () => {
    setIsValidating(true);
    setError(undefined);

    try {
      const response = await fetch(
        `/api/projects/${projectId}/context/validate-spec/${specId}`,
        { 
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
        }
      );

      if (!response.ok) {
        const errorData = await response.json().catch(() => ({}));
        throw new Error(errorData.message || 'Failed to validate spec');
      }

      const result = await response.json();
      
      setValidationResults(result.validations || []);
      
      // Calculate summary
      const total = result.validations?.length || 0;
      const passed = result.validations?.filter((v: ValidationResult) => v.status === 'passed').length || 0;
      const failed = result.validations?.filter((v: ValidationResult) => v.status === 'failed').length || 0;
      const warnings = result.validations?.filter((v: ValidationResult) => v.status === 'warning').length || 0;
      const unknown = result.validations?.filter((v: ValidationResult) => v.status === 'unknown').length || 0;
      
      setSummary({
        total_requirements: total,
        passed,
        failed,
        warnings,
        unknown,
        completion_percentage: total > 0 ? Math.round((passed / total) * 100) : 0,
      });

      if (onValidationComplete) {
        onValidationComplete(result.validations || []);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to validate spec');
    } finally {
      setIsValidating(false);
    }
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'passed':
        return <CheckCircle2 className="h-5 w-5 text-green-500" />;
      case 'failed':
        return <XCircle className="h-5 w-5 text-red-500" />;
      case 'warning':
        return <AlertCircle className="h-5 w-5 text-yellow-500" />;
      case 'unknown':
        return <AlertCircle className="h-5 w-5 text-gray-400" />;
      default:
        return null;
    }
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'passed':
        return 'border-green-200 bg-green-50';
      case 'failed':
        return 'border-red-200 bg-red-50';
      case 'warning':
        return 'border-yellow-200 bg-yellow-50';
      case 'unknown':
        return 'border-gray-200 bg-gray-50';
      default:
        return 'border-gray-200';
    }
  };

  return (
    <div className="space-y-4">
      {/* Validation Header */}
      <Card>
        <CardHeader>
          <CardTitle>Spec Implementation Validation</CardTitle>
          <CardDescription>
            Verify that code implements all spec requirements and scenarios
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <Button 
            onClick={runValidation} 
            disabled={isValidating}
            className="w-full"
          >
            {isValidating && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
            {isValidating ? 'Validating...' : 'Run Validation'}
          </Button>

          {error && (
            <Alert variant="destructive">
              <AlertDescription>{error}</AlertDescription>
            </Alert>
          )}

          {/* Summary */}
          {summary && (
            <div className="space-y-3">
              <div className="flex items-center justify-between">
                <span className="text-sm font-medium">Completion</span>
                <span className="text-sm font-semibold">{summary.completion_percentage}%</span>
              </div>
              <Progress value={summary.completion_percentage} className="h-2" />
              
              <div className="grid grid-cols-4 gap-2 pt-2">
                <div className="flex flex-col items-center p-2 rounded-lg bg-green-50">
                  <span className="text-xs text-muted-foreground">Passed</span>
                  <span className="text-lg font-semibold text-green-600">{summary.passed}</span>
                </div>
                <div className="flex flex-col items-center p-2 rounded-lg bg-red-50">
                  <span className="text-xs text-muted-foreground">Failed</span>
                  <span className="text-lg font-semibold text-red-600">{summary.failed}</span>
                </div>
                <div className="flex flex-col items-center p-2 rounded-lg bg-yellow-50">
                  <span className="text-xs text-muted-foreground">Warnings</span>
                  <span className="text-lg font-semibold text-yellow-600">{summary.warnings}</span>
                </div>
                <div className="flex flex-col items-center p-2 rounded-lg bg-gray-50">
                  <span className="text-xs text-muted-foreground">Unknown</span>
                  <span className="text-lg font-semibold text-gray-600">{summary.unknown}</span>
                </div>
              </div>
            </div>
          )}
        </CardContent>
      </Card>

      {/* Validation Results */}
      {validationResults.length > 0 && (
        <div className="space-y-3">
          {validationResults.map((result, index) => (
            <Card key={index} className={`${getStatusColor(result.status)} border`}>
              <CardContent className="pt-6">
                <div className="flex items-start gap-3">
                  {getStatusIcon(result.status)}
                  <div className="flex-1 space-y-2">
                    <div className="flex items-center justify-between">
                      <div className="font-medium">{result.requirement_name}</div>
                      <Badge variant={
                        result.status === 'passed' ? 'default' :
                        result.status === 'failed' ? 'destructive' :
                        result.status === 'warning' ? 'secondary' :
                        'outline'
                      }>
                        {result.status}
                      </Badge>
                    </div>

                    {/* Details */}
                    {result.details && result.details.length > 0 && (
                      <div className="space-y-1">
                        {result.details.map((detail, i) => (
                          <div key={i} className="text-sm text-muted-foreground">
                            {detail}
                          </div>
                        ))}
                      </div>
                    )}

                    {/* Code References */}
                    {result.code_references && result.code_references.length > 0 && (
                      <div className="mt-3 space-y-2">
                        <div className="flex items-center gap-2 text-xs font-medium text-muted-foreground">
                          <Code2 className="h-3 w-3" />
                          <span>Code References</span>
                        </div>
                        {result.code_references.map((ref, i) => (
                          <div 
                            key={i} 
                            className="flex items-start gap-2 text-xs bg-white/50 dark:bg-gray-900/50 rounded p-2"
                          >
                            <FileCode className="h-4 w-4 text-muted-foreground mt-0.5" />
                            <div className="flex-1">
                              <div className="font-mono text-xs">
                                {ref.file_path}:{ref.line_number}
                              </div>
                              <div className="text-muted-foreground">
                                {ref.symbol_name}
                              </div>
                              {ref.snippet && (
                                <pre className="mt-1 text-xs bg-gray-100 dark:bg-gray-800 p-2 rounded overflow-x-auto">
                                  {ref.snippet}
                                </pre>
                              )}
                            </div>
                          </div>
                        ))}
                      </div>
                    )}
                  </div>
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      )}

      {/* Empty State */}
      {!isValidating && validationResults.length === 0 && !error && (
        <Card>
          <CardContent className="pt-6">
            <div className="text-center text-muted-foreground py-8">
              <Shield className="h-12 w-12 mx-auto mb-3 opacity-50" />
              <p>No validation results yet</p>
              <p className="text-sm mt-1">Click "Run Validation" to check spec implementation</p>
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  );
}
