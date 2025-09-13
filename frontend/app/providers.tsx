import {ThemeProvider as NextThemesProvider, ThemeProviderProps} from 'next-themes';
import { AuthProvider } from '@/lib/auth';
import { Toaster } from '@/components/ui/toaster';

export function AppProvider({ children, ...props }: ThemeProviderProps) {
  return (
      <NextThemesProvider {...props}>
        <AuthProvider>
          {children}
          <Toaster />
        </AuthProvider>
      </NextThemesProvider>
  );
}