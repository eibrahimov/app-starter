import { Checkbox as ThemesCheckbox } from "@radix-ui/themes";

interface CheckboxProps {
  checked: boolean;
  onCheckedChange: (checked: boolean) => void;
  "aria-label"?: string;
}

// `color="grass"` is intentional: the checkbox reads as a "done"/success control
// (e.g. the Items todo list) regardless of the global accent, so it is pinned to
// green rather than inheriting `theme.config.ts`'s `accentColor`. Re-theming the
// app does not change it; edit this line if you want it to follow the accent.
export function Checkbox({
  checked,
  onCheckedChange,
  "aria-label": ariaLabel,
}: CheckboxProps) {
  return (
    <ThemesCheckbox
      checked={checked}
      onCheckedChange={(value) => onCheckedChange(value === true)}
      aria-label={ariaLabel}
      size="3"
      color="grass"
    />
  );
}
