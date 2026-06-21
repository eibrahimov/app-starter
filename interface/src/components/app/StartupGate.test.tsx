import { Theme } from "@radix-ui/themes";
import { screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { renderWithClient } from "../../test-utils";
import { StartupGate, startupView } from "./StartupGate";

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

// The render branch is decided by this pure function, so the copy flip and the
// terminal/recover transitions are tested here without driving real retry timers.
describe("startupView", () => {
  const base = {
    isSuccess: false,
    isError: false,
    isFetching: true,
    failureCount: 0,
  };

  it("is 'ready' once the probe succeeds", () => {
    expect(startupView({ ...base, isSuccess: true })).toBe("ready");
  });

  it("is 'starting' on the first attempts", () => {
    expect(startupView({ ...base, failureCount: 0 })).toBe("starting");
    expect(startupView({ ...base, failureCount: 3 })).toBe("starting");
  });

  it("flips to 'slow' after more than SLOW_AFTER failed probes", () => {
    expect(startupView({ ...base, failureCount: 4 })).toBe("slow");
  });

  it("is 'failed' only when the error is settled (not refetching)", () => {
    expect(
      startupView({
        ...base,
        isError: true,
        isFetching: false,
        failureCount: 8,
      }),
    ).toBe("failed");
  });

  it("returns to the splash while a Retry refetch is in flight (recover path)", () => {
    expect(
      startupView({
        ...base,
        isError: true,
        isFetching: true,
        failureCount: 8,
      }),
    ).toBe("slow");
  });
});
