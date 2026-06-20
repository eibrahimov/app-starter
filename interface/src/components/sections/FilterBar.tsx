import { Button, Flex } from "@radix-ui/themes";

interface FilterBarProps<T extends string> {
  options: readonly T[];
  value: T;
  onChange: (value: T) => void;
}

// Uses the raw Themes `Button` (not the wrapped `ui/Button`) on purpose: the
// pill toggles need the `soft` variant and `highContrast` selected state, which
// the wrapped Button deliberately does not expose (it only maps the app's
// semantic variants). This is the one section that reaches past the primitive.
export function FilterBar<T extends string>({
  options,
  value,
  onChange,
}: FilterBarProps<T>) {
  return (
    <Flex gap="1" align="center">
      {options.map((option) => {
        const selected = value === option;
        return (
          <Button
            key={option}
            type="button"
            size="2"
            variant={selected ? "soft" : "ghost"}
            highContrast={selected}
            aria-pressed={selected}
            onClick={() => onChange(option)}
            style={{ textTransform: "capitalize" }}
          >
            {option}
          </Button>
        );
      })}
    </Flex>
  );
}
