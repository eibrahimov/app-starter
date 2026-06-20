import { Flex } from "@radix-ui/themes";
import type { ReactNode } from "react";

interface ToolbarProps {
  children: ReactNode;
  className?: string;
}

export function Toolbar({ children, className }: ToolbarProps) {
  return (
    <Flex wrap="wrap" gap="2" className={className}>
      {children}
    </Flex>
  );
}
