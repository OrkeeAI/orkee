// ABOUTME: GitHub integration settings component for project configuration
// ABOUTME: Allows configuration of GitHub owner, repo, token, labels, and sync settings

import { useState, useEffect } from 'react';
import { Github, Save, AlertCircle, CheckCircle2, ExternalLink, Key } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Switch } from '@/components/ui/switch';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { projectsService, type GitHubConfig } from '@/services/projects';
import { useToast } from '@/hooks/use-toast';

interface GitHubSettingsProps {
  projectId: string;
}

export function GitHubSettings({ projectId }: GitHubSettingsProps) {
  const [config, setConfig] = useState<GitHubConfig>({
    githubSyncEnabled: false,
  });
  const [isLoading, setIsLoading] = useState(true);
  const [isSaving, setIsSaving] = useState(false);
  const [hasToken, setHasToken] = useState(false);
  const [tokenInput, setTokenInput] = useState('');
  const { toast } = useToast();

  useEffect(() => {
    loadConfig();
  }, [projectId]);

  const loadConfig = async () => {
    setIsLoading(true);
    try {
      const githubConfig = await projectsService.getGitHubConfig(projectId);
      if (githubConfig) {
        setConfig(githubConfig);
        setHasToken(!!githubConfig.githubTokenEncrypted);
      }
    } catch (error) {
      console.error('Failed to load GitHub config:', error);
      toast({
        title: 'Failed to load configuration',
        description: error instanceof Error ? error.message : 'Unknown error',
        variant: 'destructive',
      });
    } finally {
      setIsLoading(false);
    }
  };

  const handleSave = async () => {
    setIsSaving(true);
    try {
      // If token is provided, encrypt and save it
      const updates: Partial<GitHubConfig> = {
        ...config,
      };

      if (tokenInput.trim()) {
        // Token encryption happens on the backend
        updates.githubTokenEncrypted = tokenInput.trim();
      }

      await projectsService.updateGitHubConfig(projectId, updates);

      toast({
        title: 'Settings saved',
        description: 'GitHub integration settings have been updated',
      });

      setTokenInput('');
      await loadConfig();
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Failed to save settings';
      toast({
        title: 'Save failed',
        description: errorMessage,
        variant: 'destructive',
      });
    } finally {
      setIsSaving(false);
    }
  };

  if (isLoading) {
    return (
      <Card>
        <CardHeader>
          <div className="flex items-center gap-2">
            <Github className="h-5 w-5" />
            <CardTitle>GitHub Integration</CardTitle>
          </div>
        </CardHeader>
        <CardContent>
          <p className="text-muted-foreground">Loading...</p>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center gap-2">
          <Github className="h-5 w-5" />
          <CardTitle>GitHub Integration</CardTitle>
        </div>
        <CardDescription>
          Configure GitHub integration for syncing Epics and Tasks to Issues
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        {/* Enable/Disable Toggle */}
        <div className="flex items-center justify-between p-3 border rounded-md">
          <div className="space-y-0.5">
            <Label htmlFor="github-enabled">Enable GitHub Sync</Label>
            <p className="text-sm text-muted-foreground">
              Sync Epics and Tasks to GitHub Issues
            </p>
          </div>
          <Switch
            id="github-enabled"
            checked={config.githubSyncEnabled}
            onCheckedChange={(checked) =>
              setConfig({ ...config, githubSyncEnabled: checked })
            }
            disabled={isSaving}
          />
        </div>

        {/* Repository Configuration */}
        {config.githubSyncEnabled && (
          <>
            <Alert>
              <AlertCircle className="h-4 w-4" />
              <AlertDescription className="text-xs">
                Configure your GitHub repository details. You'll need a personal access token with
                <code className="mx-1 px-1 py-0.5 bg-muted rounded text-xs">repo</code> permissions.
              </AlertDescription>
            </Alert>

            <div className="space-y-4">
              {/* Owner */}
              <div className="space-y-2">
                <Label htmlFor="github-owner">Repository Owner</Label>
                <Input
                  id="github-owner"
                  placeholder="username or org-name"
                  value={config.githubOwner || ''}
                  onChange={(e) =>
                    setConfig({ ...config, githubOwner: e.target.value })
                  }
                  disabled={isSaving}
                />
                <p className="text-xs text-muted-foreground">
                  The GitHub username or organization that owns the repository
                </p>
              </div>

              {/* Repo */}
              <div className="space-y-2">
                <Label htmlFor="github-repo">Repository Name</Label>
                <Input
                  id="github-repo"
                  placeholder="repository-name"
                  value={config.githubRepo || ''}
                  onChange={(e) =>
                    setConfig({ ...config, githubRepo: e.target.value })
                  }
                  disabled={isSaving}
                />
                <p className="text-xs text-muted-foreground">
                  The name of the GitHub repository
                </p>
              </div>

              {/* Repository URL Preview */}
              {config.githubOwner && config.githubRepo && (
                <Alert>
                  <Github className="h-4 w-4" />
                  <AlertDescription className="text-xs">
                    Repository:
                    <a
                      href={`https://github.com/${config.githubOwner}/${config.githubRepo}`}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="ml-1 text-primary hover:underline inline-flex items-center gap-1"
                    >
                      github.com/{config.githubOwner}/{config.githubRepo}
                      <ExternalLink className="h-3 w-3" />
                    </a>
                  </AlertDescription>
                </Alert>
              )}

              {/* Personal Access Token */}
              <div className="space-y-2 pt-4 border-t">
                <div className="flex items-center justify-between">
                  <Label htmlFor="github-token">Personal Access Token</Label>
                  {hasToken && (
                    <span className="text-xs text-green-600 flex items-center gap-1">
                      <CheckCircle2 className="h-3 w-3" />
                      Token configured
                    </span>
                  )}
                </div>
                <Input
                  id="github-token"
                  type="password"
                  placeholder={hasToken ? '••••••••••••' : 'ghp_...'}
                  value={tokenInput}
                  onChange={(e) => setTokenInput(e.target.value)}
                  disabled={isSaving}
                />
                <p className="text-xs text-muted-foreground">
                  Create a personal access token at{' '}
                  <a
                    href="https://github.com/settings/tokens/new"
                    target="_blank"
                    rel="noopener noreferrer"
                    className="text-primary hover:underline"
                  >
                    GitHub Settings
                  </a>
                  {' '}with <code className="px-1 py-0.5 bg-muted rounded">repo</code> permissions.
                  Leave empty to keep existing token.
                </p>
              </div>

              {/* Default Assignee */}
              <div className="space-y-2">
                <Label htmlFor="github-assignee">Default Assignee (Optional)</Label>
                <Input
                  id="github-assignee"
                  placeholder="username"
                  value={config.githubDefaultAssignee || ''}
                  onChange={(e) =>
                    setConfig({ ...config, githubDefaultAssignee: e.target.value })
                  }
                  disabled={isSaving}
                />
                <p className="text-xs text-muted-foreground">
                  Automatically assign new issues to this GitHub username
                </p>
              </div>
            </div>
          </>
        )}

        {/* Save Button */}
        <div className="pt-4 border-t">
          <Button onClick={handleSave} disabled={isSaving} className="w-full">
            {isSaving ? (
              <>
                <Key className="mr-2 h-4 w-4 animate-spin" />
                Saving...
              </>
            ) : (
              <>
                <Save className="mr-2 h-4 w-4" />
                Save GitHub Settings
              </>
            )}
          </Button>
        </div>

        {/* Security Notice */}
        <Alert>
          <Key className="h-4 w-4" />
          <AlertDescription className="text-xs">
            Your GitHub token is encrypted and stored securely in your local database. It never
            leaves your machine.
          </AlertDescription>
        </Alert>
      </CardContent>
    </Card>
  );
}
