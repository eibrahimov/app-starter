import { screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { renderWithTheme } from "../../test-utils";
import { PageHeader } from "./PageHeader";

describe("PageHeader", () => {
  it("renders the title as a level-1 heading", () => {
    renderWithTheme(<PageHeader title="Items" />);
    const heading = screen.getByRole("heading", { level: 1 });
    expect(heading.textContent).toBe("Items");
  });

  it("renders without children", () => {
    renderWithTheme(<PageHeader title="Posts" />);
    expect(screen.getByRole("heading", { name: "Posts" })).toBeTruthy();
  });

  it("renders optional children alongside the title", () => {
    renderWithTheme(
      <PageHeader title="Items">
        <button type="button">New item</button>
      </PageHeader>,
    );
    expect(screen.getByRole("heading", { name: "Items" })).toBeTruthy();
    expect(screen.getByText("New item")).toBeTruthy();
  });

  it("renders multiple children in the right slot", () => {
    renderWithTheme(
      <PageHeader title="Items">
        <span>first</span>
        <span>second</span>
      </PageHeader>,
    );
    expect(screen.getByText("first")).toBeTruthy();
    expect(screen.getByText("second")).toBeTruthy();
  });
});
