// ABOUTME: Template management UI for sandbox configuration templates
// ABOUTME: Placeholder component for Phase 6 full implementation

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { FileText, Plus } from 'lucide-react'
import { Alert, AlertDescription } from '@/components/ui/alert'

export function TemplateManagement() {
  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <FileText className="h-5 w-5" />
          Sandbox Templates
        </CardTitle>
        <CardDescription>
          Save and reuse sandbox configurations
        </CardDescription>
      </CardHeader>
      <CardContent>
        <Alert>
          <AlertDescription>
            <p className="font-medium mb-2">Template Management - Coming Soon</p>
            <p className="text-sm">
              Template management will be fully implemented in Phase 6 Advanced Features.
              For now, you can create sandboxes with custom configurations directly.
            </p>
          </AlertDescription>
        </Alert>

        <div className="mt-4">
          <Button variant="outline" disabled>
            <Plus className="h-4 w-4 mr-2" />
            Create Template
          </Button>
        </div>
      </CardContent>
    </Card>
  )
}
