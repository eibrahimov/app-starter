import { Theme } from "@radix-ui/themes";
import { screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { renderWithClient } from "../test-utils";
import { HomePage } from "./Home";

// Mock the typed API client and stub its methods so the test never hits the
// network. `vi.hoisted` lets the (hoisted) mock factory reference `getMock`.
const { getMock } = vi.hoisted(() => ({ getMock: vi.fn() }));
vi.mock(
  "../api/client",
  () =>
    ({
      api: { GET: getMock, POST: vi.fn(), DELETE: vi.fn() },
    }) as unknown as typeof import("../api/client"),
);

describe("HomePage", () => {
  it("shows the backend version once the health query resolves", async () => {
    getMock.mockResolvedValue({
      data: { status: "ok", version: "9.9.9", database: "ok" },
      error: undefined,
    });

    // Themes components need Theme context to resolve their tokens in jsdom.
    renderWithClient(
      <Theme>
        <HomePage />
      </Theme>,
    );

    await waitFor(() => expect(screen.getByText(/Backend ok/i)).toBeTruthy());
    expect(screen.getByText(/v9\.9\.9/)).toBeTruthy();
  });
});
