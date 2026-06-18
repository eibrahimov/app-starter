import { cva } from "class-variance-authority";

interface FilterBarProps<T extends string> {
  options: readonly T[];
  value: T;
  onChange: (value: T) => void;
}

const filterButtonVariants = cva(
  "rounded-md px-3 py-1 text-xs capitalize focus-ring coarse:min-h-11",
  {
    variants: {
      selected: {
        true: "bg-accent text-accent-foreground",
        false: "text-muted-foreground hover:text-foreground",
      },
    },
  },
);

export function FilterBar<T extends string>({
  options,
  value,
  onChange,
}: FilterBarProps<T>) {
  return (
    <div className="flex gap-1">
      {options.map((option) => {
        const selected = value === option;
        return (
          <button
            key={option}
            type="button"
            aria-pressed={selected}
            onClick={() => onChange(option)}
            className={filterButtonVariants({ selected })}
          >
            {option}
          </button>
        );
      })}
    </div>
  );
}
