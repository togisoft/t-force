'use client';

import { useAuth } from '@/lib/auth';
import { useRouter } from 'next/navigation';
import { useEffect, useState } from 'react';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardFooter } from '@/components/ui/card';
import { Loader2, Mail, Eye, EyeOff, UserPlus, Lock, AlertCircle, User, Shield, CheckCircle2 } from 'lucide-react';
import { Input } from '@/components/ui/input';
import { Form, FormControl, FormField, FormItem, FormLabel, FormMessage } from '@/components/ui/form';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { TwoFactorForm } from '@/components/auth/two-factor-form';
import { Progress } from '@/components/ui/progress';
import { useToast } from '@/hooks/use-toast';
import Link from 'next/link';

// Enhanced Zod schemas
const loginFormSchema = z.object({
  email: z.string().email({ message: "Please enter a valid email address." }),
  password: z.string().min(6, { message: "Password must be at least 6 characters." }),
});

const registerFormSchema = z.object({
  name: z.string().min(2, { message: "Name must be at least 2 characters." }),
  email: z.string().email({ message: "Please enter a valid email address." }),
  password: z
      .string()
      .min(8, 'Password must be at least 8 characters')
      .regex(/[A-Z]/, 'Password must contain at least one uppercase letter')
      .regex(/[a-z]/, 'Password must contain at least one lowercase letter')
      .regex(/[0-9]/, 'Password must contain at least one number')
      .regex(/[^A-Za-z0-9]/, 'Password must contain at least one special character'),
  confirmPassword: z.string(),
}).refine((data) => data.password === data.confirmPassword, {
  message: "Passwords do not match.",
  path: ["confirmPassword"],
});

type LoginFormValues = z.infer<typeof loginFormSchema>;
type RegisterFormValues = z.infer<typeof registerFormSchema>;

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

export default function AuthPage() {
  const { isAuthenticated, requires2FA, login, register } = useAuth();
  const router = useRouter();
  const { toast } = useToast();
  const [isLoading, setIsLoading] = useState(false);
  const [isRegistering, setIsRegistering] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [registerError, setRegisterError] = useState<string | null>(null);
  const [registrationSuccess, setRegistrationSuccess] = useState(false);
  const [showPassword, setShowPassword] = useState(false);
  const [showConfirmPassword, setShowConfirmPassword] = useState(false);
  const [activeTab, setActiveTab] = useState("login");

  const loginForm = useForm<LoginFormValues>({
    resolver: zodResolver(loginFormSchema),
    defaultValues: { email: "", password: "" },
  });

  const registerForm = useForm<RegisterFormValues>({
    resolver: zodResolver(registerFormSchema),
    defaultValues: { name: "", email: "", password: "", confirmPassword: "" },
  });

  const password = registerForm.watch('password');
  const confirmPassword = registerForm.watch('confirmPassword');
  const passwordStrength = getPasswordStrength(password || '');

  useEffect(() => {
    if (isAuthenticated) {
      router.replace('/dashboard');
    }
  }, [isAuthenticated, router]);

  const onLoginSubmit = async (data: LoginFormValues) => {
    setIsLoading(true);
    setError(null);
    
    try {
      const result = await login(data);
      if (!result.success) {
        setError(result.error || "Authentication failed.");
        toast({
          title: "Sign In Failed",
          description: result.error || "Authentication failed. Please check your credentials.",
          variant: "destructive",
        });
      }
    } catch (error) {
      const errorMessage = "Authentication failed. Please try again.";
      setError(errorMessage);
      toast({
        title: "Sign In Failed",
        description: errorMessage,
        variant: "destructive",
      });
    } finally {
      setIsLoading(false);
    }
  };

  const onRegisterSubmit = async (data: RegisterFormValues) => {
    setIsRegistering(true);
    setRegisterError(null);
    setRegistrationSuccess(false);
    
    try {
      const result = await register({ name: data.name, email: data.email, password: data.password });
      
      if (result.success) {
        setRegistrationSuccess(true);
        
        // Show success toast
        toast({
          title: "Registration Successful!",
          description: "Your account has been created successfully. Please sign in with your credentials.",
          variant: "default",
        });
        
        // Clear the form
        registerForm.reset();
        
        // Switch to login tab after a short delay
        setTimeout(() => {
          setActiveTab("login");
          setRegistrationSuccess(false);
        }, 2000);
      } else {
        setRegisterError(result.error || "Registration failed.");
        toast({
          title: "Registration Failed",
          description: result.error || "Registration failed. Please try again.",
          variant: "destructive",
        });
      }
    } catch (error) {
      const errorMessage = "Registration failed. Please try again.";
      setRegisterError(errorMessage);
      toast({
        title: "Registration Failed",
        description: errorMessage,
        variant: "destructive",
      });
    } finally {
      setIsRegistering(false);
    }
  };

  if (isAuthenticated) {
    return (
        <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-background via-accent/20 to-muted/20">
          <Card className="p-8 border-0 shadow-xl auth-card">
            <div className="text-center space-y-4">
              <div className="w-16 h-16 mx-auto bg-gradient-to-br from-chart-3 to-chart-4 rounded-full flex items-center justify-center">
                <CheckCircle2 className="w-8 h-8 text-primary-foreground" />
              </div>
              <h2 className="text-xl font-semibold text-foreground">Welcome Back!</h2>
              <p className="text-muted-foreground">Redirecting to your dashboard...</p>
              <Loader2 className="h-6 w-6 animate-spin text-primary mx-auto" />
            </div>
          </Card>
        </div>
    );
  }

  return (
      <main className="min-h-screen flex flex-col items-center justify-center p-4 bg-gradient-to-br from-background via-accent/20 to-muted/20 relative overflow-hidden">

        {/* Hero Section */}
        <div className="w-full max-w-md mx-auto mb-8 text-center z-10">
          <div className="relative">
            <div className="w-20 h-20 mx-auto mb-6 bg-gradient-to-br from-primary to-chart-2 rounded-2xl shadow-xl flex items-center justify-center transform rotate-3 hover:rotate-0 transition-transform duration-300">
              <Shield className="w-10 h-10 text-primary-foreground" />
            </div>
            <h1 className="text-5xl font-extrabold tracking-tight text-gradient mb-3">
              T-Force
            </h1>
            <p className="text-foreground text-lg font-medium">
              Secure Communication Platform
            </p>
            <p className="text-muted-foreground text-sm mt-2">
              Modern, secure, and user-friendly authentication for your applications
            </p>
          </div>
        </div>

        <Card className="max-w-md w-full shadow-2xl z-10 border-0 auth-card overflow-hidden">
          {requires2FA ? (
              <CardContent className="pt-6">
                                  <div className="text-center mb-6">
                  <div className="w-16 h-16 mx-auto bg-gradient-to-br from-chart-4 to-chart-5 rounded-full flex items-center justify-center mb-4">
                    <Shield className="w-8 h-8 text-primary-foreground" />
                  </div>
                  <h2 className="text-xl font-semibold text-foreground">Two-Factor Authentication</h2>
                  <p className="text-muted-foreground text-sm mt-1">Enter the verification code from your authenticator app</p>
                </div>
                <TwoFactorForm />
              </CardContent>
          ) : (
              <>
                <Tabs value={activeTab} onValueChange={setActiveTab} className="w-full">
                  <CardHeader className="pb-4">
                    <TabsList className="grid w-full grid-cols-2 bg-muted/80 backdrop-blur-sm">
                      <TabsTrigger
                          value="login"
                          className="data-[state=active]:bg-card data-[state=active]:shadow-sm transition-all duration-200"
                      >
                        <Lock className="w-4 h-4 mr-2" />
                        Sign In
                      </TabsTrigger>
                      <TabsTrigger
                          value="register"
                          className="data-[state=active]:bg-card data-[state=active]:shadow-sm transition-all duration-200"
                      >
                        <UserPlus className="w-4 h-4 mr-2" />
                        Create Account
                      </TabsTrigger>
                    </TabsList>
                  </CardHeader>

                  <CardContent className="space-y-6">
                    <TabsContent value="login" className="space-y-6 mt-0">
                      <Form {...loginForm}>
                        <form onSubmit={loginForm.handleSubmit(onLoginSubmit)} className="space-y-5">
                          {/* Email Field */}
                          <FormField
                              control={loginForm.control}
                              name="email"
                              render={({ field }) => (
                                  <FormItem>
                                    <FormLabel className="text-gray-700 font-medium">Email Address</FormLabel>
                                    <FormControl>
                                      <div className="relative group">
                                        <Mail className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-gray-400 group-focus-within:text-blue-500 transition-colors duration-200" />
                                        <Input
                                            placeholder="you@example.com"
                                            type="email"
                                            autoComplete="email"
                                            className="pl-10 h-12 border-gray-300 focus:border-blue-500 focus:ring-blue-500 transition-all duration-200 hover:border-gray-400"
                                            {...field}
                                        />
                                      </div>
                                    </FormControl>
                                    <FormMessage />
                                  </FormItem>
                              )}
                          />

                          {/* Password Field */}
                          <FormField
                              control={loginForm.control}
                              name="password"
                              render={({ field }) => (
                                  <FormItem>
                                    <FormLabel className="text-gray-700 font-medium">Password</FormLabel>
                                    <FormControl>
                                      <div className="relative group">
                                        <Lock className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-gray-400 group-focus-within:text-blue-500 transition-colors duration-200" />
                                        <Input
                                            type={showPassword ? "text" : "password"}
                                            placeholder="••••••••"
                                            autoComplete="current-password"
                                            className="pl-10 pr-10 h-12 border-gray-300 focus:border-blue-500 focus:ring-blue-500 transition-all duration-200 hover:border-gray-400"
                                            {...field}
                                        />
                                        <Button
                                            type="button"
                                            variant="ghost"
                                            size="sm"
                                            className="absolute right-0 top-0 h-full px-3 hover:bg-transparent"
                                            onClick={() => setShowPassword(!showPassword)}
                                        >
                                          {showPassword ?
                                              <EyeOff className="h-4 w-4 text-gray-400 hover:text-gray-600 transition-colors duration-200" /> :
                                              <Eye className="h-4 w-4 text-gray-400 hover:text-gray-600 transition-colors duration-200" />
                                          }
                                        </Button>
                                      </div>
                                    </FormControl>
                                    <FormMessage />
                                    <div className="text-right mt-2">
                                      <Link
                                          href="/forgot-password"
                                          className="text-sm text-blue-600 hover:text-blue-800 transition-colors duration-200 underline-offset-4 hover:underline"
                                      >
                                        Forgot password?
                                      </Link>
                                    </div>
                                  </FormItem>
                              )}
                          />

                          {error && (
                              <Alert variant="destructive" className="border-red-200 bg-red-50">
                                <AlertCircle className="h-4 w-4" />
                                <AlertTitle>Sign In Failed</AlertTitle>
                                <AlertDescription>{error}</AlertDescription>
                              </Alert>
                          )}

                          <Button
                              type="submit"
                              className="w-full h-12 text-base font-medium bg-gradient-to-r from-primary to-chart-2 hover:from-primary/90 hover:to-chart-2/90 shadow-lg hover:shadow-xl transition-all duration-200 transform hover:scale-[1.02] disabled:transform-none disabled:opacity-50 button-glow"
                              disabled={isLoading}
                          >
                            {isLoading ? (
                                <>
                                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                                  Signing In...
                                </>
                            ) : (
                                <>
                                  <Lock className="mr-2 h-4 w-4" />
                                  Sign In
                                </>
                            )}
                          </Button>
                        </form>
                      </Form>
                    </TabsContent>

                    <TabsContent value="register" className="space-y-6 mt-0">
                      <Form {...registerForm}>
                        <form onSubmit={registerForm.handleSubmit(onRegisterSubmit)} className="space-y-5">
                          {/* Name Field */}
                          <FormField
                              control={registerForm.control}
                              name="name"
                              render={({ field }) => (
                                  <FormItem>
                                    <FormLabel className="text-gray-700 font-medium">Full Name</FormLabel>
                                    <FormControl>
                                      <div className="relative group">
                                        <User className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-gray-400 group-focus-within:text-blue-500 transition-colors duration-200" />
                                        <Input
                                            placeholder="John Doe"
                                            autoComplete="name"
                                            className="pl-10 h-12 border-gray-300 focus:border-blue-500 focus:ring-blue-500 transition-all duration-200 hover:border-gray-400"
                                            {...field}
                                        />
                                      </div>
                                    </FormControl>
                                    <FormMessage />
                                  </FormItem>
                              )}
                          />

                          {/* Email Field */}
                          <FormField
                              control={registerForm.control}
                              name="email"
                              render={({ field }) => (
                                  <FormItem>
                                    <FormLabel className="text-gray-700 font-medium">Email Address</FormLabel>
                                    <FormControl>
                                      <div className="relative group">
                                        <Mail className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-gray-400 group-focus-within:text-blue-500 transition-colors duration-200" />
                                        <Input
                                            placeholder="you@example.com"
                                            type="email"
                                            autoComplete="email"
                                            className="pl-10 h-12 border-gray-300 focus:border-blue-500 focus:ring-blue-500 transition-all duration-200 hover:border-gray-400"
                                            {...field}
                                        />
                                      </div>
                                    </FormControl>
                                    <FormMessage />
                                  </FormItem>
                              )}
                          />

                          {/* Password Field */}
                          <FormField
                              control={registerForm.control}
                              name="password"
                              render={({ field }) => (
                                  <FormItem>
                                    <FormLabel className="text-gray-700 font-medium">Password</FormLabel>
                                    <FormControl>
                                      <div className="relative group">
                                        <Lock className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-gray-400 group-focus-within:text-blue-500 transition-colors duration-200" />
                                        <Input
                                            type={showPassword ? "text" : "password"}
                                            placeholder="Create a strong password"
                                            autoComplete="new-password"
                                            className="pl-10 pr-10 h-12 border-gray-300 focus:border-blue-500 focus:ring-blue-500 transition-all duration-200 hover:border-gray-400"
                                            {...field}
                                        />
                                        <Button
                                            type="button"
                                            variant="ghost"
                                            size="sm"
                                            className="absolute right-0 top-0 h-full px-3 hover:bg-transparent"
                                            onClick={() => setShowPassword(!showPassword)}
                                        >
                                          {showPassword ?
                                              <EyeOff className="h-4 w-4 text-gray-400 hover:text-gray-600 transition-colors duration-200" /> :
                                              <Eye className="h-4 w-4 text-gray-400 hover:text-gray-600 transition-colors duration-200" />
                                          }
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
                                                  <div className={`w-1.5 h-1.5 rounded-full ${
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

                          {/* Confirm Password Field */}
                          <FormField
                              control={registerForm.control}
                              name="confirmPassword"
                              render={({ field }) => (
                                  <FormItem>
                                    <FormLabel className="text-gray-700 font-medium">Confirm Password</FormLabel>
                                    <FormControl>
                                      <div className="relative group">
                                        <Lock className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-gray-400 group-focus-within:text-blue-500 transition-colors duration-200" />
                                        <Input
                                            type={showConfirmPassword ? "text" : "password"}
                                            placeholder="Confirm your password"
                                            autoComplete="new-password"
                                            className="pl-10 pr-10 h-12 border-gray-300 focus:border-blue-500 focus:ring-blue-500 transition-all duration-200 hover:border-gray-400"
                                            {...field}
                                        />
                                        <Button
                                            type="button"
                                            variant="ghost"
                                            size="sm"
                                            className="absolute right-0 top-0 h-full px-3 hover:bg-transparent"
                                            onClick={() => setShowConfirmPassword(!showConfirmPassword)}
                                        >
                                          {showConfirmPassword ?
                                              <EyeOff className="h-4 w-4 text-gray-400 hover:text-gray-600 transition-colors duration-200" /> :
                                              <Eye className="h-4 w-4 text-gray-400 hover:text-gray-600 transition-colors duration-200" />
                                          }
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

                          {registrationSuccess && (
                              <Alert className="border-green-200 bg-green-50">
                                <CheckCircle2 className="h-4 w-4 text-green-600" />
                                <AlertTitle className="text-green-800">Registration Successful!</AlertTitle>
                                <AlertDescription className="text-green-700">
                                  Your account has been created successfully. Redirecting to sign in...
                                </AlertDescription>
                              </Alert>
                          )}

                          {registerError && (
                              <Alert variant="destructive" className="border-red-200 bg-red-50">
                                <AlertCircle className="h-4 w-4" />
                                <AlertTitle>Registration Failed</AlertTitle>
                                <AlertDescription>{registerError}</AlertDescription>
                              </Alert>
                          )}

                          <Button
                              type="submit"
                              className="w-full h-12 text-base font-medium bg-gradient-to-r from-chart-3 to-chart-4 hover:from-chart-3/90 hover:to-chart-4/90 shadow-lg hover:shadow-xl transition-all duration-200 transform hover:scale-[1.02] disabled:transform-none disabled:opacity-50 button-glow"
                              disabled={isRegistering || passwordStrength.score < 5 || registrationSuccess}
                          >
                            {registrationSuccess ? (
                                <>
                                  <CheckCircle2 className="mr-2 h-4 w-4" />
                                  Account Created!
                                </>
                            ) : isRegistering ? (
                                <>
                                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                                  Creating Account...
                                </>
                            ) : (
                                <>
                                  <UserPlus className="mr-2 h-4 w-4" />
                                  Create Account
                                </>
                            )}
                          </Button>
                        </form>
                      </Form>
                    </TabsContent>

                    {/* Social Login Divider */}
                    <div className="relative my-6">
                      <div className="absolute inset-0 flex items-center">
                        <span className="w-full border-t border-border" />
                      </div>
                      <div className="relative flex justify-center text-xs uppercase">
                        <span className="bg-card px-3 text-muted-foreground font-medium">Or continue with</span>
                      </div>
                    </div>

                    {/* Social Login Buttons */}
                    <div className="grid grid-cols-2 gap-3">
                      <Button
                          variant="outline"
                          className="h-12 border-border hover:border-border/80 hover:bg-accent/50 transition-all duration-200 transform hover:scale-[1.02]"
                          onClick={() => window.location.href = `/api/auth/oauth/google`}
                      >
                        <svg className="mr-2 h-5 w-5" viewBox="0 0 24 24">
                          <path fill="#4285F4" d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z"/>
                          <path fill="#34A853" d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z"/>
                          <path fill="#FBBC05" d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l3.66-2.84z"/>
                          <path fill="#EA4335" d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z"/>
                        </svg>
                        Google
                      </Button>
                      <Button
                          variant="outline"
                          className="h-12 border-border hover:border-border/80 hover:bg-accent/50 transition-all duration-200 transform hover:scale-[1.02]"
                          onClick={() => window.location.href = `/api/auth/oauth/github`}
                      >
                        <svg className="mr-2 h-5 w-5 fill-current" viewBox="0 0 24 24">
                          <path d="M12 .297c-6.63 0-12 5.373-12 12 0 5.303 3.438 9.8 8.205 11.385.6.113.82-.258.82-.577 0-.285-.01-1.04-.015-2.04-3.338.724-4.042-1.61-4.042-1.61C4.422 18.07 3.633 17.7 3.633 17.7c-1.087-.744.084-.729.084-.729 1.205.084 1.838 1.236 1.838 1.236 1.07 1.835 2.809 1.305 3.495.998.108-.776.417-1.305.76-1.605-2.665-.3-5.466-1.332-5.466-5.93 0-1.31.465-2.38 1.235-3.22-.135-.303-.54-1.523.105-3.176 0 0 1.005-.322 3.3 1.23.96-.267 1.98-.399 3-.405 1.02.006 2.04.138 3 .405 2.28-1.552 3.285-1.23 3.285-1.23.645 1.653.24 2.873.12 3.176.765.84 1.23 1.91 1.23 3.22 0 4.61-2.805 5.625-5.475 5.92.42.36.81 1.096.81 2.22 0 1.606-.015 2.896-.015 3.286 0 .315.21.69.825.57C20.565 22.092 24 17.592 24 12.297c0-6.627-5.373-12-12-12" />
                        </svg>
                        GitHub
                      </Button>
                    </div>

                  </CardContent>
                </Tabs>

                <CardFooter className="flex flex-col space-y-4 pt-6 gradient-bg">
                  <div className="text-center">
                    <p className="text-xs text-muted-foreground px-4 leading-relaxed">
                      By continuing, you agree to our{' '}
                      <Link href="/" className="text-primary hover:text-primary/80 transition-colors duration-200 underline-offset-4 hover:underline">
                        Terms of Service
                      </Link>{' '}
                      and{' '}
                      <Link href="/" className="text-primary hover:text-primary/80 transition-colors duration-200 underline-offset-4 hover:underline">
                        Privacy Policy
                      </Link>
                    </p>
                  </div>
                </CardFooter>
              </>
          )}
        </Card>

      </main>
  );
}