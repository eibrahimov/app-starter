interface Stat {
  label: string;
  value: number | string;
}

interface StatGroupProps {
  stats: Stat[];
}

// Renders a compact summary line, e.g. "1 draft / 0 published / 2 archived".
export function StatGroup({ stats }: StatGroupProps) {
  return (
    <p className="text-xs text-zinc-500">
      {stats.map((stat, index) => (
        <span key={stat.label}>
          {index > 0 && " / "}
          {stat.value} {stat.label}
        </span>
      ))}
    </p>
  );
}
