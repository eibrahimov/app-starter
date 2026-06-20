import { Spinner as ThemesSpinner } from "@radix-ui/themes";

export function Spinner() {
  return (
    <span role="status" aria-label="Loading">
      <ThemesSpinner size="2" />
    </span>
  );
}
