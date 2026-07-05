import { withAuth } from "next-auth/middleware";
import { NextResponse } from "next/server";
import type { NextRequest } from "next/server";

// Set AUTH_BYPASS=true in .env.local to skip auth in development. This runs
// server-side (the proxy), so it's a plain server env var — NOT NEXT_PUBLIC_ —
// and never ships in the client bundle.
function devBypass(req: NextRequest) {
  if (process.env.AUTH_BYPASS === "true") {
    return NextResponse.next();
  }
}

const authMiddleware = withAuth({ pages: { signIn: "/auth/signin" } });

export default function proxy(req: NextRequest, event: any) {
  const bypass = devBypass(req);
  if (bypass) return bypass;
  return (authMiddleware as any)(req, event);
}

export const config = {
  matcher: ["/((?!_next/static|_next/image|favicon.ico).*)"],
};
