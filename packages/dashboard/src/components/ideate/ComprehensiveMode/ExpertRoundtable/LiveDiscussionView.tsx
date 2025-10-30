// ABOUTME: Real-time discussion view with SSE streaming for roundtable messages
// ABOUTME: Displays live chat interface with expert messages, auto-scroll, and connection status

import { useEffect, useRef } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { MessageSquare, User, Bot, Shield, AlertCircle, Wifi, WifiOff } from 'lucide-react';
import { useRoundtableStream } from '@/hooks/useIdeate';
import type { RoundtableMessage, MessageRole, ExpertPersona } from '@/services/ideate';

interface LiveDiscussionViewProps {
  roundtableId: string;
  participants: ExpertPersona[];
  isActive?: boolean;
}

export function LiveDiscussionView({
  roundtableId,
  participants,
  isActive = true,
}: LiveDiscussionViewProps) {
  const { messages, isConnected, error } = useRoundtableStream(roundtableId, isActive);
  const scrollAreaRef = useRef<HTMLDivElement>(null);
  const bottomRef = useRef<HTMLDivElement>(null);

  // Auto-scroll to bottom when new messages arrive
  useEffect(() => {
    if (bottomRef.current) {
      bottomRef.current.scrollIntoView({ behavior: 'smooth' });
    }
  }, [messages]);

  const getExpertByRole = (expertId: string | null) => {
    if (!expertId) return null;
    return participants.find((p) => p.id === expertId);
  };

  const getRoleIcon = (role: MessageRole) => {
    switch (role) {
      case 'expert':
        return User;
      case 'moderator':
        return Shield;
      case 'user':
        return MessageSquare;
      case 'system':
        return Bot;
      default:
        return MessageSquare;
    }
  };

  const getRoleBadgeVariant = (role: MessageRole): 'default' | 'secondary' | 'outline' | 'destructive' => {
    switch (role) {
      case 'expert':
        return 'default';
      case 'moderator':
        return 'secondary';
      case 'user':
        return 'outline';
      case 'system':
        return 'secondary';
      default:
        return 'secondary';
    }
  };

  const formatTimestamp = (timestamp: string) => {
    const date = new Date(timestamp);
    return date.toLocaleTimeString('en-US', {
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  const renderMessage = (message: RoundtableMessage) => {
    const expert = message.expert_id ? getExpertByRole(message.expert_id) : null;
    const Icon = getRoleIcon(message.role);

    return (
      <div key={message.id} className="group mb-4">
        <div className="flex gap-3">
          <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-primary/10">
            <Icon className="h-4 w-4 text-primary" />
          </div>
          <div className="flex-1 space-y-1">
            <div className="flex items-center gap-2">
              <span className="font-semibold text-sm">
                {expert?.name || message.role}
              </span>
              {expert && (
                <Badge variant="secondary" className="text-xs">
                  {expert.role}
                </Badge>
              )}
              <Badge variant={getRoleBadgeVariant(message.role)} className="text-xs">
                {message.role}
              </Badge>
              <span className="text-xs text-muted-foreground">
                {formatTimestamp(message.created_at)}
              </span>
            </div>
            <div className="text-sm text-foreground whitespace-pre-wrap leading-relaxed">
              {message.content}
            </div>
            {message.metadata && Object.keys(message.metadata).length > 0 && (
              <div className="text-xs text-muted-foreground pt-1">
                {Object.entries(message.metadata).map(([key, value]) => (
                  <span key={key} className="mr-3">
                    {key}: {String(value)}
                  </span>
                ))}
              </div>
            )}
          </div>
        </div>
      </div>
    );
  };

  return (
    <Card>
      <CardHeader className="pb-3">
        <div className="flex items-center justify-between">
          <div>
            <CardTitle className="flex items-center gap-2">
              <MessageSquare className="h-5 w-5" />
              Live Discussion
            </CardTitle>
            <CardDescription>
              Real-time roundtable conversation
            </CardDescription>
          </div>
          <div className="flex items-center gap-2">
            <Badge
              variant={isConnected ? 'default' : 'destructive'}
              className="flex items-center gap-1"
            >
              {isConnected ? (
                <>
                  <Wifi className="h-3 w-3" />
                  Connected
                </>
              ) : (
                <>
                  <WifiOff className="h-3 w-3" />
                  Disconnected
                </>
              )}
            </Badge>
            {messages.length > 0 && (
              <Badge variant="secondary">{messages.length} messages</Badge>
            )}
          </div>
        </div>
      </CardHeader>
      <CardContent>
        {error && (
          <Alert variant="destructive" className="mb-4">
            <AlertCircle className="h-4 w-4" />
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        )}

        <ScrollArea className="h-[600px] pr-4" ref={scrollAreaRef}>
          {messages.length === 0 ? (
            <div className="flex flex-col items-center justify-center h-full text-center p-8">
              <MessageSquare className="h-12 w-12 text-muted-foreground mb-4" />
              <p className="text-sm text-muted-foreground">
                {isActive
                  ? 'Waiting for discussion to begin...'
                  : 'Discussion has not started yet'}
              </p>
            </div>
          ) : (
            <div className="space-y-1">
              {messages.map((message) => renderMessage(message))}
              <div ref={bottomRef} />
            </div>
          )}
        </ScrollArea>
      </CardContent>
    </Card>
  );
}
