import type { ThemeProps } from "@radix-ui/themes";

// Single source of truth for global theming. Restyle the whole app here, or ask
// an agent in natural language ("accent teal, large radius, 105% scaling").
// The full vocabulary (every prop and its allowed values) lives in
// docs/radix-reference.md. Do NOT add `appearance` here: light/dark is driven by
// the `.dark` class on <html> (ThemeProvider + the pre-hydration script in
// index.html), which Radix Themes reads natively. Passing `appearance` would
// cause a flash on load.
export const themeConfig = {
  accentColor: "indigo",
  grayColor: "auto",
  panelBackground: "translucent",
  radius: "medium",
  scaling: "100%",
} satisfies Partial<ThemeProps>;
