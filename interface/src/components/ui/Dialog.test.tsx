import { Theme } from "@radix-ui/themes";
import { fireEvent, render, screen } from "@testing-library/react";
import type { ReactNode } from "react";
import { describe, expect, it, vi } from "vitest";
import { Dialog } from "./Dialog";

function renderWithTheme(ui: ReactNode) {
  return render(<Theme>{ui}</Theme>);
}

describe("Dialog", () => {
  it("renders the title when open", () => {
    renderWithTheme(
      <Dialog open title="Edit item">
        <p>Body content</p>
      </Dialog>,
    );
    expect(screen.getByRole("heading", { name: "Edit item" })).toBeTruthy();
  });

  it("renders children when open", () => {
    renderWithTheme(
      <Dialog open title="Edit item">
        <p>Body content</p>
      </Dialog>,
    );
    expect(screen.getByText("Body content")).toBeTruthy();
  });

  it("renders the description when provided", () => {
    renderWithTheme(
      <Dialog open title="Edit item" description="Change the item details">
        <p>Body content</p>
      </Dialog>,
    );
    expect(screen.getByText("Change the item details")).toBeTruthy();
  });

  it("still shows the title when no description is given", () => {
    renderWithTheme(
      <Dialog open title="Edit item">
        <p>Body content</p>
      </Dialog>,
    );
    expect(screen.getByRole("heading", { name: "Edit item" })).toBeTruthy();
    expect(screen.queryByText("Change the item details")).toBeNull();
  });

  it("does not render content while closed", () => {
    renderWithTheme(
      <Dialog open={false} title="Hidden title">
        <p>Hidden body</p>
      </Dialog>,
    );
    expect(screen.queryByText("Hidden title")).toBeNull();
    expect(screen.queryByText("Hidden body")).toBeNull();
  });

  it("renders the trigger for a closed dialog", () => {
    renderWithTheme(
      <Dialog
        open={false}
        title="Edit item"
        trigger={<button type="button">Open dialog</button>}
      >
        <p>Body content</p>
      </Dialog>,
    );
    expect(screen.getByText("Open dialog")).toBeTruthy();
  });

  it("opens via the trigger and calls onOpenChange", () => {
    const onOpenChange = vi.fn();
    renderWithTheme(
      <Dialog
        onOpenChange={onOpenChange}
        title="Edit item"
        trigger={<button type="button">Open dialog</button>}
      >
        <p>Body content</p>
      </Dialog>,
    );
    expect(screen.queryByText("Body content")).toBeNull();
    fireEvent.click(screen.getByText("Open dialog"));
    expect(onOpenChange).toHaveBeenCalledWith(true);
    expect(screen.getByText("Body content")).toBeTruthy();
  });
});
