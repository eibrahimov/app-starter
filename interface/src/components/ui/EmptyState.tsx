import type { ReactNode } from "react";

interface EmptyStateProps {
  message: string;
  children?: ReactNode;
}

export function EmptyState({ message, children }: EmptyStateProps) {
  return (
    <p className="text-sm text-zinc-500">
      {message}
      {children}
    </p>
  );
}
