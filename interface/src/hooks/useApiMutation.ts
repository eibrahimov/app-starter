import {
  useMutation,
  useQueryClient,
  type UseMutationResult,
} from "@tanstack/react-query";
import type { MaybeOptionalInit } from "openapi-fetch";
import type { PathsWithMethod } from "openapi-typescript-helpers";
import { api } from "../api/client";
import type { paths } from "../api/schema";

type PostPaths = PathsWithMethod<paths, "post">;
type DeletePaths = PathsWithMethod<paths, "delete">;
type WriteMethod = "post" | "delete";

// The openapi-fetch init ({ body, params }) for a given method + path.
type WriteInit<M extends WriteMethod, P> = M extends "post"
  ? P extends PostPaths
    ? MaybeOptionalInit<paths[P], "post">
    : never
  : P extends DeletePaths
    ? MaybeOptionalInit<paths[P], "delete">
    : never;

interface MutationOptions {
  invalidateKeys?: readonly (readonly unknown[])[];
  onSuccess?: () => void;
}

// Type-safe POST/DELETE wrapper. The mutation variable is the openapi-fetch
// init object ({ body, params }), fully typed against the schema. On success it
// invalidates each key in invalidateKeys, then runs the caller's onSuccess.
export function useApiMutation<
  M extends WriteMethod,
  P extends M extends "post" ? PostPaths : DeletePaths,
>(
  method: M,
  path: P,
  options?: MutationOptions,
): UseMutationResult<unknown, unknown, WriteInit<M, P>> {
  const queryClient = useQueryClient();
  // openapi-fetch throws the typed error response body, which is not guaranteed
  // to be an Error instance — so the error channel is `unknown`, not `Error`.
  return useMutation<unknown, unknown, WriteInit<M, P>>({
    mutationFn: async (init: WriteInit<M, P>) => {
      // Casts bridge the generic path/init through openapi-fetch's overloads;
      // the public signature keeps callers fully type-checked.
      const call = method === "post" ? api.POST : api.DELETE;
      const { data, error } = await call(path as never, init as never);
      if (error) throw error;
      return data;
    },
    onSuccess: () => {
      for (const key of options?.invalidateKeys ?? []) {
        queryClient.invalidateQueries({ queryKey: key });
      }
      options?.onSuccess?.();
    },
  });
}
