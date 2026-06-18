import type { HTMLAttributes } from "react";
import { cn } from "./cn";

type CardProps = HTMLAttributes<HTMLElement> & {
  as?: "div" | "li";
};

// The row shell shared by list items. Render as <li> inside a <ul>, or <div>
// elsewhere.
export function Card({ as = "div", className, ...props }: CardProps) {
  const Tag = as;
  return (
    <Tag
      className={cn(
        "flex items-center gap-3 rounded-md border border-border px-3 py-2",
        className,
      )}
      {...props}
    />
  );
}
