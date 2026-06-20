import { Button, Flex } from "@radix-ui/themes";

interface FilterBarProps<T extends string> {
  options: readonly T[];
  value: T;
  onChange: (value: T) => void;
}

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
