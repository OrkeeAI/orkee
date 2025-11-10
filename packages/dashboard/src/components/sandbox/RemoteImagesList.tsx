// ABOUTME: Remote Docker Hub images list component
// ABOUTME: Searchable list with tabs for search results and user images

import { useEffect, useState, useCallback } from 'react';
import { Input } from '@/components/ui/input';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Search, RefreshCw, Star, Download, Shield } from 'lucide-react';
import {
  searchDockerHubImages,
  listUserDockerHubImages,
  type DockerHubImage,
} from '@/services/docker';
import { setDefaultImage } from '@/services/sandbox';
import { useToast } from '@/hooks/use-toast';

interface RemoteImagesListProps {
  username?: string | null;
  isLoggedIn?: boolean;
}

// Debounce hook
function useDebounce<T>(value: T, delay: number): T {
  const [debouncedValue, setDebouncedValue] = useState<T>(value);

  useEffect(() => {
    const handler = setTimeout(() => {
      setDebouncedValue(value);
    }, delay);

    return () => {
      clearTimeout(handler);
    };
  }, [value, delay]);

  return debouncedValue;
}

export function RemoteImagesList({ username, isLoggedIn }: RemoteImagesListProps) {
  const [searchQuery, setSearchQuery] = useState('');
  const [searchResults, setSearchResults] = useState<DockerHubImage[]>([]);
  const [userImages, setUserImages] = useState<DockerHubImage[]>([]);
  const [activeTab, setActiveTab] = useState<'search' | 'user'>('search');
  const [isLoading, setIsLoading] = useState(false);
  const { toast } = useToast();

  const debouncedSearchQuery = useDebounce(searchQuery, 500);

  // Search Docker Hub
  const performSearch = useCallback(async (query: string) => {
    if (query.length < 3) {
      setSearchResults([]);
      return;
    }

    try {
      setIsLoading(true);
      const results = await searchDockerHubImages(query, 25);
      setSearchResults(results);
    } catch (error) {
      toast({
        title: 'Search failed',
        description: error instanceof Error ? error.message : 'Unknown error',
        variant: 'destructive',
      });
    } finally {
      setIsLoading(false);
    }
  }, [toast]);

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

  // Trigger search when debounced query changes
  useEffect(() => {
    if (activeTab === 'search') {
      performSearch(debouncedSearchQuery);
    }
  }, [debouncedSearchQuery, activeTab, performSearch]);

  // Load user images when tab changes or login status changes
  useEffect(() => {
    if (activeTab === 'user') {
      loadUserImages();
    }
  }, [activeTab, loadUserImages]);

  const handleUseImage = async (image: DockerHubImage) => {
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
          <Button size="sm" onClick={() => handleUseImage(image)}>
            Use Image
          </Button>
        </div>
      </CardContent>
    </Card>
  );

  return (
    <div className="space-y-4">
      <Tabs value={activeTab} onValueChange={(v) => setActiveTab(v as 'search' | 'user')}>
        <TabsList className="grid w-full grid-cols-2">
          <TabsTrigger value="search">Search Results</TabsTrigger>
          <TabsTrigger value="user" disabled={!isLoggedIn}>
            My Images {!isLoggedIn && '(Login Required)'}
          </TabsTrigger>
        </TabsList>

        <TabsContent value="search" className="space-y-4">
          <div className="relative">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
            <Input
              placeholder="Search Docker Hub (min 3 characters)..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="pl-9"
            />
          </div>

          {isLoading ? (
            <div className="flex items-center justify-center py-8">
              <RefreshCw className="h-6 w-6 animate-spin text-muted-foreground" />
            </div>
          ) : searchQuery.length < 3 ? (
            <div className="text-center py-8 text-sm text-muted-foreground">
              Enter at least 3 characters to search
            </div>
          ) : searchResults.length === 0 ? (
            <div className="text-center py-8 text-sm text-muted-foreground">
              No results found for "{searchQuery}"
            </div>
          ) : (
            <div className="space-y-3 max-h-[600px] overflow-y-auto">
              {searchResults.map((image) => (
                <ImageCard key={image.name} image={image} />
              ))}
            </div>
          )}
        </TabsContent>

        <TabsContent value="user" className="space-y-4">
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
        </TabsContent>
      </Tabs>
    </div>
  );
}
