
import { useState, useCallback } from 'react';
import { useToast } from '@/hooks/use-toast';

export interface Room {
  id: string;
  name: string;
  description: string | null;
  created_by: string;
  created_at: string;
  is_protected: boolean;
  is_owner: boolean;
  room_code: string;
  user_count: number;
}

interface UseRoomsOptions {
  userId?: string | null;
}

interface UseRoomsReturn {
  rooms: Room[];
  selectedRoom: Room | null;
  isLoadingRooms: boolean;
  roomUsers: { [roomId: string]: number };
  fetchRooms: () => Promise<void>;
  createRoom: (name: string, description?: string, password?: string) => Promise<Room | null>;
  joinProtectedRoom: (roomId: string, password: string) => Promise<boolean>;
  joinRoomByCode: (code: string, password?: string) => Promise<Room | null>;
  deleteRoom: (roomId: string) => Promise<boolean>;
  updateRoomUserCount: (roomId: string, count: number) => void;
  selectRoom: (room: Room | null) => void;
  leaveRoomMembership: (roomId: string) => Promise<boolean>;
}

export function useRooms({ userId }: UseRoomsOptions): UseRoomsReturn {
  const [rooms, setRooms] = useState<Room[]>([]);
  const [selectedRoom, setSelectedRoom] = useState<Room | null>(null);
  const [isLoadingRooms, setIsLoadingRooms] = useState(false);
  const [roomUsers, setRoomUsers] = useState<{ [roomId: string]: number }>({});
  const { toast } = useToast();
  // Prefer relative API paths and let Next.js rewrites proxy to backend
  const API_BASE = '/api';

  const fetchRooms = useCallback(async () => {
    setIsLoadingRooms(true);
    try {
      const response = await fetch(`${API_BASE}/chat/rooms`, {
        method: 'GET',
        credentials: 'include',
        headers: { 'Content-Type': 'application/json' },
      });

      if (!response.ok) {
        throw new Error(`Failed to fetch rooms: ${response.status}`);
      }

      const data = await response.json();

      const roomsArray = Array.isArray(data) ? data : (data.rooms || []);
      setRooms(roomsArray);
    } catch (error) {
      console.error('Error fetching rooms:', error);
      toast({ title: 'Error', description: 'Failed to load chat rooms.', variant: 'destructive' });
    } finally {
      setIsLoadingRooms(false);
    }
  }, [toast]);

  const createRoom = useCallback(async (name: string, description?: string, password?: string): Promise<Room | null> => {
    try {
      const response = await fetch(`${API_BASE}/chat/rooms`, {
        method: 'POST',
        credentials: 'include',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ name, description: description || null, password: password || null })
      });

      if (!response.ok) {
        throw new Error('Failed to create room');
      }

      const newRoom = await response.json();
      setRooms(prev => [newRoom, ...prev]);

      toast({ title: 'Success', description: 'Room created successfully!' });

      return newRoom;
    } catch (error) {
      console.error('Error creating room:', error);
      toast({ title: 'Error', description: 'Failed to create room.', variant: 'destructive' });
      return null;
    }
  }, [toast]);

  const joinProtectedRoom = useCallback(async (roomId: string, password: string): Promise<boolean> => {
    try {
      const response = await fetch(`${API_BASE}/chat/rooms/${roomId}/verify-password`, {
        method: 'POST',
        credentials: 'include',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ password })
      });

      if (!response.ok) {
        throw new Error('Invalid password');
      }

      toast({ title: 'Success', description: 'Room joined successfully!' });

      return true;
    } catch (error) {
      console.error('Error joining protected room:', error);
      toast({ title: 'Error', description: 'Invalid password for protected room.', variant: 'destructive' });
      return false;
    }
  }, [toast]);

  const joinRoomByCode = useCallback(async (code: string, password?: string): Promise<Room | null> => {
    try {
      const response = await fetch(`${API_BASE}/chat/rooms/join-by-code`, {
        method: 'POST',
        credentials: 'include',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ room_code: code.trim().toUpperCase(), password: password || null })
      });

      if (!response.ok) {
        const errorData = await response.json().catch(() => null);
        const errorMessage = errorData?.message || 'Failed to join room';
        throw new Error(errorMessage);
      }

      const data = await response.json().catch(() => ({}));

      let room: any = null;
      if (data && typeof data === 'object') {
        if (data.room) {
          room = data.room;
        } else if (data.data && data.data.room) {
          room = data.data.room;
        } else if (data.id && data.name) {
          room = data;
        }
      }

      if (!room) {
        const msg = (data && (data.message || data.error)) || 'Failed to join room';
        throw new Error(msg);
      }

      setRooms(prev => {
        const exists = prev.some(r => r.id === room.id);
        return exists ? prev : [room, ...prev];
      });

      toast({ title: 'Success', description: (data && (data.message || data.info)) || `Joined room: ${room.name}` });

      return room as Room;
    } catch (error) {
      console.error('Error joining room by code:', error);
      toast({ title: 'Error', description: error instanceof Error ? error.message : 'Failed to join room. Check the code and try again.', variant: 'destructive' });
      return null;
    }
  }, [toast]);

  const deleteRoom = useCallback(async (roomId: string): Promise<boolean> => {
    try {
      const response = await fetch(`${API_BASE}/chat/rooms/${roomId}`, {
        method: 'DELETE',
        credentials: 'include',
        headers: { 'Content-Type': 'application/json' }
      });

      if (!response.ok) {
        throw new Error('Failed to delete room');
      }

      setRooms(prev => prev.filter(room => room.id !== roomId));

      if (selectedRoom?.id === roomId) {
        setSelectedRoom(null);
      }

      toast({ title: 'Success', description: 'Room deleted successfully.' });

      return true;
    } catch (error) {
      console.error('Error deleting room:', error);
      toast({ title: 'Error', description: 'Failed to delete room.', variant: 'destructive' });
      return false;
    }
  }, [toast, selectedRoom]);

  const leaveRoomMembership = useCallback(async (roomId: string): Promise<boolean> => {
    try {
      const response = await fetch(`${API_BASE}/chat/rooms/${roomId}/membership`, {
        method: 'DELETE',
        credentials: 'include',
        headers: { 'Content-Type': 'application/json' },
      });

      if (!response.ok) {
        // Try to surface backend error message
        let errorMessage = 'Failed to leave room';
        const ct = response.headers.get('content-type') || '';
        if (ct.includes('application/json')) {
          const data = await response.json().catch(() => null as any);
          if (data && (data.message || data.error)) {
            errorMessage = data.message || data.error;
          }
        } else {
          const txt = await response.text().catch(() => '');
          if (txt) errorMessage = txt;
        }
        throw new Error(errorMessage);
      }

      setRooms(prev => prev.filter(room => room.id !== roomId));

      if (selectedRoom?.id === roomId) {
        setSelectedRoom(null);
      }

      toast({ title: 'Success', description: 'You have left the room.' });

      return true;
    } catch (error) {
      console.error('Error leaving room membership:', error);
      const description = error instanceof Error ? error.message : 'Failed to leave the room.';
      toast({ title: 'Error', description, variant: 'destructive' });
      return false;
    }
  }, [toast, selectedRoom]);

  const updateRoomUserCount = useCallback((roomId: string, count: number) => {
    setRoomUsers(prev => ({ ...prev, [roomId]: count }));
    setRooms(prev => prev.map(room => room.id === roomId ? { ...room, user_count: count } : room));
  }, []);

  const selectRoom = useCallback((room: Room | null) => { setSelectedRoom(room); }, []);

  return {
    rooms,
    selectedRoom,
    isLoadingRooms,
    roomUsers,
    fetchRooms,
    createRoom,
    joinProtectedRoom,
    joinRoomByCode,
    deleteRoom,
    updateRoomUserCount,
    selectRoom,
    leaveRoomMembership,
  };
}