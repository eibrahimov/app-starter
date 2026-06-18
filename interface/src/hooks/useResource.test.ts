import { renderHook, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { withClient } from "../test-utils";
import { useResource } from "./useResource";

const { getMock, postMock } = vi.hoisted(() => ({
  getMock: vi.fn(),
  postMock: vi.fn(),
}));
vi.mock(
  "../api/client",
  () =>
    ({
      api: { GET: getMock, POST: postMock, DELETE: vi.fn() },
    }) as unknown as typeof import("../api/client"),
);

describe("useResource", () => {
  it("surfaces list data from the API", async () => {
    getMock.mockResolvedValue({
      data: [{ id: "1", title: "write tests", done: false }],
      error: undefined,
    });

    const { result } = renderHook(
      () =>
        useResource({
          key: "items",
          listPath: "/api/v1/items",
          createPath: "/api/v1/items",
        }),
      { wrapper: withClient() },
    );

    await waitFor(() => expect(result.current.list.data).toBeTruthy());
    expect(result.current.list.data?.[0]?.title).toBe("write tests");
  });

  it("invalidates the resource key after a create", async () => {
    getMock.mockResolvedValue({ data: [], error: undefined });
    postMock.mockResolvedValue({ data: { id: "2" }, error: undefined });

    const { result } = renderHook(
      () =>
        useResource({
          key: "items",
          listPath: "/api/v1/items",
          createPath: "/api/v1/items",
        }),
      { wrapper: withClient() },
    );

    await waitFor(() => expect(result.current.list.isSuccess).toBe(true));
    result.current.create?.mutate({ body: { title: "new" } });

    await waitFor(() => expect(postMock).toHaveBeenCalledTimes(1));
    expect(getMock.mock.calls.length).toBeGreaterThan(1);
  });

  it("runs the onSuccess callback after a create", async () => {
    getMock.mockResolvedValue({ data: [], error: undefined });
    postMock.mockResolvedValue({ data: { id: "3" }, error: undefined });
    const onSuccess = vi.fn();

    const { result } = renderHook(
      () =>
        useResource({
          key: "items",
          listPath: "/api/v1/items",
          createPath: "/api/v1/items",
          onSuccess,
        }),
      { wrapper: withClient() },
    );

    await waitFor(() => expect(result.current.list.isSuccess).toBe(true));
    result.current.create?.mutate({ body: { title: "new" } });

    await waitFor(() => expect(onSuccess).toHaveBeenCalledTimes(1));
  });
});
