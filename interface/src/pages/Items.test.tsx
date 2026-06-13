import { screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { renderWithClient } from "../test-utils";
import { ItemsPage } from "./Items";

const { getMock } = vi.hoisted(() => ({ getMock: vi.fn() }));
vi.mock(
  "../api/client",
  () =>
    ({
      api: { GET: getMock, POST: vi.fn(), DELETE: vi.fn() },
    }) as unknown as typeof import("../api/client"),
);

describe("ItemsPage", () => {
  it("renders items returned by the API", async () => {
    getMock.mockResolvedValue({
      data: [{ id: "1", title: "write tests", done: false }],
      error: undefined,
    });

    renderWithClient(<ItemsPage />);

    await waitFor(() => expect(screen.getByText("write tests")).toBeTruthy());
  });
});
