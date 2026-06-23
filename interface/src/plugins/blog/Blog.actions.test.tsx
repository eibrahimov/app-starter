import { Theme } from "@radix-ui/themes";
import { fireEvent, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { renderWithClient } from "../../test-utils";
import { BlogPage } from "./Blog";

const { getMock, postMock } = vi.hoisted(() => ({
  getMock: vi.fn(),
  postMock: vi.fn(),
}));
vi.mock(
  "../../api/client",
  () =>
    ({
      api: { GET: getMock, POST: postMock },
    }) as unknown as typeof import("../../api/client"),
);

// BlogPage issues two GETs -- the post list and the aggregate stats -- so the
// mock routes on the request path and returns the right shape for each. The list
// contains one draft and one published post to exercise both row actions.
function mockBlogApi() {
  getMock.mockImplementation((path: string) => {
    if (path === "/api/v1/blog/stats") {
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

function renderPage() {
  return renderWithClient(
    <Theme>
      <BlogPage />
    </Theme>,
  );
}

describe("BlogPage actions", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("publishes a draft row via POST /api/v1/blog/{id}/publish", async () => {
    mockBlogApi();

    renderPage();

    const publishButton = await screen.findByRole("button", {
      name: "Publish",
    });
    fireEvent.click(publishButton);

    await waitFor(() =>
      expect(postMock).toHaveBeenCalledWith("/api/v1/blog/{id}/publish", {
        params: { path: { id: "1" } },
      }),
    );
  });

  it("archives a published row via POST /api/v1/blog/{id}/archive", async () => {
    mockBlogApi();

    renderPage();

    const archiveButton = await screen.findByRole("button", {
      name: "Archive",
    });
    fireEvent.click(archiveButton);

    await waitFor(() =>
      expect(postMock).toHaveBeenCalledWith("/api/v1/blog/{id}/archive", {
        params: { path: { id: "2" } },
      }),
    );
  });

  it("only shows the action matching each row's status", async () => {
    mockBlogApi();

    renderPage();

    await screen.findByRole("button", { name: "Publish" });
    expect(screen.getAllByRole("button", { name: "Publish" })).toHaveLength(1);
    expect(screen.getAllByRole("button", { name: "Archive" })).toHaveLength(1);
  });

  it("creates a post via POST /api/v1/blog after typing a title", async () => {
    mockBlogApi();

    renderPage();

    await screen.findByRole("button", { name: "Publish" });

    const input = screen.getByPlaceholderText("Draft a new post");
    fireEvent.change(input, { target: { value: "fresh post" } });
    fireEvent.click(screen.getByRole("button", { name: "Draft" }));

    await waitFor(() =>
      expect(postMock).toHaveBeenCalledWith("/api/v1/blog", {
        body: { title: "fresh post" },
      }),
    );
  });

  it("submits a new post when Enter is pressed in the title input", async () => {
    mockBlogApi();

    renderPage();

    await screen.findByRole("button", { name: "Publish" });

    const input = screen.getByPlaceholderText("Draft a new post");
    fireEvent.change(input, { target: { value: "via enter" } });
    fireEvent.keyDown(input, { key: "Enter" });

    await waitFor(() =>
      expect(postMock).toHaveBeenCalledWith("/api/v1/blog", {
        body: { title: "via enter" },
      }),
    );
  });

  it("does not create a post when the title is blank or whitespace", async () => {
    mockBlogApi();

    renderPage();

    await screen.findByRole("button", { name: "Publish" });

    const input = screen.getByPlaceholderText("Draft a new post");
    fireEvent.change(input, { target: { value: "   " } });
    fireEvent.click(screen.getByRole("button", { name: "Draft" }));

    expect(postMock).not.toHaveBeenCalled();
  });

  it("clears the title input after a successful create", async () => {
    mockBlogApi();

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
    mockBlogApi();

    renderPage();

    await screen.findByRole("button", { name: "Publish" });
    getMock.mockClear();

    fireEvent.click(screen.getByRole("button", { name: "draft" }));

    await waitFor(() =>
      expect(getMock).toHaveBeenCalledWith("/api/v1/blog", {
        params: { query: { status: "draft" } },
      }),
    );
  });

  it("requests the unfiltered list on the initial 'all' filter", async () => {
    mockBlogApi();

    renderPage();

    await waitFor(() =>
      expect(getMock).toHaveBeenCalledWith("/api/v1/blog", {
        params: { query: {} },
      }),
    );
  });

  it("renders the per-status badge for each post", async () => {
    mockBlogApi();

    renderPage();

    await screen.findByRole("button", { name: "Archive" });
    expect(screen.getByText("a")).toBeTruthy();
    expect(screen.getByText("b")).toBeTruthy();
    expect(screen.getByText("published", { selector: "span" })).toBeTruthy();
  });

  it("resolves each status to its Badge tone via the typed statusTone map", async () => {
    mockBlogApi();

    renderPage();

    await screen.findByRole("button", { name: "Archive" });
    expect(
      screen
        .getByText("draft", { selector: "span" })
        .getAttribute("data-accent-color"),
    ).toBe("amber");
    expect(
      screen
        .getByText("published", { selector: "span" })
        .getAttribute("data-accent-color"),
    ).toBe("grass");
  });
});
