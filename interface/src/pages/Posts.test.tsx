import { Theme } from "@radix-ui/themes";
import { screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { renderWithClient } from "../test-utils";
import { PostsPage } from "./Posts";

const { getMock } = vi.hoisted(() => ({ getMock: vi.fn() }));
vi.mock(
  "../api/client",
  () =>
    ({
      api: { GET: getMock, POST: vi.fn() },
    }) as unknown as typeof import("../api/client"),
);

// PostsPage issues two GETs -- the list and the aggregate stats -- so the mock
// routes on the request path and returns the right shape for each.
function mockPostsApi() {
  getMock.mockImplementation((path: string) => {
    if (path === "/api/v1/posts/stats") {
      return Promise.resolve({
        data: { draft: 1, published: 0, archived: 0 },
        error: undefined,
      });
    }
    return Promise.resolve({
      data: [{ id: "1", title: "hello world", status: "draft" }],
      error: undefined,
    });
  });
}

// Radix Themes components read their config from a <Theme> context.
function renderPage() {
  return renderWithClient(
    <Theme>
      <PostsPage />
    </Theme>,
  );
}

describe("PostsPage", () => {
  it("renders posts returned by the API", async () => {
    mockPostsApi();

    renderPage();

    await waitFor(() => expect(screen.getByText("hello world")).toBeTruthy());
  });

  it("renders the aggregate stats summary", async () => {
    mockPostsApi();

    renderPage();

    await waitFor(() => expect(screen.getByText(/1 draft/)).toBeTruthy());
  });
});
