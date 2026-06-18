import { cva, type VariantProps } from "class-variance-authority";
import type { ButtonHTMLAttributes } from "react";
import { cn } from "./cn";

// The text variants share a muted base and differ only in their hover accent,
// so a caller never has to patch the hover colour through className. Colours are
// semantic tokens, so both light and dark themes resolve from one definition.
export const buttonVariants = cva(
  "inline-flex items-center justify-center focus-ring transition-colors disabled:opacity-50 coarse:min-h-11",
  {
    variants: {
      variant: {
        primary:
          "rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:opacity-90",
        ghost: "text-xs text-muted-foreground hover:text-foreground",
        danger: "text-xs text-muted-foreground hover:text-destructive",
        success: "text-xs text-muted-foreground hover:text-success",
        warning: "text-xs text-muted-foreground hover:text-warning",
      },
    },
    defaultVariants: { variant: "primary" },
  },
);

export type ButtonVariant = NonNullable<
  VariantProps<typeof buttonVariants>["variant"]
>;

interface ButtonProps
  extends ButtonHTMLAttributes<HTMLButtonElement>,
    VariantProps<typeof buttonVariants> {}

// Defaults to type="button" so callers satisfy the explicit-type rule for free;
// pass type="submit" for form submissions.
export function Button({ variant, type, className, ...props }: ButtonProps) {
  return (
    <button
      type={type ?? "button"}
      className={cn(buttonVariants({ variant }), className)}
      {...props}
    />
  );
}
