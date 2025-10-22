import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Info } from 'lucide-react';

interface ContextHistoryProps {
  projectId: string;
}

export function ContextHistory({ projectId }: ContextHistoryProps) {
  return (
    <Card>
      <CardHeader>
        <CardTitle>Context History</CardTitle>
        <CardDescription>
          View and restore previously generated context snapshots
        </CardDescription>
      </CardHeader>
      <CardContent>
        <Alert>
          <Info className="h-4 w-4" />
          <AlertDescription>
            History feature coming in Phase 4. This will allow you to view, compare, and restore
            previously generated context configurations.
          </AlertDescription>
        </Alert>
      </CardContent>
    </Card>
  );
}
