import type { ReactNode } from "react";
import { cn } from "../ui/cn";

interface ToolbarProps {
  children: ReactNode;
  className?: string;
}

export function Toolbar({ children, className }: ToolbarProps) {
  return (
    <div className={cn("flex flex-wrap gap-2", className)}>{children}</div>
  );
}
