// lib/auth.tsx
'use client';

import { useRouter } from 'next/navigation';
import { useState, useEffect, createContext, useContext, ReactNode } from 'react';

// Types
export interface User {
  id: string;
  email: string;
  name: string;
  profile_image?: string | null;
  role: string;
  provider: string;
  is_active: boolean;
}

export interface Session {
  id: string;
  ip_address: string;
  user_agent: string;
  device_type: string;
  browser: string;
  os: string;
  last_active_at: string;
  created_at: string;
  is_active: boolean;
  is_current: boolean;
}

export interface AuthState {
  user: User | null;
  token: string | null;
  isLoading: boolean;
  isAuthenticated: boolean;
  requires2FA: boolean;
  temp2FAToken: string | null;
  pending2FAUser: User | null;
  sessions: Session[];
  isLoadingSessions: boolean;
  currentSessionId: string | null;
}

export interface LoginCredentials {
  email: string;
  password: string;
}

export interface RegisterCredentials {
  name: string;
  email: string;
  password: string;
  profile_image?: string | null;
}

export interface TwoFactorVerifyCredentials {
  code: string;
}

export interface AuthContextType extends AuthState {
  login: (credentials: LoginCredentials) => Promise<{ success: boolean; requires2FA?: boolean; error?: string }>;
  register: (credentials: RegisterCredentials) => Promise<{ success: boolean; error?: string }>;
  logout: () => Promise<void>;
  logoutWithMessage: (message: string) => Promise<void>;
  verify2FA: (credentials: TwoFactorVerifyCredentials) => Promise<{ success: boolean; error?: string }>;
  fetchSessions: () => Promise<void>;
  terminateSession: (sessionId: string) => Promise<{ success: boolean; error?: string }>;
  terminateAllSessions: () => Promise<{ success: boolean; error?: string }>;
  refreshUser: () => Promise<void>;
  handleOAuthToken: (token: string, requires2fa?: boolean) => Promise<{ success: boolean; requires2FA?: boolean; error?: string }>;
  handleApiError: (response: Response, silent?: boolean) => Promise<boolean>; // Returns true if handled (logout occurred)
}

// Create a context for authentication
const AuthContext = createContext<AuthContextType | null>(null);

// Helper function to store auth data
const storeAuthData = (user: User, token: string) => {
  if (typeof window !== 'undefined') {
    localStorage.setItem('auth_user', JSON.stringify(user));
    localStorage.setItem('auth_token', token);
  }
};

// Helper function to remove auth data
const removeAuthData = () => {
  if (typeof window !== 'undefined') {
    localStorage.removeItem('auth_user');
    localStorage.removeItem('auth_token');
  }
};

// Helper function to clear auth cookie
const clearAuthCookie = () => {
  if (typeof window !== 'undefined') {
    document.cookie = 'auth_token=; expires=Thu, 01 Jan 1970 00:00:00 UTC; path=/;';
  }
};

// Helper function to show notification
const showNotification = (message: string, type: 'error' | 'warning' | 'info' = 'error') => {
  if (typeof window !== 'undefined') {
    if (type === 'error') {
      alert(`Session Error: ${message}`);
    } else {
      console.warn(`Auth Notification: ${message}`);
    }
  }
};

// Auth Provider component
export const AuthProvider: React.FC<{ children: ReactNode }> = ({ children }) => {
  const [authState, setAuthState] = useState<AuthState>({
    user: null,
    token: null,
    isLoading: true,
    isAuthenticated: false,
    requires2FA: false,
    temp2FAToken: null,
    pending2FAUser: null,
    sessions: [],
    isLoadingSessions: false,
    currentSessionId: null,
  });

  const router = useRouter();

  // Central error handler for API responses
  const handleApiError = async (response: Response, silent: boolean = false): Promise<boolean> => {
    if (response.status === 401) {
      try {
        const errorData = await response.json();

        if (errorData.error === 'UserNotFound') {
          if (!silent) await logoutWithMessage('Your account no longer exists. Please log in again.');
          else await logout();
          return true;
        }

        if (errorData.error === 'Unauthorized') {
          if (!silent) await logoutWithMessage('Your session has expired. Please log in again.');
          else await logout();
          return true;
        }

        if (!silent) await logoutWithMessage('Authentication failed. Please log in again.');
        else await logout();
        return true;
      } catch (parseError) {
        if (!silent) await logoutWithMessage('Session expired. Please log in again.');
        else await logout();
        return true;
      }
    }

    return false;
  };

  // On component mount, verify the session and fetch the user
  useEffect(() => {
    const verifyAuthOnLoad = async () => {
      try {
        // Eğer zaten 2FA durumundaysak, /me çağrısı yapma
        if (authState.requires2FA && authState.temp2FAToken) {
          setAuthState(prev => ({ ...prev, isLoading: false }));
          return;
        }

        // Use /api/me to both validate the session and get fresh user data
        const response = await fetch(`/api/me`, {
          method: 'GET',
          credentials: 'include',
        });

        if (response.ok) {
          const userData: User = await response.json();

          if (userData.is_active) {
            setAuthState(prev => ({
              ...prev,
              user: userData,
              isAuthenticated: true,
              isLoading: false,
              requires2FA: false, // Başarılı /me çağrısı 2FA'nın tamamlandığını gösterir
              temp2FAToken: null,
              pending2FAUser: null,
            }));
          } else {
            await logoutWithMessage('Your account is deactivated. Please contact support.');
          }
        } else {
          // İlk yükleme sırasında silent=true kullanarak alert göstermeyi engelle
          // Ayrıca 2FA durumunu korumak için logout çağrısından önce kontrol et
          if (!authState.requires2FA) {
            await handleApiError(response, true);
          }
          setAuthState(prev => ({ ...prev, isLoading: false }));
        }
      } catch (error) {
        console.error('Failed to verify authentication:', error);
        // İlk yüklemede bağlantı hatası için de alert gösterme
        // Ve 2FA durumunu koru
        setAuthState(prev => ({ ...prev, isLoading: false }));
      }
    };

    verifyAuthOnLoad();
  }, []); // authState.requires2FA dependency'sini kaldırdık çünkü sonsuz döngüye neden oluyordu

  const login = async (credentials: LoginCredentials) => {
    try {
      const response = await fetch(`/api/auth/login`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify(credentials),
      });

      if (response.ok) {
        const data = await response.json();

        if (data.requires_2fa) {
          setAuthState(prev => ({
            ...prev,
            requires2FA: true,
            temp2FAToken: data.temp_token,
            pending2FAUser: data.user,
            isAuthenticated: false,
            isLoading: false // 2FA durumunda loading'i false yap
          }));
          return { success: true, requires2FA: true };
        }

        setAuthState(prev => ({
          ...prev,
          user: data.user,
          token: data.token || null,
          isAuthenticated: true,
          requires2FA: false,
          temp2FAToken: null,
          pending2FAUser: null,
          isLoading: false
        }));
        return { success: true };
      }

      const errorText = await response.text();
      return { success: false, error: errorText || 'Login failed' };
    } catch (error) {
      console.error('Login error:', error);
      return { success: false, error: 'Login failed' };
    }
  };

  const register = async (credentials: RegisterCredentials) => {
    try {
      const response = await fetch(`/api/auth/register`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify(credentials),
      });

      if (response.ok) {
        return { success: true };
      }

      const errorText = await response.text();
      return { success: false, error: errorText || 'Registration failed' };
    } catch (error) {
      console.error('Registration error:', error);
      return { success: false, error: 'Registration failed' };
    }
  };

  const logout = async () => {
    try {
      await fetch(`/api/auth/logout`, { method: 'GET', credentials: 'include' });
    } catch {}

    setAuthState(prev => ({
      ...prev,
      user: null,
      token: null,
      isAuthenticated: false,
      requires2FA: false,
      temp2FAToken: null,
      pending2FAUser: null
    }));
    removeAuthData();
    clearAuthCookie();
    router.push('/');
  };

  const logoutWithMessage = async (message: string) => {
    await logout();
    showNotification(message);
  };

  const verify2FA = async (credentials: TwoFactorVerifyCredentials) => {
    try {
      // Get the temp token from auth state
      const tempToken = authState.temp2FAToken;
      if (!tempToken) {
        return { success: false, error: 'No temporary token found. Please login again.' };
      }

      const response = await fetch(`/api/auth/verify-2fa`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify({
          temp_token: tempToken,
          code: credentials.code
        }),
      });

      if (response.ok) {
        const data = await response.json();
        setAuthState(prev => ({
          ...prev,
          user: data.user,
          isAuthenticated: true,
          requires2FA: false,
          temp2FAToken: null,
          pending2FAUser: null
        }));
        return { success: true };
      }

      const errorText = await response.text();
      return { success: false, error: errorText || '2FA verification failed' };
    } catch (error) {
      console.error('2FA verify error:', error);
      return { success: false, error: '2FA verification failed' };
    }
  };

  const fetchSessions = async () => {
    setAuthState(prev => ({ ...prev, isLoadingSessions: true }));
    try {
      const response = await fetch(`/api/auth/sessions`, { method: 'GET', credentials: 'include' });
      if (response.ok) {
        const data = await response.json();
        setAuthState(prev => ({ ...prev, sessions: data || [], isLoadingSessions: false }));
      } else {
        await handleApiError(response);
        setAuthState(prev => ({ ...prev, isLoadingSessions: false }));
      }
    } catch (error) {
      console.error('Fetch sessions error:', error);
      setAuthState(prev => ({ ...prev, isLoadingSessions: false }));
    }
  };

  const terminateSession = async (sessionId: string) => {
    try {
      const response = await fetch(`/api/auth/sessions/${sessionId}`, { method: 'DELETE', credentials: 'include' });
      if (response.ok) {
        await fetchSessions();
        return { success: true };
      }
      const errorText = await response.text();
      return { success: false, error: errorText || 'Failed to terminate session' };
    } catch (error) {
      console.error('Terminate session error:', error);
      return { success: false, error: 'Failed to terminate session' };
    }
  };

  const terminateAllSessions = async () => {
    try {
      const response = await fetch(`/api/auth/sessions`, { method: 'DELETE', credentials: 'include' });
      if (response.ok) {
        await fetchSessions();
        return { success: true };
      }
      const errorText = await response.text();
      return { success: false, error: errorText || 'Failed to terminate all sessions' };
    } catch (error) {
      console.error('Terminate all sessions error:', error);
      return { success: false, error: 'Failed to terminate all sessions' };
    }
  };

  const refreshUser = async () => {
    try {
      const userResponse = await fetch(`/api/me`, { method: 'GET', credentials: 'include' });
      if (userResponse.ok) {
        const userData: User = await userResponse.json();
        setAuthState(prev => ({ ...prev, user: userData }));
      }
    } catch (error) {
      console.error('Refresh user error:', error);
    }
  };

  const handleOAuthToken = async (token: string, requires2fa?: boolean) => {
    try {
      const response = await fetch(`/api/auth/sync`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify({
          token: token || undefined,
          requires2fa: requires2fa
        }),
      });

      if (response.ok) {
        const data = await response.json();

        if (data.requires_2fa) {
          // 2FA required - set temporary state
          setAuthState(prev => ({
            ...prev,
            requires2FA: true,
            temp2FAToken: data.temp_token,
            pending2FAUser: data.user,
            isAuthenticated: false
          }));
          return { success: true, requires2FA: true };
        } else {
          // Normal login success
          setAuthState(prev => ({
            ...prev,
            user: data.user,
            token: data.token || null,
            isAuthenticated: true,
            requires2FA: false,
            temp2FAToken: null,
            pending2FAUser: null
          }));
          return { success: true };
        }
      }

      const errorText = await response.text();
      return { success: false, error: errorText || 'Failed to sync OAuth token' };
    } catch (error) {
      console.error('Handle OAuth token error:', error);
      return { success: false, error: 'Failed to sync OAuth token' };
    }
  };

  const handleApiErrorWrapper = async (response: Response, silent?: boolean) => handleApiError(response, silent);

  return (
      <AuthContext.Provider value={{ ...authState, login, register, logout, logoutWithMessage, verify2FA, fetchSessions, terminateSession, terminateAllSessions, refreshUser, handleOAuthToken, handleApiError: handleApiErrorWrapper }}>
        {children}
      </AuthContext.Provider>
  );
};

export const useAuth = (): AuthContextType => {
  const context = useContext(AuthContext);
  if (!context) {
    throw new Error('useAuth must be used within an AuthProvider');
  }
  return context;
};

export const useProtectedRoute = (redirectTo: string = '/'): void => {
  const { isAuthenticated, isLoading } = useAuth();
  const router = useRouter();

  useEffect(() => {
    if (!isLoading && !isAuthenticated) {
      router.push(redirectTo);
    }
  }, [isAuthenticated, isLoading, redirectTo, router]);
};

export const useAdminRoute = (redirectTo: string = '/dashboard'): void => {
  const { user, isLoading } = useAuth();
  const router = useRouter();

  useEffect(() => {
    if (!isLoading && (!user || user.role !== 'admin')) {
      router.push(redirectTo);
    }
  }, [user, isLoading, redirectTo, router]);
};