import type { HTMLAttributes } from "react";
import { Card as ThemesCard, Flex } from "@radix-ui/themes";

type CardProps = HTMLAttributes<HTMLElement> & {
  as?: "div" | "li";
};

// The row shell shared by list items. Render as <li> inside a <ul>, or <div>
// elsewhere. Built on the Radix Themes Card; the inner Flex provides the
// horizontal row layout (centered, gapped) the list items rely on.
export function Card({ as = "div", children, ...props }: CardProps) {
  const content = (
    <Flex align="center" gap="3">
      {children}
    </Flex>
  );

  if (as === "li") {
    return (
      <ThemesCard asChild size="1" {...props}>
        <li>{content}</li>
      </ThemesCard>
    );
  }

  return (
    <ThemesCard size="1" {...props}>
      {content}
    </ThemesCard>
  );
}
