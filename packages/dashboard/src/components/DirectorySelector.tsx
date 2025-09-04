import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { FolderOpen } from 'lucide-react';
import { DirectoryBrowser } from './DirectoryBrowser';

interface DirectorySelectorProps {
  id: string;
  label: string;
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  required?: boolean;
}

export function DirectorySelector({ 
  id, 
  label, 
  value, 
  onChange, 
  placeholder = "/path/to/project",
  required = false 
}: DirectorySelectorProps) {
  const [showDirectoryBrowser, setShowDirectoryBrowser] = useState(false);

  const handleBrowse = () => {
    setShowDirectoryBrowser(true);
  };

  const handleDirectorySelect = (path: string) => {
    onChange(path);
    setShowDirectoryBrowser(false);
  };

  return (
    <>
      <div className="grid gap-2">
        <Label htmlFor={id}>{label} {required && '*'}</Label>
        <div className="flex gap-2">
          <Input
            id={id}
            value={value}
            onChange={(e) => onChange(e.target.value)}
            placeholder={placeholder}
            required={required}
            className="flex-1"
          />
          <Button
            type="button"
            variant="outline"
            size="icon"
            onClick={handleBrowse}
            title="Browse for directory"
          >
            <FolderOpen className="h-4 w-4" />
          </Button>
        </div>
      </div>
      
      {showDirectoryBrowser && (
        <DirectoryBrowser
          onSelect={handleDirectorySelect}
          onCancel={() => setShowDirectoryBrowser(false)}
          initialPath={value}
        />
      )}
    </>
  );
}