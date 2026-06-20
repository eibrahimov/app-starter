import { Flex, Text } from "@radix-ui/themes";
import type { UseQueryResult } from "@tanstack/react-query";
import type { ReactNode } from "react";
import { EmptyState } from "../ui/EmptyState";
import { ErrorState } from "../ui/ErrorState";
import { Spinner } from "../ui/Spinner";

interface DataListProps<T> {
  query: UseQueryResult<T[]>;
  // Returns the list row (e.g. a <Card as="li">) and supplies its own key.
  renderItem: (item: T, index: number) => ReactNode;
  emptyMessage: string;
  loadingMessage?: string;
  errorMessage?: string;
  className?: string;
}

// Owns the loading/error/empty/data triage that pages otherwise copy-paste.
export function DataList<T>({
  query,
  renderItem,
  emptyMessage,
  loadingMessage = "Loading…",
  errorMessage = "Something went wrong.",
  className,
}: DataListProps<T>) {
  const items = query.data ?? [];

  if (query.isLoading) {
    return (
      <Flex aria-busy="true" align="center" gap="2">
        <Spinner />
        <Text size="2" color="gray">
          {loadingMessage}
        </Text>
      </Flex>
    );
  }
  // Only show the error state when there is nothing else to render. With
  // keepPreviousData a failed refetch keeps the prior list visible instead of
  // wiping it out for an error line.
  if (query.isError && items.length === 0) {
    return <ErrorState message={errorMessage} />;
  }
  if (items.length === 0) return <EmptyState message={emptyMessage} />;

  return (
    <Flex asChild direction="column" gap="2" className={className}>
      <ul>{items.map(renderItem)}</ul>
    </Flex>
  );
}
