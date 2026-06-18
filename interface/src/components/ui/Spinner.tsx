import { cn } from "./cn";

export function Spinner({ className }: { className?: string }) {
  return (
    <span
      role="status"
      aria-label="Loading"
      className={cn(
        "inline-block size-4 animate-spin rounded-full border-2 border-muted border-t-foreground motion-reduce:animate-none",
        className,
      )}
    />
  );
}
