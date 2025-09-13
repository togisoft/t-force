import { useCallback, useMemo } from 'react';
import { EmojiData, MessageReaction } from '../types';
import React from 'react';
import {
  ThumbsUp,
  ThumbsDown,
  Heart,
  Star,
  Laugh,
  Frown,
  Angry,
  Zap,
  Award,
  Trophy,
  Gift,
  Music,
  Sparkles,
  Flame,
} from 'lucide-react';

export function useReactions() {
  // Define available emoji reactions
  const availableEmojis = useMemo<EmojiData[]>(() => [
    {
      emoji: '👍',
      icon: React.createElement(ThumbsUp, { className: "h-4 w-4" }),
      label: 'Thumbs Up'
    },
    {
      emoji: '👎',
      icon: React.createElement(ThumbsDown, { className: "h-4 w-4" }),
      label: 'Thumbs Down'
    },
    {
      emoji: '❤️',
      icon: React.createElement(Heart, { className: "h-4 w-4" }),
      label: 'Heart'
    },
    {
      emoji: '⭐',
      icon: React.createElement(Star, { className: "h-4 w-4" }),
      label: 'Star'
    },
    {
      emoji: '😄',
      icon: React.createElement(Laugh, { className: "h-4 w-4" }),
      label: 'Laugh'
    },
    {
      emoji: '😢',
      icon: React.createElement(Frown, { className: "h-4 w-4" }),
      label: 'Sad'
    },
    {
      emoji: '😠',
      icon: React.createElement(Angry, { className: "h-4 w-4" }),
      label: 'Angry'
    },
    {
      emoji: '⚡',
      icon: React.createElement(Zap, { className: "h-4 w-4" }),
      label: 'Zap'
    },
    {
      emoji: '🏆',
      icon: React.createElement(Award, { className: "h-4 w-4" }),
      label: 'Award'
    },
    {
      emoji: '🏅',
      icon: React.createElement(Trophy, { className: "h-4 w-4" }),
      label: 'Trophy'
    },
    {
      emoji: '🎁',
      icon: React.createElement(Gift, { className: "h-4 w-4" }),
      label: 'Gift'
    },
    {
      emoji: '🎵',
      icon: React.createElement(Music, { className: "h-4 w-4" }),
      label: 'Music'
    },
    {
      emoji: '✨',
      icon: React.createElement(Sparkles, { className: "h-4 w-4" }),
      label: 'Sparkles'
    },
    {
      emoji: '🔥',
      icon: React.createElement(Flame, { className: "h-4 w-4" }),
      label: 'Fire'
    },
  ], []);

  // Get emoji data by emoji character
  const getEmojiData = useCallback((emoji: string) => {
    return availableEmojis.find(e => e.emoji === emoji) || {
      emoji,
      icon: null,
      label: emoji
    };
  }, [availableEmojis]);

  // Format reaction users for display
  const formatReactionUsers = useCallback((users: { user_name: string }[]) => {
    if (!users || users.length === 0) return '';

    if (users.length === 1) {
      return users[0].user_name;
    }

    if (users.length === 2) {
      return `${users[0].user_name} and ${users[1].user_name}`;
    }

    return `${users[0].user_name}, ${users[1].user_name}, and ${users.length - 2} more`;
  }, []);

  // Check if the current user has reacted with a specific emoji
  const hasUserReacted = useCallback((
      messageReactions: MessageReaction[] | undefined,
      emoji: string,
      userId: string | null
  ) => {
    if (!messageReactions || !userId) return false;

    const reaction = messageReactions.find(r => r.emoji === emoji);
    if (!reaction) return false;

    return reaction.users.some(u => u.user_id === userId);
  }, []);

  return {
    availableEmojis,
    getEmojiData,
    formatReactionUsers,
    hasUserReacted
  };
}