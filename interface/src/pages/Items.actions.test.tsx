import { fireEvent, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { renderWithClient } from "../test-utils";
import { ItemsPage } from "./Items";

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

function seedOneItem() {
  getMock.mockResolvedValue({
    data: [{ id: "1", title: "write tests", done: false }],
    error: undefined,
  });
  postMock.mockResolvedValue({ data: {}, error: undefined });
  deleteMock.mockResolvedValue({ data: {}, error: undefined });
}

describe("ItemsPage actions", () => {
  // Shared vi.fn() mocks accumulate calls across tests; reset counts each time.
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("creates an item via POST when typing a title and clicking Add", async () => {
    seedOneItem();

    renderWithClient(<ItemsPage />);
    await waitFor(() => expect(screen.getByText("write tests")).toBeTruthy());

    const input = screen.getByPlaceholderText("What needs doing?");
    fireEvent.change(input, { target: { value: "buy milk" } });
    fireEvent.click(screen.getByText("Add"));

    await waitFor(() => expect(postMock).toHaveBeenCalledTimes(1));
    expect(postMock).toHaveBeenCalledWith(
      "/api/v1/items",
      expect.objectContaining({ body: { title: "buy milk" } }),
    );
  });

  it("creates an item when pressing Enter in the input", async () => {
    seedOneItem();

    renderWithClient(<ItemsPage />);
    await waitFor(() => expect(screen.getByText("write tests")).toBeTruthy());

    const input = screen.getByPlaceholderText("What needs doing?");
    fireEvent.change(input, { target: { value: "press enter" } });
    fireEvent.keyDown(input, { key: "Enter" });

    await waitFor(() => expect(postMock).toHaveBeenCalledTimes(1));
    expect(postMock).toHaveBeenCalledWith(
      "/api/v1/items",
      expect.objectContaining({ body: { title: "press enter" } }),
    );
  });

  it("does not submit when the title is blank or whitespace", async () => {
    seedOneItem();

    renderWithClient(<ItemsPage />);
    await waitFor(() => expect(screen.getByText("write tests")).toBeTruthy());

    const input = screen.getByPlaceholderText("What needs doing?");
    fireEvent.change(input, { target: { value: "   " } });
    fireEvent.click(screen.getByText("Add"));

    expect(postMock).not.toHaveBeenCalled();
  });

  it("does not submit on a non-Enter keypress", async () => {
    seedOneItem();

    renderWithClient(<ItemsPage />);
    await waitFor(() => expect(screen.getByText("write tests")).toBeTruthy());

    const input = screen.getByPlaceholderText("What needs doing?");
    fireEvent.change(input, { target: { value: "typing" } });
    fireEvent.keyDown(input, { key: "a" });

    expect(postMock).not.toHaveBeenCalled();
  });

  it("clears the input after a successful create", async () => {
    seedOneItem();

    renderWithClient(<ItemsPage />);
    await waitFor(() => expect(screen.getByText("write tests")).toBeTruthy());

    const input = screen.getByPlaceholderText(
      "What needs doing?",
    ) as HTMLInputElement;
    fireEvent.change(input, { target: { value: "buy milk" } });
    fireEvent.click(screen.getByText("Add"));

    await waitFor(() => expect(input.value).toBe(""));
  });

  it("toggles an item via POST to the toggle path when the checkbox is clicked", async () => {
    seedOneItem();

    renderWithClient(<ItemsPage />);
    await waitFor(() => expect(screen.getByText("write tests")).toBeTruthy());

    fireEvent.click(screen.getByLabelText("write tests"));

    await waitFor(() =>
      expect(postMock).toHaveBeenCalledWith(
        "/api/v1/items/{id}/toggle",
        expect.objectContaining({ params: { path: { id: "1" } } }),
      ),
    );
  });

  it("deletes an item via DELETE when Delete is clicked", async () => {
    seedOneItem();

    renderWithClient(<ItemsPage />);
    await waitFor(() => expect(screen.getByText("write tests")).toBeTruthy());

    fireEvent.click(screen.getByText("Delete"));

    await waitFor(() =>
      expect(deleteMock).toHaveBeenCalledWith(
        "/api/v1/items/{id}",
        expect.objectContaining({ params: { path: { id: "1" } } }),
      ),
    );
  });

  it("renders the page header and the add control", async () => {
    seedOneItem();

    renderWithClient(<ItemsPage />);
    await waitFor(() => expect(screen.getByText("write tests")).toBeTruthy());

    expect(screen.getByText("Items")).toBeTruthy();
    expect(screen.getByText("Add")).toBeTruthy();
    expect(screen.getByPlaceholderText("What needs doing?")).toBeTruthy();
  });
});
