import { Button, Card, Code, Flex, Heading, Text } from "@radix-ui/themes";
import type { FallbackProps } from "react-error-boundary";

// Offline-safe fallback for the error boundaries (root in main.tsx + route-level
// in router.tsx). It renders only static Radix Themes primitives, so it works
// even when the backend or network is unreachable — it must never itself depend
// on a fetch. "Try again" re-mounts the boundary's subtree to recover from a
// transient error; "Reload" reloads the SPA from its embedded (desktop) or
// served (web) assets, both of which are local to the app origin. For a failed
// data query (not a render crash) use `ui/ErrorState` instead.
export function ErrorFallback({ error, resetErrorBoundary }: FallbackProps) {
  const message = error instanceof Error ? error.message : String(error);
  return (
    <Card role="alert" m="4" size="3">
      <Flex direction="column" gap="3" p="2">
        <Heading size="4">Something went wrong</Heading>
        <Text size="2" color="gray">
          The interface hit an unexpected error. Your data is safe — retrying
          won't lose anything.
        </Text>
        {message ? (
          <Code
            size="1"
            variant="soft"
            color="red"
            style={{ whiteSpace: "pre-wrap", wordBreak: "break-word" }}
          >
            {message}
          </Code>
        ) : null}
        <Flex gap="3" mt="1">
          <Button onClick={resetErrorBoundary}>Try again</Button>
          <Button
            variant="soft"
            color="gray"
            onClick={() => window.location.reload()}
          >
            Reload
          </Button>
        </Flex>
      </Flex>
    </Card>
  );
}
