// ABOUTME: Docker Hub user images list component
// ABOUTME: Displays authenticated user's images from Docker Hub registry

import { useEffect, useState, useCallback } from 'react';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { RefreshCw, Star, Download, Shield, ArrowDown } from 'lucide-react';
import {
  listUserDockerHubImages,
  pullDockerImage,
  type DockerHubImage,
} from '@/services/docker';
import { setDefaultImage } from '@/services/sandbox';
import { useToast } from '@/hooks/use-toast';

interface RemoteImagesListProps {
  username?: string | null;
  isLoggedIn?: boolean;
}

export function RemoteImagesList({ username, isLoggedIn }: RemoteImagesListProps) {
  const [userImages, setUserImages] = useState<DockerHubImage[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const { toast } = useToast();

  // Load user images
  const loadUserImages = useCallback(async () => {
    if (!username || !isLoggedIn) {
      setUserImages([]);
      return;
    }

    try {
      setIsLoading(true);
      const results = await listUserDockerHubImages(username);
      setUserImages(results);
    } catch (error) {
      toast({
        title: 'Failed to load user images',
        description: error instanceof Error ? error.message : 'Unknown error',
        variant: 'destructive',
      });
    } finally {
      setIsLoading(false);
    }
  }, [username, isLoggedIn, toast]);

  // Load user images when login status changes
  useEffect(() => {
    loadUserImages();
  }, [loadUserImages]);

  const handlePullImage = async (image: DockerHubImage) => {
    const imageTag = `${image.name}:latest`;
    try {
      await pullDockerImage({ image_tag: imageTag });
      toast({
        title: 'Image pulled successfully',
        description: `${imageTag} has been downloaded to local Docker`,
      });
    } catch (error) {
      toast({
        title: 'Failed to pull image',
        description: error instanceof Error ? error.message : 'Unknown error',
        variant: 'destructive',
      });
    }
  };

  const handleSetDefault = async (image: DockerHubImage) => {
    const imageTag = `${image.name}:latest`;
    try {
      await setDefaultImage(imageTag);
      toast({
        title: 'Default image updated',
        description: `${imageTag} is now the default for new sandboxes`,
      });
    } catch (error) {
      toast({
        title: 'Failed to set default image',
        description: error instanceof Error ? error.message : 'Unknown error',
        variant: 'destructive',
      });
    }
  };

  const formatNumber = (num: number): string => {
    if (num >= 1000000) {
      return `${(num / 1000000).toFixed(1)}M`;
    }
    if (num >= 1000) {
      return `${(num / 1000).toFixed(1)}K`;
    }
    return num.toString();
  };

  const ImageCard = ({ image }: { image: DockerHubImage }) => (
    <Card>
      <CardHeader>
        <div className="flex items-start justify-between">
          <div className="flex-1">
            <CardTitle className="text-base flex items-center gap-2">
              {image.name}
              {image.is_official && (
                <Badge variant="default" className="bg-blue-500">
                  <Shield className="h-3 w-3 mr-1" />
                  Official
                </Badge>
              )}
            </CardTitle>
            <CardDescription className="mt-2 line-clamp-2">
              {image.description || 'No description available'}
            </CardDescription>
          </div>
        </div>
      </CardHeader>
      <CardContent>
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-4 text-sm text-muted-foreground">
            <div className="flex items-center gap-1">
              <Star className="h-4 w-4" />
              {formatNumber(image.star_count)}
            </div>
            <div className="flex items-center gap-1">
              <Download className="h-4 w-4" />
              {formatNumber(image.pull_count)}
            </div>
          </div>
          <div className="flex items-center gap-2">
            <Button size="sm" variant="outline" onClick={() => handlePullImage(image)}>
              <ArrowDown className="h-4 w-4 mr-1" />
              Pull
            </Button>
            <Button size="sm" onClick={() => handleSetDefault(image)}>
              <Star className="h-4 w-4 mr-1" />
              Set as Default
            </Button>
          </div>
        </div>
      </CardContent>
    </Card>
  );

  if (!isLoggedIn) {
    return (
      <div className="text-center py-8 text-sm text-muted-foreground">
        Login to Docker Hub to view your images
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <p className="text-sm text-muted-foreground">
          {userImages.length} image{userImages.length !== 1 ? 's' : ''} found
        </p>
        <Button variant="outline" size="sm" onClick={loadUserImages}>
          <RefreshCw className="h-4 w-4 mr-2" />
          Refresh
        </Button>
      </div>

      {isLoading ? (
        <div className="flex items-center justify-center py-8">
          <RefreshCw className="h-6 w-6 animate-spin text-muted-foreground" />
        </div>
      ) : userImages.length === 0 ? (
        <div className="text-center py-8 text-sm text-muted-foreground">
          No images found for user {username}
        </div>
      ) : (
        <div className="space-y-3 max-h-[600px] overflow-y-auto">
          {userImages.map((image) => (
            <ImageCard key={image.name} image={image} />
          ))}
        </div>
      )}
    </div>
  );
}
