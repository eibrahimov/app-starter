import { Dialog as RadixDialog, VisuallyHidden } from "@radix-ui/themes";
import type { ReactNode } from "react";

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
      {trigger && <RadixDialog.Trigger>{trigger}</RadixDialog.Trigger>}
      <RadixDialog.Content maxWidth="448px">
        <RadixDialog.Title>{title}</RadixDialog.Title>
        {description ? (
          <RadixDialog.Description>{description}</RadixDialog.Description>
        ) : (
          <VisuallyHidden>
            <RadixDialog.Description>{title}</RadixDialog.Description>
          </VisuallyHidden>
        )}
        {children}
      </RadixDialog.Content>
    </RadixDialog.Root>
  );
}
