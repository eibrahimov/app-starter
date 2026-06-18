import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { Dialog } from "./Dialog";

describe("Dialog", () => {
  it("renders the title when open", () => {
    render(
      <Dialog open title="Edit item">
        <p>Body content</p>
      </Dialog>,
    );
    expect(screen.getByRole("heading", { name: "Edit item" })).toBeTruthy();
  });

  it("renders children when open", () => {
    render(
      <Dialog open title="Edit item">
        <p>Body content</p>
      </Dialog>,
    );
    expect(screen.getByText("Body content")).toBeTruthy();
  });

  it("renders the description when provided", () => {
    render(
      <Dialog open title="Edit item" description="Change the item details">
        <p>Body content</p>
      </Dialog>,
    );
    expect(screen.getByText("Change the item details")).toBeTruthy();
  });

  it("still shows the title when no description is given", () => {
    render(
      <Dialog open title="Edit item">
        <p>Body content</p>
      </Dialog>,
    );
    expect(screen.getByRole("heading", { name: "Edit item" })).toBeTruthy();
    expect(screen.queryByText("Change the item details")).toBeNull();
  });

  it("does not render content while closed", () => {
    render(
      <Dialog open={false} title="Hidden title">
        <p>Hidden body</p>
      </Dialog>,
    );
    expect(screen.queryByText("Hidden title")).toBeNull();
    expect(screen.queryByText("Hidden body")).toBeNull();
  });

  it("renders the trigger for a closed dialog", () => {
    render(
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
    render(
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
