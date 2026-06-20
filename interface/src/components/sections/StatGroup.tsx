import { Text } from "@radix-ui/themes";

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
    <Text as="p" size="1" color="gray">
      {stats.map((stat) => `${stat.value} ${stat.label}`).join(" / ")}
    </Text>
  );
}
