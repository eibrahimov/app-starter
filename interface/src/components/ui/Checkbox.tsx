import { Checkbox as ThemesCheckbox } from "@radix-ui/themes";

interface CheckboxProps {
  checked: boolean;
  onCheckedChange: (checked: boolean) => void;
  className?: string;
  "aria-label"?: string;
}

export function Checkbox({
  checked,
  onCheckedChange,
  className,
  "aria-label": ariaLabel,
}: CheckboxProps) {
  return (
    <ThemesCheckbox
      checked={checked}
      onCheckedChange={(value) => onCheckedChange(value === true)}
      aria-label={ariaLabel}
      className={className}
      size="3"
      color="grass"
    />
  );
}
