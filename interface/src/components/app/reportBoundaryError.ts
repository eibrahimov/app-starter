import type { ErrorInfo } from "react";

// The single `onError` seam shared by the root (main.tsx) and route-level
// (router.tsx) error boundaries. Today it only logs to the console; this is the
// one place to wire opt-in crash reporting later (e.g. tauri-plugin-sentry,
// shipped OFF by default per docs/desktop-features.md §5) without touching the
// boundary call sites. `error` is `unknown` because a thrown value need not be
// an Error — react-error-boundary forwards it as-is.
export function reportBoundaryError(error: unknown, info: ErrorInfo): void {
  console.error(
    "UI error boundary caught an error:",
    error,
    info.componentStack,
  );
}
