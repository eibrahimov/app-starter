import * as RadixDropdown from "@radix-ui/react-dropdown-menu";
import type { ReactNode } from "react";
import { cx } from "./cx";

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
          className="min-w-[8rem] rounded-md border border-zinc-800 bg-zinc-900 p-1 text-zinc-100 shadow-xl outline-none"
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
      className={cx(
        "cursor-pointer rounded px-2 py-1 text-sm text-zinc-300 outline-none data-[highlighted]:bg-zinc-800 data-[highlighted]:text-zinc-100",
        className,
      )}
    >
      {children}
    </RadixDropdown.Item>
  );
}
