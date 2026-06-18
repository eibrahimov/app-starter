import type { InputHTMLAttributes } from "react";
import { cn } from "./cn";

type InputProps = InputHTMLAttributes<HTMLInputElement>;

export function Input({ className, ...props }: InputProps) {
  return (
    <input
      className={cn(
        "rounded-md border border-input bg-card px-3 py-2 text-sm focus-ring",
        className,
      )}
      {...props}
    />
  );
}
