"use client";

import { signIn } from "next-auth/react";
import { useSearchParams } from "next/navigation";
import { Suspense } from "react";

function SignInContent() {
  const searchParams = useSearchParams();
  const callbackUrl = searchParams.get("callbackUrl") ?? "/";

  return (
    <main className="flex min-h-screen items-center justify-center bg-zinc-100">
      <div className="flex flex-col items-center gap-6 rounded-lg bg-white p-10 shadow-md">
        <h1 className="text-2xl font-semibold text-zinc-800">Libretto</h1>
        <p className="text-sm text-zinc-500">Technical Service Information</p>
        <button
          onClick={() => signIn("pocket-id", { callbackUrl })}
          className="rounded bg-zinc-800 px-6 py-2 text-sm text-white hover:bg-zinc-700"
        >
          Sign in
        </button>
      </div>
    </main>
  );
}

export default function SignInPage() {
  return (
    <Suspense>
      <SignInContent />
    </Suspense>
  );
}
