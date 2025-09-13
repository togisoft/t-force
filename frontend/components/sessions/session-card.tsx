import { Session } from '@/lib/auth';
import { formatDistanceToNow } from 'date-fns';
import { Laptop, Smartphone, Tablet, Monitor, Globe, Clock, MapPin } from 'lucide-react';
import { Card, CardContent, CardFooter } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip';

interface SessionCardProps {
  session: Session;
  isCurrentSession: boolean;
  onTerminate: (sessionId: string) => void;
  isTerminating?: boolean;
}

export function SessionCard({ session, isCurrentSession, onTerminate, isTerminating = false }: SessionCardProps) {
  // Format dates
  const lastActiveAt = new Date(session.last_active_at);
  const createdAt = new Date(session.created_at);
  
  const lastActiveFormatted = formatDistanceToNow(lastActiveAt, { addSuffix: true });
  const createdAtFormatted = formatDistanceToNow(createdAt, { addSuffix: true });
  
  // Get device icon
  const DeviceIcon = () => {
    switch (session.device_type.toLowerCase()) {
      case 'mobile':
        return <Smartphone className="h-5 w-5" />;
      case 'tablet':
        return <Tablet className="h-5 w-5" />;
      case 'desktop':
      default:
        return <Laptop className="h-5 w-5" />;
    }
  };
  
  return (
    <Card className={`overflow-hidden ${isCurrentSession ? 'border-primary' : ''}`}>
      <CardContent className="p-6">
        <div className="flex items-start justify-between">
          <div className="flex items-center gap-3">
            <div className="rounded-full bg-muted p-2">
              <DeviceIcon />
            </div>
            <div>
              <h3 className="font-medium">
                {session.browser} on {session.os}
                {isCurrentSession && (
                  <Badge variant="outline" className="ml-2 bg-primary/10 text-primary">
                    Current
                  </Badge>
                )}
              </h3>
              <p className="text-sm text-muted-foreground">{session.device_type}</p>
            </div>
          </div>
        </div>
        
        <div className="mt-4 grid grid-cols-2 gap-3 text-sm">
          <div className="flex items-center gap-2">
            <MapPin className="h-4 w-4 text-muted-foreground" />
            <span>{session.ip_address}</span>
          </div>
          <div className="flex items-center gap-2">
            <Clock className="h-4 w-4 text-muted-foreground" />
            <TooltipProvider>
              <Tooltip>
                <TooltipTrigger asChild>
                  <span>{lastActiveFormatted}</span>
                </TooltipTrigger>
                <TooltipContent>
                  <p>Last active: {lastActiveAt.toLocaleString()}</p>
                </TooltipContent>
              </Tooltip>
            </TooltipProvider>
          </div>
          <div className="flex items-center gap-2">
            <Globe className="h-4 w-4 text-muted-foreground" />
            <span className="truncate">{session.user_agent.substring(0, 30)}...</span>
          </div>
          <div className="flex items-center gap-2">
            <Clock className="h-4 w-4 text-muted-foreground" />
            <TooltipProvider>
              <Tooltip>
                <TooltipTrigger asChild>
                  <span>Created {createdAtFormatted}</span>
                </TooltipTrigger>
                <TooltipContent>
                  <p>Created: {createdAt.toLocaleString()}</p>
                </TooltipContent>
              </Tooltip>
            </TooltipProvider>
          </div>
        </div>
      </CardContent>
      
      <CardFooter className="bg-muted/50 p-3">
        {isCurrentSession ? (
          <p className="text-xs text-muted-foreground">This is your current session</p>
        ) : (
          <Button 
            variant="destructive" 
            size="sm" 
            onClick={() => onTerminate(session.id)}
            disabled={isTerminating}
          >
            {isTerminating ? 'Terminating...' : 'Terminate Session'}
          </Button>
        )}
      </CardFooter>
    </Card>
  );
}