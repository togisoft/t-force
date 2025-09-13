"use client";
import React, { useEffect, useLayoutEffect, useRef, useState, useCallback, useMemo } from "react";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { Button } from "@/components/ui/button";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import {
  Loader2,
  RotateCcw,
  Smile,
  Clock,
  Volume2,
  VolumeX,
  CheckCircle,
  AlertCircle,
  ChevronDown,
  Heart,
  ThumbsUp,
  Laugh,
  Eye,
  Frown,
  Flame,
  Brain,
  PartyPopper,
  Hash,
  Zap,
} from "lucide-react";
import { format } from "date-fns";
import { Message, MessageReaction } from "../types";
import { cn } from "@/lib/utils";
import { MessageContent } from "@/app/dashboard/chat/components/MessageContent";

interface MessageListProps {
  messages: Message[];
  isLoadingMessages: boolean;
  typingUsers: Record<string, { name: string; timestamp: number }>;
  userId: string;
  onRetryMessage: (message: Message) => void;
  onAddReaction: (messageId: string, emoji: string) => void;
  onRemoveReaction: (messageId: string, emoji: string) => void;
  formatReactionUsers: (users: { user_name: string }[]) => string;
  hasUserReacted: (
    reactions: MessageReaction[] | undefined,
    emoji: string,
    userId: string
  ) => boolean;
  websocketReady?: boolean;
  lastSoundMessageIdRef: React.MutableRefObject<string>;
}

// Enhanced emoji data with icons and labels
const EMOJI_DATA = [
  { emoji: "👍", icon: <ThumbsUp className="h-4 w-4" />, label: "Thumbs Up" },
  { emoji: "❤️", icon: <Heart className="h-4 w-4" />, label: "Heart" },
  { emoji: "😂", icon: <Laugh className="h-4 w-4" />, label: "Laugh" },
  { emoji: "😮", icon: <Eye className="h-4 w-4" />, label: "Wow" },
  { emoji: "😢", icon: <Frown className="h-4 w-4" />, label: "Sad" },
  { emoji: "🙏", icon: <Heart className="h-4 w-4" />, label: "Pray" },
  { emoji: "🔥", icon: <Flame className="h-4 w-4" />, label: "Fire" },
  { emoji: "🤔", icon: <Brain className="h-4 w-4" />, label: "Think" },
  { emoji: "🎉", icon: <PartyPopper className="h-4 w-4" />, label: "Party" },
  { emoji: "💯", icon: <Hash className="h-4 w-4" />, label: "Hundred" },
  { emoji: "👏", icon: <Heart className="h-4 w-4" />, label: "Clap" },
  { emoji: "🤯", icon: <Zap className="h-4 w-4" />, label: "Mind Blown" },
];

// Memoized status icon component
const MessageStatusIcon = React.memo(({ message, userId }: { message: Message; userId: string }) => {
  if (message.user_id !== userId) return null;

  switch (message.status) {
    case "sending":
      return (
        <TooltipProvider>
          <Tooltip>
            <TooltipTrigger asChild>
              <Loader2 className="h-3.5 w-3.5 animate-spin opacity-60 ml-2" />
            </TooltipTrigger>
            <TooltipContent>Sending...</TooltipContent>
          </Tooltip>
        </TooltipProvider>
      );
    case "sent":
      return (
        <TooltipProvider>
          <Tooltip>
            <TooltipTrigger asChild>
              <CheckCircle className="h-3.5 w-3.5 opacity-60 ml-2 text-green-500" />
            </TooltipTrigger>
            <TooltipContent>Sent</TooltipContent>
          </Tooltip>
        </TooltipProvider>
      );
    case "failed":
      return (
        <TooltipProvider>
          <Tooltip>
            <TooltipTrigger asChild>
              <AlertCircle className="h-3.5 w-3.5 opacity-60 ml-2 text-red-500" />
            </TooltipTrigger>
            <TooltipContent>Failed to send</TooltipContent>
          </Tooltip>
        </TooltipProvider>
      );
    default:
      return null;
  }
});

MessageStatusIcon.displayName = 'MessageStatusIcon';

// Enhanced Reaction Button Component
const ReactionButton = React.memo(({ 
  reaction, 
  messageId, 
  userId, 
  hasUserReacted, 
  onReactionClick 
}: {
  reaction: MessageReaction;
  messageId: string;
  userId: string;
  hasUserReacted: (reactions: MessageReaction[] | undefined, emoji: string, userId: string) => boolean;
  onReactionClick: (messageId: string, emoji: string, reactions?: MessageReaction[]) => void;
}) => {
  const userReacted = hasUserReacted([reaction], reaction.emoji, userId);
  
  return (
    <TooltipProvider>
      <Tooltip>
        <TooltipTrigger asChild>
          <Button
            variant={userReacted ? "default" : "outline"}
            size="sm"
            className={cn(
              "h-7 px-2 text-xs rounded-full border transition-all duration-200 hover:scale-105",
              userReacted
                ? "bg-gradient-to-r from-blue-500 to-purple-500 text-white shadow-lg hover:shadow-xl"
                : "bg-white/80 dark:bg-gray-800/80 hover:bg-gray-100 dark:hover:bg-gray-700 backdrop-blur-sm"
            )}
            onClick={() => onReactionClick(messageId, reaction.emoji, [reaction])}
          >
            <span className="mr-1 text-sm">{reaction.emoji}</span>
            <span className="font-semibold">{reaction.count}</span>
          </Button>
        </TooltipTrigger>
        <TooltipContent className="bg-gray-900 text-white border-gray-700 shadow-lg">
          {reaction.users.map(u => u.user_name).join(", ")}
        </TooltipContent>
      </Tooltip>
    </TooltipProvider>
  );
});

ReactionButton.displayName = 'ReactionButton';

export function MessageList({
  messages,
  isLoadingMessages,
  typingUsers,
  userId,
  onRetryMessage,
  onAddReaction,
  onRemoveReaction,
  formatReactionUsers,
  hasUserReacted,
  websocketReady = true,
  lastSoundMessageIdRef,
}: MessageListProps) {
  // --- Refs ---
  const wrapperRef = useRef<HTMLDivElement>(null);
  const viewportRef = useRef<HTMLDivElement | null>(null);
  const audioRef = useRef<HTMLAudioElement | null>(null);
  // Use the external ref to track last sound message ID
  const lastSoundMessageId = lastSoundMessageIdRef;
  const lastReadMessageId = useRef<string>('');
  const scrollListenerRef = useRef<((e: Event) => void) | null>(null);

  // --- State ---
  const [isAtBottom, setIsAtBottom] = useState(true);
  const [soundEnabled, setSoundEnabled] = useState(true);
  const [unreadCount, setUnreadCount] = useState(0);
  const [showScrollButton, setShowScrollButton] = useState(false);
  const [isScrolling, setIsScrolling] = useState(false);

  // --- Sound Setup ---
  useEffect(() => {
    audioRef.current = new Audio("/sound.mp3");
    audioRef.current.volume = 0.3;
    audioRef.current.preload = "auto";

    return () => {
      if (audioRef.current) {
        audioRef.current.pause();
        audioRef.current = null;
      }
    };
  }, []);

  // --- Stable callback functions ---
  const playNotificationSound = useCallback(() => {
    if (soundEnabled && audioRef.current) {
      audioRef.current.currentTime = 0;
      audioRef.current.play().catch((error) => {
        console.log("Could not play notification sound:", error);
      });
    }
  }, [soundEnabled]);

  const handleReactionClick = useCallback((
    messageId: string,
    emoji: string,
    reactions?: MessageReaction[]
  ) => {
    const userReacted = hasUserReacted(reactions, emoji, userId);
    if (userReacted) {
      onRemoveReaction(messageId, emoji);
    } else {
      onAddReaction(messageId, emoji);
    }
  }, [hasUserReacted, userId, onAddReaction, onRemoveReaction]);

  const toggleSound = useCallback(() => {
    setSoundEnabled(prev => !prev);
  }, []);

  const handleRetry = useCallback((message: Message) => {
    console.log(`Retrying message: ${message.id}`);
    onRetryMessage(message);
  }, [onRetryMessage]);

  const scrollToBottom = useCallback((behavior: ScrollBehavior = 'smooth', force: boolean = false) => {
    if (!viewportRef.current || (!force && isScrolling)) return;
    
    setIsScrolling(true);
    const viewport = viewportRef.current;
    
    // Direct scroll to bottom - most reliable method
    const scrollToEnd = () => {
      viewport.scrollTop = viewport.scrollHeight;
    };
    
    if (behavior === 'smooth') {
      // Smooth scroll with animation
      viewport.scrollTo({
        top: viewport.scrollHeight,
        behavior: 'smooth'
      });
      // Ensure we reach the bottom after animation
      setTimeout(scrollToEnd, 600);
    } else {
      // Instant scroll
      scrollToEnd();
    }
    
    // Reset scrolling flag
    setTimeout(() => {
      setIsScrolling(false);
      setIsAtBottom(true);
      setUnreadCount(0);
      setShowScrollButton(false);
    }, behavior === 'smooth' ? 800 : 100);
  }, [isScrolling]);



  const handleScrollToBottom = useCallback(() => {
    scrollToBottom('smooth');
    setUnreadCount(0);
    setShowScrollButton(false);
    // Mark all messages as read - use ref to access current messages without dependency
    if (messages.length > 0) {
      lastReadMessageId.current = messages[messages.length - 1].id;
    }
  }, [scrollToBottom, messages.length]);

  // Check if at bottom - simplified and more reliable
  const checkIfAtBottom = useCallback(() => {
    if (!viewportRef.current) return true;
    
    const { scrollTop, scrollHeight, clientHeight } = viewportRef.current;
    // Within 5px of bottom is considered "at bottom"
    return scrollHeight - (scrollTop + clientHeight) <= 5;
  }, []);

  // --- Scrolling Logic ---
  useEffect(() => {
    if (!wrapperRef.current) return;

    // Find the scroll viewport more reliably
    const vp = wrapperRef.current.querySelector(
      "[data-radix-scroll-area-viewport]"
    ) as HTMLDivElement | null;

    if (!vp) {
      console.warn('Scroll viewport not found');
      return;
    }

    viewportRef.current = vp;

    // Remove existing listener
    if (scrollListenerRef.current) {
      vp.removeEventListener("scroll", scrollListenerRef.current);
    }

    const handleScroll = () => {
      // Don't handle scroll events during programmatic scrolling
      if (isScrolling) return;

      const atBottom = checkIfAtBottom();
      setIsAtBottom(atBottom);
      
      if (atBottom) {
        setUnreadCount(0);
        setShowScrollButton(false);
        if (messages.length > 0) {
          lastReadMessageId.current = messages[messages.length - 1].id;
        }
      } else {
        setShowScrollButton(unreadCount > 0);
      }
    };

    scrollListenerRef.current = handleScroll;
    vp.addEventListener("scroll", handleScroll, { passive: true });
    
    // Initial check
    handleScroll();
    
    return () => {
      if (scrollListenerRef.current) {
        vp.removeEventListener("scroll", scrollListenerRef.current);
      }
    };
  }, [checkIfAtBottom, messages.length, unreadCount, isScrolling]);

  // Update isAtBottom when messages change (for initial load)
  useEffect(() => {
    if (messages.length > 0 && viewportRef.current) {
      const atBottom = checkIfAtBottom();
      setIsAtBottom(atBottom);
    }
  }, [messages.length, checkIfAtBottom]);

  // On initial load, scroll to bottom
  useLayoutEffect(() => {
    if (!isLoadingMessages && messages.length > 0) {
      setTimeout(() => {
        scrollToBottom('auto', true);
        if (messages.length > 0) {
          lastReadMessageId.current = messages[messages.length - 1].id;
        }
      }, 50);
    }
  }, [isLoadingMessages, messages.length, scrollToBottom]);

  // Handle new messages with sound, scroll, and unread count
  useEffect(() => {
    if (messages.length === 0) return;

    const lastMessage = messages[messages.length - 1];
    const isNewMessage = lastMessage.id !== lastSoundMessageId.current;

    // Only process truly new messages, not messages loaded from fetchMessages
    if (isNewMessage && lastMessage.status === 'sent') {
      lastSoundMessageId.current = lastMessage.id;
      
      // Handle own messages - always scroll to bottom
      if (lastMessage.user_id === userId) {
        scrollToBottom('smooth');
        lastReadMessageId.current = lastMessage.id;
        return;
      }
      
      // For other users' messages
      const messageTime = new Date(lastMessage.created_at).getTime();
      const now = Date.now();
      const isRecentMessage = (now - messageTime) < 5000;
      
      // Play notification for recent messages
      if (isRecentMessage) {
        playNotificationSound();
      }
      
      // Auto-scroll if at bottom, otherwise show unread indicator
      if (isAtBottom) {
        scrollToBottom('smooth');
        lastReadMessageId.current = lastMessage.id;
      } else {
        setUnreadCount(prev => prev + 1);
        setShowScrollButton(true);
      }
    }
  }, [messages, userId, playNotificationSound, scrollToBottom, isAtBottom]);

  // Memoized message renderer with enhanced bubbles
  const renderMessage = useCallback((message: Message) => {
    const isOwn = message.user_id === userId;
    const hasReactions = message.reactions && message.reactions.length > 0;
    const isPending = message.status === "sending";

    return (
      <div
        key={message.id}
        className={cn(
          "flex gap-3 mb-4 group/message transition-all duration-300 hover:scale-[1.01]",
          isOwn ? "justify-end" : "justify-start",
          isPending && "opacity-70"
        )}
      >
        {!isOwn && (
          <Avatar className="h-9 w-9 flex-shrink-0 mt-1 ring-2 ring-gray-200 dark:ring-gray-700">
            <AvatarImage src={message.user_profile_image || undefined} />
            <AvatarFallback className="text-xs bg-gradient-to-br from-blue-400 to-purple-500 text-white font-semibold">
              {message.user_name.charAt(0).toUpperCase()}
            </AvatarFallback>
          </Avatar>
        )}

        <div
          className={cn(
            "flex flex-col max-w-[75%] min-w-0",
            isOwn ? "items-end" : "items-start"
          )}
        >
          {!isOwn && (
            <div className="text-xs text-muted-foreground mb-1 ml-1 font-medium flex items-center gap-1">
              <span>{message.user_name}</span>
              <span className="text-muted-foreground/60">•</span>
              <span>{format(new Date(message.created_at), "HH:mm")}</span>
            </div>
          )}

          {/* Enhanced Message Bubble */}
          <div
            className={cn(
              "relative rounded-2xl px-4 py-3 shadow-lg transition-all duration-200 hover:shadow-xl",
              "backdrop-blur-sm border",
              isOwn
                ? "bg-gradient-to-br from-blue-500 via-blue-600 to-purple-600 text-white rounded-br-md border-blue-400/30"
                : "bg-white/90 dark:bg-gray-800/90 rounded-bl-md border-gray-200/50 dark:border-gray-700/50",
              isPending && "border-dashed border-muted-foreground/30 animate-pulse"
            )}
          >
            {/* Message Content */}
            <div className="whitespace-pre-wrap break-words text-sm leading-relaxed">
              <MessageContent content={message.content} />
            </div>

            {/* Retry Button for Failed Messages */}
            {isOwn && message.status === "failed" && (
              <TooltipProvider>
                <Tooltip>
                  <TooltipTrigger asChild>
                    <Button
                      variant="ghost"
                      size="sm"
                      className="absolute -bottom-8 right-0 h-7 px-2 text-red-500 hover:bg-red-50 dark:hover:bg-red-950/20"
                      onClick={() => handleRetry(message)}
                    >
                      <RotateCcw className="h-3 w-3 mr-1" /> Retry
                    </Button>
                  </TooltipTrigger>
                  <TooltipContent>Message failed to send</TooltipContent>
                </Tooltip>
              </TooltipProvider>
            )}
          </div>

          {/* Enhanced Reactions & Timestamp */}
          <div
            className={cn(
              "flex flex-col mt-2",
              isOwn ? "items-end" : "items-start"
            )}
          >
            {/* Reactions */}
            {hasReactions && (
              <div className="flex flex-wrap gap-1 mb-2">
                {message.reactions!.map((reaction) => (
                  <ReactionButton
                    key={reaction.id}
                    reaction={reaction}
                    messageId={message.id}
                    userId={userId}
                    hasUserReacted={hasUserReacted}
                    onReactionClick={handleReactionClick}
                  />
                ))}
              </div>
            )}

            {/* Timestamp and Actions */}
            <div className="flex items-center gap-2 text-xs text-muted-foreground/80">
              {isOwn && (
                <>
                  <span>{format(new Date(message.created_at), "HH:mm")}</span>
                  <MessageStatusIcon message={message} userId={userId} />
                </>
              )}

              {/* Enhanced Reaction Button */}
              {!isPending && (
                <Popover>
                  <PopoverTrigger asChild>
                    <Button
                      variant="ghost"
                      size="icon"
                      className="h-7 w-7 p-0 opacity-0 group-hover/message:opacity-70 hover:opacity-100 transition-all duration-200 hover:scale-110 hover:bg-blue-50 dark:hover:bg-blue-950/20"
                    >
                      <Smile className="h-4 w-4" />
                    </Button>
                  </PopoverTrigger>
                  <PopoverContent className="w-auto p-3 bg-white/95 dark:bg-gray-900/95 backdrop-blur-sm border shadow-xl">
                    <div className="grid grid-cols-6 gap-2">
                      {EMOJI_DATA.map((emojiItem) => (
                        <TooltipProvider key={emojiItem.emoji}>
                          <Tooltip>
                            <TooltipTrigger asChild>
                              <Button
                                variant="ghost"
                                size="sm"
                                className="h-10 w-10 hover:bg-blue-50 dark:hover:bg-blue-950/20 rounded-full transition-all duration-200 hover:scale-110"
                                onClick={() => handleReactionClick(message.id, emojiItem.emoji, message.reactions)}
                              >
                                <span className="text-lg">{emojiItem.emoji}</span>
                              </Button>
                            </TooltipTrigger>
                            <TooltipContent className="bg-gray-900 text-white border-gray-700">
                              {emojiItem.label}
                            </TooltipContent>
                          </Tooltip>
                        </TooltipProvider>
                      ))}
                    </div>
                  </PopoverContent>
                </Popover>
              )}
            </div>
          </div>
        </div>

        {isOwn && (
          <Avatar className="h-9 w-9 flex-shrink-0 mt-1 ring-2 ring-blue-400/30">
            <AvatarImage src={message.user_profile_image || undefined} />
            <AvatarFallback className="text-xs bg-gradient-to-br from-blue-500 to-purple-600 text-white font-semibold">
              You
            </AvatarFallback>
          </Avatar>
        )}
      </div>
    );
  }, [userId, handleReactionClick, hasUserReacted, handleRetry]);

  // Memoized rendered messages with duplicate filtering to prevent re-rendering all messages
  const renderedMessages = useMemo(() => {
    // Remove duplicates based on ID and content to prevent double rendering
    const uniqueMessages = messages.filter((message, index, arr) => {
      // Check if this is the first occurrence of this message ID
      const firstOccurrence = arr.findIndex(m => m.id === message.id);
      if (firstOccurrence !== index) {
        return false;
      }
      
      // Additional check for content duplicates (in case of temp ID issues)
      const sameContentMessages = arr.filter(m => 
        m.content === message.content && 
        m.user_id === message.user_id &&
        Math.abs(new Date(m.created_at).getTime() - new Date(message.created_at).getTime()) < 5000 // Within 5 seconds
      );
      
      if (sameContentMessages.length > 1) {
        // Keep the one with 'sent' status or the latest one
        const sentMessage = sameContentMessages.find(m => m.status === 'sent');
        if (sentMessage && sentMessage.id !== message.id) {
          return false;
        }
        
        // If no sent message, keep the latest one
        const latestMessage = sameContentMessages.reduce((latest, current) => 
          new Date(current.created_at) > new Date(latest.created_at) ? current : latest
        );
        
        return latestMessage.id === message.id;
      }
      
      return true;
    });
    
    return uniqueMessages.map(renderMessage);
  }, [messages, renderMessage]);

  return (
    <div
      ref={wrapperRef}
      className="h-full relative bg-gradient-to-br from-gray-50 via-blue-50/30 to-purple-50/30 dark:from-gray-900 dark:via-blue-950/20 dark:to-purple-950/20"
    >
      {/* WebSocket Connection Status */}
      {!websocketReady && (
        <div className="absolute top-4 left-4 z-10">
          <div className="flex items-center gap-2 bg-yellow-100 dark:bg-yellow-900/30 text-yellow-800 dark:text-yellow-200 px-3 py-1.5 rounded-full text-xs shadow-lg">
            <Loader2 className="h-3 w-3 animate-spin" />
            Connecting...
          </div>
        </div>
      )}

      {/* Sound Toggle Button */}
      <div className="absolute top-4 right-4 z-10">
        <TooltipProvider>
          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                variant="ghost"
                size="icon"
                className="h-9 w-9 bg-white/80 dark:bg-gray-800/80 hover:bg-white dark:hover:bg-gray-700 backdrop-blur-sm shadow-lg hover:shadow-xl transition-all duration-200 hover:scale-105"
                onClick={toggleSound}
              >
                {soundEnabled ? (
                  <Volume2 className="h-4 w-4 text-muted-foreground" />
                ) : (
                  <VolumeX className="h-4 w-4 text-muted-foreground" />
                )}
              </Button>
            </TooltipTrigger>
            <TooltipContent className="bg-gray-900 text-white border-gray-700">
              {soundEnabled ? "Mute sounds" : "Enable sounds"}
            </TooltipContent>
          </Tooltip>
        </TooltipProvider>
      </div>

      <ScrollArea className="h-full">
        <div className="px-4 py-4 pr-6">
          {isLoadingMessages ? (
            <div className="flex justify-center py-8">
              <div className="flex items-center gap-3 bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-full px-4 py-2 shadow-lg">
                <Loader2 className="h-5 w-5 animate-spin text-blue-500" />
                <span className="text-sm text-muted-foreground">Loading messages...</span>
              </div>
            </div>
          ) : messages.length === 0 ? (
            <div className="flex flex-col items-center justify-center h-full py-16 text-center">
              <div className="rounded-full bg-gradient-to-br from-blue-100 to-purple-100 dark:from-blue-900/30 dark:to-purple-900/30 p-6 mb-4 shadow-lg">
                <Clock className="h-10 w-10 text-blue-500 dark:text-blue-400" />
              </div>
              <div className="text-muted-foreground mb-1 text-lg font-light">
                No messages yet
              </div>
              <div className="text-sm text-muted-foreground/70">
                Start the conversation!
              </div>
            </div>
          ) : (
            <>
              {renderedMessages}
            </>
          )}
        </div>
      </ScrollArea>

      {/* Enhanced Scroll to Bottom Button */}
      {showScrollButton && (
        <div className="absolute bottom-4 right-4 z-10">
          <TooltipProvider>
            <Tooltip>
              <TooltipTrigger asChild>
                <Button
                  onClick={handleScrollToBottom}
                  className="h-14 w-14 rounded-full bg-gradient-to-r from-blue-500 to-purple-600 hover:from-blue-600 hover:to-purple-700 text-white shadow-xl hover:shadow-2xl transition-all duration-300 hover:scale-110 backdrop-blur-sm border-2 border-white/20"
                >
                  <div className="relative flex items-center justify-center">
                    <ChevronDown className="h-6 w-6" />
                    {unreadCount > 0 && (
                      <div className="absolute -top-2 -right-2 bg-red-500 text-white text-xs rounded-full h-7 w-7 flex items-center justify-center font-bold shadow-lg border-2 border-white animate-pulse">
                        {unreadCount > 99 ? '99+' : unreadCount}
                      </div>
                    )}
                  </div>
                </Button>
              </TooltipTrigger>
              <TooltipContent className="bg-gray-900 text-white border-gray-700 shadow-lg">
                {unreadCount > 0 
                  ? `${unreadCount} yeni mesaj${unreadCount > 1 ? '' : ''}`
                  : 'En alta git'
                }
              </TooltipContent>
            </Tooltip>
          </TooltipProvider>
        </div>
      )}

      {/* Connection Status Indicator */}
      {!websocketReady && (
        <div className="absolute bottom-3 right-6 bg-yellow-100 dark:bg-yellow-900/30 text-yellow-800 dark:text-yellow-200 px-3 py-1.5 rounded-full text-xs shadow-lg">
          Reconnecting...
        </div>
      )}
    </div>
  );
}