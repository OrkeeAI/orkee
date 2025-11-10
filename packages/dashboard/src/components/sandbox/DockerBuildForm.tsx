// ABOUTME: Docker image build form component
// ABOUTME: Form for building Docker images with file picker and validation

import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Hammer, Plus, X } from 'lucide-react';
import { buildDockerImage, type BuildImageRequest, type BuildImageResponse } from '@/services/docker';
import { useToast } from '@/hooks/use-toast';

interface DockerBuildFormProps {
  username?: string | null;
  onBuildStart?: () => void;
  onBuildComplete?: (response: BuildImageResponse) => void;
}

export function DockerBuildForm({ username, onBuildStart, onBuildComplete }: DockerBuildFormProps) {
  const [dockerfilePath, setDockerfilePath] = useState('');
  const [buildContext, setBuildContext] = useState('');
  const [imageName, setImageName] = useState('');
  const [imageTag, setImageTag] = useState('latest');
  const [labels, setLabels] = useState<Record<string, string>>({});
  const [newLabelKey, setNewLabelKey] = useState('');
  const [newLabelValue, setNewLabelValue] = useState('');
  const [isBuilding, setIsBuilding] = useState(false);
  const [errors, setErrors] = useState<Record<string, string>>({});
  const { toast } = useToast();

  const validate = (): boolean => {
    const newErrors: Record<string, string> = {};

    if (!dockerfilePath.trim()) {
      newErrors.dockerfilePath = 'Dockerfile path is required';
    }

    if (!buildContext.trim()) {
      newErrors.buildContext = 'Build context is required';
    }

    if (!imageName.trim()) {
      newErrors.imageName = 'Image name is required';
    } else if (!/^[a-z0-9][a-z0-9._-]*$/.test(imageName)) {
      newErrors.imageName = 'Image name must be lowercase alphanumeric with dots, dashes, or underscores';
    }

    if (!imageTag.trim()) {
      newErrors.imageTag = 'Image tag is required';
    } else if (!/^[a-zA-Z0-9._-]+$/.test(imageTag)) {
      newErrors.imageTag = 'Image tag must be alphanumeric with dots, dashes, or underscores';
    }

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleAddLabel = () => {
    if (newLabelKey.trim() && newLabelValue.trim()) {
      setLabels((prev) => ({
        ...prev,
        [newLabelKey.trim()]: newLabelValue.trim(),
      }));
      setNewLabelKey('');
      setNewLabelValue('');
    }
  };

  const handleRemoveLabel = (key: string) => {
    setLabels((prev) => {
      const updated = { ...prev };
      delete updated[key];
      return updated;
    });
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!validate()) {
      return;
    }

    const fullImageTag = username && !imageName.includes('/')
      ? `${username}/${imageName}:${imageTag}`
      : `${imageName}:${imageTag}`;

    const request: BuildImageRequest = {
      dockerfile_path: dockerfilePath,
      build_context: buildContext,
      image_tag: fullImageTag,
      labels: {
        ...labels,
        'orkee.sandbox': 'true',
      },
    };

    try {
      setIsBuilding(true);
      onBuildStart?.();
      const response = await buildDockerImage(request);

      toast({
        title: 'Build successful',
        description: `Image ${response.image_tag} built successfully`,
      });

      onBuildComplete?.(response);

      // Clear form
      setDockerfilePath('');
      setBuildContext('');
      setImageName('');
      setImageTag('latest');
      setLabels({});
    } catch (error) {
      toast({
        title: 'Build failed',
        description: error instanceof Error ? error.message : 'Unknown error',
        variant: 'destructive',
      });
    } finally {
      setIsBuilding(false);
    }
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Hammer className="h-5 w-5" />
          Build Docker Image
        </CardTitle>
        <CardDescription>
          Build a custom Docker image for your sandboxes
        </CardDescription>
      </CardHeader>
      <CardContent>
        <form onSubmit={handleSubmit} className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="dockerfilePath">
              Dockerfile Path <span className="text-destructive">*</span>
            </Label>
            <Input
              id="dockerfilePath"
              placeholder="/path/to/Dockerfile"
              value={dockerfilePath}
              onChange={(e) => setDockerfilePath(e.target.value)}
              disabled={isBuilding}
            />
            {errors.dockerfilePath && (
              <p className="text-sm text-destructive">{errors.dockerfilePath}</p>
            )}
          </div>

          <div className="space-y-2">
            <Label htmlFor="buildContext">
              Build Context <span className="text-destructive">*</span>
            </Label>
            <Input
              id="buildContext"
              placeholder="/path/to/build/context"
              value={buildContext}
              onChange={(e) => setBuildContext(e.target.value)}
              disabled={isBuilding}
            />
            {errors.buildContext && (
              <p className="text-sm text-destructive">{errors.buildContext}</p>
            )}
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-2">
              <Label htmlFor="imageName">
                Image Name <span className="text-destructive">*</span>
              </Label>
              <Input
                id="imageName"
                placeholder="my-sandbox-image"
                value={imageName}
                onChange={(e) => setImageName(e.target.value)}
                disabled={isBuilding}
              />
              {errors.imageName && (
                <p className="text-sm text-destructive">{errors.imageName}</p>
              )}
              {username && !imageName.includes('/') && imageName && (
                <p className="text-xs text-muted-foreground">
                  Will create: {username}/{imageName}
                </p>
              )}
            </div>

            <div className="space-y-2">
              <Label htmlFor="imageTag">
                Tag <span className="text-destructive">*</span>
              </Label>
              <Input
                id="imageTag"
                placeholder="latest"
                value={imageTag}
                onChange={(e) => setImageTag(e.target.value)}
                disabled={isBuilding}
              />
              {errors.imageTag && (
                <p className="text-sm text-destructive">{errors.imageTag}</p>
              )}
            </div>
          </div>

          <div className="space-y-2">
            <Label>Additional Labels (Optional)</Label>
            <div className="flex gap-2">
              <Input
                placeholder="Key"
                value={newLabelKey}
                onChange={(e) => setNewLabelKey(e.target.value)}
                disabled={isBuilding}
                className="flex-1"
              />
              <Input
                placeholder="Value"
                value={newLabelValue}
                onChange={(e) => setNewLabelValue(e.target.value)}
                disabled={isBuilding}
                className="flex-1"
              />
              <Button
                type="button"
                variant="outline"
                size="icon"
                onClick={handleAddLabel}
                disabled={isBuilding || !newLabelKey || !newLabelValue}
              >
                <Plus className="h-4 w-4" />
              </Button>
            </div>

            {Object.keys(labels).length > 0 && (
              <div className="space-y-2 mt-2">
                {Object.entries(labels).map(([key, value]) => (
                  <div key={key} className="flex items-center justify-between bg-muted p-2 rounded">
                    <span className="text-sm">
                      <span className="font-medium">{key}:</span> {value}
                    </span>
                    <Button
                      type="button"
                      variant="ghost"
                      size="sm"
                      onClick={() => handleRemoveLabel(key)}
                      disabled={isBuilding}
                    >
                      <X className="h-4 w-4" />
                    </Button>
                  </div>
                ))}
              </div>
            )}
          </div>

          <div className="flex gap-2 pt-4">
            <Button
              type="submit"
              disabled={isBuilding}
              className="flex-1"
            >
              {isBuilding ? (
                <>Building...</>
              ) : (
                <>
                  <Hammer className="h-4 w-4 mr-2" />
                  Build Image
                </>
              )}
            </Button>
            <Button
              type="button"
              variant="outline"
              onClick={() => {
                setDockerfilePath('');
                setBuildContext('');
                setImageName('');
                setImageTag('latest');
                setLabels({});
                setErrors({});
              }}
              disabled={isBuilding}
            >
              Clear
            </Button>
          </div>
        </form>
      </CardContent>
    </Card>
  );
}
