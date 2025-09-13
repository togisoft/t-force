import React, { useState, useRef, useCallback, useEffect } from 'react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip';
import { Send, Image, Loader2, Smile, Mic, MicOff, Video } from 'lucide-react';
import { useToast } from '@/hooks/use-toast';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import { useReactions } from '../hooks/useReactions';
import { VoiceMessage } from '@/components/chat/VoiceMessage';

interface MessageInputProps {
  selectedRoom: any;
  isConnected: boolean;
  onSendMessage: (content: string) => boolean;
  onTypingIndicator: (isTyping: boolean) => void;
  onUploadImage?: (file: File) => Promise<string | null>;
  onUploadVideo?: (file: File) => Promise<string | null>;
  onSendVoiceMessage?: (audioBlob: Blob) => Promise<boolean>;
}

export function MessageInput({
                               selectedRoom,
                               isConnected,
                               onSendMessage,
                               onTypingIndicator,
                               onUploadImage,
                               onUploadVideo,
                               onSendVoiceMessage,
                             }: MessageInputProps) {
  const [message, setMessage] = useState('');
  const [isSending, setIsSending] = useState(false);
  const [uploadingType, setUploadingType] = useState<null | 'image' | 'video'>(null);
  const isUploading = uploadingType !== null;
  const [isTyping, setIsTyping] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const videoInputRef = useRef<HTMLInputElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);
  const typingTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const lastSentMessageRef = useRef<{ content: string; timestamp: number } | null>(null);
  const { toast } = useToast();
  const { availableEmojis } = useReactions();

  // Typing indicator management with debouncing
  useEffect(() => {
    if (message.trim()) {
      if (!isTyping) {
        setIsTyping(true);
        onTypingIndicator(true);
      }
      
      // Clear existing timeout
      if (typingTimeoutRef.current) {
        clearTimeout(typingTimeoutRef.current);
      }
      
      // Set new timeout to stop typing indicator
      typingTimeoutRef.current = setTimeout(() => {
        setIsTyping(false);
        onTypingIndicator(false);
      }, 2000);
    } else {
      if (isTyping) {
        setIsTyping(false);
        onTypingIndicator(false);
      }
      if (typingTimeoutRef.current) {
        clearTimeout(typingTimeoutRef.current);
        typingTimeoutRef.current = null;
      }
    }
    
    // Cleanup on unmount
    return () => {
      if (typingTimeoutRef.current) {
        clearTimeout(typingTimeoutRef.current);
      }
    };
  }, [message, isTyping, onTypingIndicator]);

  const handleSubmit = useCallback((e: React.FormEvent) => {
    e.preventDefault();
    if (!message.trim() || !isConnected || isSending) return;
    
    const trimmedMessage = message.trim();
    const now = Date.now();
    
    // Prevent duplicate submissions within 500ms
    if (lastSentMessageRef.current && 
        lastSentMessageRef.current.content === trimmedMessage && 
        now - lastSentMessageRef.current.timestamp < 500) {
      console.log('Duplicate submission prevented');
      return;
    }
    
    setIsSending(true);
    lastSentMessageRef.current = { content: trimmedMessage, timestamp: now };
    
    try {
      const success = onSendMessage(trimmedMessage);
      if (success) {
        setMessage('');
        setIsTyping(false);
        onTypingIndicator(false);
        // Clear typing timeout
        if (typingTimeoutRef.current) {
          clearTimeout(typingTimeoutRef.current);
          typingTimeoutRef.current = null;
        }
        // Focus input after sending
        setTimeout(() => {
          inputRef.current?.focus();
        }, 100);
      } else {
        toast({
          title: "Failed to send",
          description: "Message could not be sent. Please try again.",
          variant: "destructive",
        });
      }
    } catch (error) {
      console.error('Error in handleSubmit:', error);
      toast({
        title: "Error",
        description: "An unexpected error occurred.",
        variant: "destructive",
      });
    } finally {
      setIsSending(false);
    }
  }, [message, isConnected, isSending, onSendMessage, onTypingIndicator, toast]);

  const handleInputChange = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    setMessage(e.target.value);
  }, []);

  const handleKeyPress = useCallback((e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey && !isSending) {
      e.preventDefault();
      e.stopPropagation();
      handleSubmit(e as any);
    }
  }, [handleSubmit, isSending]);

  const handleImageUpload = useCallback(async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file || !onUploadImage) return;
    
    // Validate file type
    if (!file.type.startsWith('image/')) {
      toast({
        title: "Invalid file type",
        description: "Please select an image file.",
        variant: "destructive",
      });
      return;
    }
    
    // Validate file size (max 10MB)
    if (file.size > 10 * 1024 * 1024) {
      toast({
        title: "File too large",
        description: "Please select an image smaller than 10MB.",
        variant: "destructive",
      });
      return;
    }
    
    setUploadingType('image');
    try {
      const imageUrl = await onUploadImage(file);
      if (imageUrl) {
        // Send the image message
        const imageMessage = `[image](${imageUrl})`;
        const success = onSendMessage(imageMessage);
        if (success) {
          toast({
            title: "Success",
            description: "Image uploaded and sent successfully!",
          });
        }
      }
    } catch (error) {
      console.error('Error uploading image:', error);
      toast({
        title: "Upload failed",
        description: "Failed to upload image. Please try again.",
        variant: "destructive",
      });
    } finally {
      setUploadingType(null);
      // Reset file input
      if (fileInputRef.current) {
        fileInputRef.current.value = '';
      }
    }
  }, [onUploadImage, onSendMessage, toast]);

  const openImagePicker = useCallback(() => {
    fileInputRef.current?.click();
  }, []);

  const handleVideoUpload = useCallback(async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file || !onUploadVideo) return;

    const allowedTypes = ['video/mp4', 'video/webm', 'video/ogg'];
    if (!allowedTypes.includes(file.type)) {
      toast({
        title: 'Invalid file type',
        description: 'Please select an MP4, WebM, or Ogg video.',
        variant: 'destructive',
      });
      return;
    }

    // Validate file size (max 50MB)
    if (file.size > 50 * 1024 * 1024) {
      toast({
        title: 'File too large',
        description: 'Please select a video smaller than 50MB.',
        variant: 'destructive',
      });
      return;
    }

    setUploadingType('video');
    try {
      const videoUrl = await onUploadVideo(file);
      if (videoUrl) {
        const videoMessage = `[video](${videoUrl})`;
        const success = onSendMessage(videoMessage);
        if (success) {
          toast({
            title: 'Success',
            description: 'Video uploaded and sent successfully!',
          });
        }
      }
    } catch (error) {
      console.error('Error uploading video:', error);
      toast({
        title: 'Upload failed',
        description: 'Failed to upload video. Please try again.',
        variant: 'destructive',
      });
    } finally {
      setUploadingType(null);
      if (videoInputRef.current) {
        videoInputRef.current.value = '';
      }
    }
  }, [onUploadVideo, onSendMessage, toast]);

  const openVideoPicker = useCallback(() => {
    videoInputRef.current?.click();
  }, []);

  const handleEmojiSelect = useCallback((emoji: string) => {
    if (inputRef.current) {
      const startPos = inputRef.current.selectionStart || 0;
      const endPos = inputRef.current.selectionEnd || 0;
      const newMessage = message.substring(0, startPos) + emoji + message.substring(endPos);
      setMessage(newMessage);
      // Focus input and set cursor position after emoji
      setTimeout(() => {
        if (inputRef.current) {
          inputRef.current.focus();
          inputRef.current.selectionStart = startPos + emoji.length;
          inputRef.current.selectionEnd = startPos + emoji.length;
        }
      }, 0);
    }
  }, [message]);

  const handleVoiceMessage = useCallback(async (audioBlob: Blob) => {
    if (!onSendVoiceMessage) {
      toast({
        title: "Voice messages not supported",
        description: "Voice message feature is not available.",
        variant: "destructive",
      });
      return;
    }

    try {
      const success = await onSendVoiceMessage(audioBlob);
      if (success) {
        toast({
          title: "Voice message sent",
          description: "Your voice message has been sent successfully!",
        });
      }
    } catch (error) {
      console.error('Error sending voice message:', error);
      toast({
        title: "Failed to send voice message",
        description: "Please try again.",
        variant: "destructive",
      });
    }
  }, [onSendVoiceMessage, toast]);

  if (!selectedRoom) {
    return null;
  }

  return (
      <div className="bg-white/95 dark:bg-gray-900/95 backdrop-blur-sm border-t border-gray-200/50 dark:border-gray-700/50 p-4 shadow-lg">
        <form onSubmit={handleSubmit} className="flex items-end gap-3">
          <div className="flex items-center gap-1">
            <input
                ref={fileInputRef}
                type="file"
                accept="image/*"
                onChange={handleImageUpload}
                className="hidden"
            />
            {/* Hidden video input */}
            <input
                ref={videoInputRef}
                type="file"
                accept="video/*"
                onChange={handleVideoUpload}
                className="hidden"
            />
            <TooltipProvider>
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button
                      type="button"
                      variant="ghost"
                      size="icon"
                      onClick={openImagePicker}
                      disabled={!isConnected || isUploading}
                      className="h-10 w-10 rounded-full text-gray-500 hover:text-blue-600 hover:bg-blue-50 dark:text-gray-400 dark:hover:text-blue-400 dark:hover:bg-blue-950/20 transition-all duration-200 hover:scale-105"
                  >
                    {isUploading && uploadingType === 'image' ? (
                        <Loader2 className="h-4 w-4 animate-spin" />
                    ) : (
                        <Image className="h-4 w-4" />
                    )}
                  </Button>
                </TooltipTrigger>
                <TooltipContent>
                  <p>{isUploading && uploadingType === 'image' ? 'Uploading...' : 'Upload image'}</p>
                </TooltipContent>
              </Tooltip>
            </TooltipProvider>
            {/* Video upload button */}
            <TooltipProvider>
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button
                      type="button"
                      variant="ghost"
                      size="icon"
                      onClick={openVideoPicker}
                      disabled={!isConnected || isUploading}
                      className="h-10 w-10 rounded-full text-gray-500 hover:text-blue-600 hover:bg-blue-50 dark:text-gray-400 dark:hover:text-blue-400 dark:hover:bg-blue-950/20 transition-all duration-200 hover:scale-105"
                  >
                    {isUploading && uploadingType === 'video' ? (
                        <Loader2 className="h-4 w-4 animate-spin" />
                    ) : (
                        <Video className="h-4 w-4" />
                    )}
                  </Button>
                </TooltipTrigger>
                <TooltipContent>
                  <p>{isUploading && uploadingType === 'video' ? 'Uploading...' : 'Upload video'}</p>
                </TooltipContent>
              </Tooltip>
            </TooltipProvider>
            <Popover>
              <PopoverTrigger asChild>
                <Button
                    type="button"
                    variant="ghost"
                    size="icon"
                    className="h-10 w-10 rounded-full text-gray-500 hover:text-blue-600 hover:bg-blue-50 dark:text-gray-400 dark:hover:text-blue-400 dark:hover:bg-blue-950/20 transition-all duration-200 hover:scale-105"
                >
                  <Smile className="h-4 w-4" />
                </Button>
              </PopoverTrigger>
              <PopoverContent className="w-auto p-3 bg-white/95 dark:bg-gray-900/95 backdrop-blur-sm border border-gray-200/50 dark:border-gray-700/50 shadow-xl">
                <div className="grid grid-cols-8 gap-2 max-h-40 overflow-y-auto">
                  {availableEmojis.map((emojiObj) => (
                      <Button
                          key={emojiObj.emoji}
                          variant="ghost"
                          size="icon"
                          className="h-10 w-10 p-0 hover:bg-blue-50 dark:hover:bg-blue-950/20 rounded-full transition-all duration-200 hover:scale-110"
                          onClick={() => handleEmojiSelect(emojiObj.emoji)}
                      >
                        <span className="text-lg">{emojiObj.emoji}</span>
                      </Button>
                  ))}
                </div>
              </PopoverContent>
            </Popover>

          </div>
          <div className="flex-1 relative">
            <Input
                ref={inputRef}
                value={message}
                onChange={handleInputChange}
                onKeyDown={handleKeyPress}
                placeholder={
                  !isConnected
                      ? "Disconnected. Reconnecting..."
                      : isUploading
                          ? (uploadingType === 'video' ? 'Uploading video...' : 'Uploading image...')
                          : "Type a message..."
                }
                disabled={!isConnected || isSending || isUploading}
                className="pr-12 min-h-[44px] py-3 resize-none bg-white/80 dark:bg-gray-800/80 border border-gray-200/50 dark:border-gray-700/50 focus:ring-2 focus:ring-blue-500 focus:border-transparent rounded-full backdrop-blur-sm shadow-sm hover:shadow-md transition-all duration-200"
                maxLength={2000}
            />
            {message.length > 0 && (
                <div className="absolute right-3 top-1/2 transform -translate-y-1/2">
              <span className="text-xs text-gray-400 dark:text-gray-500">
                {2000 - message.length}
              </span>
                </div>
            )}
          </div>
          
          {/* Voice Message Button */}
          <TooltipProvider>
            <Tooltip>
              <TooltipTrigger asChild>
                <Button
                  type="button"
                  variant="ghost"
                  size="icon"
                  onClick={() => {
                    console.log('Voice button clicked');
                    // Trigger voice recording
                    const voiceButton = document.querySelector('[data-voice-record]') as HTMLButtonElement;
                    if (voiceButton) {
                      console.log('Found voice button, clicking...');
                      voiceButton.click();
                    } else {
                      console.log('Voice button not found');
                    }
                  }}
                  disabled={!isConnected}
                  className="h-10 w-10 rounded-full bg-gradient-to-r from-green-500 to-green-600 hover:from-green-600 hover:to-green-700 transition-all duration-200 shadow-md hover:shadow-lg hover:scale-105 disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  <Mic className="h-4 w-4 text-white" />
                </Button>
              </TooltipTrigger>
              <TooltipContent>
                <p>Record voice message</p>
              </TooltipContent>
            </Tooltip>
          </TooltipProvider>
          <Button
              type="submit"
              size="icon"
              disabled={!message.trim() || !isConnected || isSending || isUploading}
              className="h-10 w-10 rounded-full bg-gradient-to-r from-blue-500 to-purple-600 hover:from-blue-600 hover:to-purple-700 transition-all duration-200 flex-shrink-0 disabled:opacity-50 disabled:cursor-not-allowed shadow-md hover:shadow-lg hover:scale-105"
          >
            {isSending ? (
                <Loader2 className="h-4 w-4 animate-spin text-white" />
            ) : (
                <Send className="h-4 w-4 text-white" />
            )}
          </Button>
        </form>
        
        {/* Voice Message Component */}
        <VoiceMessage
          onSend={handleVoiceMessage}
          disabled={!isConnected}
        />
        
        <div className="mt-2 flex justify-between items-center">
          <div className="text-xs text-gray-500 dark:text-gray-400">
            {!isConnected && (
                <span className="text-red-500 dark:text-red-400">
              Connection lost. Trying to reconnect...
            </span>
            )}
            {isUploading && (
                <span>Uploading {uploadingType === 'video' ? 'video' : 'image'}...</span>
            )}
          </div>
          {message.length > 1800 && (
              <div className="text-xs text-gray-500 dark:text-gray-400">
                {2000 - message.length} characters remaining
              </div>
          )}
        </div>
      </div>
  );
}
