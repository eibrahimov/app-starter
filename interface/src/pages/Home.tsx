import { Card, Code, Container, Flex, Heading, Text } from "@radix-ui/themes";
import { useQuery } from "@tanstack/react-query";
import { api } from "../api/client";

export function HomePage() {
  const health = useQuery({
    queryKey: ["health"],
    queryFn: async () => {
      const { data, error } = await api.GET("/api/health");
      if (error) throw error;
      return data;
    },
  });

  return (
    <Container size="3">
      <Flex direction="column" gap="5">
        <Heading size="7">App Starter</Heading>
        <Text color="gray">
          Rust backend (axum + SQLite) with an embedded React UI, typed
          end-to-end via OpenAPI. Edit <Code>src/items.rs</Code> and run{" "}
          <Code>just typegen</Code> to grow the API.
        </Text>
        <Card>
          {health.isLoading && (
            <Text size="2" color="gray">
              Checking backend…
            </Text>
          )}
          {health.isError && (
            <Text size="2" color="red">
              Backend unreachable. Start it with <Code>cargo run</Code>.
            </Text>
          )}
          {health.data && (
            <Text size="2" color="grass">
              Backend {health.data.status} · v{health.data.version}
            </Text>
          )}
        </Card>
      </Flex>
    </Container>
  );
}
