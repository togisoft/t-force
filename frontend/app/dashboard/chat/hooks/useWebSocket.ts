import { useEffect, useRef, useState, useCallback } from 'react';
import { useToast } from '@/hooks/use-toast';

// --- Interfaces ---
interface WsMessage {
  type: string;
  room_id?: string;
  content?: string;
  is_typing?: boolean;
  temp_id?: string; // For message reconciliation
  message_id?: string;
  emoji?: string;
  add?: boolean;
}

interface WsResponse {
  message_type: string;
  data: any;
  timestamp: number;
  message_id?: string;
}

interface UseWebSocketOptions {
  user?: any;
  onMessage?: (message: WsResponse) => void;
  onConnectionChange?: (connected: boolean, state: string) => void;
}

interface UseWebSocketReturn {
  isConnected: boolean;
  connectionState: string;
  sendMessage: (roomId: string, content: string, tempId: string) => boolean;
  sendTyping: (roomId: string, isTyping: boolean) => boolean;
  sendReaction: (messageId: string, emoji: string, add: boolean) => boolean;
  joinRoom: (roomId: string) => void;
  leaveRoom: (roomId: string) => void;
  reconnect: () => void;
}

export function useWebSocket({ user, onMessage, onConnectionChange }: UseWebSocketOptions): UseWebSocketReturn {
  const [connectionState, setConnectionState] = useState('disconnected');
  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const reconnectAttemptsRef = useRef(0);
  const activeRoomsRef = useRef<Set<string>>(new Set());
  const handlersRef = useRef({ onMessage, onConnectionChange });
  const messageQueueRef = useRef<Array<{ message: WsMessage; priority: boolean; timestamp: number }>>([]);
  const lastPingRef = useRef<number>(0);
  const pingIntervalRef = useRef<NodeJS.Timeout | null>(null);
  const { toast } = useToast();

  useEffect(() => {
    handlersRef.current = { onMessage, onConnectionChange };
  }, [onMessage, onConnectionChange]);

  const WS_URL = process.env.NEXT_PUBLIC_WS_URL || (typeof window !== 'undefined'
      ? `${window.location.protocol === 'https:' ? 'wss' : 'ws'}://${window.location.host}/api/chat/ws`
      : 'ws://localhost:8000/api/chat/ws');

  console.log('WebSocket URL:', WS_URL);
  console.log('Current location:', typeof window !== 'undefined' ? window.location.href : 'server-side');

  const updateConnectionState = (state: string) => {
    setConnectionState(state);
    const isConnected = state === 'connected';
    handlersRef.current.onConnectionChange?.(isConnected, state);
  };

  const flushMessageQueue = useCallback(() => {
    if (!wsRef.current || wsRef.current.readyState !== WebSocket.OPEN) return;

    const now = Date.now();
    const queue = messageQueueRef.current;
    messageQueueRef.current = [];

    // Sort by priority and timestamp
    queue.sort((a, b) => {
      if (a.priority !== b.priority) return b.priority ? 1 : -1;
      return a.timestamp - b.timestamp;
    });

    for (const { message } of queue) {
      try {
        wsRef.current.send(JSON.stringify(message));
      } catch (error) {
        console.error('Failed to send queued message:', error);
        // Re-queue failed messages
        messageQueueRef.current.push({ message, priority: true, timestamp: now });
      }
    }
  }, []);

  const sendWsMessage = useCallback((message: WsMessage, isPriority = false): boolean => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      try {
        wsRef.current.send(JSON.stringify(message));
        return true;
      } catch (error) {
        console.error('Failed to send message:', error);
        return false;
      }
    } else {
      // Queue message for later
      messageQueueRef.current.push({ 
        message, 
        priority: isPriority, 
        timestamp: Date.now() 
      });
      
      if (!isPriority) {
        console.warn('WebSocket not connected. Message queued:', message);
      }
      return false;
    }
  }, []);

  const startPingInterval = useCallback(() => {
    if (pingIntervalRef.current) {
      clearInterval(pingIntervalRef.current);
    }
    
    pingIntervalRef.current = setInterval(() => {
      if (wsRef.current?.readyState === WebSocket.OPEN) {
        lastPingRef.current = Date.now();
        sendWsMessage({ type: 'ping' }, true);
      }
    }, 25000); // Send ping every 25 seconds
  }, [sendWsMessage]);

  const stopPingInterval = useCallback(() => {
    if (pingIntervalRef.current) {
      clearInterval(pingIntervalRef.current);
      pingIntervalRef.current = null;
    }
  }, []);

  const connect = useCallback(() => {
    if (!user || (wsRef.current && wsRef.current.readyState === WebSocket.OPEN)) {
      return;
    }

    if (wsRef.current) {
      wsRef.current.onclose = null;
      wsRef.current.onerror = null;
      wsRef.current.close();
    }

    updateConnectionState('connecting');
    
    try {
      console.log('Attempting to connect to WebSocket:', WS_URL);
      wsRef.current = new WebSocket(WS_URL);
    } catch (error) {
      console.error('Failed to create WebSocket:', error);
      updateConnectionState('failed');
      return;
    }

    wsRef.current.onopen = () => {
      console.log('WebSocket connected successfully.');
      reconnectAttemptsRef.current = 0;
      updateConnectionState('connected');
      startPingInterval();

      // Flush any queued messages
      setTimeout(flushMessageQueue, 100);

      // Auto-rejoin rooms on successful connection
      if (activeRoomsRef.current.size > 0) {
        console.log('Re-joining active rooms:', [...activeRoomsRef.current]);
        activeRoomsRef.current.forEach(roomId => {
          sendWsMessage({ type: 'join', room_id: roomId }, true);
        });
      }
    };

    wsRef.current.onmessage = (event) => {
      try {
        const message = JSON.parse(event.data) as WsResponse;
        
        // Handle pong messages
        if (message.message_type === 'pong') {
          const latency = Date.now() - lastPingRef.current;
          console.debug(`WebSocket latency: ${latency}ms`);
          return;
        }

        handlersRef.current.onMessage?.(message);
      } catch (error) {
        console.error('Error parsing WebSocket message:', error);
      }
    };

    wsRef.current.onclose = (event) => {
      console.warn(`WebSocket disconnected: code ${event.code}, reason: ${event.reason}`);
      stopPingInterval();
      
      if (event.code === 1000) { // Normal closure
        updateConnectionState('disconnected');
        return;
      }
      
      // Handle authentication errors
      if (event.code === 1008) { // Policy violation (often auth issues)
        console.error('WebSocket authentication failed. Please log in again.');
        toast({
          title: "Authentication Error",
          description: "Please log in again to continue chatting.",
          variant: "destructive",
        });
        updateConnectionState('failed');
        return;
      }
      
      // Attempt to reconnect on abnormal closure
      if (reconnectAttemptsRef.current < 5) {
        const delay = Math.min(1000 * Math.pow(2, reconnectAttemptsRef.current), 30000);
        updateConnectionState('reconnecting');
        reconnectTimeoutRef.current = setTimeout(() => {
          reconnectAttemptsRef.current++;
          connect();
        }, delay);
      } else {
        updateConnectionState('failed');
        toast({
          title: "Connection Lost",
          description: "Unable to reconnect to the chat server. Please refresh the page.",
          variant: "destructive",
        });
      }
    };

    wsRef.current.onerror = (error) => {
      console.error('WebSocket error:', error);
      updateConnectionState('error');
    };
  }, [user, WS_URL, toast, startPingInterval, stopPingInterval, flushMessageQueue, sendWsMessage]);

  const disconnect = useCallback(() => {
    stopPingInterval();
    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current);
      reconnectTimeoutRef.current = null;
    }
    if (wsRef.current) {
      wsRef.current.onclose = null;
      wsRef.current.onerror = null;
      wsRef.current.close(1000, 'User disconnected');
      wsRef.current = null;
    }
    updateConnectionState('disconnected');
    messageQueueRef.current = [];
  }, [stopPingInterval]);

  const joinRoom = useCallback((roomId: string) => {
    console.log(`Attempting to join room: ${roomId}`);
    activeRoomsRef.current.add(roomId);
    sendWsMessage({ type: 'join', room_id: roomId }, true);
  }, [sendWsMessage]);

  const leaveRoom = useCallback((roomId: string) => {
    console.log(`Attempting to leave room: ${roomId}`);
    activeRoomsRef.current.delete(roomId);
    sendWsMessage({ type: 'leave', room_id: roomId }, true);
  }, [sendWsMessage]);

  const sendMessage = useCallback((roomId: string, content: string, tempId: string): boolean => {
    return sendWsMessage({ type: 'message', room_id: roomId, content, temp_id: tempId });
  }, [sendWsMessage]);

  const sendTyping = useCallback((roomId: string, isTyping: boolean): boolean => {
    return sendWsMessage({ type: 'typing', room_id: roomId, is_typing: isTyping });
  }, [sendWsMessage]);

  const sendReaction = useCallback((messageId: string, emoji: string, add: boolean): boolean => {
    return sendWsMessage({ type: 'reaction', message_id: messageId, emoji, add });
  }, [sendWsMessage]);

  const reconnect = useCallback(() => {
    disconnect();
    reconnectAttemptsRef.current = 0;
    setTimeout(connect, 500);
  }, [connect, disconnect]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      disconnect();
    };
  }, [disconnect]);

  // Connect when user becomes available
  useEffect(() => {
    if (user) {
      connect();
    } else {
      disconnect();
    }
  }, [user, connect, disconnect]);

  return {
    isConnected: connectionState === 'connected',
    connectionState,
    sendMessage,
    sendTyping,
    sendReaction,
    joinRoom,
    leaveRoom,
    reconnect,
  };
}