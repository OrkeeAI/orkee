// ABOUTME: Local Docker images list component
// ABOUTME: Displays table of local images with actions (push, delete, set default)

import { useEffect, useState } from 'react';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import { Button } from '@/components/ui/button';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog';
import { RefreshCw, MoreVertical, Trash2, Upload, Star, Copy } from 'lucide-react';
import { listLocalImages, deleteDockerImage, type DockerImage } from '@/services/docker';
import { setDefaultImage } from '@/services/sandbox';
import { useToast } from '@/hooks/use-toast';

interface LocalImagesListProps {
  refreshTrigger?: number;
}

export function LocalImagesList({ refreshTrigger }: LocalImagesListProps) {
  const [images, setImages] = useState<DockerImage[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [selectedImage, setSelectedImage] = useState<DockerImage | null>(null);
  const [showDeleteDialog, setShowDeleteDialog] = useState(false);
  const [isDeleting, setIsDeleting] = useState(false);
  const { toast } = useToast();

  const loadImages = async () => {
    try {
      setIsLoading(true);
      const data = await listLocalImages();
      setImages(data);
    } catch (error) {
      toast({
        title: 'Failed to load local images',
        description: error instanceof Error ? error.message : 'Unknown error',
        variant: 'destructive',
      });
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    loadImages();
  }, [refreshTrigger]);

  const handleDelete = async () => {
    if (!selectedImage) return;

    try {
      setIsDeleting(true);
      await deleteDockerImage({
        image: `${selectedImage.repository}:${selectedImage.tag}`,
        force: false,
      });

      toast({
        title: 'Image deleted',
        description: `Successfully deleted ${selectedImage.repository}:${selectedImage.tag}`,
      });

      setShowDeleteDialog(false);
      setSelectedImage(null);
      loadImages();
    } catch (error) {
      toast({
        title: 'Failed to delete image',
        description: error instanceof Error ? error.message : 'Unknown error',
        variant: 'destructive',
      });
    } finally {
      setIsDeleting(false);
    }
  };

  const handleCopyTag = (image: DockerImage) => {
    const fullTag = `${image.repository}:${image.tag}`;
    navigator.clipboard.writeText(fullTag);
    toast({
      title: 'Copied to clipboard',
      description: fullTag,
    });
  };

  const handlePush = (image: DockerImage) => {
    // TODO: Implement push functionality
    toast({
      title: 'Push functionality coming soon',
      description: `Will push ${image.repository}:${image.tag}`,
    });
  };

  const handleSetDefault = async (image: DockerImage) => {
    const imageTag = `${image.repository}:${image.tag}`;
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

  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-8">
        <RefreshCw className="h-6 w-6 animate-spin text-muted-foreground" />
      </div>
    );
  }

  if (images.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center py-8 text-center">
        <p className="text-sm text-muted-foreground">No local images found</p>
        <p className="text-xs text-muted-foreground mt-2">
          Build an image using the Build tab
        </p>
      </div>
    );
  }

  return (
    <>
      <div className="space-y-4">
        <div className="flex items-center justify-between">
          <p className="text-sm text-muted-foreground">
            {images.length} image{images.length !== 1 ? 's' : ''} found
          </p>
          <Button variant="outline" size="sm" onClick={loadImages}>
            <RefreshCw className="h-4 w-4 mr-2" />
            Refresh
          </Button>
        </div>

        <div className="border rounded-lg">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Repository</TableHead>
                <TableHead>Tag</TableHead>
                <TableHead>Size</TableHead>
                <TableHead>Created</TableHead>
                <TableHead className="w-[50px]"></TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {images.map((image) => (
                <TableRow key={image.image_id}>
                  <TableCell className="font-medium">{image.repository}</TableCell>
                  <TableCell>{image.tag}</TableCell>
                  <TableCell>{image.size}</TableCell>
                  <TableCell className="text-muted-foreground">
                    {image.created}
                  </TableCell>
                  <TableCell>
                    <DropdownMenu>
                      <DropdownMenuTrigger asChild>
                        <Button variant="ghost" size="sm">
                          <MoreVertical className="h-4 w-4" />
                        </Button>
                      </DropdownMenuTrigger>
                      <DropdownMenuContent align="end">
                        <DropdownMenuItem onClick={() => handleCopyTag(image)}>
                          <Copy className="h-4 w-4 mr-2" />
                          Copy image tag
                        </DropdownMenuItem>
                        <DropdownMenuItem onClick={() => handlePush(image)}>
                          <Upload className="h-4 w-4 mr-2" />
                          Push to Docker Hub
                        </DropdownMenuItem>
                        <DropdownMenuItem onClick={() => handleSetDefault(image)}>
                          <Star className="h-4 w-4 mr-2" />
                          Set as default
                        </DropdownMenuItem>
                        <DropdownMenuItem
                          onClick={() => {
                            setSelectedImage(image);
                            setShowDeleteDialog(true);
                          }}
                          className="text-destructive"
                        >
                          <Trash2 className="h-4 w-4 mr-2" />
                          Delete image
                        </DropdownMenuItem>
                      </DropdownMenuContent>
                    </DropdownMenu>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </div>
      </div>

      <AlertDialog open={showDeleteDialog} onOpenChange={setShowDeleteDialog}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Delete Docker Image?</AlertDialogTitle>
            <AlertDialogDescription>
              Are you sure you want to delete{' '}
              <span className="font-medium">
                {selectedImage?.repository}:{selectedImage?.tag}
              </span>
              ? This action cannot be undone.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel disabled={isDeleting}>Cancel</AlertDialogCancel>
            <AlertDialogAction
              onClick={handleDelete}
              disabled={isDeleting}
              className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
            >
              {isDeleting ? (
                <>
                  <RefreshCw className="h-4 w-4 mr-2 animate-spin" />
                  Deleting...
                </>
              ) : (
                'Delete'
              )}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  );
}
