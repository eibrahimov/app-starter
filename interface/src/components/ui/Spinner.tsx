import { Spinner as ThemesSpinner } from "@radix-ui/themes";

export function Spinner({ className }: { className?: string }) {
  return (
    <span role="status" aria-label="Loading" className={className}>
      <ThemesSpinner size="2" />
    </span>
  );
}
