import { DropdownMenu as Menu } from "@radix-ui/themes";
import type { ReactNode } from "react";

interface DropdownMenuProps {
  trigger: ReactNode;
  children: ReactNode;
}

export function DropdownMenu({ trigger, children }: DropdownMenuProps) {
  return (
    <Menu.Root>
      <Menu.Trigger>{trigger}</Menu.Trigger>
      <Menu.Content>{children}</Menu.Content>
    </Menu.Root>
  );
}

interface DropdownMenuItemProps {
  onSelect?: () => void;
  className?: string;
  children: ReactNode;
}

export function DropdownMenuItem({
  onSelect,
  className,
  children,
}: DropdownMenuItemProps) {
  return (
    <Menu.Item onSelect={onSelect} className={className}>
      {children}
    </Menu.Item>
  );
}
