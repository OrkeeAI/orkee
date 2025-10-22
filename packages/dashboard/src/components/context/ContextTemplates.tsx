import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Info } from 'lucide-react';

interface ContextTemplatesProps {
  projectId: string;
}

export function ContextTemplates({ projectId }: ContextTemplatesProps) {
  return (
    <Card>
      <CardHeader>
        <CardTitle>Context Templates</CardTitle>
        <CardDescription>
          Save and reuse context configurations for different scenarios
        </CardDescription>
      </CardHeader>
      <CardContent>
        <Alert>
          <Info className="h-4 w-4" />
          <AlertDescription>
            Templates feature coming in Phase 3. This will allow you to save common context
            configurations and link them to OpenSpec requirements.
          </AlertDescription>
        </Alert>
      </CardContent>
    </Card>
  );
}
