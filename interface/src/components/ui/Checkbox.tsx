import * as RadixCheckbox from "@radix-ui/react-checkbox";
import { cx } from "./cx";

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
    <RadixCheckbox.Root
      checked={checked}
      onCheckedChange={(value) => onCheckedChange(value === true)}
      aria-label={ariaLabel}
      className={cx(
        "flex size-4 items-center justify-center rounded border border-zinc-700 bg-zinc-900 outline-none focus-visible:border-zinc-500 data-[state=checked]:border-emerald-500 data-[state=checked]:bg-emerald-500",
        className,
      )}
    >
      <RadixCheckbox.Indicator className="text-xs leading-none text-zinc-900">
        ✓
      </RadixCheckbox.Indicator>
    </RadixCheckbox.Root>
  );
}
