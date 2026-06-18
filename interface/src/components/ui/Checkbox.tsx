import * as RadixCheckbox from "@radix-ui/react-checkbox";
import { cn } from "./cn";

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
      className={cn(
        "flex size-4 coarse:size-6 items-center justify-center rounded border border-input bg-card focus-ring data-[state=checked]:border-success data-[state=checked]:bg-success",
        className,
      )}
    >
      <RadixCheckbox.Indicator className="text-success-foreground">
        <svg
          viewBox="0 0 12 12"
          className="size-3"
          fill="none"
          stroke="currentColor"
          strokeWidth={2}
          strokeLinecap="round"
          strokeLinejoin="round"
          aria-hidden="true"
        >
          <path d="M2.5 6.5 5 9l4.5-5" />
        </svg>
      </RadixCheckbox.Indicator>
    </RadixCheckbox.Root>
  );
}
