import { Callout, Text } from "@radix-ui/themes";
import { ExclamationTriangleIcon } from "../../theme/icons";

export function ErrorState({ message }: { message: string }) {
  // role="alert" gives the message an assertive live region so screen readers
  // announce it when an error replaces the list.
  return (
    <Callout.Root role="alert" color="red" size="1">
      <Callout.Icon>
        <ExclamationTriangleIcon />
      </Callout.Icon>
      <Callout.Text>
        <Text size="2">{message}</Text>
      </Callout.Text>
    </Callout.Root>
  );
}
