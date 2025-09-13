// app/layout.tsx
import type { Metadata } from 'next';
import { Inter } from 'next/font/google';
import './globals.css';
import { AppProvider } from './providers';

const inter = Inter({
    subsets: ['latin'],
    variable: '--font-sans',
});

export const metadata: Metadata = {
    title: 'T-Force - Secure Communication Platform',
    description: 'A modern authentication system built with Next.js 15 and Rust',
};

export default function RootLayout({
                                       children,
                                   }: {
    children: React.ReactNode;
}) {
    return (
        <html lang="en" suppressHydrationWarning>
        <body className={`${inter.variable} font-sans min-h-screen bg-background text-foreground antialiased`}>
        <AppProvider
            attribute="class"
            defaultTheme="system"
            enableSystem
            disableTransitionOnChange
        >
            <div className="min-h-screen w-full mx-auto max-w-screen-2xl overflow-hidden">{children}</div>
        </AppProvider>
        </body>
        </html>
    );
}