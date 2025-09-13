'use client';

import { useEffect, useState } from 'react';
import { useRouter, useSearchParams } from 'next/navigation';
import { useAuth } from '@/lib/auth';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Loader2 } from 'lucide-react';

export default function OAuthCallbackPage() {
    const router = useRouter();
    const searchParams = useSearchParams();
    const { handleOAuthToken, verify2FA } = useAuth();

    const [isLoading, setIsLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);
    const [requires2FA, setRequires2FA] = useState(false);
    const [verificationCode, setVerificationCode] = useState('');
    const [verifying2FA, setVerifying2FA] = useState(false);
    const [verificationError, setVerificationError] = useState<string | null>(null);

    useEffect(() => {
        const processOAuthCallback = async () => {
            try {
                // Get token from URL
                const token = searchParams.get('token');
                const requires2FA = searchParams.get('requires2fa') === 'true';

                if (!token) {
                    setError('No authentication token found in the URL.');
                    setIsLoading(false);
                    return;
                }

                // Handle the OAuth token
                const result = await handleOAuthToken(token, requires2FA);

                if (!result.success) {
                    setError(result.error || 'Failed to authenticate with the provided token.');
                    setIsLoading(false);
                    return;
                }

                if (result.requires2FA) {
                    // If 2FA is required, show the verification form
                    setRequires2FA(true);
                    setIsLoading(false);
                } else {
                    // If authentication is successful and 2FA is not required, redirect to dashboard
                    router.push('/dashboard');
                }
            } catch (error) {
                console.error('Error processing OAuth callback:', error);
                setError('An unexpected error occurred while processing the authentication.');
                setIsLoading(false);
            }
        };

        processOAuthCallback();
    }, [searchParams, handleOAuthToken, router]);

    const handleVerify2FA = async () => {
        if (!verificationCode) {
            setVerificationError('Please enter the verification code.');
            return;
        }

        setVerifying2FA(true);
        setVerificationError(null);

        try {
            const result = await verify2FA({ code: verificationCode });

            if (result.success) {
                // If 2FA verification is successful, redirect to dashboard
                router.push('/dashboard');
            } else {
                setVerificationError(result.error || 'Failed to verify the code. Please try again.');
                setVerifying2FA(false);
            }
        } catch (error) {
            console.error('Error verifying 2FA code:', error);
            setVerificationError('An unexpected error occurred during verification.');
            setVerifying2FA(false);
        }
    };

    if (isLoading) {
        return (
            <div className="flex items-center justify-center min-h-screen">
                <Card className="w-full max-w-md">
                    <CardHeader>
                        <CardTitle>Processing Authentication</CardTitle>
                        <CardDescription>Please wait while we process your authentication...</CardDescription>
                    </CardHeader>
                    <CardContent className="flex justify-center p-6">
                        <Loader2 className="h-8 w-8 animate-spin" />
                    </CardContent>
                </Card>
            </div>
        );
    }

    if (error) {
        return (
            <div className="flex items-center justify-center min-h-screen">
                <Card className="w-full max-w-md">
                    <CardHeader>
                        <CardTitle>Authentication Error</CardTitle>
                        <CardDescription>There was a problem with your authentication.</CardDescription>
                    </CardHeader>
                    <CardContent>
                        <Alert variant="destructive" className="mb-4">
                            <AlertDescription>{error}</AlertDescription>
                        </Alert>
                        <Button className="w-full" onClick={() => router.push('/')}>
                            Return to Login
                        </Button>
                    </CardContent>
                </Card>
            </div>
        );
    }

    if (requires2FA) {
        return (
            <div className="flex items-center justify-center min-h-screen">
                <Card className="w-full max-w-md">
                    <CardHeader>
                        <CardTitle>Two-Factor Authentication</CardTitle>
                        <CardDescription>Please enter the verification code from your authenticator app.</CardDescription>
                    </CardHeader>
                    <CardContent>
                        {verificationError && (
                            <Alert variant="destructive" className="mb-4">
                                <AlertDescription>{verificationError}</AlertDescription>
                            </Alert>
                        )}
                        <div className="space-y-4">
                            <div className="space-y-2">
                                <Label htmlFor="code">Verification Code</Label>
                                <Input
                                    id="code"
                                    placeholder="Enter 6-digit code"
                                    value={verificationCode}
                                    onChange={(e) => setVerificationCode(e.target.value)}
                                    disabled={verifying2FA}
                                />
                            </div>
                            <Button
                                className="w-full"
                                onClick={handleVerify2FA}
                                disabled={verifying2FA}
                            >
                                {verifying2FA ? (
                                    <>
                                        <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                                        Verifying...
                                    </>
                                ) : (
                                    'Verify'
                                )}
                            </Button>
                            <Button
                                variant="outline"
                                className="w-full"
                                onClick={() => router.push('/')}
                                disabled={verifying2FA}
                            >
                                Cancel
                            </Button>
                        </div>
                    </CardContent>
                </Card>
            </div>
        );
    }

    // This should not be reached, but just in case
    return (
        <div className="flex items-center justify-center min-h-screen">
            <Card className="w-full max-w-md">
                <CardHeader>
                    <CardTitle>Redirecting...</CardTitle>
                    <CardDescription>You will be redirected to the dashboard shortly.</CardDescription>
                </CardHeader>
                <CardContent className="flex justify-center p-6">
                    <Loader2 className="h-8 w-8 animate-spin" />
                </CardContent>
            </Card>
        </div>
    );
}