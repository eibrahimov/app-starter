import { Flex, Heading } from "@radix-ui/themes";
import type { ReactNode } from "react";

interface PageHeaderProps {
  title: string;
  children?: ReactNode;
}

export function PageHeader({ title, children }: PageHeaderProps) {
  return (
    <Flex align="baseline" justify="between" gap="2" wrap="wrap">
      <Heading as="h1" size="6" weight="bold">
        {title}
      </Heading>
      {children}
    </Flex>
  );
}
