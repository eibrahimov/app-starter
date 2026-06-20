import { Theme } from "@radix-ui/themes";
import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { EmptyState } from "./EmptyState";

function renderInTheme(ui: React.ReactNode) {
  return render(<Theme>{ui}</Theme>);
}

describe("EmptyState", () => {
  it("renders the message prop text", () => {
    renderInTheme(<EmptyState message="Nothing here yet" />);
    expect(screen.getByText("Nothing here yet")).toBeTruthy();
  });

  it("renders the message inside a paragraph element", () => {
    renderInTheme(<EmptyState message="No items" />);
    expect(screen.getByText("No items").tagName).toBe("P");
  });

  it("renders an empty message without throwing", () => {
    const { container } = renderInTheme(<EmptyState message="" />);
    const paragraph = container.querySelector("p");
    expect(paragraph).toBeTruthy();
    expect(paragraph?.textContent).toBe("");
  });
});
