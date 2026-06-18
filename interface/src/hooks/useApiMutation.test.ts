import { renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { withClient } from "../test-utils";
import { useApiMutation } from "./useApiMutation";

const { getMock, postMock, deleteMock } = vi.hoisted(() => ({
  getMock: vi.fn(),
  postMock: vi.fn(),
  deleteMock: vi.fn(),
}));
vi.mock(
  "../api/client",
  () =>
    ({
      api: { GET: getMock, POST: postMock, DELETE: deleteMock },
    }) as unknown as typeof import("../api/client"),
);

describe("useApiMutation", () => {
  // Shared vi.fn() mocks accumulate calls across tests; reset counts each time.
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("POSTs the init, invalidates keys, and runs onSuccess", async () => {
    postMock.mockResolvedValue({ data: { id: "1" }, error: undefined });
    const onSuccess = vi.fn();

    const { result } = renderHook(
      () =>
        useApiMutation("post", "/api/v1/items", {
          invalidateKeys: [["items"]],
          onSuccess,
        }),
      { wrapper: withClient() },
    );

    result.current.mutate({ body: { title: "x" } });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(postMock).toHaveBeenCalledTimes(1);
    expect(postMock).toHaveBeenCalledWith("/api/v1/items", {
      body: { title: "x" },
    });
    expect(deleteMock).not.toHaveBeenCalled();
    expect(onSuccess).toHaveBeenCalledTimes(1);
    expect(result.current.data).toEqual({ id: "1" });
  });

  it("routes the delete method to api.DELETE with path params", async () => {
    deleteMock.mockResolvedValue({ data: undefined, error: undefined });

    const { result } = renderHook(
      () => useApiMutation("delete", "/api/v1/items/{id}"),
      { wrapper: withClient() },
    );

    result.current.mutate({ params: { path: { id: "1" } } });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(deleteMock).toHaveBeenCalledTimes(1);
    expect(deleteMock).toHaveBeenCalledWith("/api/v1/items/{id}", {
      params: { path: { id: "1" } },
    });
    expect(postMock).not.toHaveBeenCalled();
  });

  it("surfaces the thrown error response when the API returns an error", async () => {
    postMock.mockResolvedValue({
      data: undefined,
      error: { message: "boom" },
    });
    const onSuccess = vi.fn();

    const { result } = renderHook(
      () => useApiMutation("post", "/api/v1/items", { onSuccess }),
      { wrapper: withClient() },
    );

    result.current.mutate({ body: { title: "bad" } });

    await waitFor(() => expect(result.current.isError).toBe(true));
    expect(result.current.error).toEqual({ message: "boom" });
    expect(onSuccess).not.toHaveBeenCalled();
  });

  it("succeeds without invalidateKeys or onSuccess options", async () => {
    postMock.mockResolvedValue({ data: { id: "2" }, error: undefined });

    const { result } = renderHook(
      () => useApiMutation("post", "/api/v1/items"),
      { wrapper: withClient() },
    );

    result.current.mutate({ body: { title: "no-options" } });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(postMock).toHaveBeenCalledTimes(1);
  });
});
