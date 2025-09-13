'use client';

import { useState } from 'react';
import { useRouter } from 'next/navigation';
import Link from 'next/link';
import { z } from 'zod';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { Loader2, Mail, ArrowLeft, CheckCircle2, Send, Clock, Shield, AlertCircle } from 'lucide-react';

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

// Form schema
const formSchema = z.object({
  email: z.string().email('Please enter a valid email address'),
});

type FormValues = z.infer<typeof formSchema>;

export default function ForgotPasswordPage() {
  const router = useRouter();
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [success, setSuccess] = useState(false);
  const [submittedEmail, setSubmittedEmail] = useState('');

  // Initialize form
  const form = useForm<FormValues>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      email: '',
    },
  });

  // Handle form submission
  const onSubmit = async (values: FormValues) => {
    setIsSubmitting(true);
    setSubmittedEmail(values.email);

    try {
      const response = await fetch(`${process.env.NEXT_PUBLIC_API_URL}/api/auth/forgot-password`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ email: values.email }),
      });

      if (!response.ok) {
        const errorData = await response.json();
        throw new Error(errorData.error || 'Failed to send password reset email');
      }

      // Show success message
      setSuccess(true);

      // Clear form
      form.reset();

      // Show success toast
      toast({
        title: "Reset Link Sent",
        description: "Please check your email for the password reset link.",
      });

    } catch (error: any) {
      console.error('Error sending password reset email:', error);
      toast({
        variant: 'destructive',
        title: 'Error',
        description: error.message || 'Failed to send password reset email. Please try again.',
      });
    } finally {
      setIsSubmitting(false);
    }
  };

  // Handle resend email
  const handleResend = async () => {
    if (!submittedEmail) return;

    setIsSubmitting(true);

    try {
      const response = await fetch(`${process.env.NEXT_PUBLIC_API_URL}/api/auth/forgot-password`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ email: submittedEmail }),
      });

      if (!response.ok) {
        const errorData = await response.json();
        throw new Error(errorData.error || 'Failed to resend password reset email');
      }

      toast({
        title: "Email Resent",
        description: "Please check your email for the new password reset link.",
      });

    } catch (error: any) {
      console.error('Error resending password reset email:', error);
      toast({
        variant: 'destructive',
        title: 'Error',
        description: error.message || 'Failed to resend email. Please try again.',
      });
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
      <div className="flex min-h-screen items-center justify-center px-4 py-12 sm:px-6 lg:px-8">
        <div className="w-full max-w-md">
          {/* Back to login link */}
          <div className="mb-6">
            <Link
                href="/"
                className="inline-flex items-center text-sm text-gray-600 hover:text-gray-900 transition-colors duration-200 group"
            >
              <ArrowLeft className="w-4 h-4 mr-1 group-hover:-translate-x-1 transition-transform duration-200" />
              Back to Sign In
            </Link>
          </div>

          <Card className="border-0 shadow-xl bg-white/80 backdrop-blur-sm overflow-hidden">
            <CardHeader className="space-y-1 text-center pb-6 bg-gradient-to-r from-blue-600/5 to-purple-600/5">
              <div className="flex items-center justify-center w-16 h-16 mx-auto mb-4 bg-gradient-to-br from-blue-600 to-purple-600 rounded-full shadow-lg">
                <Mail className="w-8 h-8 text-white" />
              </div>
              <CardTitle className="text-2xl font-bold bg-gradient-to-r from-blue-600 to-purple-600 bg-clip-text text-transparent">
                Forgot Password?
              </CardTitle>
              <CardDescription className="text-gray-600 max-w-sm mx-auto">
                No worries! Enter your email address and we'll send you a secure link to reset your password.
              </CardDescription>
            </CardHeader>

            <CardContent className="pt-6">
              {success ? (
                  <div className="space-y-6">
                    <Alert className="border-green-200 bg-green-50">
                      <CheckCircle2 className="h-5 w-5 text-green-600" />
                      <AlertTitle className="text-green-800">Email Sent Successfully!</AlertTitle>
                      <AlertDescription className="text-green-700">
                        We've sent a password reset link to <strong>{submittedEmail}</strong>.
                        Please check your inbox and spam folder.
                      </AlertDescription>
                    </Alert>

                    {/* Email not received section */}
                    <div className="bg-gray-50 rounded-lg p-4 space-y-4">
                      <h4 className="text-sm font-medium text-gray-800 flex items-center">
                        <Clock className="w-4 h-4 mr-2 text-gray-600" />
                        Didn't receive the email?
                      </h4>
                      <div className="space-y-3 text-sm text-gray-600">
                        <div className="flex items-start space-x-2">
                          <div className="w-1.5 h-1.5 bg-gray-400 rounded-full mt-2 flex-shrink-0"></div>
                          <span>Check your spam/junk folder</span>
                        </div>
                        <div className="flex items-start space-x-2">
                          <div className="w-1.5 h-1.5 bg-gray-400 rounded-full mt-2 flex-shrink-0"></div>
                          <span>Make sure you entered the correct email address</span>
                        </div>
                        <div className="flex items-start space-x-2">
                          <div className="w-1.5 h-1.5 bg-gray-400 rounded-full mt-2 flex-shrink-0"></div>
                          <span>Wait a few minutes for the email to arrive</span>
                        </div>
                      </div>

                      <Button
                          variant="outline"
                          size="sm"
                          onClick={handleResend}
                          disabled={isSubmitting}
                          className="w-full mt-4 border-blue-200 text-blue-600 hover:bg-blue-50 hover:border-blue-300 transition-all duration-200"
                      >
                        {isSubmitting ? (
                            <>
                              <Loader2 className="mr-2 h-3 w-3 animate-spin" />
                              Sending...
                            </>
                        ) : (
                            <>
                              <Send className="mr-2 h-3 w-3" />
                              Resend Email
                            </>
                        )}
                      </Button>
                    </div>
                  </div>
              ) : (
                  <div className="space-y-6">
                    <Form {...form}>
                      <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-6">
                        <FormField
                            control={form.control}
                            name="email"
                            render={({ field }) => (
                                <FormItem>
                                  <FormLabel className="text-gray-700 font-medium">Email Address</FormLabel>
                                  <FormControl>
                                    <div className="relative">
                                      <Mail className="absolute left-3 top-3 h-4 w-4 text-gray-400" />
                                      <Input
                                          placeholder="your.email@example.com"
                                          type="email"
                                          autoComplete="email"
                                          disabled={isSubmitting}
                                          className="pl-10 border-gray-300 focus:border-blue-500 focus:ring-blue-500 transition-all duration-200 hover:border-gray-400"
                                          {...field}
                                      />
                                    </div>
                                  </FormControl>
                                  <FormMessage />
                                </FormItem>
                            )}
                        />

                        <Button
                            type="submit"
                            className="w-full bg-gradient-to-r from-blue-600 to-purple-600 hover:from-blue-700 hover:to-purple-700 text-white font-medium py-2.5 transition-all duration-200 transform hover:scale-[1.02] disabled:transform-none disabled:opacity-50 shadow-lg hover:shadow-xl"
                            disabled={isSubmitting}
                        >
                          {isSubmitting ? (
                              <>
                                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                                Sending Reset Link...
                              </>
                          ) : (
                              <>
                                <Send className="mr-2 h-4 w-4" />
                                Send Reset Link
                              </>
                          )}
                        </Button>
                      </form>
                    </Form>

                    {/* Security notice */}
                    <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
                      <div className="flex items-start space-x-3">
                        <Shield className="w-5 h-5 text-blue-600 mt-0.5 flex-shrink-0" />
                        <div>
                          <h4 className="text-sm font-medium text-blue-800">Security Notice</h4>
                          <p className="text-xs text-blue-700 mt-1">
                            For your security, the reset link will expire in 24 hours.
                            If you don't receive an email, the address may not be registered with us.
                          </p>
                        </div>
                      </div>
                    </div>
                  </div>
              )}
            </CardContent>

            <CardFooter className="flex flex-col space-y-4 pt-6 bg-gray-50/50">
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

          {/* Additional info */}
          <div className="mt-8 text-center space-y-3">
            <div className="flex items-center justify-center space-x-6 text-xs text-gray-500">
              <div className="flex items-center space-x-1">
                <Shield className="w-3 h-3" />
                <span>Secure Process</span>
              </div>
              <div className="flex items-center space-x-1">
                <Clock className="w-3 h-3" />
                <span>24hr Expiry</span>
              </div>
              <div className="flex items-center space-x-1">
                <Mail className="w-3 h-3" />
                <span>Email Verification</span>
              </div>
            </div>

            {/* Common issues help */}
            <details className="bg-white/60 backdrop-blur-sm rounded-lg border border-gray-200 p-4 text-left group">
              <summary className="text-sm font-medium text-gray-700 cursor-pointer flex items-center justify-between group-open:text-blue-600 transition-colors duration-200">
              <span className="flex items-center">
                <AlertCircle className="w-4 h-4 mr-2" />
                Common Issues & Solutions
              </span>
                <span className="text-gray-400 group-open:rotate-180 transition-transform duration-200">▼</span>
              </summary>
              <div className="mt-4 space-y-3 text-xs text-gray-600 border-t border-gray-200 pt-4">
                <div>
                  <strong className="text-gray-800">Email not arriving?</strong>
                  <p>Check your spam folder and ensure the email address is correct. It may take up to 10 minutes to arrive.</p>
                </div>
                <div>
                  <strong className="text-gray-800">Link expired?</strong>
                  <p>Reset links expire after 24 hours for security. You can request a new one anytime.</p>
                </div>
                <div>
                  <strong className="text-gray-800">Account not found?</strong>
                  <p>If your email isn't registered, you won't receive a reset link. Consider creating a new account.</p>
                </div>
              </div>
            </details>
          </div>
        </div>
      </div>
  );
}