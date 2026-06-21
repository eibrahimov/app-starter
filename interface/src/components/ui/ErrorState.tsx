import { Callout, Text } from "@radix-ui/themes";
import { ExclamationTriangleIcon } from "../../theme/icons";

export function ErrorState({ message }: { message: string }) {
  // role="alert" gives the message an assertive live region so screen readers
  // announce it when an error replaces the list. This is the inline state for a
  // failed data query; for a render crash, the `components/app/ErrorFallback`
  // boundary fallback (with retry/reload) is the counterpart.
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
