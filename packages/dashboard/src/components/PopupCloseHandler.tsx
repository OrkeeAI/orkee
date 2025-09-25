import { useEffect } from 'react';
import { useLocation } from 'react-router-dom';
import { useCloudAuth } from '@/hooks/useCloud';

export function PopupCloseHandler() {
  const { isAuthenticated } = useCloudAuth();
  const location = useLocation();

  useEffect(() => {
    // Check if we're in a popup window
    if (window.opener && window.opener !== window) {
      const urlParams = new URLSearchParams(location.search);
      
      // Check for OAuth success indicators in URL
      const hasOAuthSuccess = 
        urlParams.get('oauth') === 'success' ||
        urlParams.get('oauth_success') === 'true' ||
        urlParams.get('authenticated') === 'true';
      
      // If we have OAuth success indicator OR we're authenticated, close popup
      if (hasOAuthSuccess || isAuthenticated) {
        // Notify the opener window
        try {
          window.opener.postMessage(
            { type: 'oauth-success', authenticated: true },
            window.location.origin
          );
          localStorage.setItem('oauth_success', JSON.stringify({ 
            authenticated: true, 
            timestamp: Date.now() 
          }));
        } catch (e) {
          console.error('Failed to notify opener:', e);
        }
        
        // Give a moment for the message to be sent, then close
        setTimeout(() => {
          window.close();
        }, 100);
      }
    }
  }, [isAuthenticated, location]);

  return null;
}