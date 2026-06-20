import { Badge as ThemesBadge } from "@radix-ui/themes";
import type { ComponentPropsWithoutRef } from "react";

// The tones the app speaks, kept identical to the previous cva API so callers
// (e.g. pages/Posts) do not change. Each maps to an accessible Radix Themes
// color; the e2e a11y test audits these for contrast.
export type BadgeTone = "neutral" | "emerald" | "amber" | "red" | "zinc";

const toneColor: Record<
  BadgeTone,
  ComponentPropsWithoutRef<typeof ThemesBadge>["color"]
> = {
  neutral: "gray",
  emerald: "grass",
  amber: "amber",
  red: "red",
  zinc: "gray",
};

type BadgeProps = Omit<
  ComponentPropsWithoutRef<typeof ThemesBadge>,
  "color"
> & {
  tone?: BadgeTone;
};

export function Badge({ tone = "neutral", ...props }: BadgeProps) {
  return <ThemesBadge color={toneColor[tone]} {...props} />;
}
