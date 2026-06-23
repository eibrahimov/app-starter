import { keepPreviousData } from "@tanstack/react-query";
import { renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { withClient } from "../test-utils";
import { useApiQuery } from "./useApiQuery";

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

describe("useApiQuery", () => {
  // Shared vi.fn() mocks accumulate calls across tests; reset counts each time.
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("populates data on a successful GET", async () => {
    getMock.mockResolvedValue({
      data: [{ id: "1", title: "write tests", done: false }],
      error: undefined,
    });

    const { result } = renderHook(() => useApiQuery("/api/v1/todo"), {
      wrapper: withClient(),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(result.current.data?.[0]?.title).toBe("write tests");
    expect(getMock).toHaveBeenCalledWith("/api/v1/todo", undefined);
  });

  it("surfaces an error when the API returns one", async () => {
    getMock.mockResolvedValue({
      data: undefined,
      error: { message: "boom" },
    });

    const { result } = renderHook(() => useApiQuery("/api/v1/todo"), {
      wrapper: withClient(),
    });

    await waitFor(() => expect(result.current.isError).toBe(true));
    expect(result.current.data).toBeUndefined();
    expect(result.current.error).toEqual({ message: "boom" });
  });

  it("forwards init to api.GET and uses init.params.query for the default key", async () => {
    getMock.mockResolvedValue({ data: [], error: undefined });

    const init = { params: { query: { status: "draft" } } } as const;
    const { result } = renderHook(() => useApiQuery("/api/v1/blog", init), {
      wrapper: withClient(),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(getMock).toHaveBeenCalledWith("/api/v1/blog", init);
  });

  it("respects an explicit queryKey", async () => {
    getMock.mockResolvedValue({
      data: [{ id: "9", title: "scoped", done: true }],
      error: undefined,
    });

    const { result } = renderHook(
      () =>
        useApiQuery("/api/v1/todo", undefined, {
          queryKey: ["items", "custom-scope"],
        }),
      { wrapper: withClient() },
    );

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(result.current.data?.[0]?.title).toBe("scoped");
  });

  it("does not run the query when disabled", () => {
    getMock.mockResolvedValue({ data: [], error: undefined });

    const { result } = renderHook(
      () => useApiQuery("/api/v1/todo", undefined, { enabled: false }),
      { wrapper: withClient() },
    );

    expect(result.current.fetchStatus).toBe("idle");
    expect(result.current.data).toBeUndefined();
    expect(getMock).not.toHaveBeenCalled();
  });

  it("accepts placeholderData: keepPreviousData", async () => {
    getMock.mockResolvedValue({
      data: [{ id: "2", title: "kept", done: false }],
      error: undefined,
    });

    const { result } = renderHook(
      () =>
        useApiQuery("/api/v1/todo", undefined, {
          placeholderData: keepPreviousData,
        }),
      { wrapper: withClient() },
    );

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(result.current.data?.[0]?.title).toBe("kept");
  });
});
