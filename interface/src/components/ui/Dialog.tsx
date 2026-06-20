import * as RadixDialog from "@radix-ui/react-dialog";
import type { ReactNode } from "react";
import { VisuallyHidden } from "./VisuallyHidden";

interface DialogProps {
  open?: boolean;
  onOpenChange?: (open: boolean) => void;
  trigger?: ReactNode;
  title: string;
  description?: string;
  children: ReactNode;
}

export function Dialog({
  open,
  onOpenChange,
  trigger,
  title,
  description,
  children,
}: DialogProps) {
  return (
    <RadixDialog.Root open={open} onOpenChange={onOpenChange}>
      {trigger && <RadixDialog.Trigger asChild>{trigger}</RadixDialog.Trigger>}
      <RadixDialog.Portal>
        <RadixDialog.Overlay className="fixed inset-0 bg-black/60" />
        <RadixDialog.Content className="-translate-x-1/2 -translate-y-1/2 fixed top-1/2 left-1/2 w-full max-w-md space-y-3 rounded-md border border-border bg-popover p-4 text-popover-foreground shadow-xl outline-none">
          <RadixDialog.Title className="text-sm font-semibold tracking-tight">
            {title}
          </RadixDialog.Title>
          {description ? (
            <RadixDialog.Description className="text-sm text-muted-foreground">
              {description}
            </RadixDialog.Description>
          ) : (
            <VisuallyHidden>
              <RadixDialog.Description>{title}</RadixDialog.Description>
            </VisuallyHidden>
          )}
          {children}
        </RadixDialog.Content>
      </RadixDialog.Portal>
    </RadixDialog.Root>
  );
}
