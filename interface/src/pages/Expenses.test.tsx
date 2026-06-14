import { screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { renderWithClient } from "../test-utils";
import { ExpensesPage } from "./Expenses";

const { getMock } = vi.hoisted(() => ({ getMock: vi.fn() }));
vi.mock(
  "../api/client",
  () =>
    ({
      api: { GET: getMock, POST: vi.fn(), PUT: vi.fn(), DELETE: vi.fn() },
    }) as unknown as typeof import("../api/client"),
);

describe("ExpensesPage", () => {
  it("renders expenses returned by the API", async () => {
    getMock.mockImplementation((path: string) => {
      if (path === "/api/v1/settings") {
        return Promise.resolve({ data: { base_currency: "USD" }, error: undefined });
      }
      if (path === "/api/v1/categories") {
        return Promise.resolve({ data: [], error: undefined });
      }
      // "/api/v1/expenses"
      return Promise.resolve({
        data: [
          {
            id: "1",
            amount_cents: 1299,
            description: "Coffee",
            category_id: null,
            category_name: null,
            spent_on: "2026-06-01",
            created_at: "2026-06-01T00:00:00Z",
          },
        ],
        error: undefined,
      });
    });

    renderWithClient(<ExpensesPage />);

    await waitFor(() => expect(screen.getByText("Coffee")).toBeTruthy());
    expect(screen.getByText("$12.99")).toBeTruthy();
  });
});
