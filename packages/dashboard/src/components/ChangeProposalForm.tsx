// ABOUTME: Form component for creating spec change proposals
// ABOUTME: Allows users to create change proposals with proposal, tasks, and design markdown

import { useState } from 'react';
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle, DialogTrigger } from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Badge } from '@/components/ui/badge';
import { FilePlus, AlertCircle, CheckCircle, Loader2 } from 'lucide-react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { queryKeys } from '@/lib/queryClient';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';

interface ChangeProposalFormProps {
  projectId: string;
  prdId?: string;
  trigger?: React.ReactNode;
}

interface ChangeProposalFormData {
  changeId: string;
  proposalMarkdown: string;
  tasksMarkdown: string;
  designMarkdown?: string;
  createdBy: string;
}

export function ChangeProposalForm({ projectId, prdId, trigger }: ChangeProposalFormProps) {
  const [open, setOpen] = useState(false);
  const [changeId, setChangeId] = useState('');
  const [proposalMarkdown, setProposalMarkdown] = useState('');
  const [tasksMarkdown, setTasksMarkdown] = useState('');
  const [designMarkdown, setDesignMarkdown] = useState('');
  const [createdBy, setCreatedBy] = useState('user'); // TODO: Get from auth context
  const [selectedTab, setSelectedTab] = useState('proposal');

  const queryClient = useQueryClient();

  const createChangeMutation = useMutation({
    mutationFn: async (data: ChangeProposalFormData) => {
      const response = await fetch(`/api/${projectId}/changes`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          prdId,
          proposalMarkdown: data.proposalMarkdown,
          tasksMarkdown: data.tasksMarkdown,
          designMarkdown: data.designMarkdown || undefined,
          createdBy: data.createdBy,
        }),
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || 'Failed to create change proposal');
      }

      return response.json();
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.projects });
      resetForm();
      setOpen(false);
    },
  });

  const resetForm = () => {
    setChangeId('');
    setProposalMarkdown('');
    setTasksMarkdown('');
    setDesignMarkdown('');
    setSelectedTab('proposal');
  };

  const handleSubmit = () => {
    if (!proposalMarkdown.trim() || !tasksMarkdown.trim()) {
      return;
    }

    createChangeMutation.mutate({
      changeId,
      proposalMarkdown,
      tasksMarkdown,
      designMarkdown: designMarkdown.trim() || undefined,
      createdBy,
    });
  };

  const canSubmit = proposalMarkdown.trim() && tasksMarkdown.trim() && !createChangeMutation.isPending;

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        {trigger || (
          <Button variant="outline">
            <FilePlus className="mr-2 h-4 w-4" />
            New Change Proposal
          </Button>
        )}
      </DialogTrigger>
      <DialogContent className="max-w-4xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>Create Change Proposal</DialogTitle>
          <DialogDescription>
            Propose changes to specifications with detailed proposal, tasks, and optional design notes.
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4">
          {/* Change ID */}
          <div className="space-y-2">
            <Label htmlFor="changeId">
              Change ID <span className="text-muted-foreground text-xs">(e.g., "add-oauth-support")</span>
            </Label>
            <Input
              id="changeId"
              placeholder="kebab-case-id"
              value={changeId}
              onChange={(e) => setChangeId(e.target.value)}
            />
          </div>

          {/* Tabbed Markdown Editors */}
          <Tabs value={selectedTab} onValueChange={setSelectedTab}>
            <TabsList className="grid w-full grid-cols-3">
              <TabsTrigger value="proposal">
                Proposal {!proposalMarkdown.trim() && <Badge variant="destructive" className="ml-2">Required</Badge>}
              </TabsTrigger>
              <TabsTrigger value="tasks">
                Tasks {!tasksMarkdown.trim() && <Badge variant="destructive" className="ml-2">Required</Badge>}
              </TabsTrigger>
              <TabsTrigger value="design">
                Design <Badge variant="secondary" className="ml-2">Optional</Badge>
              </TabsTrigger>
            </TabsList>

            <TabsContent value="proposal" className="space-y-4">
              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label>Proposal Markdown</Label>
                  <Textarea
                    placeholder="# Proposal Title

## Problem Statement
Describe the problem this change addresses...

## Proposed Solution
Describe the proposed solution...

## Impact
What will change and what won't..."
                    value={proposalMarkdown}
                    onChange={(e) => setProposalMarkdown(e.target.value)}
                    className="min-h-[400px] font-mono text-sm"
                  />
                  <p className="text-xs text-muted-foreground">
                    {proposalMarkdown.length} characters
                  </p>
                </div>
                <div className="space-y-2">
                  <Label>Preview</Label>
                  <div className="border rounded-md p-4 min-h-[400px] overflow-y-auto prose prose-sm dark:prose-invert max-w-none">
                    {proposalMarkdown ? (
                      <ReactMarkdown remarkPlugins={[remarkGfm]}>
                        {proposalMarkdown}
                      </ReactMarkdown>
                    ) : (
                      <p className="text-muted-foreground">Proposal preview will appear here</p>
                    )}
                  </div>
                </div>
              </div>
            </TabsContent>

            <TabsContent value="tasks" className="space-y-4">
              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label>Tasks Markdown</Label>
                  <Textarea
                    placeholder="# Implementation Tasks

- [ ] Task 1: Description
- [ ] Task 2: Description
- [ ] Task 3: Description

## Testing Tasks
- [ ] Unit tests for X
- [ ] Integration tests for Y"
                    value={tasksMarkdown}
                    onChange={(e) => setTasksMarkdown(e.target.value)}
                    className="min-h-[400px] font-mono text-sm"
                  />
                  <p className="text-xs text-muted-foreground">
                    {tasksMarkdown.length} characters
                  </p>
                </div>
                <div className="space-y-2">
                  <Label>Preview</Label>
                  <div className="border rounded-md p-4 min-h-[400px] overflow-y-auto prose prose-sm dark:prose-invert max-w-none">
                    {tasksMarkdown ? (
                      <ReactMarkdown remarkPlugins={[remarkGfm]}>
                        {tasksMarkdown}
                      </ReactMarkdown>
                    ) : (
                      <p className="text-muted-foreground">Tasks preview will appear here</p>
                    )}
                  </div>
                </div>
              </div>
            </TabsContent>

            <TabsContent value="design" className="space-y-4">
              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label>Design Markdown (Optional)</Label>
                  <Textarea
                    placeholder="# Design Notes

## Architecture
Describe architectural changes...

## API Changes
Document new or modified APIs...

## Data Models
Describe schema changes..."
                    value={designMarkdown}
                    onChange={(e) => setDesignMarkdown(e.target.value)}
                    className="min-h-[400px] font-mono text-sm"
                  />
                  <p className="text-xs text-muted-foreground">
                    {designMarkdown.length} characters
                  </p>
                </div>
                <div className="space-y-2">
                  <Label>Preview</Label>
                  <div className="border rounded-md p-4 min-h-[400px] overflow-y-auto prose prose-sm dark:prose-invert max-w-none">
                    {designMarkdown ? (
                      <ReactMarkdown remarkPlugins={[remarkGfm]}>
                        {designMarkdown}
                      </ReactMarkdown>
                    ) : (
                      <p className="text-muted-foreground">Design notes preview will appear here</p>
                    )}
                  </div>
                </div>
              </div>
            </TabsContent>
          </Tabs>

          {/* Error/Success Messages */}
          {createChangeMutation.isError && (
            <Alert variant="destructive">
              <AlertCircle className="h-4 w-4" />
              <AlertDescription>
                {createChangeMutation.error instanceof Error
                  ? createChangeMutation.error.message
                  : 'Failed to create change proposal'}
              </AlertDescription>
            </Alert>
          )}

          {createChangeMutation.isSuccess && (
            <Alert>
              <CheckCircle className="h-4 w-4" />
              <AlertDescription>
                Change proposal created successfully!
              </AlertDescription>
            </Alert>
          )}

          {/* Actions */}
          <div className="flex justify-end gap-2">
            <Button
              variant="outline"
              onClick={() => setOpen(false)}
              disabled={createChangeMutation.isPending}
            >
              Cancel
            </Button>
            <Button
              onClick={handleSubmit}
              disabled={!canSubmit}
            >
              {createChangeMutation.isPending && (
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              )}
              Create Proposal
            </Button>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}
