import * as RadixDropdown from "@radix-ui/react-dropdown-menu";
import type { ReactNode } from "react";
import { cn } from "./cn";

interface DropdownMenuProps {
  trigger: ReactNode;
  children: ReactNode;
}

export function DropdownMenu({ trigger, children }: DropdownMenuProps) {
  return (
    <RadixDropdown.Root>
      <RadixDropdown.Trigger asChild>{trigger}</RadixDropdown.Trigger>
      <RadixDropdown.Portal>
        <RadixDropdown.Content
          sideOffset={4}
          className="min-w-[8rem] rounded-md border border-border bg-popover p-1 text-popover-foreground shadow-xl outline-none"
        >
          {children}
        </RadixDropdown.Content>
      </RadixDropdown.Portal>
    </RadixDropdown.Root>
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
    <RadixDropdown.Item
      onSelect={onSelect}
      className={cn(
        "cursor-pointer rounded px-2 py-1 text-sm text-muted-foreground outline-none data-[highlighted]:bg-accent data-[highlighted]:text-accent-foreground",
        className,
      )}
    >
      {children}
    </RadixDropdown.Item>
  );
}
