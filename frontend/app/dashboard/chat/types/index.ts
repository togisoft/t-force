import React from 'react';

// Types for WebSocket communication
export type WsMessage = {
  type: 'join' | 'leave' | 'message' | 'typing' | 'reaction' | 'ping';
  room_id?: string;
  content?: string;
  is_typing?: boolean;
  message_id?: string;
  emoji?: string;
  add?: boolean;
  temp_id?: string; // For client-side message reconciliation
};

export type WsResponse = {
  message_type: string;
  data: any;
  timestamp: number;
};

// Main data model for a Room
export type Room = {
  id: string;
  name: string;
  description: string | null;
  created_by: string;
  created_at: string;
  is_protected: boolean;
  room_code: string;
  is_owner: boolean;
  user_count: number;
};

// Data model for a message reaction
export interface MessageReaction {
  id: string;
  emoji: string;
  users: { user_id: string; user_name: string }[];
  count: number;
}

// Main data model for a Message
export interface Message {
  id: string;
  content: string;
  user_id: string;
  user_name: string;
  user_profile_image?: string | null;
  room_id: string;
  created_at: string;
  reactions?: MessageReaction[];
  is_own?: boolean; // Client-side property
  status?: 'sending' | 'sent' | 'failed'; // Client-side property
}

// Data model for a user who is currently typing
export interface TypingUser {
  name: string;
  timestamp: number;
}

// Data model for available Emojis
export interface EmojiData {
  emoji: string;
  icon: React.ReactElement | null;
  label: string;
}

// Data model for the authenticated user
export type AuthenticatedUser = {
  id: string;
  name: string;
  email: string;
  profile_image: string | null;
};