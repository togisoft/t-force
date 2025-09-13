import { useState } from 'react';
import { useRouter } from 'next/navigation';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Loader2, CheckCircle2, AlertCircle } from 'lucide-react';
import { useAuth } from '@/lib/auth';

export function TwoFactorForm() {
  const { verify2FA } = useAuth();
  const router = useRouter();
  const [verificationCode, setVerificationCode] = useState('');
  const [isVerifying, setIsVerifying] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleVerify = async () => {
    if (!verificationCode || verificationCode.length < 6) {
      setError('Please enter a valid verification code');
      return;
    }

    setIsVerifying(true);
    setError(null);
    
    try {
      const result = await verify2FA({
        code: verificationCode,
      });
      
      if (!result.success) {
        setError(result.error || 'Verification failed. Please try again.');
        setIsVerifying(false);
      } else {
        // Successful verification - redirect to dashboard
        // Keep loading state true during redirection for better UX
        setTimeout(() => {
          router.replace('/dashboard');
        }, 500);
      }
    } catch (err) {
      console.error('2FA verification error:', err);
      setError('An unexpected error occurred. Please try again later.');
      setIsVerifying(false);
    }
  };

  return (
    <div className="space-y-6 pt-2">
      <div className="text-center mb-6">
        <h3 className="text-xl font-bold mb-2">Two-Factor Authentication</h3>
        <p className="text-sm text-muted-foreground">
          Please enter the verification code from your authenticator app
        </p>
      </div>
      
      <div className="space-y-4">
        <div className="space-y-2">
          <Label htmlFor="verification-code" className="text-base">Verification Code</Label>
          <Input
            id="verification-code"
            type="text"
            placeholder="Enter 6-digit code"
            value={verificationCode}
            onChange={(e) => setVerificationCode(e.target.value)}
            className="py-6 text-center text-lg tracking-widest"
            maxLength={6}
            disabled={isVerifying}
          />
        </div>
        
        {error && (
          <Alert variant="destructive" className="py-2">
            <AlertCircle className="h-4 w-4" />
            <AlertTitle>Error</AlertTitle>
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        )}
        
        <Button 
          onClick={handleVerify} 
          className="w-full bg-gradient-to-r from-primary-600 to-primary-500 hover:from-primary-700 hover:to-primary-600 button-glow py-6 text-base" 
          disabled={isVerifying || !verificationCode || verificationCode.length < 6}
        >
          {isVerifying ? (
            <>
              <Loader2 className="mr-2 h-5 w-5 animate-spin" />
              Verifying...
            </>
          ) : (
            <>
              <CheckCircle2 className="mr-2 h-5 w-5" />
              Verify Code
            </>
          )}
        </Button>
        
        <p className="text-xs text-center text-muted-foreground mt-4">
          If you're having trouble, please contact support or use one of your backup codes.
        </p>
      </div>
    </div>
  );
}