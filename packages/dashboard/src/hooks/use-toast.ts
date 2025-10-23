// ABOUTME: Toast notification hook for displaying user feedback messages
// ABOUTME: Provides a simple toast API for success, error, and info messages

import { useState, useCallback } from 'react';

export interface ToastProps {
  title: string;
  description?: string;
  variant?: 'default' | 'destructive';
}

export function useToast() {
  const [, setToasts] = useState<ToastProps[]>([]);

  const toast = useCallback((props: ToastProps) => {
    // Simple implementation - in production, this would integrate with a toast library
    // like sonner or react-hot-toast
    console.log('[Toast]', props.title, props.description);
    
    setToasts((prev) => [...prev, props]);
    
    // Auto-dismiss after 3 seconds
    setTimeout(() => {
      setToasts((prev) => prev.slice(1));
    }, 3000);
  }, []);

  return { toast };
}
