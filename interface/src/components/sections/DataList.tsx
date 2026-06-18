import type { UseQueryResult } from "@tanstack/react-query";
import type { ReactNode } from "react";
import { cx } from "../ui/cx";
import { EmptyState } from "../ui/EmptyState";
import { ErrorState } from "../ui/ErrorState";

interface DataListProps<T> {
  query: UseQueryResult<T[] | undefined>;
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
  if (query.isLoading) return <EmptyState message={loadingMessage} />;
  if (query.isError) return <ErrorState message={errorMessage} />;

  const items = query.data ?? [];
  if (items.length === 0) return <EmptyState message={emptyMessage} />;

  return (
    <ul className={cx("space-y-2", className)}>{items.map(renderItem)}</ul>
  );
}
