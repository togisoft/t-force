import { NextResponse } from 'next/server';
import type { NextRequest } from 'next/server';
import { jwtDecode } from 'jwt-decode';

interface JwtPayload {
  exp: number;
}

// List of pages accessible without authentication
const publicPages = [
  '/',
  '/oauth/callback',
  '/forgot-password',
  '/reset-password',
];

export function middleware(request: NextRequest) {
  const path = request.nextUrl.pathname;
  const token = request.cookies.get('auth_token')?.value;

  console.log(`\n--- [Middleware Execution Start] ---`);
  console.log(`[Middleware] Requested page: ${path}`);

  let hasValidToken = false;
  if (token) {
    try {
      const decoded = jwtDecode<JwtPayload>(token);
      // Check if token is not expired
      if (decoded.exp * 1000 > Date.now()) {
        hasValidToken = true;
      }
    } catch (error) {
      console.error("[Middleware] Failed to decode token:", error);
    }
  }

  console.log(`[Middleware] Has valid token?: ${hasValidToken}`);

  // Check if requested page is public
  const isPublicPage = publicPages.includes(path);
  console.log(`[Middleware] Is public page?: ${isPublicPage}`);

  // SCENARIO 1: User is AUTHENTICATED and tries to access a public page
  // Redirect them to the dashboard
  if (hasValidToken && isPublicPage) {
    console.log("-> [DECISION]: Authenticated user on a public page. Redirecting to /dashboard.");
    return NextResponse.redirect(new URL('/dashboard', request.url));
  }

  // SCENARIO 2: User is NOT AUTHENTICATED and tries to access a protected page
  // Redirect to login page
  if (!hasValidToken && !isPublicPage) {
    console.log("-> [DECISION]: Not authenticated user on a protected page. Redirecting to login (/).");
    const response = NextResponse.redirect(new URL('/', request.url));

    // Remove any old auth cookies
    if (token) {
      response.cookies.delete('auth_token');
      response.cookies.delete('auth_user');
    }
    return response;
  }

  // Allow the request to continue
  console.log("-> [DECISION]: Request allowed to proceed.");
  console.log(`--- [Middleware Execution End] ---\n`);
  return NextResponse.next();
}

export const config = {
  matcher: [
    '/((?!api|_next/static|_next/image|favicon.ico).*)',
  ],
};
