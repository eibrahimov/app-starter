import type { HTMLAttributes } from "react";
import { cx } from "./cx";

export type BadgeTone = "neutral" | "emerald" | "amber" | "red" | "zinc";

interface BadgeProps extends HTMLAttributes<HTMLSpanElement> {
  tone?: BadgeTone;
}

const tones: Record<BadgeTone, string> = {
  neutral: "text-zinc-400",
  emerald: "text-emerald-400",
  amber: "text-amber-400",
  red: "text-red-400",
  zinc: "text-zinc-500",
};

export function Badge({ tone = "neutral", className, ...props }: BadgeProps) {
  return <span className={cx("text-xs", tones[tone], className)} {...props} />;
}
