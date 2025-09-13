'use client';

import React, { useRef, useEffect, useCallback, useState, useMemo } from 'react';
import { useAuth } from '@/lib/auth';
import { Card, CardContent } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { useSearchParams } from 'next/navigation';
import { Loader2, MessageSquare, Wifi, WifiOff, RefreshCw, Menu, X } from 'lucide-react';
import { useToast } from '@/hooks/use-toast';

// Import custom hooks
import { useWebSocket } from './hooks/useWebSocket';
import { useRooms } from './hooks/useRooms';
import { useMessages } from './hooks/useMessages';
import { useReactions } from './hooks/useReactions';

// Import components
import { RoomList } from './components/RoomList';
import { MessageList } from './components/MessageList';
import { MessageInput } from './components/MessageInput';

export default function ChatPage() {
  const { user, isLoading, isAuthenticated } = useAuth();
  const { formatReactionUsers, hasUserReacted } = useReactions();
  const { toast } = useToast();
  const searchParams = useSearchParams();

  // State for pending room selection
  const [pendingRoomSelection, setPendingRoomSelection] = useState<any>(null);
  // State to track image upload progress for better UX feedback.
  const [isUploading, setIsUploading] = useState(false);
  const [roomUsers, setRoomUsers] = useState<{ [roomId: string]: number }>({});
  const [connectionRetryCount, setConnectionRetryCount] = useState(0);
  const [isInitializing, setIsInitializing] = useState(true);
  const [isMobileSidebarOpen, setIsMobileSidebarOpen] = useState(false);

  // Create refs to break circular dependencies
  const updateRoomUserCountRef = useRef<(roomId: string, count: number) => void>(() => {});
  const handleMessageEventRef = useRef<(response: any) => void>(() => {});

  // Handle WebSocket messages
  const handleWebSocketMessage = useCallback((response: any) => {
    handleMessageEventRef.current(response);
    if (response.message_type === 'user_count' && response.data && response.data.room_id) {
      updateRoomUserCountRef.current(response.data.room_id, response.data.count);
    }
  }, []);

  const handleConnectionChange = useCallback((connected: boolean, state: string) => {
    console.log('WebSocket connection status:', connected ? 'connected' : 'disconnected', 'State:', state);
    
    if (connected) {
      setConnectionRetryCount(0);
      if (pendingRoomSelection) {
        console.log('WebSocket connected, processing pending room selection:', pendingRoomSelection.id);
        joinRoom(pendingRoomSelection.id);
        setPendingRoomSelection(null);
      }
    } else if (state === 'failed' || state === 'error') {
      setConnectionRetryCount(prev => prev + 1);
    }
  }, [pendingRoomSelection]); // Note: joinRoom is stable, no need to include

  // Initialize WebSocket
  const {
    isConnected,
    connectionState,
    sendMessage: sendChatMessage,
    sendTyping: sendTypingIndicator,
    sendReaction,
    joinRoom,
    leaveRoom,
    reconnect
  } = useWebSocket({
    user,
    onMessage: handleWebSocketMessage,
    onConnectionChange: handleConnectionChange
  });

  // Initialize rooms management
  const {
    rooms,
    selectedRoom,
    isLoadingRooms,
    fetchRooms,
    createRoom,
    joinProtectedRoom,
    joinRoomByCode,
    deleteRoom,
    updateRoomUserCount,
    selectRoom,
    leaveRoomMembership,
  } = useRooms({
    userId: user?.id || null
  });

  useEffect(() => {
    updateRoomUserCountRef.current = updateRoomUserCount;
  }, [updateRoomUserCount]);

  // Initialize messages management
  const {
    messages,
    isLoadingMessages,
    typingUsers,
    sendMessage,
    sendVoiceMessage,
    retryMessage,
    handleMessageEvent,
    fetchMessages,
    lastSoundMessageIdRef,
    applyReactionLocal,
  } = useMessages({
    user,
    selectedRoom,
    sendChatMessage
  });

  useEffect(() => {
    handleMessageEventRef.current = handleMessageEvent;
  }, [handleMessageEvent]);

  // Handle room selection
  const handleRoomSelect = useCallback(async (room: any) => {
    if (!isConnected) {
      console.log('WebSocket not connected, storing room selection for later');
      setPendingRoomSelection(room);
      return;
    }

    if (selectedRoom?.id === room.id) {
      return; // Already selected
    }

    // Leave current room if any
    if (selectedRoom) {
      leaveRoom(selectedRoom.id);
    }

    // Select new room
    selectRoom(room);
    
    // Join the room via WebSocket
    joinRoom(room.id);
    
    // Close mobile sidebar on room selection
    setIsMobileSidebarOpen(false);
  }, [isConnected, selectedRoom, leaveRoom, selectRoom, joinRoom]);

  // Handle image upload
  const handleImageUpload = useCallback(async (file: File) => {
    if (!selectedRoom) {
      toast({ title: "Error", description: "Please select a room first.", variant: "destructive" });
      return null;
    }

    try {
      const formData = new FormData();
      formData.append('file', file);

      const response = await fetch('/api/chat/upload', {
        method: 'POST',
        body: formData,
        credentials: 'include',
      });

      if (!response.ok) {
        throw new Error('Upload failed');
      }

      const data = await response.json();
      return data.image_url;
    } catch (error) {
      console.error('Image upload failed:', error);
      toast({ title: "Upload Failed", description: "Failed to upload image. Please try again.", variant: "destructive" });
      return null;
    }
  }, [selectedRoom, toast]);

  // Handle video upload
  const handleVideoUpload = useCallback(async (file: File) => {
    if (!selectedRoom) {
      toast({ title: 'Error', description: 'Please select a room first.', variant: 'destructive' });
      return null;
    }

    try {
      const formData = new FormData();
      formData.append('file', file);

      const response = await fetch('/api/chat/upload-video', {
        method: 'POST',
        body: formData,
        credentials: 'include',
      });

      if (!response.ok) {
        throw new Error('Upload failed');
      }

      const data = await response.json();
      return data.video_url;
    } catch (error) {
      console.error('Video upload failed:', error);
      toast({ title: 'Upload Failed', description: 'Failed to upload video. Please try again.', variant: 'destructive' });
      return null;
    }
  }, [selectedRoom, toast]);

  // Get active typing users for the current room
  const activeTypingUsers = useMemo(() => {
    if (!selectedRoom) return [];
    
    return Object.entries(typingUsers)
      .filter(([userId, userData]) => {
        // Only show typing indicators from other users (not current user)
        return userId !== user?.id && 
               Date.now() - userData.timestamp < 5000; // Only show recent typing (within 5 seconds)
      })
      .map(([userId, userData]) => userData);
  }, [typingUsers, selectedRoom, user?.id]);

  // Get connection status display
  const getConnectionStatusDisplay = useCallback(() => {
    const isConnected = connectionState === 'connected';
    const isConnecting = connectionState === 'connecting' || connectionState === 'reconnecting';
    const isFailed = connectionState === 'failed' || connectionState === 'error';

    return {
      icon: isConnected ? <Wifi className="h-4 w-4 text-green-500" /> : <WifiOff className="h-4 w-4 text-red-500" />,
      text: isConnected ? 'Connected' : isConnecting ? 'Connecting...' : 'Disconnected',
      color: isConnected ? 'text-green-600' : isConnecting ? 'text-yellow-600' : 'text-red-600',
      bgColor: isConnected ? 'bg-green-50' : isConnecting ? 'bg-yellow-50' : 'bg-red-50',
      borderColor: isConnected ? 'border-green-200' : isConnecting ? 'border-yellow-200' : 'border-red-200',
      showRetry: isFailed && connectionRetryCount > 0,
      retryCount: connectionRetryCount
    };
  }, [connectionState, connectionRetryCount]);

  // Initialize chat when user is authenticated
  useEffect(() => {
    if (isAuthenticated && user && !isInitializing) {
      fetchRooms();
    }
  }, [isAuthenticated, user, fetchRooms, isInitializing]);

  // Set initialization complete when user loads
  useEffect(() => {
    if (!isLoading) {
      setIsInitializing(false);
    }
  }, [isLoading]);

  // Handle room code from URL
  useEffect(() => {
    const roomCode = searchParams.get('room');
    if (roomCode && rooms.length > 0) {
      const room = rooms.find(r => r.room_code === roomCode);
      if (room) {
        handleRoomSelect(room);
      }
    }
  }, [searchParams, rooms, handleRoomSelect]);

  // Show loading state
  if (isLoading || isInitializing) {
    return (
      <div className="flex items-center justify-center h-full bg-gradient-to-br from-gray-50 via-blue-50/20 to-purple-50/20 dark:from-gray-900 dark:via-blue-950/10 dark:to-purple-950/10">
        <div className="text-center">
          <div className="w-16 h-16 mx-auto mb-6 bg-gradient-to-br from-blue-100 to-purple-100 dark:from-blue-900/30 dark:to-purple-900/30 rounded-full flex items-center justify-center shadow-xl">
            <Loader2 className="h-8 w-8 animate-spin text-blue-600 dark:text-blue-400" />
          </div>
          <p className="text-gray-600 dark:text-gray-400 text-lg font-medium">Loading chat...</p>
        </div>
      </div>
    );
  }

  // Show authentication required
  if (!isAuthenticated) {
    return (
      <div className="flex items-center justify-center h-full bg-gradient-to-br from-gray-50 via-blue-50/20 to-purple-50/20 dark:from-gray-900 dark:via-blue-950/10 dark:to-purple-950/10">
        <div className="text-center">
          <div className="w-20 h-20 mx-auto mb-6 bg-gradient-to-br from-blue-100 to-purple-100 dark:from-blue-900/30 dark:to-purple-900/30 rounded-full flex items-center justify-center shadow-xl">
            <MessageSquare className="h-10 w-10 text-blue-600 dark:text-blue-400" />
          </div>
          <h2 className="text-2xl font-bold mb-3 text-gray-900 dark:text-gray-100">Authentication Required</h2>
          <p className="text-gray-600 dark:text-gray-400 text-lg">Please log in to access the chat.</p>
        </div>
      </div>
    );
  }

  const connectionStatus = getConnectionStatusDisplay();

  return (
    <div className="flex h-full bg-gradient-to-br from-background via-accent/10 to-muted/10 overflow-hidden">
      {/* Mobile Overlay */}
      {isMobileSidebarOpen && (
        <div 
          className="fixed inset-0 bg-black/50 backdrop-blur-sm z-40 md:hidden"
          onClick={() => setIsMobileSidebarOpen(false)}
        />
      )}

      {/* Sidebar */}
      <div className={`
        w-80 bg-white/95 dark:bg-gray-800/95 backdrop-blur-sm border-r border-gray-200/50 dark:border-gray-700/50 flex flex-col h-full shadow-xl
        md:relative md:translate-x-0 transition-transform duration-300 ease-in-out z-50
        ${isMobileSidebarOpen ? 'fixed translate-x-0' : 'fixed -translate-x-full md:translate-x-0'}
      `}>
        {/* Header */}
        <div className="p-4 border-b border-gray-200/50 dark:border-gray-700/50 bg-gradient-to-r from-blue-500 via-blue-600 to-purple-600 dark:from-blue-600 dark:via-blue-700 dark:to-purple-700 flex-shrink-0 shadow-lg">
          <div className="flex items-center justify-between mb-4">
            <h1 className="text-xl font-bold text-white drop-shadow-sm">Chat Rooms</h1>
            <div className="flex items-center space-x-2">
              {/* Mobile Close Button */}
              <button
                onClick={() => setIsMobileSidebarOpen(false)}
                className="md:hidden p-1.5 hover:bg-white/20 rounded-lg transition-colors duration-200"
              >
                <X className="h-5 w-5 text-white" />
              </button>
              {connectionStatus.icon}
              <span className={`text-sm font-medium text-white drop-shadow-sm hidden sm:inline`}>
                {connectionStatus.text}
              </span>
            </div>
          </div>
          
          {/* Connection Status Badge */}
          <div className={`flex items-center justify-between p-3 rounded-xl border shadow-lg ${connectionStatus.bgColor} ${connectionStatus.borderColor} backdrop-blur-sm`}>
            <div className="flex items-center space-x-2">
              {connectionStatus.icon}
              <span className={`text-sm font-medium ${connectionStatus.color}`}>
                {connectionStatus.text}
              </span>
              {connectionStatus.showRetry && (
                <Badge variant="secondary" className="text-xs bg-white/20 text-white border-white/30">
                  {connectionStatus.retryCount} retries
                </Badge>
              )}
            </div>
            {connectionStatus.showRetry && (
              <button
                onClick={reconnect}
                className="flex items-center space-x-1 text-xs text-blue-100 hover:text-white transition-colors duration-200"
              >
                <RefreshCw className="h-3 w-3" />
                <span>Retry</span>
              </button>
            )}
          </div>
        </div>

        {/* Room List */}
        <div className="flex-1 overflow-hidden">
          <RoomList
            rooms={rooms}
            selectedRoom={selectedRoom}
            userId={user?.id || ''}
            user={user}
            isLoadingRooms={isLoadingRooms}
            onSelectRoom={handleRoomSelect}
            onCreateRoom={createRoom}
            onJoinProtectedRoom={joinProtectedRoom}
            onJoinRoomByCode={joinRoomByCode}
            onDeleteRoom={deleteRoom}
            onLeaveRoom={leaveRoomMembership}
          />
        </div>
      </div>

      {/* Main Chat Area */}
      <div className="flex-1 flex flex-col h-full">
        {selectedRoom ? (
          <>
            {/* Chat Header */}
            <div className="bg-gradient-to-r from-primary via-primary to-chart-2 border-b border-border/50 p-4 flex-shrink-0 shadow-lg h-20">
              <div className="flex items-center justify-between h-full">
                {/* Mobile Menu Button */}
                <button
                  onClick={() => setIsMobileSidebarOpen(true)}
                  className="md:hidden p-2 hover:bg-white/20 rounded-lg transition-colors duration-200 mr-3"
                >
                  <Menu className="h-5 w-5 text-primary-foreground" />
                </button>
                
                <div className="flex-1 flex flex-col justify-center">
                  <h2 className="text-lg font-bold text-primary-foreground drop-shadow-sm">{selectedRoom.name}</h2>
                  <div className="flex items-center space-x-2 h-6 mt-1">
                    {activeTypingUsers.length > 0 ? (
                                              <div className="flex items-center space-x-2 bg-primary-foreground/20 backdrop-blur-sm rounded-full px-3 py-1.5 shadow-sm">
                        <div className="flex space-x-1">
                          <div className="w-1.5 h-1.5 bg-primary-foreground rounded-full animate-bounce" style={{ animationDelay: "0ms" }} />
                          <div className="w-1.5 h-1.5 bg-primary-foreground rounded-full animate-bounce" style={{ animationDelay: "150ms" }} />
                          <div className="w-1.5 h-1.5 bg-primary-foreground rounded-full animate-bounce" style={{ animationDelay: "300ms" }} />
                        </div>
                        <span className="text-xs text-primary-foreground font-semibold drop-shadow-sm">
                          {activeTypingUsers.length === 1 
                            ? `${activeTypingUsers[0].name} is typing...`
                            : `${activeTypingUsers.length} people are typing...`
                          }
                        </span>
                      </div>
                    ) : (
                      <div className="h-6"></div>
                    )}
                  </div>
                </div>
                <div className="flex items-center">
                  {selectedRoom.is_protected && (
                    <Badge variant="secondary" className="bg-white/20 text-white border-white/30 flex-shrink-0 backdrop-blur-sm shadow-sm">Protected</Badge>
                  )}
                </div>
              </div>
            </div>

            {/* Messages */}
            <div className="flex-1 overflow-hidden">
              <MessageList
                messages={messages}
                isLoadingMessages={isLoadingMessages}
                typingUsers={typingUsers}
                userId={user?.id || ''}
                onRetryMessage={retryMessage}
                onAddReaction={(messageId: string, emoji: string) => { applyReactionLocal(messageId, emoji, true); sendReaction(messageId, emoji, true); }}
                onRemoveReaction={(messageId: string, emoji: string) => { applyReactionLocal(messageId, emoji, false); sendReaction(messageId, emoji, false); }}
                formatReactionUsers={formatReactionUsers}
                hasUserReacted={hasUserReacted}
                websocketReady={isConnected}
                lastSoundMessageIdRef={lastSoundMessageIdRef}
              />
            </div>

            {/* Message Input */}
            <div className="flex-shrink-0">
              <MessageInput
                selectedRoom={selectedRoom}
                isConnected={isConnected}
                onSendMessage={(content: string) => {
                  sendMessage(content);
                  return true; // Return true to indicate success
                }}
                onTypingIndicator={(isTyping: boolean) => {
                  if (selectedRoom) {
                    sendTypingIndicator(selectedRoom.id, isTyping);
                  }
                }}
                onUploadImage={handleImageUpload}
                onUploadVideo={handleVideoUpload}
                onSendVoiceMessage={sendVoiceMessage}

              />
            </div>
          </>
        ) : (
          /* No Room Selected */
          <div className="flex-1 flex flex-col">
            {/* Mobile Header for No Room */}
            <div className="md:hidden bg-gradient-to-r from-primary via-primary to-chart-2 border-b border-border/50 p-4 flex-shrink-0 shadow-lg">
              <div className="flex items-center">
                <button
                  onClick={() => setIsMobileSidebarOpen(true)}
                  className="p-2 hover:bg-white/20 rounded-lg transition-colors duration-200 mr-3"
                >
                  <Menu className="h-5 w-5 text-primary-foreground" />
                </button>
                <h2 className="text-lg font-bold text-primary-foreground drop-shadow-sm">Chat</h2>
              </div>
            </div>
            
            {/* No Room Content */}
            <div className="flex-1 flex items-center justify-center bg-gradient-to-br from-background via-accent/10 to-muted/10">
              <div className="text-center px-4">
                                  <div className="w-20 h-20 mx-auto mb-6 bg-gradient-to-br from-accent to-muted rounded-full flex items-center justify-center shadow-xl">
                    <MessageSquare className="h-10 w-10 text-primary" />
                  </div>
                  <h2 className="text-2xl font-bold mb-3 text-foreground">Select a Room</h2>
                  <p className="text-muted-foreground text-lg mb-4">Choose a room from the sidebar to start chatting.</p>
                
                {/* Mobile-specific content */}
                <div className="md:hidden">
                  <button
                    onClick={() => setIsMobileSidebarOpen(true)}
                    className="bg-gradient-to-r from-primary to-chart-2 text-primary-foreground px-6 py-3 rounded-lg font-medium shadow-lg hover:shadow-xl transition-all duration-200 hover:scale-105 button-glow"
                  >
                    <Menu className="h-5 w-5 inline mr-2" />
                    Open Rooms
                  </button>
                </div>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}