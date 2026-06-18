import type { MaybeOptionalInit } from "openapi-fetch";
import type { PathsWithMethod } from "openapi-typescript-helpers";
import type { paths } from "../api/schema";
import { useApiMutation } from "./useApiMutation";
import { useApiQuery } from "./useApiQuery";

type GetPaths = PathsWithMethod<paths, "get">;
type PostPaths = PathsWithMethod<paths, "post">;

interface UseResourceOptions<
  ListPath extends GetPaths,
  CreatePath extends PostPaths,
> {
  // Cache scope and first query-key segment, e.g. "items".
  key: string;
  listPath: ListPath;
  createPath: CreatePath;
  listInit?: MaybeOptionalInit<paths[ListPath], "get">;
}

// Convenience layer over useApiQuery + useApiMutation for the common
// list + create case. Mutations invalidate the broad [key] prefix, so any
// sub-keyed queries (lists, stats) refresh together. Reach for the wrappers
// directly when a page needs filtered lists, stats, lifecycle actions, or a
// read-only list.
export function useResource<
  ListPath extends GetPaths,
  CreatePath extends PostPaths,
>(options: UseResourceOptions<ListPath, CreatePath>) {
  const list = useApiQuery(options.listPath, options.listInit, {
    queryKey: [options.key],
  });
  const create = useApiMutation("post", options.createPath, {
    invalidateKeys: [[options.key]],
  });
  return { list, create };
}
