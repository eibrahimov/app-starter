import { cx } from "../ui/cx";

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
    <div className="flex gap-1">
      {options.map((option) => {
        const selected = value === option;
        return (
          <button
            key={option}
            type="button"
            aria-pressed={selected}
            onClick={() => onChange(option)}
            className={cx(
              "rounded-md px-3 py-1 text-xs capitalize",
              selected
                ? "bg-zinc-800 text-zinc-100"
                : "text-zinc-500 hover:text-zinc-300",
            )}
          >
            {option}
          </button>
        );
      })}
    </div>
  );
}
