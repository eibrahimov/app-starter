import type { UseQueryResult } from "@tanstack/react-query";
import type { ReactNode } from "react";
import { cx } from "../ui/cx";
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
      <div className="flex items-center gap-2 text-sm text-zinc-500">
        <Spinner />
        <span>{loadingMessage}</span>
      </div>
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
    <ul className={cx("space-y-2", className)}>{items.map(renderItem)}</ul>
  );
}
