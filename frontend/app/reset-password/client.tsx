'use client';

import { useState, useEffect } from 'react';
import { useRouter, useSearchParams } from 'next/navigation';
import Link from 'next/link';
import { z } from 'zod';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { Loader2, Eye, EyeOff, Shield, CheckCircle2, AlertCircle, ArrowLeft, Key } from 'lucide-react';

import { Button } from '@/components/ui/button';
import {
    Card,
    CardContent,
    CardDescription,
    CardFooter,
    CardHeader,
    CardTitle,
} from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import {
    Form,
    FormControl,
    FormField,
    FormItem,
    FormLabel,
    FormMessage,
} from '@/components/ui/form';
import { toast } from '@/components/ui/use-toast';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Progress } from '@/components/ui/progress';

// Form schema with password validation
const formSchema = z.object({
    password: z
        .string()
        .min(8, 'Password must be at least 8 characters')
        .regex(/[A-Z]/, 'Password must contain at least one uppercase letter')
        .regex(/[a-z]/, 'Password must contain at least one lowercase letter')
        .regex(/[0-9]/, 'Password must contain at least one number')
        .regex(/[^A-Za-z0-9]/, 'Password must contain at least one special character'),
    confirmPassword: z.string(),
}).refine(data => data.password === data.confirmPassword, {
    message: "Passwords don't match",
    path: ['confirmPassword'],
});

type FormValues = z.infer<typeof formSchema>;

// Password strength indicator
const getPasswordStrength = (password: string) => {
    let score = 0;
    const checks = [
        { regex: /.{8,}/, label: 'At least 8 characters' },
        { regex: /[A-Z]/, label: 'Uppercase letter' },
        { regex: /[a-z]/, label: 'Lowercase letter' },
        { regex: /[0-9]/, label: 'Number' },
        { regex: /[^A-Za-z0-9]/, label: 'Special character' },
    ];

    const passedChecks = checks.map(check => ({
        ...check,
        passed: check.regex.test(password)
    }));

    score = passedChecks.filter(check => check.passed).length;

    return {
        score,
        percentage: (score / checks.length) * 100,
        checks: passedChecks,
        strength: score < 2 ? 'Weak' : score < 4 ? 'Medium' : 'Strong'
    };
};

export default function ResetPasswordPage() {
    const router = useRouter();
    const searchParams = useSearchParams();
    const [token, setToken] = useState<string | null>(null);
    const [isSubmitting, setIsSubmitting] = useState(false);
    const [success, setSuccess] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [showPassword, setShowPassword] = useState(false);
    const [showConfirmPassword, setShowConfirmPassword] = useState(false);
    const [isTokenValidating, setIsTokenValidating] = useState(true);

    // Get token from URL and validate it
    useEffect(() => {
        const urlToken = searchParams.get('token');
        if (!urlToken) {
            setError('Invalid or missing reset token. Please request a new password reset link.');
            setIsTokenValidating(false);
            return;
        }

        // Simulate token validation (you might want to validate with backend)
        setTimeout(() => {
            setToken(urlToken);
            setIsTokenValidating(false);
        }, 1500);
    }, [searchParams]);

    // Initialize form
    const form = useForm<FormValues>({
        resolver: zodResolver(formSchema),
        defaultValues: {
            password: '',
            confirmPassword: '',
        },
    });

    const password = form.watch('password');
    const passwordStrength = getPasswordStrength(password || '');

    // Handle form submission
    const onSubmit = async (values: FormValues) => {
        if (!token) {
            setError('Invalid or missing reset token. Please request a new password reset link.');
            return;
        }

        setIsSubmitting(true);

        try {
            const response = await fetch(`${process.env.NEXT_PUBLIC_API_URL}/api/auth/reset-password`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    token: token,
                    password: values.password
                }),
            });

            if (!response.ok) {
                const errorData = await response.json();
                throw new Error(errorData.error || 'Failed to reset password');
            }

            // Show success message
            setSuccess(true);

            // Clear form
            form.reset();

            // Show success toast
            toast({
                title: "Password Reset Successful",
                description: "You will be redirected to the login page shortly.",
            });

            // Redirect to login page after 3 seconds
            setTimeout(() => {
                router.push('/');
            }, 3000);

        } catch (error: any) {
            console.error('Error resetting password:', error);
            setError(error.message || 'Failed to reset password. Please try again.');

            toast({
                variant: "destructive",
                title: "Reset Failed",
                description: error.message || 'Failed to reset password. Please try again.',
            });
        } finally {
            setIsSubmitting(false);
        }
    };

    // Loading state for token validation
    if (isTokenValidating) {
        return (
            <div className="flex min-h-screen items-center justify-center">
                <Card className="w-full max-w-md border-0 shadow-xl">
                    <CardContent className="pt-6">
                        <div className="flex flex-col items-center space-y-4">
                            <div className="flex items-center justify-center w-12 h-12 bg-blue-100 rounded-full">
                                <Shield className="w-6 h-6 text-blue-600" />
                            </div>
                            <div className="text-center">
                                <h3 className="text-lg font-semibold text-gray-900">Validating Reset Token</h3>
                                <p className="text-sm text-gray-600 mt-1">Please wait while we verify your request...</p>
                            </div>
                            <Loader2 className="w-6 h-6 animate-spin text-blue-600" />
                        </div>
                    </CardContent>
                </Card>
            </div>
        );
    }

    return (
        <div className="flex min-h-screen items-center justify-center px-4 py-12 sm:px-6 lg:px-8">
            <div className="w-full max-w-md">
                {/* Back to login link */}
                <div className="mb-6">
                    <Link
                        href="/"
                        className="inline-flex items-center text-sm text-gray-600 hover:text-gray-900 transition-colors duration-200"
                    >
                        <ArrowLeft className="w-4 h-4 mr-1" />
                        Back to Sign In
                    </Link>
                </div>

                <Card className="border-0 shadow-xl bg-white/80 backdrop-blur-sm">
                    <CardHeader className="space-y-1 text-center pb-6">
                        <div className="flex items-center justify-center w-16 h-16 mx-auto mb-4 bg-gradient-to-br from-blue-600 to-purple-600 rounded-full">
                            <Key className="w-8 h-8 text-white" />
                        </div>
                        <CardTitle className="text-2xl font-bold bg-gradient-to-r from-blue-600 to-purple-600 bg-clip-text text-transparent">
                            Reset Your Password
                        </CardTitle>
                        <CardDescription className="text-gray-600">
                            Create a new secure password for your account
                        </CardDescription>
                    </CardHeader>

                    <CardContent>
                        {error && (
                            <Alert variant="destructive" className="mb-6 border-red-200 bg-red-50">
                                <AlertCircle className="h-4 w-4" />
                                <AlertTitle>Error</AlertTitle>
                                <AlertDescription>{error}</AlertDescription>
                            </Alert>
                        )}

                        {success ? (
                            <Alert className="mb-6 border-green-200 bg-green-50">
                                <CheckCircle2 className="h-4 w-4 text-green-600" />
                                <AlertTitle className="text-green-800">Success!</AlertTitle>
                                <AlertDescription className="text-green-700">
                                    Your password has been reset successfully. You will be redirected to the login page shortly.
                                </AlertDescription>
                            </Alert>
                        ) : (
                            !error && token && (
                                <Form {...form}>
                                    <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-6">
                                        <FormField
                                            control={form.control}
                                            name="password"
                                            render={({ field }) => (
                                                <FormItem>
                                                    <FormLabel className="text-gray-700 font-medium">New Password</FormLabel>
                                                    <FormControl>
                                                        <div className="relative">
                                                            <Input
                                                                placeholder="Enter your new password"
                                                                type={showPassword ? "text" : "password"}
                                                                autoComplete="new-password"
                                                                disabled={isSubmitting}
                                                                className="pr-10 border-gray-300 focus:border-blue-500 focus:ring-blue-500 transition-colors duration-200"
                                                                {...field}
                                                            />
                                                            <Button
                                                                type="button"
                                                                variant="ghost"
                                                                size="sm"
                                                                className="absolute right-0 top-0 h-full px-3 py-2 hover:bg-transparent"
                                                                onClick={() => setShowPassword(!showPassword)}
                                                                disabled={isSubmitting}
                                                            >
                                                                {showPassword ? (
                                                                    <EyeOff className="h-4 w-4 text-gray-400" />
                                                                ) : (
                                                                    <Eye className="h-4 w-4 text-gray-400" />
                                                                )}
                                                            </Button>
                                                        </div>
                                                    </FormControl>
                                                    <FormMessage />

                                                    {/* Password strength indicator */}
                                                    {password && (
                                                        <div className="mt-3 space-y-2">
                                                            <div className="flex items-center justify-between">
                                                                <span className="text-xs text-gray-600">Password Strength</span>
                                                                <span className={`text-xs font-medium ${
                                                                    passwordStrength.strength === 'Weak' ? 'text-red-600' :
                                                                        passwordStrength.strength === 'Medium' ? 'text-yellow-600' :
                                                                            'text-green-600'
                                                                }`}>
                                  {passwordStrength.strength}
                                </span>
                                                            </div>
                                                            <Progress
                                                                value={passwordStrength.percentage}
                                                                className={`h-2 ${
                                                                    passwordStrength.strength === 'Weak' ? '[&>div]:bg-red-500' :
                                                                        passwordStrength.strength === 'Medium' ? '[&>div]:bg-yellow-500' :
                                                                            '[&>div]:bg-green-500'
                                                                }`}
                                                            />
                                                            <div className="grid grid-cols-1 gap-1">
                                                                {passwordStrength.checks.map((check, index) => (
                                                                    <div key={index} className="flex items-center space-x-2">
                                                                        <div className={`w-2 h-2 rounded-full ${
                                                                            check.passed ? 'bg-green-500' : 'bg-gray-300'
                                                                        }`} />
                                                                        <span className={`text-xs ${
                                                                            check.passed ? 'text-green-700' : 'text-gray-500'
                                                                        }`}>
                                      {check.label}
                                    </span>
                                                                    </div>
                                                                ))}
                                                            </div>
                                                        </div>
                                                    )}
                                                </FormItem>
                                            )}
                                        />

                                        <FormField
                                            control={form.control}
                                            name="confirmPassword"
                                            render={({ field }) => (
                                                <FormItem>
                                                    <FormLabel className="text-gray-700 font-medium">Confirm New Password</FormLabel>
                                                    <FormControl>
                                                        <div className="relative">
                                                            <Input
                                                                placeholder="Confirm your new password"
                                                                type={showConfirmPassword ? "text" : "password"}
                                                                autoComplete="new-password"
                                                                disabled={isSubmitting}
                                                                className="pr-10 border-gray-300 focus:border-blue-500 focus:ring-blue-500 transition-colors duration-200"
                                                                {...field}
                                                            />
                                                            <Button
                                                                type="button"
                                                                variant="ghost"
                                                                size="sm"
                                                                className="absolute right-0 top-0 h-full px-3 py-2 hover:bg-transparent"
                                                                onClick={() => setShowConfirmPassword(!showConfirmPassword)}
                                                                disabled={isSubmitting}
                                                            >
                                                                {showConfirmPassword ? (
                                                                    <EyeOff className="h-4 w-4 text-gray-400" />
                                                                ) : (
                                                                    <Eye className="h-4 w-4 text-gray-400" />
                                                                )}
                                                            </Button>
                                                        </div>
                                                    </FormControl>
                                                    <FormMessage />

                                                    {/* Password match indicator */}
                                                    {field.value && password && (
                                                        <div className="flex items-center mt-2 space-x-2">
                                                            {field.value === password ? (
                                                                <>
                                                                    <CheckCircle2 className="w-4 h-4 text-green-500" />
                                                                    <span className="text-xs text-green-700">Passwords match</span>
                                                                </>
                                                            ) : (
                                                                <>
                                                                    <AlertCircle className="w-4 h-4 text-red-500" />
                                                                    <span className="text-xs text-red-700">Passwords don't match</span>
                                                                </>
                                                            )}
                                                        </div>
                                                    )}
                                                </FormItem>
                                            )}
                                        />

                                        <Button
                                            type="submit"
                                            className="w-full bg-gradient-to-r from-blue-600 to-purple-600 hover:from-blue-700 hover:to-purple-700 text-white font-medium py-2.5 transition-all duration-200 transform hover:scale-[1.02] disabled:transform-none disabled:opacity-50"
                                            disabled={isSubmitting || passwordStrength.score < 5}
                                        >
                                            {isSubmitting ? (
                                                <>
                                                    <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                                                    Resetting Password...
                                                </>
                                            ) : (
                                                <>
                                                    <Shield className="mr-2 h-4 w-4" />
                                                    Reset Password
                                                </>
                                            )}
                                        </Button>

                                        {/* Security notice */}
                                        <div className="bg-blue-50 border border-blue-200 rounded-lg p-4 mt-4">
                                            <div className="flex items-start space-x-3">
                                                <Shield className="w-5 h-5 text-blue-600 mt-0.5 flex-shrink-0" />
                                                <div>
                                                    <h4 className="text-sm font-medium text-blue-800">Security Tips</h4>
                                                    <ul className="text-xs text-blue-700 mt-1 space-y-1">
                                                        <li>• Use a unique password that you don't use elsewhere</li>
                                                        <li>• Consider using a password manager</li>
                                                        <li>• Enable two-factor authentication if available</li>
                                                    </ul>
                                                </div>
                                            </div>
                                        </div>
                                    </form>
                                </Form>
                            )
                        )}
                    </CardContent>

                    <CardFooter className="flex flex-col space-y-4 pt-6">
                        <div className="text-center">
                            <div className="text-sm text-gray-500">
                                Remember your password?{' '}
                                <Link
                                    href="/"
                                    className="font-medium text-blue-600 hover:text-blue-800 transition-colors duration-200 underline-offset-4 hover:underline"
                                >
                                    Sign in instead
                                </Link>
                            </div>
                        </div>

                        {/* Additional help */}
                        <div className="text-center">
                            <p className="text-xs text-gray-400">
                                Need help? <Link href="/support" className="text-blue-600 hover:text-blue-800 transition-colors duration-200">Contact Support</Link>
                            </p>
                        </div>
                    </CardFooter>
                </Card>

                {/* Footer info */}
                <div className="mt-8 text-center">
                    <p className="text-xs text-gray-500">
                        This password reset link will expire in 24 hours for security purposes.
                    </p>
                </div>
            </div>
        </div>
    );
}