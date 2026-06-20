import { Flex } from "@radix-ui/themes";
import type { ReactNode } from "react";

interface ToolbarProps {
  children: ReactNode;
}

export function Toolbar({ children }: ToolbarProps) {
  return (
    <Flex wrap="wrap" gap="2">
      {children}
    </Flex>
  );
}
