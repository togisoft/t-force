import { useState, useEffect } from 'react';
import { useAuth, Session } from '@/lib/auth';
import { SessionCard } from './session-card';
import { Button } from '@/components/ui/button';
import { Loader2, RefreshCw } from 'lucide-react';
import { toast } from '@/hooks/use-toast';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';

export function SessionList() {
  const { sessions, isLoadingSessions, currentSessionId, fetchSessions, terminateSession, terminateAllSessions } = useAuth();
  const [isTerminating, setIsTerminating] = useState<string | null>(null);
  const [isTerminatingAll, setIsTerminatingAll] = useState(false);
  const [isRefreshing, setIsRefreshing] = useState(false);

  // Fetch sessions on component mount
  useEffect(() => {
    fetchSessions();
  }, []);

  // Handle session termination
  const handleTerminateSession = async (sessionId: string) => {
    setIsTerminating(sessionId);
    try {
      const result = await terminateSession(sessionId);
      if (result.success) {
        toast({
          title: "Session Terminated",
          description: "The session has been successfully terminated.",
          variant: "default",
        });
      } else {
        toast({
          title: "Error",
          description: result.error || "Failed to terminate session.",
          variant: "destructive",
        });
      }
    } catch (error) {
      console.error('Error terminating session:', error);
      toast({
        title: "Error",
        description: "An unexpected error occurred while terminating the session.",
        variant: "destructive",
      });
    } finally {
      setIsTerminating(null);
    }
  };

  // Handle termination of all sessions
  const handleTerminateAllSessions = async () => {
    setIsTerminatingAll(true);
    try {
      const result = await terminateAllSessions();
      if (result.success) {
        toast({
          title: "All Sessions Terminated",
          description: "All other sessions have been successfully terminated.",
          variant: "default",
        });
      } else {
        toast({
          title: "Error",
          description: result.error || "Failed to terminate all sessions.",
          variant: "destructive",
        });
      }
    } catch (error) {
      console.error('Error terminating all sessions:', error);
      toast({
        title: "Error",
        description: "An unexpected error occurred while terminating all sessions.",
        variant: "destructive",
      });
    } finally {
      setIsTerminatingAll(false);
    }
  };

  // Handle refresh
  const handleRefresh = async () => {
    setIsRefreshing(true);
    try {
      await fetchSessions();
      toast({
        title: "Sessions Refreshed",
        description: "The session list has been refreshed.",
        variant: "default",
      });
    } catch (error) {
      console.error('Error refreshing sessions:', error);
      toast({
        title: "Error",
        description: "Failed to refresh sessions.",
        variant: "destructive",
      });
    } finally {
      setIsRefreshing(false);
    }
  };

  // Filter active sessions
  const activeSessions = sessions?.filter(session => session.is_active) || [];
  
  // Sort sessions: current session first, then by last active time
  const sortedSessions = [...activeSessions].sort((a, b) => {
    // Current session first
    if (a.id === currentSessionId) return -1;
    if (b.id === currentSessionId) return 1;
    
    // Then sort by last active time (most recent first)
    return new Date(b.last_active_at).getTime() - new Date(a.last_active_at).getTime();
  });

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h2 className="text-2xl font-bold tracking-tight">Active Sessions</h2>
        <div className="flex items-center gap-2">
          <Button 
            variant="outline" 
            size="sm" 
            onClick={handleRefresh}
            disabled={isRefreshing || isLoadingSessions}
          >
            {isRefreshing ? (
              <Loader2 className="h-4 w-4 animate-spin mr-2" />
            ) : (
              <RefreshCw className="h-4 w-4 mr-2" />
            )}
            Refresh
          </Button>
          {activeSessions.length > 1 && (
            <Button 
              variant="destructive" 
              size="sm" 
              onClick={handleTerminateAllSessions}
              disabled={isTerminatingAll || isLoadingSessions}
            >
              {isTerminatingAll ? 'Terminating...' : 'Terminate All Other Sessions'}
            </Button>
          )}
        </div>
      </div>
      
      {isLoadingSessions ? (
        <div className="flex justify-center py-8">
          <Loader2 className="h-8 w-8 animate-spin text-primary" />
        </div>
      ) : activeSessions.length === 0 ? (
        <Alert>
          <AlertTitle>No active sessions found</AlertTitle>
          <AlertDescription>
            There are no active sessions for your account.
          </AlertDescription>
        </Alert>
      ) : (
        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
          {sortedSessions.map((session) => (
            <SessionCard
              key={session.id}
              session={session}
              isCurrentSession={session.id === currentSessionId}
              onTerminate={handleTerminateSession}
              isTerminating={isTerminating === session.id}
            />
          ))}
        </div>
      )}
    </div>
  );
}