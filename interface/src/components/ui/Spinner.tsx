import { cx } from "./cx";

export function Spinner({ className }: { className?: string }) {
  return (
    <span
      role="status"
      aria-label="Loading"
      className={cx(
        "inline-block size-4 animate-spin rounded-full border-2 border-zinc-700 border-t-zinc-300",
        className,
      )}
    />
  );
}
