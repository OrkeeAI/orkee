// ABOUTME: Expert persona card component displaying expert details
// ABOUTME: Shows name, role, expertise, bio, and selection state with visual styling

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { User, Star, Check } from 'lucide-react';
import { cn } from '@/lib/utils';
import type { ExpertPersona } from '@/services/ideate';

interface ExpertCardProps {
  expert: ExpertPersona;
  isSelected?: boolean;
  onSelect?: (expert: ExpertPersona) => void;
  onDeselect?: (expert: ExpertPersona) => void;
  showActions?: boolean;
  className?: string;
}

export function ExpertCard({
  expert,
  isSelected = false,
  onSelect,
  onDeselect,
  showActions = true,
  className,
}: ExpertCardProps) {
  const handleClick = () => {
    if (!showActions) return;

    if (isSelected && onDeselect) {
      onDeselect(expert);
    } else if (!isSelected && onSelect) {
      onSelect(expert);
    }
  };

  return (
    <Card
      className={cn(
        'transition-all duration-200 hover:shadow-md',
        isSelected && 'ring-2 ring-primary',
        showActions && 'cursor-pointer',
        className
      )}
      onClick={handleClick}
    >
      <CardHeader className="pb-3">
        <div className="flex items-start justify-between">
          <div className="flex items-center gap-2">
            <div className="flex h-10 w-10 items-center justify-center rounded-full bg-primary/10">
              <User className="h-5 w-5 text-primary" />
            </div>
            <div>
              <CardTitle className="text-lg flex items-center gap-2">
                {expert.name}
                {expert.is_default && (
                  <Star className="h-4 w-4 text-yellow-500 fill-yellow-500" />
                )}
              </CardTitle>
              <CardDescription className="text-sm">{expert.role}</CardDescription>
            </div>
          </div>
          {isSelected && (
            <div className="flex h-6 w-6 items-center justify-center rounded-full bg-primary">
              <Check className="h-4 w-4 text-primary-foreground" />
            </div>
          )}
        </div>
      </CardHeader>
      <CardContent className="space-y-3">
        <div>
          <h4 className="text-sm font-medium mb-1.5">Expertise</h4>
          <div className="flex flex-wrap gap-1.5">
            {expert.expertise.map((skill, index) => (
              <Badge key={index} variant="secondary" className="text-xs">
                {skill}
              </Badge>
            ))}
          </div>
        </div>

        {expert.bio && (
          <div>
            <h4 className="text-sm font-medium mb-1.5">About</h4>
            <p className="text-sm text-muted-foreground line-clamp-3">{expert.bio}</p>
          </div>
        )}

        {showActions && (
          <div className="pt-2">
            {isSelected ? (
              <Button
                variant="outline"
                size="sm"
                onClick={(e) => {
                  e.stopPropagation();
                  onDeselect?.(expert);
                }}
                className="w-full"
              >
                Remove
              </Button>
            ) : (
              <Button
                variant="default"
                size="sm"
                onClick={(e) => {
                  e.stopPropagation();
                  onSelect?.(expert);
                }}
                className="w-full"
              >
                Select
              </Button>
            )}
          </div>
        )}
      </CardContent>
    </Card>
  );
}
