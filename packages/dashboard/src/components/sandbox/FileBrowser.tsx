// ABOUTME: File browser component for navigating sandbox filesystem
// ABOUTME: Supports file viewing, editing, and basic file operations

import { useState, useEffect } from 'react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { useToast } from '@/hooks/use-toast'
import {
  listSandboxFiles,
  readSandboxFile,
  writeSandboxFile,
  deleteSandboxFile,
  type SandboxFile,
} from '@/services/sandbox'
import {
  File,
  Folder,
  FolderOpen,
  ChevronRight,
  Home,
  RefreshCw,
  Plus,
  Trash2,
  Eye,
  Edit,
} from 'lucide-react'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Badge } from '@/components/ui/badge'

interface FileBrowserProps {
  sandboxId: string
}

export function FileBrowser({ sandboxId }: FileBrowserProps) {
  const { toast } = useToast()
  const [currentPath, setCurrentPath] = useState('/')
  const [files, setFiles] = useState<SandboxFile[]>([])
  const [loading, setLoading] = useState(false)
  const [selectedFile, setSelectedFile] = useState<SandboxFile | null>(null)
  const [fileContent, setFileContent] = useState('')
  const [isViewDialogOpen, setIsViewDialogOpen] = useState(false)
  const [isEditDialogOpen, setIsEditDialogOpen] = useState(false)

  useEffect(() => {
    loadFiles(currentPath)
  }, [sandboxId, currentPath])

  const loadFiles = async (path: string) => {
    setLoading(true)
    try {
      const fileList = await listSandboxFiles(sandboxId, path)
      setFiles(fileList)
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Failed to load files'
      toast({
        title: 'Failed to load files',
        description: errorMessage,
        variant: 'destructive',
      })
    } finally {
      setLoading(false)
    }
  }

  const handleFileClick = async (file: SandboxFile) => {
    if (file.is_directory) {
      setCurrentPath(file.path)
    } else {
      setSelectedFile(file)
      try {
        const content = await readSandboxFile(sandboxId, file.path)
        setFileContent(content)
        setIsViewDialogOpen(true)
      } catch (error) {
        const errorMessage = error instanceof Error ? error.message : 'Failed to read file'
        toast({
          title: 'Failed to read file',
          description: errorMessage,
          variant: 'destructive',
        })
      }
    }
  }

  const handleEditFile = () => {
    setIsViewDialogOpen(false)
    setIsEditDialogOpen(true)
  }

  const handleSaveFile = async () => {
    if (!selectedFile) return

    try {
      await writeSandboxFile(sandboxId, selectedFile.path, fileContent)
      toast({
        title: 'File saved',
        description: `${selectedFile.name} has been saved successfully`,
      })
      setIsEditDialogOpen(false)
      setSelectedFile(null)
      setFileContent('')
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Failed to save file'
      toast({
        title: 'Failed to save file',
        description: errorMessage,
        variant: 'destructive',
      })
    }
  }

  const handleDeleteFile = async (file: SandboxFile) => {
    if (!confirm(`Are you sure you want to delete ${file.name}?`)) return

    try {
      await deleteSandboxFile(sandboxId, file.path)
      toast({
        title: 'File deleted',
        description: `${file.name} has been deleted successfully`,
      })
      await loadFiles(currentPath)
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Failed to delete file'
      toast({
        title: 'Failed to delete file',
        description: errorMessage,
        variant: 'destructive',
      })
    }
  }

  const navigateUp = () => {
    const parts = currentPath.split('/').filter(Boolean)
    if (parts.length === 0) return
    parts.pop()
    const newPath = '/' + parts.join('/')
    setCurrentPath(newPath || '/')
  }

  const formatFileSize = (bytes: number): string => {
    if (bytes < 1024) return `${bytes} B`
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
  }

  const formatDate = (dateString: string): string => {
    return new Date(dateString).toLocaleString()
  }

  return (
    <div className="flex flex-col h-full border rounded-lg">
      {/* Header */}
      <div className="border-b p-3 bg-card">
        <div className="flex items-center gap-2">
          <Button variant="outline" size="sm" onClick={() => setCurrentPath('/')}>
            <Home className="h-4 w-4" />
          </Button>
          <Button variant="outline" size="sm" onClick={navigateUp} disabled={currentPath === '/'}>
            <ChevronRight className="h-4 w-4 rotate-180" />
          </Button>
          <div className="flex-1 flex items-center gap-1 text-sm text-muted-foreground">
            {currentPath.split('/').filter(Boolean).map((part, idx, arr) => (
              <span key={idx} className="flex items-center">
                <span>{part}</span>
                {idx < arr.length - 1 && <ChevronRight className="h-3 w-3 mx-1" />}
              </span>
            ))}
            {currentPath === '/' && <span>/</span>}
          </div>
          <Button variant="outline" size="sm" onClick={() => loadFiles(currentPath)} disabled={loading}>
            <RefreshCw className={`h-4 w-4 ${loading ? 'animate-spin' : ''}`} />
          </Button>
        </div>
      </div>

      {/* File List */}
      <ScrollArea className="flex-1">
        <div className="p-2">
          {loading ? (
            <div className="flex items-center justify-center py-8 text-muted-foreground">
              <RefreshCw className="h-5 w-5 animate-spin mr-2" />
              Loading...
            </div>
          ) : files.length === 0 ? (
            <div className="flex items-center justify-center py-8 text-muted-foreground">
              Empty directory
            </div>
          ) : (
            <div className="space-y-1">
              {files.map((file) => (
                <div
                  key={file.path}
                  className="flex items-center justify-between p-2 rounded hover:bg-accent cursor-pointer group"
                  onClick={() => handleFileClick(file)}
                >
                  <div className="flex items-center gap-2 flex-1 min-w-0">
                    {file.is_directory ? (
                      <FolderOpen className="h-4 w-4 text-blue-500 flex-shrink-0" />
                    ) : (
                      <File className="h-4 w-4 text-gray-500 flex-shrink-0" />
                    )}
                    <span className="text-sm truncate">{file.name}</span>
                  </div>
                  <div className="flex items-center gap-2">
                    {!file.is_directory && (
                      <>
                        <Badge variant="secondary" className="text-xs">
                          {formatFileSize(file.size)}
                        </Badge>
                        <Button
                          variant="ghost"
                          size="sm"
                          className="opacity-0 group-hover:opacity-100"
                          onClick={(e) => {
                            e.stopPropagation()
                            handleDeleteFile(file)
                          }}
                        >
                          <Trash2 className="h-3 w-3" />
                        </Button>
                      </>
                    )}
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      </ScrollArea>

      {/* View File Dialog */}
      <Dialog open={isViewDialogOpen} onOpenChange={setIsViewDialogOpen}>
        <DialogContent className="max-w-3xl max-h-[80vh]">
          <DialogHeader>
            <DialogTitle>{selectedFile?.name}</DialogTitle>
            <DialogDescription>
              {selectedFile && formatFileSize(selectedFile.size)} • Modified {selectedFile && formatDate(selectedFile.modified_at)}
            </DialogDescription>
          </DialogHeader>
          <ScrollArea className="max-h-96 border rounded p-4 bg-muted">
            <pre className="text-sm font-mono">{fileContent}</pre>
          </ScrollArea>
          <DialogFooter>
            <Button variant="outline" onClick={() => setIsViewDialogOpen(false)}>
              Close
            </Button>
            <Button onClick={handleEditFile}>
              <Edit className="h-4 w-4 mr-2" />
              Edit
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Edit File Dialog */}
      <Dialog open={isEditDialogOpen} onOpenChange={setIsEditDialogOpen}>
        <DialogContent className="max-w-3xl max-h-[80vh]">
          <DialogHeader>
            <DialogTitle>Edit {selectedFile?.name}</DialogTitle>
          </DialogHeader>
          <textarea
            className="w-full h-96 border rounded p-4 font-mono text-sm"
            value={fileContent}
            onChange={(e) => setFileContent(e.target.value)}
          />
          <DialogFooter>
            <Button variant="outline" onClick={() => setIsEditDialogOpen(false)}>
              Cancel
            </Button>
            <Button onClick={handleSaveFile}>
              Save
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
