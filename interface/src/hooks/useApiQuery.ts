import {
  useQuery,
  type UseQueryOptions,
  type UseQueryResult,
} from "@tanstack/react-query";
import type { MaybeOptionalInit, MethodResponse } from "openapi-fetch";
import type { PathsWithMethod } from "openapi-typescript-helpers";
import { api } from "../api/client";
import type { paths } from "../api/schema";

type Client = typeof api;
type GetPaths = PathsWithMethod<paths, "get">;
type Data<P extends GetPaths> = MethodResponse<Client, "get", P>;

interface UseApiQueryOptions<P extends GetPaths> {
  queryKey?: readonly unknown[];
  enabled?: boolean;
  // Forwarded to React Query. Pass `keepPreviousData` so a list keeps showing
  // the prior result while a new query key (e.g. a changed filter) refetches,
  // instead of flickering back to the loading state.
  placeholderData?: UseQueryOptions<Data<P>>["placeholderData"];
}

// Type-safe GET wrapper over openapi-fetch + React Query. `data` is inferred
// from the OpenAPI schema for the given path. Default query key is
// [path, query]; pass an explicit key to match a resource's invalidation scope.
export function useApiQuery<P extends GetPaths>(
  path: P,
  init?: MaybeOptionalInit<paths[P], "get">,
  options?: UseApiQueryOptions<P>,
): UseQueryResult<Data<P>> {
  const queryParams = (init as { params?: { query?: unknown } } | undefined)
    ?.params?.query;
  return useQuery({
    queryKey: options?.queryKey ?? [path, queryParams ?? null],
    queryFn: async (): Promise<Data<P>> => {
      // Cast bridges the generic path through openapi-fetch's overloads; the
      // public signature above keeps callers fully type-checked.
      const { data, error } = await api.GET(path, init as never);
      if (error) throw error;
      return data as Data<P>;
    },
    enabled: options?.enabled,
    placeholderData: options?.placeholderData,
  });
}
