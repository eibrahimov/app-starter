import { Text } from "@radix-ui/themes";

export function EmptyState({ message }: { message: string }) {
  return (
    <Text as="p" size="2" color="gray">
      {message}
    </Text>
  );
}
