/** @type {import('next').NextConfig} */
const path = require('path');

const nextConfig = {
  reactStrictMode: true,
  output: 'standalone', // Optimized for Docker deployments
  // Set the output file tracing root to resolve the workspace root warning
  outputFileTracingRoot: path.join(__dirname, '..'),
  // Remove console logs in production
  compiler: {
    removeConsole: process.env.NODE_ENV === 'production' ? {
      exclude: ['error', 'warn']
    } : false,
  },
  experimental: {
    serverActions: {
      allowedOrigins: ['localhost:3000', 'localhost:8080'],
    },
  },
  // Configure rewrites to proxy API requests to the backend in development
  async rewrites() {
    const apiUrl = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080';
    return [
      {
        // Exclude NextAuth.js routes from being proxied to the backend
        source: '/api/:path*',
        destination: `${apiUrl}/api/:path*`,
        has: [
          {
            type: 'header',
            key: 'x-skip-nextauth',
            value: '1',
          },
        ],
      },
      {
        // Explicitly proxy chat API routes to backend (no special header required)
        source: '/api/chat/:path*',
        destination: `${apiUrl}/api/chat/:path*`,
      },
      {
        // Proxy specific API routes to the backend
        source: '/api/user/:path*',
        destination: `${apiUrl}/api/user/:path*`,
      },
      {
        // Proxy specific backend auth routes
        source: '/api/auth/login',
        destination: `${apiUrl}/api/auth/login`,
      },
      {
        source: '/api/auth/register',
        destination: `${apiUrl}/api/auth/register`,
      },
      {
        source: '/api/auth/logout',
        destination: `${apiUrl}/api/auth/logout`,
      },
      {
        source: '/api/auth/sync',
        destination: `${apiUrl}/api/auth/sync`,
      },
      {
        // New: Proxy chat video upload endpoint explicitly
        source: '/api/chat/upload-video',
        destination: `${apiUrl}/api/chat/upload-video`,
      },
      {
        // New: Proxy chat video serving endpoint
        source: '/api/chat/video/:path*',
        destination: `${apiUrl}/api/chat/video/:path*`,
      },
    ];
  },
};

module.exports = nextConfig;