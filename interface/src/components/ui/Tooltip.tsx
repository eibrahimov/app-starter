import { Tooltip as ThemesTooltip } from "@radix-ui/themes";
import type { ReactNode } from "react";

interface TooltipProps {
  label: string;
  children: ReactNode;
}

export function Tooltip({ label, children }: TooltipProps) {
  return <ThemesTooltip content={label}>{children}</ThemesTooltip>;
}
