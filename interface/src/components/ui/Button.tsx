import type { ButtonHTMLAttributes } from "react";
import { cx } from "./cx";

type ButtonVariant = "primary" | "ghost" | "danger";

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
}

const variants: Record<ButtonVariant, string> = {
  primary:
    "rounded-md bg-zinc-100 px-4 py-2 text-sm font-medium text-zinc-900 hover:bg-white",
  ghost: "text-xs text-zinc-500 hover:text-zinc-300",
  danger: "text-xs text-zinc-500 hover:text-red-400",
};

// Defaults to type="button" so callers satisfy the explicit-type rule for free;
// pass type="submit" for form submissions.
export function Button({
  variant = "primary",
  type,
  className,
  ...props
}: ButtonProps) {
  return (
    <button
      type={type ?? "button"}
      className={cx(variants[variant], "disabled:opacity-50", className)}
      {...props}
    />
  );
}
