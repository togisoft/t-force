'use client';

import {useAuth, useProtectedRoute, User} from '@/lib/auth';
import {useEffect, useState, useRef} from 'react';
import {Button} from '@/components/ui/button';
import {Card, CardContent, CardDescription, CardHeader, CardTitle} from '@/components/ui/card';
import {Avatar, AvatarFallback, AvatarImage} from '@/components/ui/avatar';
import {Tabs, TabsContent, TabsList, TabsTrigger} from '@/components/ui/tabs';
import {Input} from '@/components/ui/input';
import {Label} from '@/components/ui/label';
import {
    Loader2,
    Camera,
    RefreshCw,
    Download,
    Info,
    ShieldCheck,
    ShieldOff,
    UserCheck,
    UserX,
    AlertCircle,
    UserCog,
    Trash,
    Save,
    Search,
    Filter,
    MoreHorizontal,
    Eye,
    EyeOff,
    Copy,
    CheckCircle,
    Users,
    Activity,
    Shield,
    Settings,
    User as UserIcon,
    Crown,
    Sparkles,
    MessageSquare
} from 'lucide-react';
import {useToast} from '@/hooks/use-toast';
import {SessionList} from '@/components/sessions/session-list';
import {Alert, AlertDescription, AlertTitle} from '@/components/ui/alert';
import {
    Dialog,
    DialogContent,
    DialogHeader,
    DialogTitle,
    DialogDescription,
    DialogFooter
} from '@/components/ui/dialog';
import {Table, TableBody, TableCell, TableHead, TableHeader, TableRow} from '@/components/ui/table';
import {Select, SelectContent, SelectItem, SelectTrigger, SelectValue} from '@/components/ui/select';
import {Badge} from '@/components/ui/badge';
import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuLabel,
    DropdownMenuSeparator,
    DropdownMenuTrigger
} from '@/components/ui/dropdown-menu';
import {Tooltip, TooltipContent, TooltipProvider, TooltipTrigger} from '@/components/ui/tooltip';

export default function DashboardPage() {
    const {user, isLoading, token, refreshUser} = useAuth();
    useProtectedRoute();

    const [activeTab, setActiveTab] = useState('overview');
    const [isUploading, setIsUploading] = useState(false);
    const [searchTerm, setSearchTerm] = useState('');
    const [filterRole, setFilterRole] = useState('all');
    const [filterStatus, setFilterStatus] = useState('all');
    const fileInputRef = useRef<HTMLInputElement>(null);
    const {toast} = useToast();

    // Users management state
    const [users, setUsers] = useState<User[]>([]);
    const [loadingUsers, setLoadingUsers] = useState(true);
    const [usersError, setUsersError] = useState<string | null>(null);
    const [selectedUser, setSelectedUser] = useState<User | null>(null);
    const [isRoleDialogOpen, setIsRoleDialogOpen] = useState(false);
    const [isDeleteDialogOpen, setIsDeleteDialogOpen] = useState(false);
    const [isStatusDialogOpen, setIsStatusDialogOpen] = useState(false);
    const [newRole, setNewRole] = useState<string>('');
    const [newStatus, setNewStatus] = useState<boolean>(true);
    const [isSubmitting, setIsSubmitting] = useState(false);

    // 2FA states
    const [twoFactorStatus, setTwoFactorStatus] = useState<{ enabled: boolean }>({enabled: false});
    const [twoFactorSetup, setTwoFactorSetup] = useState<{ secret: string; qr_code_url: string } | null>(null);
    const [verificationCodeForEnable, setVerificationCodeForEnable] = useState('');
    const [disable2FACode, setDisable2FACode] = useState('');
    const [regenerate2FACode, setRegenerate2FACode] = useState('');
    const [isLoading2FA, setIsLoading2FA] = useState(true);
    const [isVerifying, setIsVerifying] = useState(false);
    const [isDisabling2FA, setIsDisabling2FA] = useState(false);
    const [isRegenerating, setIsRegenerating] = useState(false);
    const [backupCodes, setBackupCodes] = useState<string[]>([]);
    const [showingInitialBackupCodes, setShowingInitialBackupCodes] = useState(false);
    const [showBackupCodes, setShowBackupCodes] = useState(false);
    const [copiedCodes, setCopiedCodes] = useState<Set<string>>(new Set());

    // Statistics for overview
    const [stats, setStats] = useState({
        totalUsers: 0,
        activeUsers: 0,
        adminUsers: 0,
        recentLogins: 0
    });

    useEffect(() => {
        if (user) {
            check2FAStatus();
            if (user.role?.toLowerCase() === 'admin') {
                fetchUsers();
            }
        }
    }, [user]);

    // Filter users based on search and filters
    const filteredUsers = users.filter(userItem => {
        const matchesSearch = userItem.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
            userItem.email.toLowerCase().includes(searchTerm.toLowerCase());
        const matchesRole = filterRole === 'all' || userItem.role.toLowerCase() === filterRole;
        const matchesStatus = filterStatus === 'all' ||
            (filterStatus === 'active' && userItem.is_active) ||
            (filterStatus === 'inactive' && !userItem.is_active);

        return matchesSearch && matchesRole && matchesStatus;
    });

    // Calculate stats when users change
    useEffect(() => {
        if (users.length > 0) {
            setStats({
                totalUsers: users.length,
                activeUsers: users.filter(u => u.is_active).length,
                adminUsers: users.filter(u => u.role.toLowerCase() === 'admin').length,
                recentLogins: users.filter(u => u.is_active).length // This would be better with actual login data
            });
        }
    }, [users]);

    const fetchUsers = async () => {
        try {
            setLoadingUsers(true);
            const response = await fetch(`/api/admin/users`, {
                headers: token ? {'Authorization': `Bearer ${token}`} : {},
                credentials: 'include',
            });

            if (!response.ok) {
                throw new Error('Failed to fetch users');
            }

            const data = await response.json();
            setUsers(data);
        } catch (err) {
            console.error('Error fetching users:', err);
            setUsersError('Failed to load users. Please try again later.');
        } finally {
            setLoadingUsers(false);
        }
    };

    const handleRoleChange = async () => {
        if (!selectedUser || !newRole) return;

        try {
            setIsSubmitting(true);
            const response = await fetch(`/api/admin/users/${selectedUser.id}/role`, {
                method: 'PUT',
                headers: {
                    ...(token ? {'Authorization': `Bearer ${token}`} : {}),
                    'Content-Type': 'application/json'
                },
                credentials: 'include',
                body: JSON.stringify({role: newRole})
            });

            if (!response.ok) {
                const errorData = await response.json();
                throw new Error(errorData.message || 'Failed to update role');
            }

            const updatedUser = await response.json();
            setUsers(users.map(user => user.id === updatedUser.id ? updatedUser : user));

            toast({
                title: "✅ Role Updated",
                description: `${selectedUser.name}'s role has been changed to ${newRole}.`,
            });

            setIsRoleDialogOpen(false);
        } catch (err: any) {
            console.error('Error updating role:', err);
            toast({
                variant: "destructive",
                title: "❌ Error",
                description: err.message || "Failed to update role. Please try again.",
            });
        } finally {
            setIsSubmitting(false);
        }
    };

    const handleDeleteUser = async () => {
        if (!selectedUser) return;

        try {
            setIsSubmitting(true);
            const response = await fetch(`/api/admin/users/${selectedUser.id}`, {
                method: 'DELETE',
                headers: token ? {'Authorization': `Bearer ${token}`} : {},
                credentials: 'include',
            });

            if (!response.ok) {
                const errorData = await response.json();
                throw new Error(errorData.message || 'Failed to delete user');
            }

            setUsers(users.filter(user => user.id !== selectedUser.id));
            toast({
                title: "🗑️ User Deleted",
                description: `${selectedUser.name} has been deleted.`,
            });

            setIsDeleteDialogOpen(false);
        } catch (err: any) {
            console.error('Error deleting user:', err);
            toast({
                variant: "destructive",
                title: "❌ Error",
                description: err.message || "Failed to delete user. Please try again.",
            });
        } finally {
            setIsSubmitting(false);
        }
    };

    const handleStatusChange = async () => {
        if (!selectedUser) return;

        try {
            setIsSubmitting(true);
            const response = await fetch(`/api/admin/users/${selectedUser.id}/status`, {
                method: 'PUT',
                headers: {
                    ...(token ? {'Authorization': `Bearer ${token}`} : {}),
                    'Content-Type': 'application/json'
                },
                credentials: 'include',
                body: JSON.stringify({is_active: newStatus})
            });

            if (!response.ok) {
                const errorData = await response.json();
                throw new Error(errorData.message || 'Failed to update status');
            }

            const updatedUser = await response.json();
            setUsers(users.map(user => user.id === updatedUser.id ? updatedUser : user));

            toast({
                title: "✅ Status Updated",
                description: `${selectedUser.name} has been ${newStatus ? 'activated' : 'deactivated'}.`,
            });

            setIsStatusDialogOpen(false);
        } catch (err: any) {
            console.error('Error updating status:', err);
            toast({
                variant: "destructive",
                title: "❌ Error",
                description: err.message || "Failed to update status. Please try again.",
            });
        } finally {
            setIsSubmitting(false);
        }
    };

    const openRoleDialog = (user: User) => {
        setSelectedUser(user);
        setNewRole(user.role);
        setIsRoleDialogOpen(true);
    };

    const openDeleteDialog = (user: User) => {
        setSelectedUser(user);
        setIsDeleteDialogOpen(true);
    };

    const openStatusDialog = (user: User) => {
        setSelectedUser(user);
        setNewStatus(!user.is_active);
        setIsStatusDialogOpen(true);
    };

    const apiFetch = async (endpoint: string, options: RequestInit = {}) => {
        const response = await fetch(`${endpoint}`, {
            ...options,
            credentials: 'include',
            headers: {
                ...(token ? {'Authorization': `Bearer ${token}`} : {}),
                'Content-Type': 'application/json',
                ...options.headers,
            },
        });

        if (!response.ok) {
            const errorData = await response.json();
            throw new Error(errorData.message || 'An error occurred');
        }
        return response.json();
    };

    const check2FAStatus = async () => {
        setIsLoading2FA(true);
        try {
            const data = await apiFetch('/api/auth/2fa/status');
            setTwoFactorStatus(data);
            if (data.enabled) setTwoFactorSetup(null);
        } catch (error) {
            toast({title: "❌ Error", description: "Failed to check 2FA status.", variant: "destructive"});
        } finally {
            setIsLoading2FA(false);
        }
    };

    const setup2FA = async () => {
        setIsLoading2FA(true);
        try {
            const data = await apiFetch('/api/auth/2fa/setup');
            setTwoFactorSetup(data);
        } catch (error) {
            toast({title: "❌ Error", description: "Failed to start 2FA setup.", variant: "destructive"});
        } finally {
            setIsLoading2FA(false);
        }
    };

    const verify2FA = async () => {
        if (!verificationCodeForEnable) return;
        setIsVerifying(true);
        try {
            const data = await apiFetch('/api/auth/2fa/verify', {
                method: 'POST',
                body: JSON.stringify({code: verificationCodeForEnable}),
            });
            if (data.success) {
                toast({title: "✅ Success", description: "Two-factor authentication has been enabled."});
                await check2FAStatus();
                const backupData = await apiFetch('/api/auth/2fa/backup-codes');
                setBackupCodes(backupData.backup_codes);
                setShowingInitialBackupCodes(true);
                setVerificationCodeForEnable('');
            }
        } catch (error: any) {
            toast({title: "❌ Error", description: error.message, variant: "destructive"});
        } finally {
            setIsVerifying(false);
        }
    };

    const disable2FA = async () => {
        if (!disable2FACode) return;
        setIsDisabling2FA(true);
        try {
            await apiFetch('/api/auth/2fa/disable', {
                method: 'POST',
                body: JSON.stringify({code: disable2FACode}),
            });
            toast({title: "✅ Success", description: "Two-factor authentication has been disabled."});
            await check2FAStatus();
            setDisable2FACode('');
            setTwoFactorSetup(null);
            setBackupCodes([]);
        } catch (error: any) {
            toast({title: "❌ Error", description: error.message, variant: "destructive"});
        } finally {
            setIsDisabling2FA(false);
        }
    };

    const regenerateBackupCodes = async () => {
        if (!regenerate2FACode) return;
        setIsRegenerating(true);
        try {
            const data = await apiFetch('/api/auth/2fa/regenerate-backup-codes', {
                method: 'POST',
                body: JSON.stringify({code: regenerate2FACode}),
            });
            setBackupCodes(data.backup_codes);
            toast({title: "✅ Success", description: "New backup codes have been generated. Please save them."});
            setRegenerate2FACode('');
            setShowingInitialBackupCodes(true);
        } catch (error: any) {
            toast({title: "❌ Error", description: error.message, variant: "destructive"});
        } finally {
            setIsRegenerating(false);
        }
    };

    const copyBackupCode = async (code: string) => {
        await navigator.clipboard.writeText(code);
        // @ts-ignore
        setCopiedCodes(prev => new Set([...prev, code]));
        toast({title: "📋 Copied", description: "Backup code copied to clipboard."});
        setTimeout(() => setCopiedCodes(prev => {
            const newSet = new Set(prev);
            newSet.delete(code);
            return newSet;
        }), 2000);
    };

    const downloadBackupCodes = () => {
        const content = "T-Force Backup Codes\n=====================\n\n" + backupCodes.join('\n');
        const blob = new Blob([content], {type: 'text/plain'});
        const url = URL.createObjectURL(blob);
        const link = document.createElement('a');
        link.href = url;
        link.download = 'tforce-backup-codes.txt';
        document.body.appendChild(link);
        link.click();
        document.body.removeChild(link);
        URL.revokeObjectURL(url);
    };

    const handleProfilePictureUpload = async (file: File) => {
        setIsUploading(true);
        try {
            const formData = new FormData();
            formData.append('file', file);
            const response = await fetch(`/api/user/profile/upload`, {
                method: 'POST',
                headers: token ? {'Authorization': `Bearer ${token}`} : {},
                credentials: 'include',
                body: formData,
            });

            if (!response.ok) {
                const errorData = await response.json();
                throw new Error(errorData.message || 'Upload failed');
            }

            await refreshUser();
            toast({title: "✅ Success", description: "Profile picture has been updated."});
        } catch (error: any) {
            toast({title: "❌ Upload Failed", description: error.message, variant: "destructive"});
        } finally {
            setIsUploading(false);
        }
    };

    const triggerFileInput = () => fileInputRef.current?.click();
    const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        const file = e.target.files?.[0];
        if (file) handleProfilePictureUpload(file);
    };

    const getUserInitials = (name: string) => {
        if (!name) return '?';
        return name.split(' ').map(p => p[0]).join('').toUpperCase().substring(0, 2);
    };

    const getRoleIcon = (role: string) => {
        switch (role.toLowerCase()) {
            case 'admin':
                return <Crown className="h-4 w-4 text-amber-500"/>;
            case 'moderator':
                return <Shield className="h-4 w-4 text-blue-500"/>;
            default:
                return <UserIcon className="h-4 w-4 text-gray-500"/>;
        }
    };

    const getProviderColor = (provider: string) => {
        switch (provider.toLowerCase()) {
            case 'google':
                return 'bg-red-50 text-red-700 border-red-200';
            case 'github':
                return 'bg-gray-50 text-gray-700 border-gray-200';
            case 'local':
                return 'bg-blue-50 text-blue-700 border-blue-200';
            default:
                return 'bg-gray-50 text-gray-700 border-gray-200';
        }
    };

    if (isLoading || !user) {
        return (
            <div className="flex h-[calc(100vh-4rem)] items-center justify-center">
                <div className="text-center space-y-4">
                    <Loader2 className="h-10 w-10 animate-spin text-primary mx-auto"/>
                    <p className="text-muted-foreground">Loading your dashboard...</p>
                </div>
            </div>
        );
    }

    return (
        <TooltipProvider>
            <div className="max-w-7xl mx-auto space-y-8 p-6">
                {/* Header Section */}
                <div className="space-y-2">
                    <div className="flex items-center gap-3">
                        <div className="p-2 bg-primary/10 rounded-lg">
                            <Sparkles className="h-6 w-6 text-primary"/>
                        </div>
                        <div>
                            <h1 className="text-4xl font-bold tracking-tight bg-gradient-to-r from-gray-900 to-gray-600 dark:from-gray-100 dark:to-gray-400 bg-clip-text text-transparent">
                                Dashboard
                            </h1>
                            <p className="text-muted-foreground text-lg">
                                Welcome back, <span className="font-medium text-foreground">{user.name}</span>!
                                Here's what's happening with your account.
                            </p>
                        </div>
                    </div>
                </div>

                <Tabs value={activeTab} onValueChange={setActiveTab} className="w-full">
                    <div className="border-b">
                        <TabsList className="grid w-full max-w-2xl grid-cols-6 h-12 bg-muted/50">
                            <TabsTrigger value="overview" className="flex items-center gap-2">
                                <Activity className="h-4 w-4"/>
                                <span className="hidden sm:inline">Overview</span>
                            </TabsTrigger>
                            <TabsTrigger value="profile" className="flex items-center gap-2">
                                <UserIcon className="h-4 w-4"/>
                                <span className="hidden sm:inline">Profile</span>
                            </TabsTrigger>
                            <TabsTrigger value="settings" className="flex items-center gap-2">
                                <Settings className="h-4 w-4"/>
                                <span className="hidden sm:inline">Security</span>
                            </TabsTrigger>
                            <TabsTrigger value="chat" className="flex items-center gap-2"
                                         onClick={() => window.location.href = '/dashboard/chat'}>
                                <MessageSquare className="h-4 w-4"/>
                                <span className="hidden sm:inline">Chat</span>
                            </TabsTrigger>
                            {user?.role?.toLowerCase() === 'admin' && (
                                <TabsTrigger value="users" className="flex items-center gap-2">
                                    <Users className="h-4 w-4"/>
                                    <span className="hidden sm:inline">Users</span>
                                </TabsTrigger>
                            )}
                        </TabsList>
                    </div>

                    {/* Overview Tab */}
                    <TabsContent value="overview" className="mt-8">
                        <div className="grid gap-6">
                            {/* Stats Cards */}
                            <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
                                <Card className="relative overflow-hidden">
                                    <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                                        <CardTitle className="text-sm font-medium">Account Status</CardTitle>
                                        <div
                                            className="h-8 w-8 bg-green-500/10 rounded-full flex items-center justify-center">
                                            <ShieldCheck className="h-4 w-4 text-green-500"/>
                                        </div>
                                    </CardHeader>
                                    <CardContent>
                                        <div className="text-2xl font-bold text-green-500">Active</div>
                                        <p className="text-xs text-muted-foreground mt-1">
                                            Signed in via {user.provider}
                                        </p>
                                        <div
                                            className="absolute inset-0 bg-gradient-to-br from-green-50/50 to-transparent dark:from-green-950/20"/>
                                    </CardContent>
                                </Card>

                                <Card className="relative overflow-hidden">
                                    <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                                        <CardTitle className="text-sm font-medium">Two-Factor Auth</CardTitle>
                                        <div className={`h-8 w-8 rounded-full flex items-center justify-center ${
                                            twoFactorStatus.enabled
                                                ? 'bg-green-500/10'
                                                : 'bg-amber-500/10'
                                        }`}>
                                            {twoFactorStatus.enabled
                                                ? <ShieldCheck className="h-4 w-4 text-green-500"/>
                                                : <ShieldOff className="h-4 w-4 text-amber-500"/>
                                            }
                                        </div>
                                    </CardHeader>
                                    <CardContent>
                                        <div className={`text-2xl font-bold ${
                                            twoFactorStatus.enabled ? 'text-green-500' : 'text-amber-500'
                                        }`}>
                                            {twoFactorStatus.enabled ? 'Enabled' : 'Disabled'}
                                        </div>
                                        <p className="text-xs text-muted-foreground mt-1">
                                            {twoFactorStatus.enabled
                                                ? 'Your account is secure'
                                                : 'Enable for extra security'
                                            }
                                        </p>
                                        {!twoFactorStatus.enabled && (
                                            <Button
                                                variant="link"
                                                size="sm"
                                                className="p-0 h-auto text-xs mt-1"
                                                onClick={() => setActiveTab('settings')}
                                            >
                                                Enable Now →
                                            </Button>
                                        )}
                                        <div className={`absolute inset-0 bg-gradient-to-br ${
                                            twoFactorStatus.enabled
                                                ? 'from-green-50/50'
                                                : 'from-amber-50/50'
                                        } to-transparent dark:opacity-20`}/>
                                    </CardContent>
                                </Card>

                                {user?.role?.toLowerCase() === 'admin' && (
                                    <>
                                        <Card className="relative overflow-hidden">
                                            <CardHeader
                                                className="flex flex-row items-center justify-between space-y-0 pb-2">
                                                <CardTitle className="text-sm font-medium">Total Users</CardTitle>
                                                <div
                                                    className="h-8 w-8 bg-blue-500/10 rounded-full flex items-center justify-center">
                                                    <Users className="h-4 w-4 text-blue-500"/>
                                                </div>
                                            </CardHeader>
                                            <CardContent>
                                                <div
                                                    className="text-2xl font-bold text-blue-500">{stats.totalUsers}</div>
                                                <p className="text-xs text-muted-foreground mt-1">
                                                    {stats.activeUsers} active users
                                                </p>
                                                <div
                                                    className="absolute inset-0 bg-gradient-to-br from-blue-50/50 to-transparent dark:from-blue-950/20"/>
                                            </CardContent>
                                        </Card>

                                        <Card className="relative overflow-hidden">
                                            <CardHeader
                                                className="flex flex-row items-center justify-between space-y-0 pb-2">
                                                <CardTitle className="text-sm font-medium">Admin Users</CardTitle>
                                                <div
                                                    className="h-8 w-8 bg-purple-500/10 rounded-full flex items-center justify-center">
                                                    <Crown className="h-4 w-4 text-purple-500"/>
                                                </div>
                                            </CardHeader>
                                            <CardContent>
                                                <div
                                                    className="text-2xl font-bold text-purple-500">{stats.adminUsers}</div>
                                                <p className="text-xs text-muted-foreground mt-1">
                                                    Administrative accounts
                                                </p>
                                                <div
                                                    className="absolute inset-0 bg-gradient-to-br from-purple-50/50 to-transparent dark:from-purple-950/20"/>
                                            </CardContent>
                                        </Card>
                                    </>
                                )}
                            </div>

                            {/* Quick Actions */}
                            <Card>
                                <CardHeader>
                                    <CardTitle className="flex items-center gap-2">
                                        <Activity className="h-5 w-5"/>
                                        Quick Actions
                                    </CardTitle>
                                    <CardDescription>
                                        Commonly used features for managing your account
                                    </CardDescription>
                                </CardHeader>
                                <CardContent>
                                    <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
                                        <Button
                                            variant="outline"
                                            className="h-auto p-4 flex flex-col items-center gap-2"
                                            onClick={() => setActiveTab('profile')}
                                        >
                                            <Camera className="h-6 w-6"/>
                                            <div className="text-center">
                                                <div className="font-medium">Update Profile</div>
                                                <div className="text-xs text-muted-foreground">Change photo & info</div>
                                            </div>
                                        </Button>

                                        <Button
                                            variant="outline"
                                            className="h-auto p-4 flex flex-col items-center gap-2"
                                            onClick={() => setActiveTab('settings')}
                                        >
                                            <Shield className="h-6 w-6"/>
                                            <div className="text-center">
                                                <div className="font-medium">Security Settings</div>
                                                <div className="text-xs text-muted-foreground">2FA & sessions</div>
                                            </div>
                                        </Button>

                                        {user?.role?.toLowerCase() === 'admin' && (
                                            <Button
                                                variant="outline"
                                                className="h-auto p-4 flex flex-col items-center gap-2"
                                                onClick={() => setActiveTab('users')}
                                            >
                                                <Users className="h-6 w-6"/>
                                                <div className="text-center">
                                                    <div className="font-medium">Manage Users</div>
                                                    <div className="text-xs text-muted-foreground">User administration
                                                    </div>
                                                </div>
                                            </Button>
                                        )}
                                    </div>
                                </CardContent>
                            </Card>
                        </div>
                    </TabsContent>

                    {/* Profile Tab */}
                    <TabsContent value="profile" className="mt-8">
                        <div className="grid gap-6 lg:grid-cols-3">
                            <div className="lg:col-span-2 space-y-6">
                                <Card>
                                    <CardHeader>
                                        <CardTitle className="flex items-center gap-2">
                                            <UserIcon className="h-5 w-5"/>
                                            Profile Information
                                        </CardTitle>
                                        <CardDescription>
                                            Manage your public profile information and avatar
                                        </CardDescription>
                                    </CardHeader>
                                    <CardContent>
                                        <div className="flex items-start gap-6">
                                            <div className="relative group">
                                                <Avatar className="h-24 w-24 ring-2 ring-border">
                                                    <AvatarImage src={user.profile_image || undefined} alt={user.name}/>
                                                    <AvatarFallback
                                                        className="text-2xl bg-gradient-to-br from-blue-500 to-purple-600 text-white">
                                                        {getUserInitials(user.name)}
                                                    </AvatarFallback>
                                                </Avatar>
                                                <button
                                                    onClick={triggerFileInput}
                                                    className="absolute inset-0 bg-black/60 rounded-full flex items-center justify-center opacity-0 group-hover:opacity-100 transition-all duration-200 cursor-pointer"
                                                    disabled={isUploading}
                                                >
                                                    {isUploading ? (
                                                        <Loader2 className="h-6 w-6 text-white animate-spin"/>
                                                    ) : (
                                                        <Camera className="h-6 w-6 text-white"/>
                                                    )}
                                                </button>
                                                <input
                                                    type="file"
                                                    ref={fileInputRef}
                                                    onChange={handleFileChange}
                                                    accept="image/*"
                                                    className="hidden"
                                                />
                                            </div>
                                            <div className="flex-1 space-y-3">
                                                <div>
                                                    <h3 className="text-xl font-semibold">{user.name}</h3>
                                                    <p className="text-muted-foreground">{user.email}</p>
                                                </div>
                                                <div className="flex items-center gap-2">
                                                    {getRoleIcon(user.role)}
                                                    <Badge variant="secondary" className="font-medium">
                                                        {user.role}
                                                    </Badge>
                                                    <Badge variant="outline"
                                                           className={getProviderColor(user.provider)}>
                                                        {user.provider}
                                                    </Badge>
                                                </div>
                                            </div>
                                        </div>
                                    </CardContent>
                                </Card>

                                <Card>
                                    <CardHeader>
                                        <CardTitle>Update Display Name</CardTitle>
                                        <CardDescription>
                                            Change how your name appears to other users
                                        </CardDescription>
                                    </CardHeader>
                                    <CardContent>
                                        <UpdateNameForm user={user} apiFetch={apiFetch} refreshUser={refreshUser}/>
                                    </CardContent>
                                </Card>

                                <Card>
                                    <CardHeader>
                                        <CardTitle>Change Password</CardTitle>
                                        <CardDescription>
                                            Update your account password for better security
                                        </CardDescription>
                                    </CardHeader>
                                    <CardContent>
                                        <UpdatePasswordForm apiFetch={apiFetch}/>
                                    </CardContent>
                                </Card>
                            </div>

                            {/* Profile Sidebar */}
                            <div className="space-y-6">
                                <Card>
                                    <CardHeader>
                                        <CardTitle className="text-lg">Account Stats</CardTitle>
                                    </CardHeader>
                                    <CardContent className="space-y-4">
                                        <div className="flex justify-between items-center">
                                            <span className="text-sm text-muted-foreground">Account Type</span>
                                            <Badge variant="outline">{user.provider}</Badge>
                                        </div>
                                        <div className="flex justify-between items-center">
                                            <span className="text-sm text-muted-foreground">Role</span>
                                            <div className="flex items-center gap-1">
                                                {getRoleIcon(user.role)}
                                                <span className="text-sm font-medium">{user.role}</span>
                                            </div>
                                        </div>
                                        <div className="flex justify-between items-center">
                                            <span className="text-sm text-muted-foreground">Status</span>
                                            <Badge variant="outline"
                                                   className="bg-green-50 text-green-700 border-green-200">
                                                Active
                                            </Badge>
                                        </div>
                                    </CardContent>
                                </Card>

                                <Card>
                                    <CardHeader>
                                        <CardTitle className="text-lg">Security</CardTitle>
                                    </CardHeader>
                                    <CardContent className="space-y-4">
                                        <div className="flex justify-between items-center">
                                            <span className="text-sm text-muted-foreground">Two-Factor Auth</span>
                                            <Badge variant={twoFactorStatus.enabled ? "default" : "secondary"}>
                                                {twoFactorStatus.enabled ? "Enabled" : "Disabled"}
                                            </Badge>
                                        </div>
                                        <Button
                                            variant="outline"
                                            size="sm"
                                            className="w-full"
                                            onClick={() => setActiveTab('settings')}
                                        >
                                            Security Settings
                                        </Button>
                                    </CardContent>
                                </Card>
                            </div>
                        </div>
                    </TabsContent>

                    {/* Settings Tab */}
                    <TabsContent value="settings" className="mt-8 space-y-8">
                        {/* 2FA Section */}
                        <Card>
                            <CardHeader>
                                <CardTitle className="flex items-center gap-2">
                                    <Shield className="h-5 w-5"/>
                                    Two-Factor Authentication
                                </CardTitle>
                                <CardDescription>
                                    Protect your account with an additional layer of security
                                </CardDescription>
                            </CardHeader>
                            <CardContent>
                                {isLoading2FA ? (
                                    <div className="flex justify-center items-center p-12">
                                        <div className="text-center space-y-2">
                                            <Loader2 className="h-8 w-8 animate-spin text-primary mx-auto"/>
                                            <p className="text-muted-foreground">Loading security settings...</p>
                                        </div>
                                    </div>
                                ) : twoFactorStatus.enabled ? (
                                    // 2FA ENABLED VIEW
                                    <div className="space-y-6">
                                        <Alert
                                            className="bg-green-50 dark:bg-green-950 border-green-200 dark:border-green-800">
                                            <ShieldCheck className="h-4 w-4 !text-green-600"/>
                                            <AlertTitle className="text-green-800 dark:text-green-300">
                                                Two-Factor Authentication is Active
                                            </AlertTitle>
                                            <AlertDescription className="text-green-700 dark:text-green-400">
                                                Your account is protected with an authenticator app. Great job on
                                                staying secure! 🛡️
                                            </AlertDescription>
                                        </Alert>

                                        {/* Backup Codes Management */}
                                        {(showingInitialBackupCodes && backupCodes.length > 0) && (
                                            <div
                                                className="space-y-4 p-6 border-2 border-dashed border-amber-200 dark:border-amber-800 rounded-lg bg-amber-50/50 dark:bg-amber-950/20">
                                                <div className="flex items-center gap-2">
                                                    <Info className="h-5 w-5 text-amber-600"/>
                                                    <h3 className="font-semibold text-amber-800 dark:text-amber-300">
                                                        Save Your Backup Codes
                                                    </h3>
                                                </div>
                                                <p className="text-sm text-amber-700 dark:text-amber-400 mb-4">
                                                    Store these codes securely. Each can only be used once, and they're
                                                    your only way to access your account if you lose your authenticator
                                                    app.
                                                </p>
                                                <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
                                                    {backupCodes.map(code => (
                                                        <Tooltip key={code}>
                                                            <TooltipTrigger asChild>
                                                                <button
                                                                    onClick={() => copyBackupCode(code)}
                                                                    className="relative p-3 bg-white dark:bg-gray-900 rounded-lg border-2 border-amber-200 dark:border-amber-800 font-mono text-sm hover:bg-amber-50 dark:hover:bg-amber-950/50 transition-colors group"
                                                                >
                                                                    <span className="block">{code}</span>
                                                                    {copiedCodes.has(code) ? (
                                                                        <CheckCircle
                                                                            className="absolute -top-1 -right-1 h-4 w-4 text-green-500 bg-white dark:bg-gray-900 rounded-full"/>
                                                                    ) : (
                                                                        <Copy
                                                                            className="absolute -top-1 -right-1 h-4 w-4 text-gray-400 bg-white dark:bg-gray-900 rounded-full opacity-0 group-hover:opacity-100 transition-opacity"/>
                                                                    )}
                                                                </button>
                                                            </TooltipTrigger>
                                                            <TooltipContent>
                                                                <p>Click to copy</p>
                                                            </TooltipContent>
                                                        </Tooltip>
                                                    ))}
                                                </div>
                                                <div className="flex gap-2 pt-2">
                                                    <Button
                                                        onClick={downloadBackupCodes}
                                                        variant="secondary"
                                                        size="sm"
                                                        className="flex items-center gap-2"
                                                    >
                                                        <Download className="h-4 w-4"/>
                                                        Download Codes
                                                    </Button>
                                                    <Button
                                                        onClick={() => setShowingInitialBackupCodes(false)}
                                                        variant="outline"
                                                        size="sm"
                                                    >
                                                        I've Saved Them
                                                    </Button>
                                                </div>
                                            </div>
                                        )}

                                        {/* Control Buttons */}
                                        <div className="grid md:grid-cols-2 gap-6 pt-4">
                                            <Card>
                                                <CardHeader className="pb-3">
                                                    <CardTitle className="text-base">Regenerate Backup Codes</CardTitle>
                                                    <CardDescription className="text-sm">
                                                        Generate new backup codes and invalidate old ones
                                                    </CardDescription>
                                                </CardHeader>
                                                <CardContent className="space-y-3">
                                                    <Input
                                                        placeholder="Enter authenticator code"
                                                        value={regenerate2FACode}
                                                        onChange={e => setRegenerate2FACode(e.target.value)}
                                                    />
                                                    <Button
                                                        variant="outline"
                                                        onClick={regenerateBackupCodes}
                                                        disabled={isRegenerating || !regenerate2FACode}
                                                        className="w-full"
                                                    >
                                                        {isRegenerating ? (
                                                            <>
                                                                <Loader2 className="mr-2 h-4 w-4 animate-spin"/>
                                                                Generating...
                                                            </>
                                                        ) : (
                                                            <>
                                                                <RefreshCw className="mr-2 h-4 w-4"/>
                                                                Regenerate Codes
                                                            </>
                                                        )}
                                                    </Button>
                                                </CardContent>
                                            </Card>

                                            <Card>
                                                <CardHeader className="pb-3">
                                                    <CardTitle className="text-base text-destructive">Disable
                                                        2FA</CardTitle>
                                                    <CardDescription className="text-sm">
                                                        Remove two-factor authentication from your account
                                                    </CardDescription>
                                                </CardHeader>
                                                <CardContent className="space-y-3">
                                                    <Input
                                                        placeholder="Enter authenticator code"
                                                        value={disable2FACode}
                                                        onChange={e => setDisable2FACode(e.target.value)}
                                                    />
                                                    <Button
                                                        variant="destructive"
                                                        onClick={disable2FA}
                                                        disabled={isDisabling2FA || !disable2FACode}
                                                        className="w-full"
                                                    >
                                                        {isDisabling2FA ? (
                                                            <>
                                                                <Loader2 className="mr-2 h-4 w-4 animate-spin"/>
                                                                Disabling...
                                                            </>
                                                        ) : (
                                                            <>
                                                                <ShieldOff className="mr-2 h-4 w-4"/>
                                                                Disable 2FA
                                                            </>
                                                        )}
                                                    </Button>
                                                </CardContent>
                                            </Card>
                                        </div>
                                    </div>
                                ) : twoFactorSetup ? (
                                    // 2FA SETUP VIEW
                                    <div className="space-y-6">
                                        <Alert>
                                            <Info className="h-4 w-4"/>
                                            <AlertTitle>Setting up Two-Factor Authentication</AlertTitle>
                                            <AlertDescription>
                                                Follow these steps to secure your account with 2FA using any
                                                authenticator app.
                                            </AlertDescription>
                                        </Alert>

                                        <div className="grid lg:grid-cols-2 gap-8">
                                            <div className="space-y-6">
                                                <div className="text-center space-y-4">
                                                    <div
                                                        className="inline-flex items-center justify-center w-8 h-8 rounded-full bg-primary text-primary-foreground text-sm font-medium">
                                                        1
                                                    </div>
                                                    <h3 className="text-lg font-semibold">Scan QR Code</h3>
                                                    <div className="flex justify-center">
                                                        <div
                                                            className="p-4 bg-white rounded-xl border-2 border-border shadow-sm">
                                                            <img
                                                                src={twoFactorSetup.qr_code_url}
                                                                alt="2FA QR Code"
                                                                className="w-48 h-48"
                                                            />
                                                        </div>
                                                    </div>
                                                    <p className="text-sm text-muted-foreground">
                                                        Use Google Authenticator, Authy, or any compatible authenticator
                                                        app
                                                    </p>
                                                </div>
                                            </div>

                                            <div className="space-y-6">
                                                <div className="space-y-4">
                                                    <div className="flex items-center gap-2">
                                                        <div
                                                            className="inline-flex items-center justify-center w-8 h-8 rounded-full bg-primary text-primary-foreground text-sm font-medium">
                                                            2
                                                        </div>
                                                        <h3 className="text-lg font-semibold">Or Enter Key Manually</h3>
                                                    </div>
                                                    <div className="p-4 bg-muted rounded-lg">
                                                        <p className="text-sm text-muted-foreground mb-2">Secret
                                                            Key:</p>
                                                        <div
                                                            className="font-mono text-sm bg-background p-3 rounded border break-all">
                                                            {twoFactorSetup.secret}
                                                        </div>
                                                    </div>
                                                </div>

                                                <div className="space-y-4">
                                                    <div className="flex items-center gap-2">
                                                        <div
                                                            className="inline-flex items-center justify-center w-8 h-8 rounded-full bg-primary text-primary-foreground text-sm font-medium">
                                                            3
                                                        </div>
                                                        <h3 className="text-lg font-semibold">Verify & Enable</h3>
                                                    </div>
                                                    <div className="space-y-3">
                                                        <Label htmlFor="verify-code">Enter the 6-digit code from your
                                                            app:</Label>
                                                        <Input
                                                            id="verify-code"
                                                            placeholder="000000"
                                                            value={verificationCodeForEnable}
                                                            onChange={e => setVerificationCodeForEnable(e.target.value)}
                                                            maxLength={6}
                                                            className="text-center text-lg font-mono"
                                                        />
                                                        <Button
                                                            onClick={verify2FA}
                                                            disabled={isVerifying || !verificationCodeForEnable || verificationCodeForEnable.length !== 6}
                                                            className="w-full"
                                                            size="lg"
                                                        >
                                                            {isVerifying ? (
                                                                <>
                                                                    <Loader2 className="mr-2 h-4 w-4 animate-spin"/>
                                                                    Verifying...
                                                                </>
                                                            ) : (
                                                                <>
                                                                    <ShieldCheck className="mr-2 h-4 w-4"/>
                                                                    Enable Two-Factor Auth
                                                                </>
                                                            )}
                                                        </Button>
                                                    </div>
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                ) : (
                                    // 2FA NOT ENABLED VIEW
                                    <div className="text-center py-12 space-y-4">
                                        <div
                                            className="inline-flex items-center justify-center w-16 h-16 rounded-full bg-amber-100 dark:bg-amber-950">
                                            <ShieldOff className="h-8 w-8 text-amber-600"/>
                                        </div>
                                        <div className="space-y-2">
                                            <h3 className="text-xl font-semibold">Two-Factor Authentication is
                                                Disabled</h3>
                                            <p className="text-muted-foreground max-w-md mx-auto">
                                                Protect your account by adding an extra layer of security. Two-factor
                                                authentication
                                                requires a second verification step when logging in.
                                            </p>
                                        </div>
                                        <Button onClick={setup2FA} size="lg" className="mt-4">
                                            <Shield className="mr-2 h-4 w-4"/>
                                            Enable Two-Factor Authentication
                                        </Button>
                                    </div>
                                )}
                            </CardContent>
                        </Card>

                        {/* Active Sessions Section */}
                        <Card>
                            <CardHeader>
                                <CardTitle className="flex items-center gap-2">
                                    <Activity className="h-5 w-5"/>
                                    Active Sessions
                                </CardTitle>
                                <CardDescription>
                                    Monitor and manage your active sessions across different devices and browsers
                                </CardDescription>
                            </CardHeader>
                            <CardContent>
                                <SessionList/>
                            </CardContent>
                        </Card>
                    </TabsContent>

                    {/* Users Management Tab */}
                    {user?.role?.toLowerCase() === 'admin' && (
                        <TabsContent value="users" className="mt-8">
                            <Card>
                                <CardHeader>
                                    <div className="flex items-center justify-between">
                                        <div>
                                            <CardTitle className="flex items-center gap-2">
                                                <Users className="h-5 w-5"/>
                                                User Management
                                            </CardTitle>
                                            <CardDescription>
                                                Manage user accounts, roles, and permissions
                                            </CardDescription>
                                        </div>
                                        <Button onClick={fetchUsers} variant="outline" size="sm"
                                                disabled={loadingUsers}>
                                            <RefreshCw
                                                className={`h-4 w-4 mr-2 ${loadingUsers ? 'animate-spin' : ''}`}/>
                                            Refresh
                                        </Button>
                                    </div>
                                </CardHeader>
                                <CardContent>
                                    {loadingUsers ? (
                                        <div className="flex items-center justify-center py-12">
                                            <div className="text-center space-y-2">
                                                <Loader2 className="h-8 w-8 animate-spin text-primary mx-auto"/>
                                                <p className="text-muted-foreground">Loading users...</p>
                                            </div>
                                        </div>
                                    ) : usersError ? (
                                        <Alert variant="destructive">
                                            <AlertCircle className="h-4 w-4"/>
                                            <AlertTitle>Error Loading Users</AlertTitle>
                                            <AlertDescription>{usersError}</AlertDescription>
                                        </Alert>
                                    ) : (
                                        <div className="space-y-4">
                                            {/* Search and Filters */}
                                            <div className="flex flex-col sm:flex-row gap-4">
                                                <div className="flex-1">
                                                    <div className="relative">
                                                        <Search
                                                            className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground"/>
                                                        <Input
                                                            placeholder="Search users by name or email..."
                                                            value={searchTerm}
                                                            onChange={(e) => setSearchTerm(e.target.value)}
                                                            className="pl-10"
                                                        />
                                                    </div>
                                                </div>
                                                <div className="flex gap-2">
                                                    <Select value={filterRole} onValueChange={setFilterRole}>
                                                        <SelectTrigger className="w-32">
                                                            <SelectValue placeholder="Role"/>
                                                        </SelectTrigger>
                                                        <SelectContent>
                                                            <SelectItem value="all">All Roles</SelectItem>
                                                            <SelectItem value="admin">Admin</SelectItem>
                                                            <SelectItem value="user">User</SelectItem>
                                                        </SelectContent>
                                                    </Select>
                                                    <Select value={filterStatus} onValueChange={setFilterStatus}>
                                                        <SelectTrigger className="w-32">
                                                            <SelectValue placeholder="Status"/>
                                                        </SelectTrigger>
                                                        <SelectContent>
                                                            <SelectItem value="all">All Status</SelectItem>
                                                            <SelectItem value="active">Active</SelectItem>
                                                            <SelectItem value="inactive">Inactive</SelectItem>
                                                        </SelectContent>
                                                    </Select>
                                                </div>
                                            </div>

                                            {/* Results Summary */}
                                            <div
                                                className="flex items-center justify-between text-sm text-muted-foreground">
                        <span>
                          Showing {filteredUsers.length} of {users.length} users
                        </span>
                                                {(searchTerm || filterRole !== 'all' || filterStatus !== 'all') && (
                                                    <Button
                                                        variant="ghost"
                                                        size="sm"
                                                        onClick={() => {
                                                            setSearchTerm('');
                                                            setFilterRole('all');
                                                            setFilterStatus('all');
                                                        }}
                                                    >
                                                        Clear filters
                                                    </Button>
                                                )}
                                            </div>

                                            {/* Users Table */}
                                            <div className="border rounded-lg">
                                                <Table>
                                                    <TableHeader>
                                                        <TableRow>
                                                            <TableHead>User</TableHead>
                                                            <TableHead>Email</TableHead>
                                                            <TableHead>Provider</TableHead>
                                                            <TableHead>Role</TableHead>
                                                            <TableHead>Status</TableHead>
                                                            <TableHead className="text-right">Actions</TableHead>
                                                        </TableRow>
                                                    </TableHeader>
                                                    <TableBody>
                                                        {filteredUsers.length === 0 ? (
                                                            <TableRow>
                                                                <TableCell colSpan={6}
                                                                           className="text-center py-8 text-muted-foreground">
                                                                    {searchTerm || filterRole !== 'all' || filterStatus !== 'all'
                                                                        ? 'No users match your filters'
                                                                        : 'No users found'
                                                                    }
                                                                </TableCell>
                                                            </TableRow>
                                                        ) : (
                                                            filteredUsers.map((userItem) => (
                                                                <TableRow key={userItem.id} className="group">
                                                                    <TableCell>
                                                                        <div className="flex items-center gap-3">
                                                                            <Avatar className="h-8 w-8">
                                                                                <AvatarImage
                                                                                    src={userItem.profile_image || undefined}
                                                                                    alt={userItem.name}/>
                                                                                <AvatarFallback
                                                                                    className="text-xs bg-gradient-to-br from-blue-500 to-purple-600 text-white">
                                                                                    {getUserInitials(userItem.name)}
                                                                                </AvatarFallback>
                                                                            </Avatar>
                                                                            <div className="flex items-center gap-2">
                                                                                <span
                                                                                    className="font-medium">{userItem.name}</span>
                                                                                {userItem.id === user?.id && (
                                                                                    <Badge variant="outline"
                                                                                           className="text-xs">You</Badge>
                                                                                )}
                                                                            </div>
                                                                        </div>
                                                                    </TableCell>
                                                                    <TableCell
                                                                        className="font-mono text-sm">{userItem.email}</TableCell>
                                                                    <TableCell>
                                                                        <Badge variant="outline"
                                                                               className={`${getProviderColor(userItem.provider)} text-xs`}>
                                                                            {userItem.provider}
                                                                        </Badge>
                                                                    </TableCell>
                                                                    <TableCell>
                                                                        <div className="flex items-center gap-2">
                                                                            {getRoleIcon(userItem.role)}
                                                                            <Badge
                                                                                variant={userItem.role.toLowerCase() === 'admin' ? "default" : "secondary"}>
                                                                                {userItem.role}
                                                                            </Badge>
                                                                        </div>
                                                                    </TableCell>
                                                                    <TableCell>
                                                                        <Badge
                                                                            variant={userItem.is_active ? "outline" : "destructive"}
                                                                            className={userItem.is_active
                                                                                ? "bg-green-50 text-green-700 border-green-200 hover:bg-green-100 dark:bg-green-950 dark:text-green-300 dark:border-green-800"
                                                                                : ""
                                                                            }
                                                                        >
                                                                            {userItem.is_active ? "Active" : "Inactive"}
                                                                        </Badge>
                                                                    </TableCell>
                                                                    <TableCell>
                                                                        <div className="flex items-center justify-end">
                                                                            <DropdownMenu>
                                                                                <DropdownMenuTrigger asChild>
                                                                                    <Button variant="ghost" size="icon"
                                                                                            className="h-8 w-8">
                                                                                        <MoreHorizontal
                                                                                            className="h-4 w-4"/>
                                                                                        <span className="sr-only">Open menu</span>
                                                                                    </Button>
                                                                                </DropdownMenuTrigger>
                                                                                <DropdownMenuContent align="end"
                                                                                                     className="w-48">
                                                                                    <DropdownMenuLabel>Actions</DropdownMenuLabel>
                                                                                    <DropdownMenuSeparator/>
                                                                                    <DropdownMenuItem
                                                                                        onClick={() => openRoleDialog(userItem)}
                                                                                        disabled={userItem.id === user?.id}
                                                                                    >
                                                                                        <UserCog
                                                                                            className="mr-2 h-4 w-4"/>
                                                                                        Change Role
                                                                                    </DropdownMenuItem>
                                                                                    <DropdownMenuItem
                                                                                        onClick={() => openStatusDialog(userItem)}
                                                                                        disabled={userItem.id === user?.id && userItem.is_active}
                                                                                    >
                                                                                        {userItem.is_active ? (
                                                                                            <>
                                                                                                <UserX
                                                                                                    className="mr-2 h-4 w-4 text-destructive"/>
                                                                                                Deactivate
                                                                                            </>
                                                                                        ) : (
                                                                                            <>
                                                                                                <UserCheck
                                                                                                    className="mr-2 h-4 w-4 text-green-500"/>
                                                                                                Activate
                                                                                            </>
                                                                                        )}
                                                                                    </DropdownMenuItem>
                                                                                    <DropdownMenuSeparator/>
                                                                                    <DropdownMenuItem
                                                                                        onClick={() => openDeleteDialog(userItem)}
                                                                                        disabled={userItem.id === user?.id}
                                                                                        className="text-destructive"
                                                                                    >
                                                                                        <Trash
                                                                                            className="mr-2 h-4 w-4"/>
                                                                                        Delete User
                                                                                    </DropdownMenuItem>
                                                                                </DropdownMenuContent>
                                                                            </DropdownMenu>
                                                                        </div>
                                                                    </TableCell>
                                                                </TableRow>
                                                            ))
                                                        )}
                                                    </TableBody>
                                                </Table>
                                            </div>
                                        </div>
                                    )}
                                </CardContent>
                            </Card>
                        </TabsContent>
                    )}
                </Tabs>

                {/* Dialogs */}
                <Dialog open={isRoleDialogOpen} onOpenChange={setIsRoleDialogOpen}>
                    <DialogContent>
                        <DialogHeader>
                            <DialogTitle>Change User Role</DialogTitle>
                            <DialogDescription>
                                Update {selectedUser?.name}'s role and permissions.
                            </DialogDescription>
                        </DialogHeader>
                        <div className="space-y-4 py-4">
                            <div className="space-y-2">
                                <Label htmlFor="role">Select Role</Label>
                                <Select value={newRole} onValueChange={setNewRole}>
                                    <SelectTrigger>
                                        <SelectValue placeholder="Select a role"/>
                                    </SelectTrigger>
                                    <SelectContent>
                                        <SelectItem value="user">User</SelectItem>
                                        <SelectItem value="admin">Admin</SelectItem>
                                    </SelectContent>
                                </Select>
                            </div>
                        </div>
                        <DialogFooter>
                            <Button variant="outline" onClick={() => setIsRoleDialogOpen(false)}>
                                Cancel
                            </Button>
                            <Button onClick={handleRoleChange} disabled={isSubmitting}>
                                {isSubmitting ? <Loader2 className="mr-2 h-4 w-4 animate-spin"/> : null}
                                Update Role
                            </Button>
                        </DialogFooter>
                    </DialogContent>
                </Dialog>

                <Dialog open={isStatusDialogOpen} onOpenChange={setIsStatusDialogOpen}>
                    <DialogContent>
                        <DialogHeader>
                            <DialogTitle>
                                {newStatus ? 'Activate' : 'Deactivate'} User
                            </DialogTitle>
                            <DialogDescription>
                                Are you sure you want to {newStatus ? 'activate' : 'deactivate'} {selectedUser?.name}?
                                {!newStatus && " They won't be able to access their account."}
                            </DialogDescription>
                        </DialogHeader>
                        <DialogFooter>
                            <Button variant="outline" onClick={() => setIsStatusDialogOpen(false)}>
                                Cancel
                            </Button>
                            <Button
                                onClick={handleStatusChange}
                                disabled={isSubmitting}
                                variant={newStatus ? "default" : "destructive"}
                            >
                                {isSubmitting ? <Loader2 className="mr-2 h-4 w-4 animate-spin"/> : null}
                                {newStatus ? 'Activate' : 'Deactivate'}
                            </Button>
                        </DialogFooter>
                    </DialogContent>
                </Dialog>

                <Dialog open={isDeleteDialogOpen} onOpenChange={setIsDeleteDialogOpen}>
                    <DialogContent>
                        <DialogHeader>
                            <DialogTitle>Delete User</DialogTitle>
                            <DialogDescription>
                                Are you sure you want to permanently delete {selectedUser?.name}?
                                This action cannot be undone and will remove all of their data.
                            </DialogDescription>
                        </DialogHeader>
                        <DialogFooter>
                            <Button variant="outline" onClick={() => setIsDeleteDialogOpen(false)}>
                                Cancel
                            </Button>
                            <Button onClick={handleDeleteUser} disabled={isSubmitting} variant="destructive">
                                {isSubmitting ? <Loader2 className="mr-2 h-4 w-4 animate-spin"/> : null}
                                Delete User
                            </Button>
                        </DialogFooter>
                    </DialogContent>
                </Dialog>
            </div>
        </TooltipProvider>
    );
}

// Placeholder components - you'll need to implement these
function UpdateNameForm({user, apiFetch, refreshUser}: any) {
    const [name, setName] = useState(user.name);
    const [isUpdating, setIsUpdating] = useState(false);
    const {toast} = useToast();

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        if (!name.trim() || name === user.name) return;

        setIsUpdating(true);
        try {
            await apiFetch('/api/user/profile', {
                method: 'PUT',
                body: JSON.stringify({name: name.trim()}),
            });
            await refreshUser();
            toast({title: "✅ Success", description: "Your name has been updated."});
        } catch (error: any) {
            toast({title: "❌ Error", description: error.message, variant: "destructive"});
        } finally {
            setIsUpdating(false);
        }
    };

    return (
        <form onSubmit={handleSubmit} className="space-y-4">
            <div className="space-y-2">
                <Label htmlFor="name">Display Name</Label>
                <Input
                    id="name"
                    value={name}
                    onChange={(e) => setName(e.target.value)}
                    placeholder="Enter your display name"
                />
            </div>
            <Button type="submit" disabled={isUpdating || !name.trim() || name === user.name}>
                {isUpdating ? <Loader2 className="mr-2 h-4 w-4 animate-spin"/> : <Save className="mr-2 h-4 w-4"/>}
                Update Name
            </Button>
        </form>
    );
}

function UpdatePasswordForm({apiFetch}: any) {
    const [currentPassword, setCurrentPassword] = useState('');
    const [newPassword, setNewPassword] = useState('');
    const [confirmPassword, setConfirmPassword] = useState('');
    const [isUpdating, setIsUpdating] = useState(false);
    const [showPasswords, setShowPasswords] = useState(false);
    const {toast} = useToast();

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        if (newPassword !== confirmPassword) {
            toast({title: "❌ Error", description: "New passwords don't match.", variant: "destructive"});
            return;
        }

        setIsUpdating(true);
        try {
            await apiFetch('/api/user/password', {
                method: 'PUT',
                body: JSON.stringify({
                    current_password: currentPassword,
                    new_password: newPassword,
                }),
            });
            toast({title: "✅ Success", description: "Your password has been updated."});
            setCurrentPassword('');
            setNewPassword('');
            setConfirmPassword('');
        } catch (error: any) {
            toast({title: "❌ Error", description: error.message, variant: "destructive"});
        } finally {
            setIsUpdating(false);
        }
    };

    return (
        <form onSubmit={handleSubmit} className="space-y-4">
            <div className="space-y-2">
                <Label htmlFor="current">Current Password</Label>
                <div className="relative">
                    <Input
                        id="current"
                        type={showPasswords ? "text" : "password"}
                        value={currentPassword}
                        onChange={(e) => setCurrentPassword(e.target.value)}
                        placeholder="Enter current password"
                    />
                </div>
            </div>
            <div className="space-y-2">
                <Label htmlFor="new">New Password</Label>
                <div className="relative">
                    <Input
                        id="new"
                        type={showPasswords ? "text" : "password"}
                        value={newPassword}
                        onChange={(e) => setNewPassword(e.target.value)}
                        placeholder="Enter new password"
                    />
                </div>
            </div>
            <div className="space-y-2">
                <Label htmlFor="confirm">Confirm New Password</Label>
                <div className="relative">
                    <Input
                        id="confirm"
                        type={showPasswords ? "text" : "password"}
                        value={confirmPassword}
                        onChange={(e) => setConfirmPassword(e.target.value)}
                        placeholder="Confirm new password"
                    />
                    <Button
                        type="button"
                        variant="ghost"
                        size="icon"
                        className="absolute right-0 top-0 h-full px-3"
                        onClick={() => setShowPasswords(!showPasswords)}
                    >
                        {showPasswords ? <EyeOff className="h-4 w-4"/> : <Eye className="h-4 w-4"/>}
                    </Button>
                </div>
            </div>
            <Button
                type="submit"
                disabled={isUpdating || !currentPassword || !newPassword || !confirmPassword}
            >
                {isUpdating ? <Loader2 className="mr-2 h-4 w-4 animate-spin"/> : <Save className="mr-2 h-4 w-4"/>}
                Update Password
            </Button>
        </form>
    );
}