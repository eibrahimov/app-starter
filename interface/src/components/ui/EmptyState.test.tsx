import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { EmptyState } from "./EmptyState";

describe("EmptyState", () => {
  it("renders the message prop text", () => {
    render(<EmptyState message="Nothing here yet" />);
    expect(screen.getByText("Nothing here yet")).toBeTruthy();
  });

  it("renders the message inside a paragraph element", () => {
    render(<EmptyState message="No items" />);
    expect(screen.getByText("No items").tagName).toBe("P");
  });

  it("applies the muted text styling", () => {
    render(<EmptyState message="Empty" />);
    expect(screen.getByText("Empty").className).toBe(
      "text-sm text-muted-foreground",
    );
  });

  it("renders an empty message without throwing", () => {
    const { container } = render(<EmptyState message="" />);
    const paragraph = container.querySelector("p");
    expect(paragraph).toBeTruthy();
    expect(paragraph?.textContent).toBe("");
  });
});
