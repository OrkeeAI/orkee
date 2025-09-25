import { useEffect } from 'react';
import { useNavigate, useLocation } from 'react-router-dom';
import { useCloudAuth } from '@/hooks/useCloud';

export default function OAuthCallback() {
  const navigate = useNavigate();
  const location = useLocation();
  const { refreshAuthStatus, isAuthenticated } = useCloudAuth();

  useEffect(() => {
    // Check if we're in a popup and authenticated (cloud dashboard loaded in popup)
    if (window.opener && isAuthenticated) {
      // We're authenticated and in a popup - notify opener and close
      window.opener.postMessage({ type: 'oauth-success', authenticated: true }, window.location.origin);
      localStorage.setItem('oauth_success', JSON.stringify({ authenticated: true, timestamp: Date.now() }));
      window.close();
      return;
    }

    // Handle OAuth callback with code/state
    const handleCallback = async () => {
      const urlParams = new URLSearchParams(window.location.search);
      const code = urlParams.get('code');
      const state = urlParams.get('state');
      const error = urlParams.get('error');

      if (error) {
        console.error('OAuth error:', error);
        // If we're in a popup, close it
        if (window.opener) {
          window.opener.postMessage({ type: 'oauth-error', error }, window.location.origin);
          window.close();
        } else {
          navigate('/settings');
        }
        return;
      }

      if (code && state) {
        try {
          // Store the auth info for the opener window to pick up
          localStorage.setItem('oauth_callback', JSON.stringify({ code, state, timestamp: Date.now() }));
          
          // If we're in a popup, close it
          if (window.opener) {
            // Send message to opener
            window.opener.postMessage({ type: 'oauth-callback', code, state }, window.location.origin);
            window.close();
          } else {
            // If not in popup, handle auth directly
            await refreshAuthStatus();
            navigate('/');
          }
        } catch (error) {
          console.error('Failed to handle OAuth callback:', error);
          if (window.opener) {
            window.close();
          } else {
            navigate('/settings');
          }
        }
      } else if (!isAuthenticated) {
        // No code or state and not authenticated
        console.error('OAuth callback missing code or state');
        if (window.opener) {
          window.close();
        } else {
          navigate('/settings');
        }
      }
    };

    handleCallback();
  }, [navigate, refreshAuthStatus, isAuthenticated, location]);

  return (
    <div className="flex items-center justify-center min-h-screen">
      <div className="text-center">
        <h1 className="text-2xl font-bold mb-4">Completing authentication...</h1>
        <div className="animate-spin rounded-full h-8 w-8 border-2 border-gray-300 border-t-gray-600 mx-auto"></div>
      </div>
    </div>
  );
}