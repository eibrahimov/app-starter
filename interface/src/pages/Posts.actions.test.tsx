import { Theme } from "@radix-ui/themes";
import { fireEvent, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { renderWithClient } from "../test-utils";
import { PostsPage } from "./Posts";

const { getMock, postMock } = vi.hoisted(() => ({
  getMock: vi.fn(),
  postMock: vi.fn(),
}));
vi.mock(
  "../api/client",
  () =>
    ({
      api: { GET: getMock, POST: postMock },
    }) as unknown as typeof import("../api/client"),
);

// PostsPage issues two GETs -- the post list and the aggregate stats -- so the
// mock routes on the request path and returns the right shape for each. The
// list contains one draft and one published post to exercise both row actions.
function mockPostsApi() {
  getMock.mockImplementation((path: string) => {
    if (path === "/api/v1/posts/stats") {
      return Promise.resolve({
        data: { draft: 1, published: 1, archived: 0 },
        error: undefined,
      });
    }
    return Promise.resolve({
      data: [
        { id: "1", title: "a", status: "draft" },
        { id: "2", title: "b", status: "published" },
      ],
      error: undefined,
    });
  });
  postMock.mockResolvedValue({ data: {}, error: undefined });
}

// Radix Themes components read their config from a <Theme> context.
function renderPage() {
  return renderWithClient(
    <Theme>
      <PostsPage />
    </Theme>,
  );
}

describe("PostsPage actions", () => {
  // Shared vi.fn() mocks accumulate calls across tests; reset counts each time.
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("publishes a draft row via POST /api/v1/posts/{id}/publish", async () => {
    mockPostsApi();

    renderPage();

    const publishButton = await screen.findByRole("button", {
      name: "Publish",
    });
    fireEvent.click(publishButton);

    await waitFor(() =>
      expect(postMock).toHaveBeenCalledWith("/api/v1/posts/{id}/publish", {
        params: { path: { id: "1" } },
      }),
    );
  });

  it("archives a published row via POST /api/v1/posts/{id}/archive", async () => {
    mockPostsApi();

    renderPage();

    const archiveButton = await screen.findByRole("button", {
      name: "Archive",
    });
    fireEvent.click(archiveButton);

    await waitFor(() =>
      expect(postMock).toHaveBeenCalledWith("/api/v1/posts/{id}/archive", {
        params: { path: { id: "2" } },
      }),
    );
  });

  it("only shows the action matching each row's status", async () => {
    mockPostsApi();

    renderPage();

    // The draft row exposes Publish and the published row exposes Archive --
    // exactly one of each, never both on the same row.
    await screen.findByRole("button", { name: "Publish" });
    expect(screen.getAllByRole("button", { name: "Publish" })).toHaveLength(1);
    expect(screen.getAllByRole("button", { name: "Archive" })).toHaveLength(1);
  });

  it("creates a post via POST /api/v1/posts after typing a title", async () => {
    mockPostsApi();

    renderPage();

    await screen.findByRole("button", { name: "Publish" });

    const input = screen.getByPlaceholderText("Draft a new post");
    fireEvent.change(input, { target: { value: "fresh post" } });
    fireEvent.click(screen.getByRole("button", { name: "Draft" }));

    await waitFor(() =>
      expect(postMock).toHaveBeenCalledWith("/api/v1/posts", {
        body: { title: "fresh post" },
      }),
    );
  });

  it("submits a new post when Enter is pressed in the title input", async () => {
    mockPostsApi();

    renderPage();

    await screen.findByRole("button", { name: "Publish" });

    const input = screen.getByPlaceholderText("Draft a new post");
    fireEvent.change(input, { target: { value: "via enter" } });
    fireEvent.keyDown(input, { key: "Enter" });

    await waitFor(() =>
      expect(postMock).toHaveBeenCalledWith("/api/v1/posts", {
        body: { title: "via enter" },
      }),
    );
  });

  it("does not create a post when the title is blank or whitespace", async () => {
    mockPostsApi();

    renderPage();

    await screen.findByRole("button", { name: "Publish" });

    const input = screen.getByPlaceholderText("Draft a new post");
    fireEvent.change(input, { target: { value: "   " } });
    fireEvent.click(screen.getByRole("button", { name: "Draft" }));

    // Whitespace-only titles are rejected before any request is issued.
    expect(postMock).not.toHaveBeenCalled();
  });

  it("clears the title input after a successful create", async () => {
    mockPostsApi();

    renderPage();

    await screen.findByRole("button", { name: "Publish" });

    const input = screen.getByPlaceholderText(
      "Draft a new post",
    ) as HTMLInputElement;
    fireEvent.change(input, { target: { value: "to be cleared" } });
    fireEvent.click(screen.getByRole("button", { name: "Draft" }));

    await waitFor(() => expect(input.value).toBe(""));
  });

  it("refetches the list with a status filter when a FilterBar option is chosen", async () => {
    mockPostsApi();

    renderPage();

    await screen.findByRole("button", { name: "Publish" });
    getMock.mockClear();

    fireEvent.click(screen.getByRole("button", { name: "draft" }));

    // Switching to the "draft" filter refetches the list under a new query key,
    // passing the chosen status through as a query param.
    await waitFor(() =>
      expect(getMock).toHaveBeenCalledWith("/api/v1/posts", {
        params: { query: { status: "draft" } },
      }),
    );
  });

  it("requests the unfiltered list on the initial 'all' filter", async () => {
    mockPostsApi();

    renderPage();

    // The default "all" filter sends an empty query object, not a status param.
    await waitFor(() =>
      expect(getMock).toHaveBeenCalledWith("/api/v1/posts", {
        params: { query: {} },
      }),
    );
  });

  it("renders the per-status badge for each post", async () => {
    mockPostsApi();

    renderPage();

    // Badge text echoes each row's status; the published row's badge proves the
    // statusTone lookup renders for non-draft statuses too.
    await screen.findByRole("button", { name: "Archive" });
    expect(screen.getByText("a")).toBeTruthy();
    expect(screen.getByText("b")).toBeTruthy();
    // "published" also appears as a FilterBar pill, so scope to the Badge span.
    expect(screen.getByText("published", { selector: "span" })).toBeTruthy();
  });
});
