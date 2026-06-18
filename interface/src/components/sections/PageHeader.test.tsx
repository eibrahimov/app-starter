import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { PageHeader } from "./PageHeader";

describe("PageHeader", () => {
  it("renders the title as a level-1 heading", () => {
    render(<PageHeader title="Items" />);
    const heading = screen.getByRole("heading", { level: 1 });
    expect(heading.textContent).toBe("Items");
  });

  it("renders without children", () => {
    render(<PageHeader title="Posts" />);
    expect(screen.getByRole("heading", { name: "Posts" })).toBeTruthy();
  });

  it("renders optional children alongside the title", () => {
    render(
      <PageHeader title="Items">
        <button type="button">New item</button>
      </PageHeader>,
    );
    expect(screen.getByRole("heading", { name: "Items" })).toBeTruthy();
    expect(screen.getByText("New item")).toBeTruthy();
  });

  it("renders multiple children in the right slot", () => {
    render(
      <PageHeader title="Items">
        <span>first</span>
        <span>second</span>
      </PageHeader>,
    );
    expect(screen.getByText("first")).toBeTruthy();
    expect(screen.getByText("second")).toBeTruthy();
  });
});
