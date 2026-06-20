import { screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { renderWithTheme } from "../../test-utils";
import { EmptyState } from "./EmptyState";

describe("EmptyState", () => {
  it("renders the message prop text", () => {
    renderWithTheme(<EmptyState message="Nothing here yet" />);
    expect(screen.getByText("Nothing here yet")).toBeTruthy();
  });

  it("renders the message inside a paragraph element", () => {
    renderWithTheme(<EmptyState message="No items" />);
    expect(screen.getByText("No items").tagName).toBe("P");
  });

  it("renders an empty message without throwing", () => {
    const { container } = renderWithTheme(<EmptyState message="" />);
    const paragraph = container.querySelector("p");
    expect(paragraph).toBeTruthy();
    expect(paragraph?.textContent).toBe("");
  });
});
