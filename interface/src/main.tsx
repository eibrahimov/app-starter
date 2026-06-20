import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { RouterProvider } from "@tanstack/react-router";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { Theme, ThemePanel } from "@radix-ui/themes";
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
        <QueryClientProvider client={queryClient}>
          <RouterProvider router={router} />
        </QueryClientProvider>
        {import.meta.env.DEV ? <ThemePanel defaultOpen={false} /> : null}
      </Theme>
    </ThemeProvider>
  </StrictMode>,
);
