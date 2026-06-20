import { Theme } from "@radix-ui/themes";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render } from "@testing-library/react";
import type { ReactElement, ReactNode } from "react";

function freshClient() {
  return new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
}

// Render a component wrapped in a Radix <Theme>. Themes components read Theme
// context to resolve their tokens, so they need this provider in jsdom.
export function renderWithTheme(ui: ReactNode) {
  return render(<Theme>{ui}</Theme>);
}

// Render a component wrapped in a fresh React Query client. Retries are off so
// a failed query surfaces immediately instead of being retried during the test.
export function renderWithClient(ui: ReactElement) {
  return render(
    <QueryClientProvider client={freshClient()}>{ui}</QueryClientProvider>,
  );
}

// A wrapper for renderHook that provides a fresh React Query client.
export function withClient() {
  const client = freshClient();
  return ({ children }: { children: ReactNode }) => (
    <QueryClientProvider client={client}>{children}</QueryClientProvider>
  );
}
