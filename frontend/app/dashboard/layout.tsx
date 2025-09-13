'use client';

import { useAuth, useProtectedRoute } from '@/lib/auth';
import { useRouter, usePathname } from 'next/navigation';
import { useState, useEffect } from 'react';
import { Button } from '@/components/ui/button';
import { Avatar, AvatarFallback, AvatarImage } from '@/components/ui/avatar';
import { Badge } from '@/components/ui/badge';
import { TooltipProvider } from '@/components/ui/tooltip';
import {
    Loader2,
    LogOut,
    Sun,
    Moon,
    User,
    Shield,
    ChevronDown,
    Crown,
    Lock,
    Monitor,
    Sparkles,
    Github, MessageSquare
} from 'lucide-react';
import { useTheme } from 'next-themes';
import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuLabel,
    DropdownMenuSeparator,
    DropdownMenuTrigger,
    DropdownMenuGroup,
    DropdownMenuSub,
    DropdownMenuSubContent,
    DropdownMenuSubTrigger
} from '@/components/ui/dropdown-menu';
import { cn } from '@/lib/utils';
import Link from "next/link";

// Google Icon Component
const GoogleIcon = ({ className = "h-4 w-4" }: { className?: string }) => (
    <svg className={className} viewBox="0 0 24 24">
        <path
            fill="currentColor"
            d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z"
        />
        <path
            fill="currentColor"
            d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z"
        />
        <path
            fill="currentColor"
            d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z"
        />
        <path
            fill="currentColor"
            d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z"
        />
    </svg>
);

export default function DashboardLayout({ children }: { children: React.ReactNode }) {
    const { user, logout, isLoading, isAuthenticated } = useAuth();
    useProtectedRoute();
    const { theme, setTheme } = useTheme();
    const router = useRouter();
    const pathname = usePathname();
    const [isSigningOut, setIsSigningOut] = useState(false);
    const [mounted, setMounted] = useState(false);

    useEffect(() => {
        setMounted(true);
    }, []);

    const handleSignOut = async () => {
        setIsSigningOut(true);
        try {
            await logout();
        } catch (error) {
            console.error('Error during logout:', error);
            setIsSigningOut(false);
        }
    };

    const getUserInitials = (name: string) => {
        if (!name) return '?';
        return name.split(' ').map(part => part[0]).join('').toUpperCase().substring(0, 2);
    };

    const getRoleBadgeColor = (role: string) => {
        switch (role?.toLowerCase()) {
            case 'admin':
                return 'bg-gradient-to-r from-chart-4 to-chart-5 text-primary-foreground';
            case 'moderator':
                return 'bg-gradient-to-r from-primary to-chart-2 text-primary-foreground';
            default:
                return 'bg-muted text-muted-foreground';
        }
    };

    const getProviderInfo = (provider: string) => {
        switch (provider?.toLowerCase()) {
            case 'google':
                return {
                    color: 'bg-destructive/10 text-destructive border-destructive/20',
                    icon: <GoogleIcon className="h-3 w-3" />,
                    name: 'Google'
                };
            case 'github':
                return {
                    color: 'bg-muted text-muted-foreground border-border',
                    icon: <Github className="h-3 w-3" />,
                    name: 'GitHub'
                };
            case 'local':
                return {
                    color: 'bg-primary/10 text-primary border-primary/20',
                    icon: <Lock className="h-3 w-3" />,
                    name: 'Local'
                };
            default:
                return {
                    color: 'bg-muted text-muted-foreground border-border',
                    icon: <User className="h-3 w-3" />,
                    name: 'Unknown'
                };
        }
    };

    // Loading state
    if (isLoading || !isAuthenticated) {
        return (
            <div className="flex h-screen items-center justify-center bg-background">
                <div className="text-center space-y-4">
                    <div className="relative">
                        <div className="h-16 w-16 bg-primary/20 rounded-full animate-pulse mx-auto"></div>
                        <Loader2 className="h-8 w-8 animate-spin text-primary absolute inset-0 m-auto" />
                    </div>
                    <div className="space-y-2">
                        <div className="h-4 w-32 bg-muted rounded animate-pulse mx-auto"></div>
                        <div className="h-3 w-24 bg-muted rounded animate-pulse mx-auto"></div>
                    </div>
                </div>
            </div>
        );
    }

    const providerInfo = getProviderInfo(user?.provider || '');

    return (
        <TooltipProvider>
            <div className="h-screen bg-background flex flex-col overflow-hidden">
                {/* Header */}
                <header className="flex-shrink-0 z-50 border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
                    <div className="container mx-auto px-4 sm:px-6 lg:px-8">
                        <div className="flex h-12 items-center justify-between">
                            {/* Logo Section */}
                            <div className="flex items-center gap-2">
                                <a href={"/dashboard"} className="relative">
                                    <div className="h-7 w-7 bg-gradient-to-r from-primary to-chart-2 rounded-lg flex items-center justify-center shadow-lg">
                                        <Lock className="h-3 w-3 text-primary-foreground" />
                                    </div>
                                    <div className="absolute -top-0.5 -right-0.5 h-2.5 w-2.5 bg-chart-3 rounded-full border border-background"></div>
                                </a>
                                <div>
                                    <h1 className="font-bold text-lg bg-gradient-to-r from-foreground to-muted-foreground bg-clip-text text-transparent">
                                        T-Force
                                    </h1>
                                    <p className="text-xs text-muted-foreground -mt-0.5">Communication Platform</p>
                                </div>
                            </div>

                            {/* User Menu */}
                            <DropdownMenu>
                                <DropdownMenuTrigger asChild>
                                    <Button variant="ghost" className="flex items-center gap-2 px-2 py-1 h-auto hover:bg-muted transition-colors">
                                        <Avatar className="h-7 w-7 ring-2 ring-border">
                                            <AvatarImage src={user?.profile_image || undefined} alt={user?.name || 'User'} />
                                            <AvatarFallback className="bg-gradient-to-br from-primary to-chart-2 text-primary-foreground text-xs font-medium">
                                                {getUserInitials(user?.name || '')}
                                            </AvatarFallback>
                                        </Avatar>
                                        <div className="hidden sm:block text-left">
                                            <div className="flex items-center gap-2">
                                                <p className="text-sm font-medium leading-none">{user?.name || 'Loading...'}</p>
                                                {user?.role && (
                                                    <Badge className={cn("text-xs px-1.5 py-0.5", getRoleBadgeColor(user.role))}>
                                                        {user?.role?.toLowerCase() === 'admin' && <Crown className="h-3 w-3 mr-1" />}
                                                        {user?.role?.toLowerCase() === 'moderator' && <Shield className="h-3 w-3 mr-1" />}
                                                        {user.role}
                                                    </Badge>
                                                )}
                                            </div>
                                            <div className="flex items-center gap-1 mt-1">
                                                {providerInfo.icon}
                                                <p className="text-xs text-muted-foreground">{user?.email || 'Loading...'}</p>
                                            </div>
                                        </div>
                                        <ChevronDown className="hidden sm:block h-4 w-4 text-muted-foreground" />
                                    </Button>
                                </DropdownMenuTrigger>
                                <DropdownMenuContent align="end" className="w-80 p-2">
                                    {/* User Info Header */}
                                    <div className="flex items-center gap-3 p-3 bg-muted rounded-lg mb-2">
                                        <Avatar className="h-12 w-12 ring-2 ring-background">
                                            <AvatarImage src={user?.profile_image || undefined} alt={user?.name || 'User'} />
                                            <AvatarFallback className="bg-gradient-to-br from-primary to-chart-2 text-primary-foreground">
                                                {getUserInitials(user?.name || '')}
                                            </AvatarFallback>
                                        </Avatar>
                                        <div className="flex-1 min-w-0">
                                            <div className="flex items-center gap-2 mb-1">
                                                <p className="text-sm font-semibold truncate">{user?.name || 'Loading...'}</p>
                                                {user?.role && (
                                                    <Badge className={cn("text-xs px-2 py-0.5", getRoleBadgeColor(user.role))}>
                                                        {user?.role?.toLowerCase() === 'admin' && <Crown className="h-3 w-3 mr-1" />}
                                                        {user?.role?.toLowerCase() === 'moderator' && <Shield className="h-3 w-3 mr-1" />}
                                                        {user.role}
                                                    </Badge>
                                                )}
                                            </div>
                                            <p className="text-xs text-muted-foreground truncate">{user?.email || 'Loading...'}</p>
                                            <div className="flex items-center gap-1 mt-1">
                                                <Badge variant="outline" className={cn("text-xs px-1.5 py-0.5 flex items-center gap-1", providerInfo.color)}>
                                                    {providerInfo.icon}
                                                    <span>{providerInfo.name}</span>
                                                </Badge>
                                                <Badge variant="outline" className="text-xs px-1.5 py-0.5 bg-chart-3/10 text-chart-3 border-chart-3/20 flex items-center gap-1">
                                                    <div className="h-1.5 w-1.5 bg-chart-3 rounded-full"></div>
                                                    <span>Online</span>
                                                </Badge>
                                            </div>
                                        </div>
                                    </div>

                                    <DropdownMenuSeparator />

                                    <DropdownMenuGroup>
                                        <DropdownMenuLabel className="text-xs text-muted-foreground px-3 py-2">Applications</DropdownMenuLabel>

                                        <DropdownMenuItem className="flex items-center gap-3 p-3">
                                            <div className="h-8 w-8 bg-purple-100 dark:bg-purple-900 rounded-lg flex items-center justify-center">
                                                <MessageSquare className="h-4 w-4 text-purple-600 dark:text-purple-400" />
                                            </div>
                                            <div>
                                                <Link href={"/dashboard/chat"} className="font-medium">ChatForce</Link>
                                                <p className="text-xs text-muted-foreground">Room based chat application</p>
                                            </div>
                                        </DropdownMenuItem>


                                    </DropdownMenuGroup>

                                    <DropdownMenuSeparator />

                                    {/* Theme Selector */}
                                    <DropdownMenuSub>
                                        <DropdownMenuSubTrigger className="flex items-center gap-3 p-3">
                                            <div className="h-8 w-8 bg-muted rounded-lg flex items-center justify-center">
                                                {mounted && theme === 'dark' ? (
                                                    <Moon className="h-4 w-4" />
                                                ) : (
                                                    <Sun className="h-4 w-4" />
                                                )}
                                            </div>
                                            <div>
                                                <p className="font-medium">Appearance</p>
                                                <p className="text-xs text-muted-foreground capitalize">
                                                    {mounted ? theme : 'system'} theme
                                                </p>
                                            </div>
                                        </DropdownMenuSubTrigger>
                                        <DropdownMenuSubContent className="w-48">
                                            <DropdownMenuItem onClick={() => setTheme('light')} className="flex items-center gap-3 p-3">
                                                <Sun className="h-4 w-4" />
                                                <div>
                                                    <p className="font-medium">Light</p>
                                                    <p className="text-xs text-muted-foreground">Clean and bright</p>
                                                </div>
                                                {theme === 'light' && <Sparkles className="h-3 w-3 ml-auto text-primary" />}
                                            </DropdownMenuItem>
                                            <DropdownMenuItem onClick={() => setTheme('dark')} className="flex items-center gap-3 p-3">
                                                <Moon className="h-4 w-4" />
                                                <div>
                                                    <p className="font-medium">Dark</p>
                                                    <p className="text-xs text-muted-foreground">Easy on the eyes</p>
                                                </div>
                                                {theme === 'dark' && <Sparkles className="h-3 w-3 ml-auto text-primary" />}
                                            </DropdownMenuItem>
                                            <DropdownMenuItem onClick={() => setTheme('system')} className="flex items-center gap-3 p-3">
                                                <Monitor className="h-4 w-4" />
                                                <div>
                                                    <p className="font-medium">System</p>
                                                    <p className="text-xs text-muted-foreground">Follow device setting</p>
                                                </div>
                                                {theme === 'system' && <Sparkles className="h-3 w-3 ml-auto text-primary" />}
                                            </DropdownMenuItem>
                                        </DropdownMenuSubContent>
                                    </DropdownMenuSub>

                                    <DropdownMenuSeparator />

                                    <DropdownMenuItem
                                        onClick={handleSignOut}
                                        disabled={isSigningOut}
                                        className="flex items-center gap-3 p-3 text-red-600 dark:text-red-400 focus:text-red-600 dark:focus:text-red-400 focus:bg-red-50 dark:focus:bg-red-950"
                                    >
                                        <div className="h-8 w-8 bg-red-100 dark:bg-red-900 rounded-lg flex items-center justify-center">
                                            {isSigningOut ? (
                                                <Loader2 className="h-4 w-4 animate-spin" />
                                            ) : (
                                                <LogOut className="h-4 w-4" />
                                            )}
                                        </div>
                                        <div>
                                            <p className="font-medium">
                                                {isSigningOut ? 'Signing out...' : 'Sign Out'}
                                            </p>
                                            <p className="text-xs text-muted-foreground">
                                                {isSigningOut ? 'Please wait...' : 'End your session safely'}
                                            </p>
                                        </div>
                                    </DropdownMenuItem>
                                </DropdownMenuContent>
                            </DropdownMenu>
                        </div>
                    </div>
                </header>

                {/* Main Content */}
                <main className={pathname?.includes('/chat') ? "flex-1 overflow-hidden" : "flex-1 overflow-auto"}>
                    {children}
                </main>

                {/* Footer - Hide on chat page */}
                {!pathname?.includes('/chat') && (
                    <footer className="flex-shrink-0 border-t bg-muted/30">
                        <div className="container mx-auto px-4 sm:px-6 lg:px-8 py-3">
                            <div className="flex flex-col sm:flex-row items-center justify-between gap-2">
                                <div className="flex items-center gap-2 text-sm text-muted-foreground">
                                    <Lock className="h-3 w-3 text-primary" />
                                    <span>Powered by T-Force</span>
                                    <Badge variant="outline" className="text-xs">v1.0</Badge>
                                </div>
                                <div className="text-sm text-muted-foreground">
                                    <span>© 2025 T-Force. All rights reserved.</span>
                                </div>
                            </div>
                        </div>
                    </footer>
                )}
            </div>
        </TooltipProvider>
    );
}