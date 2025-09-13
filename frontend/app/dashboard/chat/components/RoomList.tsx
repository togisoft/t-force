import React, { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger, DialogFooter } from '@/components/ui/dialog';
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from '@/components/ui/dropdown-menu';
import { Plus, Lock, Users, MoreVertical, Trash2, Hash, Loader2, Copy, Share2, MessageCircle, Crown, Shield, LogOut } from 'lucide-react';
import { useToast } from '@/hooks/use-toast';
import type { Room } from '../hooks/useRooms';
import {useWebSocket} from "../hooks/useWebSocket";
import { cn } from '@/lib/utils';

interface RoomListProps {
  rooms: Room[];
  selectedRoom: Room | null;
  userId: string;
  user: any;
  isLoadingRooms: boolean;
  onSelectRoom: (room: Room) => void;
  onCreateRoom: (name: string, description?: string, password?: string) => Promise<Room | null>;
  onJoinProtectedRoom: (roomId: string, password: string) => Promise<boolean>;
  onJoinRoomByCode: (code: string, password?: string) => Promise<Room | null>;
  onDeleteRoom: (roomId: string) => Promise<boolean>;
  onLeaveRoom: (roomId: string) => Promise<boolean>;
}

export function RoomList({
                           rooms,
                           selectedRoom,
                           userId,
                           user,
                           isLoadingRooms,
                           onSelectRoom,
                           onCreateRoom,
                           onJoinProtectedRoom,
                           onJoinRoomByCode,
                           onDeleteRoom,
                           onLeaveRoom,
                         }: RoomListProps) {
  const [isCreateDialogOpen, setIsCreateDialogOpen] = useState(false);
  const [isJoinDialogOpen, setIsJoinDialogOpen] = useState(false);
  const [isPasswordDialogOpen, setIsPasswordDialogOpen] = useState(false);
  const [newRoomName, setNewRoomName] = useState('');
  const [newRoomDescription, setNewRoomDescription] = useState('');
  const [newRoomPassword, setNewRoomPassword] = useState('');
  const [joinCode, setJoinCode] = useState('');
  const [joinPassword, setJoinPassword] = useState('');
  const [roomPassword, setRoomPassword] = useState('');
  const [pendingRoom, setPendingRoom] = useState<Room | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const { toast } = useToast();

  const { isConnected, joinRoom, leaveRoom } = useWebSocket({
    user,
  });

  const handleCreateRoom = async () => {
    if (!newRoomName.trim()) {
      toast({
        title: "Error",
        description: "Room name is required.",
        variant: "destructive",
      });
      return;
    }
    setIsSubmitting(true);
    try {
      const room = await onCreateRoom(newRoomName, newRoomDescription, newRoomPassword);
      if (room) {
        setIsCreateDialogOpen(false);
        setNewRoomName('');
        setNewRoomDescription('');
        setNewRoomPassword('');
        onSelectRoom(room);
      }
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleJoinByCode = async () => {
    if (!joinCode.trim()) {
      toast({
        title: "Error",
        description: "Room code is required.",
        variant: "destructive",
      });
      return;
    }
    setIsSubmitting(true);
    try {
      const room = await onJoinRoomByCode(joinCode, joinPassword);
      if (room) {
        setIsJoinDialogOpen(false);
        setJoinCode('');
        setJoinPassword('');
        onSelectRoom(room);
      }
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleRoomClick = (room: Room) => {
    if (room.is_protected && !room.is_owner) {
      setPendingRoom(room);
      setIsPasswordDialogOpen(true);
    } else {
      onSelectRoom(room);
      joinRoom(room.id);
    }
  };

  const handlePasswordSubmit = async () => {
    if (!pendingRoom || !roomPassword.trim()) {
      toast({
        title: "Error",
        description: "Password is required.",
        variant: "destructive",
      });
      return;
    }
    setIsSubmitting(true);
    try {
      const success = await onJoinProtectedRoom(pendingRoom.id, roomPassword);
      if (success) {
        setIsPasswordDialogOpen(false);
        setRoomPassword('');
        onSelectRoom(pendingRoom);
        joinRoom(pendingRoom.id);
        setPendingRoom(null);
      }
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleDeleteRoom = async (room: Room) => {
    if (confirm("Are you sure you want to leave this room? This will close the room for all members.")) {
      await onDeleteRoom(room.id);
      leaveRoom(room.id);
    }
  };

  const handleLeaveRoom = async (room: Room) => {
    // First leave via WebSocket
    leaveRoom(room.id);

    // Then remove membership via API and update UI state
    const ok = await onLeaveRoom(room.id);
    if (ok) {
      toast({
        title: "Left Room",
        description: `You have left "${room.name}".`,
      });
    } else {
      toast({
        title: "Error",
        description: `Could not leave "${room.name}". Please try again.`,
        variant: "destructive",
      });
    }
  };

  const handleCopyRoomCode = (roomCode: string) => {
    navigator.clipboard.writeText(roomCode);
    toast({
      title: "Copied!",
      description: "Room code copied to clipboard.",
    });
  };

  const handleShareRoomCode = async (roomCode: string) => {
    if (navigator.share) {
      try {
        await navigator.share({
          title: "Join my room",
          text: `Room code: ${roomCode}`,
        });
      } catch (error) {
        toast({
          title: "Error",
          description: "Failed to share room code.",
          variant: "destructive",
        });
      }
    } else {
      toast({
        title: "Not supported",
        description: "Your browser does not support sharing.",
      });
    }
  };

  return (
      <div className="h-full flex flex-col bg-gradient-to-br from-background via-accent/5 to-muted/5">
        {/* Header */}
        <div className="p-3 sm:p-4 border-b border-border/50 bg-card/95 backdrop-blur-sm flex-shrink-0 shadow-sm">
          <div className="flex items-center justify-between mb-3">
            <h2 className="text-lg font-bold text-foreground">Rooms</h2>
            <div className="flex space-x-1 sm:space-x-2">
              <Dialog open={isJoinDialogOpen} onOpenChange={setIsJoinDialogOpen}>
                <DialogTrigger asChild>
                  <Button variant="outline" size="sm" className="h-8 px-2 sm:px-3 bg-gradient-to-r from-primary/10 to-chart-2/10 border-primary/20 hover:from-primary/20 hover:to-chart-2/20 transition-all duration-200">
                    <Hash className="h-4 w-4 sm:mr-1" />
                    <span className="hidden sm:inline">Join</span>
                  </Button>
                </DialogTrigger>
                <DialogContent className="sm:max-w-md">
                  <DialogHeader>
                    <DialogTitle className="text-lg">Join Room by Code</DialogTitle>
                  </DialogHeader>
                  <div className="space-y-4">
                    <div>
                      <Label htmlFor="join-code" className="text-sm font-medium">Room Code</Label>
                      <Input
                          id="join-code"
                          placeholder="Enter room code"
                          value={joinCode}
                          onChange={(e) => setJoinCode(e.target.value)}
                          className="mt-1"
                      />
                    </div>
                    <div>
                      <Label htmlFor="join-password" className="text-sm font-medium">Password (if required)</Label>
                      <Input
                          id="join-password"
                          type="password"
                          placeholder="Enter password"
                          value={joinPassword}
                          onChange={(e) => setJoinPassword(e.target.value)}
                          className="mt-1"
                      />
                    </div>
                  </div>
                  <DialogFooter>
                    <Button variant="outline" onClick={() => setIsJoinDialogOpen(false)}>
                      Cancel
                    </Button>
                    <Button onClick={handleJoinByCode} disabled={isSubmitting} className="bg-gradient-to-r from-primary to-chart-2 hover:from-primary/90 hover:to-chart-2/90 shadow-md hover:shadow-lg transition-all duration-200">
                      {isSubmitting ? (
                          <>
                            <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                            Joining...
                          </>
                      ) : (
                          'Join Room'
                      )}
                    </Button>
                  </DialogFooter>
                </DialogContent>
              </Dialog>
              <Dialog open={isCreateDialogOpen} onOpenChange={setIsCreateDialogOpen}>
                <DialogTrigger asChild>
                  <Button size="sm" className="h-8 px-2 sm:px-3 bg-gradient-to-r from-chart-3 to-chart-4 hover:from-chart-3/90 hover:to-chart-4/90">
                    <Plus className="h-4 w-4 sm:mr-1" />
                    <span className="hidden sm:inline">Create</span>
                  </Button>
                </DialogTrigger>
                <DialogContent className="sm:max-w-md">
                  <DialogHeader>
                    <DialogTitle className="text-lg">Create New Room</DialogTitle>
                  </DialogHeader>
                  <div className="space-y-4">
                    <div>
                      <Label htmlFor="room-name" className="text-sm font-medium">Room Name</Label>
                      <Input
                          id="room-name"
                          placeholder="Enter room name"
                          value={newRoomName}
                          onChange={(e) => setNewRoomName(e.target.value)}
                          className="mt-1"
                      />
                    </div>
                    <div>
                      <Label htmlFor="room-description" className="text-sm font-medium">Description (Optional)</Label>
                      <Textarea
                          id="room-description"
                          placeholder="Enter room description"
                          value={newRoomDescription}
                          onChange={(e) => setNewRoomDescription(e.target.value)}
                          className="mt-1"
                          rows={3}
                      />
                    </div>
                    <div>
                      <Label htmlFor="room-password" className="text-sm font-medium">Password (Optional)</Label>
                      <Input
                          id="room-password"
                          type="password"
                          placeholder="Set room password"
                          value={newRoomPassword}
                          onChange={(e) => setNewRoomPassword(e.target.value)}
                          className="mt-1"
                      />
                    </div>
                  </div>
                  <DialogFooter>
                    <Button variant="outline" onClick={() => setIsCreateDialogOpen(false)}>
                      Cancel
                    </Button>
                    <Button onClick={handleCreateRoom} disabled={isSubmitting} className="bg-gradient-to-r from-chart-3 to-chart-4 hover:from-chart-3/90 hover:to-chart-4/90 shadow-md hover:shadow-lg transition-all duration-200">
                      {isSubmitting ? (
                          <>
                            <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                            Creating...
                          </>
                      ) : (
                          'Create Room'
                      )}
                    </Button>
                  </DialogFooter>
                </DialogContent>
              </Dialog>
            </div>
          </div>
        </div>

        {/* Room List */}
        <div className="flex-1 overflow-hidden bg-gradient-to-br from-background via-accent/5 to-muted/5">
          <ScrollArea className="h-full">
            <div className="p-2 sm:p-3">
              {isLoadingRooms ? (
                  <div className="space-y-3">
                    {[...Array(5)].map((_, i) => (
                        <div key={i} className="p-4 rounded-xl animate-pulse bg-muted">
                          <div className="flex items-center space-x-3">
                            <div className="w-10 h-10 bg-accent rounded-full"></div>
                            <div className="flex-1">
                              <div className="h-4 bg-accent rounded w-3/4 mb-2"></div>
                              <div className="h-3 bg-accent rounded w-1/2"></div>
                            </div>
                          </div>
                        </div>
                    ))}
                  </div>
              ) : rooms.length === 0 ? (
                  <div className="text-center py-12">
                    <div className="w-20 h-20 mx-auto mb-6 bg-gradient-to-br from-accent to-muted rounded-full flex items-center justify-center shadow-xl">
                      <MessageCircle className="h-10 w-10 text-primary" />
                    </div>
                    <h3 className="text-xl font-bold text-foreground mb-3">No rooms yet</h3>
                    <p className="text-muted-foreground mb-6 text-lg">Create your first room to start chatting</p>
                    <Button 
                      onClick={() => setIsCreateDialogOpen(true)}
                      className="bg-gradient-to-r from-primary to-chart-2 hover:from-primary/90 hover:to-chart-2/90 shadow-lg hover:shadow-xl transition-all duration-200 hover:scale-105 button-glow"
                    >
                      <Plus className="h-4 w-4 mr-2" />
                      Create Room
                    </Button>
                  </div>
              ) : (
                  <div className="space-y-2">
                    {rooms.map((room) => (
                        <div
                            key={room.id}
                            className={cn(
                                "group p-3 sm:p-4 rounded-xl border cursor-pointer transition-all duration-300 hover:shadow-lg hover:scale-[1.02] backdrop-blur-sm",
                                selectedRoom?.id === room.id 
                                  ? "bg-gradient-to-r from-primary/10 to-chart-2/10 border-primary/30 shadow-md" 
                                  : "bg-card/90 border-border/50 hover:bg-card hover:shadow-xl"
                            )}
                            onClick={() => handleRoomClick(room)}
                        >
                          <div className="flex items-start space-x-3">
                            {/* Room Icon */}
                            <div className={cn(
                                "w-10 h-10 rounded-full flex items-center justify-center flex-shrink-0 shadow-md transition-all duration-200",
                                selectedRoom?.id === room.id
                                  ? "bg-gradient-to-r from-primary to-chart-2 text-primary-foreground shadow-lg"
                                  : "bg-gradient-to-r from-muted to-accent text-muted-foreground"
                            )}>
                              {room.is_protected ? (
                                  <Shield className="h-5 w-5" />
                              ) : (
                                  <MessageCircle className="h-5 w-5" />
                              )}
                            </div>

                            {/* Room Info */}
                            <div className="flex-1 min-w-0">
                              <div className="flex items-center justify-between mb-1">
                                <div className="flex items-center space-x-2 flex-1 min-w-0">
                                  <h3 className="font-semibold text-foreground truncate">
                                    {room.name}
                                  </h3>
                                  {room.is_protected && (
                                      <Lock className="h-4 w-4 text-chart-4 flex-shrink-0" />
                                  )}
                                  {room.is_owner && (
                                      <Badge variant="outline" className="text-xs px-2 py-0 bg-chart-3/10 text-chart-3 border-chart-3/20">
                                        <Crown className="h-3 w-3 mr-1" />
                                        Owner
                                      </Badge>
                                  )}
                                </div>
                                <DropdownMenu>
                                  <DropdownMenuTrigger asChild>
                                    <Button
                                        variant="ghost"
                                        size="sm"
                                        className="h-6 w-6 p-0 opacity-0 group-hover:opacity-100 transition-opacity"
                                        onClick={(e) => e.stopPropagation()}
                                    >
                                      <MoreVertical className="h-3 w-3" />
                                    </Button>
                                  </DropdownMenuTrigger>
                                  <DropdownMenuContent align="end">
                                    <DropdownMenuItem
                                        onClick={(e) => {
                                          e.stopPropagation();
                                          room.is_owner ? handleDeleteRoom(room) : handleLeaveRoom(room);
                                        }}
                                        className={room.is_owner ? "text-destructive" : ""}
                                    >
                                      <LogOut className="h-4 w-4 mr-2" />
                                      Leave Room
                                    </DropdownMenuItem>
                                  </DropdownMenuContent>
                                </DropdownMenu>
                              </div>
                              
                              {room.description && (
                                  <p className="text-sm text-muted-foreground mb-2 line-clamp-2">
                                    {room.description}
                                  </p>
                              )}
                              
                              <div className="flex items-center justify-between">
                                <div className="flex items-center space-x-2">
                                  <span className="text-xs text-muted-foreground font-mono">
                                    {room.room_code}
                                  </span>
                                  <div className="flex space-x-1 opacity-0 group-hover:opacity-100 transition-opacity">
                                    <Button
                                        variant="ghost"
                                        size="sm"
                                        className="h-6 w-6 p-0 hover:bg-accent"
                                        onClick={(e) => {
                                          e.stopPropagation();
                                          handleCopyRoomCode(room.room_code);
                                        }}
                                    >
                                      <Copy className="h-3 w-3" />
                                    </Button>
                                    <Button
                                        variant="ghost"
                                        size="sm"
                                        className="h-6 w-6 p-0 hover:bg-accent"
                                        onClick={(e) => {
                                          e.stopPropagation();
                                          handleShareRoomCode(room.room_code);
                                        }}
                                    >
                                      <Share2 className="h-3 w-3" />
                                    </Button>
                                  </div>
                                </div>
                                <div className="flex items-center space-x-1 text-xs text-muted-foreground">
                                  <Users className="h-3 w-3" />
                                  <span>{room.user_count || 0}</span>
                                </div>
                              </div>
                            </div>
                          </div>
                        </div>
                    ))}
                  </div>
              )}
            </div>
          </ScrollArea>
        </div>

        {/* Password Dialog */}
        <Dialog open={isPasswordDialogOpen} onOpenChange={setIsPasswordDialogOpen}>
          <DialogContent className="sm:max-w-md">
            <DialogHeader>
              <DialogTitle className="text-lg">Enter Room Password</DialogTitle>
            </DialogHeader>
            <div>
              <Label htmlFor="room-password-input" className="text-sm font-medium">Password</Label>
              <Input
                  id="room-password-input"
                  type="password"
                  placeholder="Enter room password"
                  value={roomPassword}
                  onChange={(e) => setRoomPassword(e.target.value)}
                  onKeyPress={(e) => e.key === 'Enter' && handlePasswordSubmit()}
                  className="mt-1"
              />
            </div>
            <DialogFooter>
              <Button
                  variant="outline"
                  onClick={() => {
                    setIsPasswordDialogOpen(false);
                    setRoomPassword('');
                    setPendingRoom(null);
                  }}
              >
                Cancel
              </Button>
              <Button onClick={handlePasswordSubmit} disabled={isSubmitting} className="bg-gradient-to-r from-blue-500 to-purple-600 hover:from-blue-600 hover:to-purple-700 shadow-md hover:shadow-lg transition-all duration-200">
                {isSubmitting ? (
                    <>
                      <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                      Joining...
                    </>
                ) : (
                    'Join Room'
                )}
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      </div>
  );
}
