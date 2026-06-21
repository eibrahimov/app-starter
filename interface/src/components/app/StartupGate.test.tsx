import { Theme } from "@radix-ui/themes";
import { screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { renderWithClient } from "../../test-utils";
import { StartupGate } from "./StartupGate";

// Mock the typed API client so the gate's health probe never hits the network.
const { getMock } = vi.hoisted(() => ({ getMock: vi.fn() }));
vi.mock(
  "../../api/client",
  () =>
    ({
      api: { GET: getMock, POST: vi.fn(), DELETE: vi.fn() },
    }) as unknown as typeof import("../../api/client"),
);

describe("StartupGate", () => {
  it("renders children once the health probe succeeds", async () => {
    getMock.mockResolvedValue({
      data: { status: "ok", version: "9.9.9", database: "ok" },
      error: undefined,
    });

    renderWithClient(
      <Theme>
        <StartupGate>
          <div>app ready</div>
        </StartupGate>
      </Theme>,
    );

    await waitFor(() => expect(screen.getByText("app ready")).toBeTruthy());
  });

  it("shows the splash and withholds children while the probe fails", async () => {
    getMock.mockRejectedValue(new Error("connection refused"));

    renderWithClient(
      <Theme>
        <StartupGate>
          <div>app ready</div>
        </StartupGate>
      </Theme>,
    );

    await waitFor(() => expect(screen.getByText("Starting…")).toBeTruthy());
    expect(screen.queryByText("app ready")).toBeNull();
  });
});
