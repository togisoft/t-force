'use client';

import { useEffect, useState, Suspense } from 'react';
import { useRouter, useSearchParams } from 'next/navigation';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from '@/components/ui/card';
import { AlertCircle, ArrowLeft, Home, RefreshCw, Loader2 } from 'lucide-react';
import Link from 'next/link';

// Map of error codes to user-friendly messages
const errorMessages: Record<string, { title: string; description: string; action: string }> = {
  'NotFound': {
    title: 'Page Not Found',
    description: 'The authentication endpoint you are trying to access does not exist or has been moved.',
    action: 'Please try signing in from the main page or contact support if the problem persists.'
  },
  'Configuration': {
    title: 'Server Configuration Error',
    description: 'There is a problem with the server configuration. This is not your fault.',
    action: 'Please try again later or contact support if the problem persists.'
  },
  'AccessDenied': {
    title: 'Access Denied',
    description: 'You do not have permission to sign in.',
    action: 'Please sign in with an authorized account or contact support for assistance.'
  },
  'Verification': {
    title: 'Verification Error',
    description: 'The verification link may have expired or already been used.',
    action: 'Please request a new verification link.'
  },
  'OAuthSignin': {
    title: 'OAuth Sign In Error',
    description: 'There was a problem with the OAuth sign-in process.',
    action: 'Please try again or use a different sign-in method.'
  },
  'OAuthCallback': {
    title: 'OAuth Callback Error',
    description: 'There was a problem with the OAuth callback process.',
    action: 'Please try again or use a different sign-in method.'
  },
  'OAuthCreateAccount': {
    title: 'Account Creation Error',
    description: 'There was a problem creating your account.',
    action: 'Please try again or use a different sign-in method.'
  },
  'EmailCreateAccount': {
    title: 'Account Creation Error',
    description: 'There was a problem creating your account with this email.',
    action: 'Please try again or use a different email address.'
  },
  'Callback': {
    title: 'Callback Error',
    description: 'There was a problem with the authentication callback.',
    action: 'Please try again or contact support if the problem persists.'
  },
  'OAuthAccountNotLinked': {
    title: 'Account Not Linked',
    description: 'This email is already associated with another account.',
    action: 'Please sign in using the original provider you used to create your account.'
  },
  'EmailSignin': {
    title: 'Email Sign In Error',
    description: 'There was a problem sending the email with the sign-in link.',
    action: 'Please check your email address and try again.'
  },
  'CredentialsSignin': {
    title: 'Invalid Credentials',
    description: 'The email or password you entered is incorrect.',
    action: 'Please check your credentials and try again.'
  },
  'SessionRequired': {
    title: 'Authentication Required',
    description: 'You must be signed in to access this page.',
    action: 'Please sign in to continue.'
  },
  'Default': {
    title: 'Authentication Error',
    description: 'An unexpected error occurred during authentication.',
    action: 'Please try again or contact support if the problem persists.'
  }
};

// Component that uses useSearchParams
function ErrorContent() {
  const router = useRouter();
  const searchParams = useSearchParams();
  const [errorType, setErrorType] = useState<string>('Default');

  useEffect(() => {
    // Get error type from URL
    const error = searchParams.get('error');
    if (error && errorMessages[error]) {
      setErrorType(error);
    } else {
      setErrorType('Default');
    }
  }, [searchParams]);

  const errorInfo = errorMessages[errorType] || errorMessages['Default'];

  return (
    <Card className="max-w-md w-full shadow-lg border-red-200">
      <CardHeader className="space-y-1">
        <div className="flex items-center gap-2">
          <AlertCircle className="h-6 w-6 text-red-500" />
          <CardTitle className="text-2xl font-bold text-red-700">
            {errorInfo.title}
          </CardTitle>
        </div>
        <CardDescription className="text-base">
          {errorInfo.description}
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <p className="text-sm text-muted-foreground">
          {errorInfo.action}
        </p>
        
        {errorType === 'CredentialsSignin' && (
          <div className="text-sm text-muted-foreground">
            <p>If you've forgotten your password, please contact your administrator.</p>
          </div>
        )}
        
        {errorType === 'OAuthAccountNotLinked' && (
          <div className="text-sm text-muted-foreground">
            <p>To link multiple providers to one account, first sign in with the original provider you used to create your account, then link additional providers from your profile settings.</p>
          </div>
        )}
      </CardContent>
      <CardFooter className="flex flex-col sm:flex-row gap-3">
        <Button 
          variant="default" 
          className="w-full sm:w-auto flex items-center gap-2"
          onClick={() => router.push('/')}
        >
          <Home className="h-4 w-4" />
          Return to Sign In
        </Button>
        <Button 
          variant="outline" 
          className="w-full sm:w-auto flex items-center gap-2"
          onClick={() => router.back()}
        >
          <ArrowLeft className="h-4 w-4" />
          Go Back
        </Button>
        <Button 
          variant="ghost" 
          className="w-full sm:w-auto flex items-center gap-2"
          onClick={() => window.location.reload()}
        >
          <RefreshCw className="h-4 w-4" />
          Retry
        </Button>
      </CardFooter>
    </Card>
  );
}

// Loading fallback component
function ErrorLoadingFallback() {
  return (
    <Card className="max-w-md w-full shadow-lg">
      <CardHeader className="space-y-1">
        <div className="flex items-center gap-2">
          <Loader2 className="h-6 w-6 animate-spin text-primary" />
          <CardTitle className="text-2xl font-bold">
            Loading...
          </CardTitle>
        </div>
        <CardDescription>
          Please wait while we process your request.
        </CardDescription>
      </CardHeader>
      <CardContent className="flex justify-center py-6">
        <Loader2 className="h-8 w-8 animate-spin text-primary" />
      </CardContent>
    </Card>
  );
}

// Main error page component with Suspense boundary
export default function ErrorPage() {
  return (
    <main className="min-h-screen flex flex-col items-center justify-center p-4 bg-background">
      <Suspense fallback={<ErrorLoadingFallback />}>
        <ErrorContent />
      </Suspense>
    </main>
  );
}