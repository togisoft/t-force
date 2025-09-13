import { useState, useCallback, useEffect, useRef } from 'react';
import { useToast } from '@/hooks/use-toast';
import { Message } from '../types';

interface UseMessagesOptions {
  user?: any;
  selectedRoom: any;
  sendChatMessage: (roomId: string, content: string, tempId: string) => boolean;
}

interface UseMessagesReturn {
  messages: Message[];
  isLoadingMessages: boolean;
  typingUsers: Record<string, { name: string; timestamp: number }>;
  sendMessage: (content: string) => void;
  sendVoiceMessage: (audioBlob: Blob) => Promise<boolean>;
  retryMessage: (message: Message) => void;
  handleMessageEvent: (event: any) => void;
  fetchMessages: () => Promise<void>;
  lastSoundMessageIdRef: React.MutableRefObject<string>;
  applyReactionLocal: (messageId: string, emoji: string, add: boolean) => void;
}

export function useMessages({ user, selectedRoom, sendChatMessage }: UseMessagesOptions): UseMessagesReturn {
  const [messages, setMessages] = useState<Message[]>([]);
  const [isLoadingMessages, setIsLoadingMessages] = useState(false);
  const [typingUsers, setTypingUsers] = useState<Record<string, { name: string; timestamp: number }>>({});
  const pendingMessagesRef = useRef<Map<string, { message: Message; retryCount: number; timestamp: number }>>(new Map());
  const messageIdsRef = useRef<Set<string>>(new Set());
  const lastSoundMessageIdRef = useRef<string>('');
  const sentVoiceMessagesRef = useRef<Set<string>>(new Set()); // Track sent voice messages
  const { toast } = useToast();

  const fetchMessages = useCallback(async () => {
    if (!user || !selectedRoom) return;
    setIsLoadingMessages(true);
    try {
      const response = await fetch(`/api/chat/rooms/${selectedRoom.id}/messages`, {
        credentials: 'include',
      });
      if (!response.ok) throw new Error('Failed to fetch messages');
      const data = await response.json();
      const loadedMessages = (Array.isArray(data) ? data : data.messages || []).map((msg: any) => ({
        ...msg,
        status: 'sent' as const,
      }));
      
      const sortedMessages = loadedMessages.sort((a: Message, b: Message) =>
          new Date(a.created_at).getTime() - new Date(b.created_at).getTime()
      );
      
      setMessages(sortedMessages);
      
      // Track message IDs to prevent duplicates and set last message as processed for sound
      sortedMessages.forEach((msg: Message) => messageIdsRef.current.add(msg.id));
      
      // Mark the latest message as already processed to prevent notification on room entry
      if (sortedMessages.length > 0) {
        const latestMessage = sortedMessages[sortedMessages.length - 1];
        console.log('Setting lastSoundMessageId to prevent notification on room entry:', latestMessage.id);
        lastSoundMessageIdRef.current = latestMessage.id;
      }
      
    } catch (error) {
      console.error('Failed to fetch messages:', error);
      toast({ title: "Error", description: "Failed to load messages.", variant: "destructive" });
    } finally {
      setIsLoadingMessages(false);
    }
  }, [user, selectedRoom, toast]);

  const retryPendingMessage = useCallback((tempId: string) => {
    const pending = pendingMessagesRef.current.get(tempId);
    if (!pending) return;

    const { message, retryCount } = pending;
    if (retryCount >= 3) {
      // Max retries reached, mark as failed
      setMessages(prev => prev.map(msg =>
          msg.id === tempId ? { ...msg, status: 'failed' } : msg
      ));
      pendingMessagesRef.current.delete(tempId);
      toast({ 
        title: "Message Failed", 
        description: "Message could not be sent after multiple attempts.", 
        variant: "destructive" 
      });
      return;
    }

    // Retry sending
    const success = sendChatMessage(selectedRoom.id, message.content, tempId);
    if (success) {
      pendingMessagesRef.current.set(tempId, { 
        message, 
        retryCount: retryCount + 1, 
        timestamp: Date.now() 
      });
    }
  }, [selectedRoom, sendChatMessage, toast]);

  const sendVoiceMessage = useCallback(async (audioBlob: Blob): Promise<boolean> => {
    if (!user || !selectedRoom) return false;

    const tempId = `temp_${Date.now()}_${Math.random()}`;
    
    try {
      const formData = new FormData();
      formData.append('room_id', selectedRoom.id);
      formData.append('audio', audioBlob, 'voice_message.webm');

      const response = await fetch(`/api/chat/voice`, {
        method: 'POST',
        credentials: 'include',
        body: formData,
      });

      if (!response.ok) {
        throw new Error('Failed to upload voice message');
      }

      const data = await response.json();
      
      // Add temporary message to UI
      const tempMessage: Message = {
        id: tempId,
        content: '[audio](uploading...)',
        user_id: user.id,
        user_name: user.name,
        user_profile_image: user.profile_image,
        room_id: selectedRoom.id,
        created_at: new Date().toISOString(),
        status: 'sending',
      };

      setMessages(prev => [...prev, tempMessage]);

      // Update with real message data
      const realMessage: Message = {
        id: data.message_id,
        content: data.audio_url ? `[audio](${data.audio_url})` : '[audio](uploaded)',
        user_id: user.id,
        user_name: user.name,
        user_profile_image: user.profile_image,
        room_id: selectedRoom.id,
        created_at: new Date().toISOString(),
        status: 'sent',
      };

      setMessages(prev => prev.map(msg => msg.id === tempId ? realMessage : msg));
      messageIdsRef.current.add(data.message_id);
      
      // Track this voice message to prevent duplicate WebSocket messages
      sentVoiceMessagesRef.current.add(data.message_id);
      
      // Note: Backend already handles WebSocket broadcasting for voice messages
      // No need to send via WebSocket from frontend
      
      return true;
    } catch (error) {
      console.error('Failed to send voice message:', error);
      
      // Mark as failed
      setMessages(prev => prev.map(msg => 
        msg.id === tempId ? { ...msg, status: 'failed' } : msg
      ));
      
      toast({ 
        title: "Voice Message Failed", 
        description: "Failed to send voice message. Please try again.", 
        variant: "destructive" 
      });
      
      return false;
    }
  }, [user, selectedRoom,  toast]);

  const sendMessage = useCallback((content: string) => {
    if (!selectedRoom || !user || !content.trim()) return;

    const trimmedContent = content.trim();
    const tempId = `temp-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
    
    // Prevent duplicate messages by checking if the same content was sent recently
    const recentMessages = Array.from(pendingMessagesRef.current.values())
      .filter(pending => Date.now() - pending.timestamp < 1000); // Within 1 second
    
    const isDuplicate = recentMessages.some(pending => 
      pending.message.content === trimmedContent && 
      pending.message.user_id === user.id
    );
    
    if (isDuplicate) {
      console.log('Duplicate message prevented:', trimmedContent);
      return;
    }

    const tempMessage: Message = {
      id: tempId,
      content: trimmedContent,
      user_id: user.id,
      user_name: user.name || 'You',
      user_profile_image: user.profile_image || null,
      room_id: selectedRoom.id,
      created_at: new Date().toISOString(),
      status: 'sending',
    };

    // Add to messages immediately
    console.log('Adding temp message:', tempId, trimmedContent);
    setMessages(prev => [...prev, tempMessage]);

    // Track as pending
    pendingMessagesRef.current.set(tempId, {
      message: tempMessage,
      retryCount: 0,
      timestamp: Date.now(),
    });

    // Send via WebSocket
    const success = sendChatMessage(selectedRoom.id, trimmedContent, tempId);
    if (!success) {
      // If sending failed, mark as failed immediately
      setMessages(prev => prev.map(msg =>
          msg.id === tempId ? { ...msg, status: 'failed' } : msg
      ));
      pendingMessagesRef.current.delete(tempId);
      toast({ 
        title: "Send Failed", 
        description: "Failed to send message. Please try again.", 
        variant: "destructive" 
      });
    }
  }, [selectedRoom, user, sendChatMessage, toast]);

  const retryMessage = useCallback((message: Message) => {
    if (message.status === 'failed') {
      // Remove the failed message
      setMessages(prev => prev.filter(msg => msg.id !== message.id));
      // Send again
      sendMessage(message.content);
    }
  }, [sendMessage]);

  // Helper to immutably apply a reaction update
  const applyReactionUpdate = useCallback((messageId: string, emoji: string, userId: string, userName: string, add: boolean) => {
    setMessages(prev => prev.map(msg => {
      if (msg.id !== messageId) return msg;

      const reactions = Array.isArray(msg.reactions)
        ? msg.reactions.map(r => ({ ...r, users: [...r.users] }))
        : [] as NonNullable<Message['reactions']>;

      const idx = reactions.findIndex(r => r.emoji === emoji);

      if (add) {
        if (idx >= 0) {
          const r = reactions[idx];
          const already = r.users.some(u => u.user_id === userId);
          if (!already) {
            reactions[idx] = {
              ...r,
              users: [...r.users, { user_id: userId, user_name: userName }],
              count: (r.count || 0) + 1,
            };
          }
        } else {
          reactions.push({
            id: `${messageId}-${emoji}`,
            emoji,
            users: [{ user_id: userId, user_name: userName }],
            count: 1,
          });
        }
      } else {
        if (idx >= 0) {
          const r = reactions[idx];
          const filteredUsers = r.users.filter(u => u.user_id !== userId);
          const decreased = r.users.length !== filteredUsers.length;
          const newCount = Math.max(0, (r.count || 0) - (decreased ? 1 : 0));

          if (filteredUsers.length === 0 || newCount === 0) {
            reactions.splice(idx, 1);
          } else {
            reactions[idx] = { ...r, users: filteredUsers, count: newCount };
          }
        }
      }

      return { ...msg, reactions };
    }));
  }, []);

  // Expose a local optimistic updater for reactions
  const applyReactionLocal = useCallback((messageId: string, emoji: string, add: boolean) => {
    if (!user) return;
    applyReactionUpdate(messageId, emoji, user.id, user.name, add);
  }, [user, applyReactionUpdate]);

  const handleMessageEvent = useCallback((response: any) => {
    console.log('Handling message event:', response);

    switch (response.message_type) {
      case 'message_ack':
        // Handle message acknowledgment
        const { temp_id, message_id, success } = response.data;
        console.log('Message ACK received:', { temp_id, message_id, success });
        if (success && temp_id) {
          setMessages(prev => {
            const updatedMessages = prev.map(msg => {
              if (msg.id === temp_id) {
                // Replace temp message with confirmed message
                const confirmedMessage = {
                  ...msg,
                  id: message_id,
                  status: 'sent' as const,
                };
                // Add the confirmed message ID to prevent duplicates
                if (message_id) {
                  messageIdsRef.current.add(message_id);
                }
                console.log('ACK: Updated temp message to confirmed:', temp_id, '->', message_id);
                return confirmedMessage;
              }
              return msg;
            });
            // Sort messages by created_at timestamp
            return updatedMessages.sort((a, b) => 
              new Date(a.created_at).getTime() - new Date(b.created_at).getTime()
            );
          });
          pendingMessagesRef.current.delete(temp_id);
        } else {
          // Mark as failed
          setMessages(prev => prev.map(msg =>
              msg.id === temp_id ? { ...msg, status: 'failed' } : msg
          ));
          pendingMessagesRef.current.delete(temp_id);
        }
        break;

      case 'message':
        // Handle incoming message
        const messageData = response.data;
        const messageId = messageData.id || response.message_id;
        
        // Prevent duplicate messages by checking ID
        if (messageId && messageIdsRef.current.has(messageId)) {
          console.log('Duplicate message ignored (ID):', messageId, 'Content:', messageData.content);
          return;
        }

        // Check if this is a voice message we already sent via API
        if (messageId && sentVoiceMessagesRef.current.has(messageId)) {
          console.log('Voice message already sent via API, ignoring WebSocket duplicate:', messageId);
          return;
        }

        // Check if this is our own message that we already have as temp message
        const isOwnMessage = messageData.user_id === user?.id;
        if (isOwnMessage) {
          // Check if we have a pending temp message with same content
          const existingTempMessage = Array.from(pendingMessagesRef.current.entries())
            .find(([_, pending]) => 
              pending.message.content === messageData.content &&
              pending.message.user_id === messageData.user_id
            );
          
          if (existingTempMessage) {
            // This is our own message coming back - update the temp message instead of adding new
            const [tempId, pending] = existingTempMessage;
            console.log('Updating temp message to confirmed:', tempId, '->', messageId);
            setMessages(prev => {
              const updatedMessages = prev.map(msg => {
                if (msg.id === tempId) {
                  const confirmedMessage = {
                    ...msg,
                    id: messageId,
                    status: 'sent' as const,
                  };
                  return confirmedMessage;
                }
                return msg;
              });
              // Sort messages by created_at timestamp
              return updatedMessages.sort((a, b) => 
                new Date(a.created_at).getTime() - new Date(b.created_at).getTime()
              );
            });
            
            // Add to message IDs and remove from pending
            if (messageId) {
              messageIdsRef.current.add(messageId);
            }
            pendingMessagesRef.current.delete(tempId);
            return;
          }
        }

        const newMessage: Message = {
          id: messageId || `msg-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
          content: messageData.content,
          user_id: messageData.user_id,
          user_name: messageData.user_name,
          user_profile_image: messageData.user_profile_image,
          room_id: messageData.room_id,
          created_at: messageData.timestamp ? new Date(messageData.timestamp * 1000).toISOString() : new Date().toISOString(),
          status: 'sent',
        };

        // Add to message IDs to prevent duplicates
        if (messageId) {
          messageIdsRef.current.add(messageId);
        }

        console.log('Adding new message from server:', messageId, messageData.content);
        setMessages(prev => {
          const updatedMessages = [...prev, newMessage];
          // Sort messages by created_at timestamp
          return updatedMessages.sort((a, b) => 
            new Date(a.created_at).getTime() - new Date(b.created_at).getTime()
          );
        });
        break;

      case 'typing':
        const typingData = response.data;
        if (typingData.is_typing) {
          setTypingUsers(prev => ({
            ...prev,
            [typingData.user_id]: {
              name: typingData.user_name,
              timestamp: Date.now(),
            },
          }));
        } else {
          setTypingUsers(prev => {
            const newTypingUsers = { ...prev };
            delete newTypingUsers[typingData.user_id];
            return newTypingUsers;
          });
        }
        break;

      case 'reaction': {
        const { message_id, user_id, user_name, emoji, add } = response.data || {};
        if (message_id && emoji && typeof add === 'boolean' && user_id) {
          applyReactionUpdate(message_id, emoji, user_id, user_name || 'Unknown', add);
        }
        break;
      }

      case 'user_joined':
      case 'user_left':
        // Handle user join/leave events
        console.log('User event:', response.message_type, response.data);
        break;

      default:
        console.log('Unhandled message type:', response.message_type);
    }
  }, [user, applyReactionUpdate]);

  // Clean up old pending messages periodically
  useEffect(() => {
    const cleanupInterval = setInterval(() => {
      const now = Date.now();
      const timeout = 30000; // 30 seconds

      pendingMessagesRef.current.forEach((pending, tempId) => {
        if (now - pending.timestamp > timeout) {
          console.log('Cleaning up old pending message:', tempId);
          setMessages(prev => prev.filter(msg => msg.id !== tempId));
          pendingMessagesRef.current.delete(tempId);
        }
      });
    }, 10000); // Check every 10 seconds

    return () => clearInterval(cleanupInterval);
  }, []);

  // Reset messages when room changes
  useEffect(() => {
    if (selectedRoom) {
      setMessages([]);
      messageIdsRef.current.clear();
      pendingMessagesRef.current.clear();
      sentVoiceMessagesRef.current.clear(); // Clear sent voice messages tracking
      setTypingUsers({});
      lastSoundMessageIdRef.current = ''; // Reset sound tracking
      fetchMessages();
    }
  }, [selectedRoom?.id, fetchMessages]);

  // Auto-retry failed messages
  useEffect(() => {
    const retryInterval = setInterval(() => {
      pendingMessagesRef.current.forEach((pending, tempId) => {
        if (Date.now() - pending.timestamp > 5000) { // 5 seconds
          retryPendingMessage(tempId);
        }
      });
    }, 5000);

    return () => clearInterval(retryInterval);
  }, [retryPendingMessage]);

  return {
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
  };
}