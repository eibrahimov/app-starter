import { useQuery, type UseQueryResult } from "@tanstack/react-query";
import type { MaybeOptionalInit, MethodResponse } from "openapi-fetch";
import type { PathsWithMethod } from "openapi-typescript-helpers";
import { api } from "../api/client";
import type { paths } from "../api/schema";

type Client = typeof api;
type GetPaths = PathsWithMethod<paths, "get">;

interface UseApiQueryOptions {
  queryKey?: readonly unknown[];
  enabled?: boolean;
}

// Type-safe GET wrapper over openapi-fetch + React Query. `data` is inferred
// from the OpenAPI schema for the given path. Default query key is
// [path, query]; pass an explicit key to match a resource's invalidation scope.
export function useApiQuery<P extends GetPaths>(
  path: P,
  init?: MaybeOptionalInit<paths[P], "get">,
  options?: UseApiQueryOptions,
): UseQueryResult<MethodResponse<Client, "get", P>> {
  const queryParams = (init as { params?: { query?: unknown } } | undefined)
    ?.params?.query;
  return useQuery({
    queryKey: options?.queryKey ?? [path, queryParams ?? null],
    queryFn: async (): Promise<MethodResponse<Client, "get", P>> => {
      // Cast bridges the generic path through openapi-fetch's overloads; the
      // public signature above keeps callers fully type-checked.
      const { data, error } = await api.GET(path, init as never);
      if (error) throw error;
      return data as MethodResponse<Client, "get", P>;
    },
    enabled: options?.enabled,
  });
}
