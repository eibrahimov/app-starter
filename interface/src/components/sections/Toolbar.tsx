import type { ReactNode } from "react";
import { cx } from "../ui/cx";

interface ToolbarProps {
  children: ReactNode;
  className?: string;
}

export function Toolbar({ children, className }: ToolbarProps) {
  return <div className={cx("flex gap-2", className)}>{children}</div>;
}
