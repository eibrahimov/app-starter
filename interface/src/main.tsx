import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { RouterProvider } from "@tanstack/react-router";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { ErrorBoundary } from "react-error-boundary";
import { Theme, ThemePanel } from "@radix-ui/themes";
import { ErrorFallback } from "./components/app/ErrorFallback";
import { reportBoundaryError } from "./components/app/reportBoundaryError";
import { StartupGate } from "./components/app/StartupGate";
import { ThemeProvider } from "./components/theme/ThemeProvider";
import { router } from "./router";
import { themeConfig } from "./theme/theme.config";
import "./styles.css";

const queryClient = new QueryClient();

const rootElement = document.getElementById("root");
if (!rootElement) {
  throw new Error("Failed to find the #root element to mount into.");
}

createRoot(rootElement).render(
  <StrictMode>
    <ThemeProvider>
      <Theme {...themeConfig}>
        {/* Root error boundary: a render error anywhere below white-screens the
            SPA without it. The fallback is offline-safe so it survives even when
            the backend is unreachable. */}
        <ErrorBoundary
          FallbackComponent={ErrorFallback}
          onError={reportBoundaryError}
        >
          <QueryClientProvider client={queryClient}>
            {/* Hold the UI behind the readiness probe so the desktop webview does
                not render against a sidecar that has not bound its port yet. */}
            <StartupGate>
              <RouterProvider router={router} />
            </StartupGate>
          </QueryClientProvider>
        </ErrorBoundary>
        {import.meta.env.DEV ? <ThemePanel defaultOpen={false} /> : null}
      </Theme>
    </ThemeProvider>
  </StrictMode>,
);
