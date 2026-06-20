import { screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { renderWithTheme } from "../../test-utils";
import { Toolbar } from "./Toolbar";

describe("Toolbar", () => {
  it("renders its children", () => {
    renderWithTheme(
      <Toolbar>
        <button type="button">Add</button>
      </Toolbar>,
    );
    expect(screen.getByRole("button", { name: "Add" })).toBeTruthy();
  });

  it("renders multiple children", () => {
    renderWithTheme(
      <Toolbar>
        <span>first</span>
        <span>second</span>
      </Toolbar>,
    );
    expect(screen.getByText("first")).toBeTruthy();
    expect(screen.getByText("second")).toBeTruthy();
  });

  it("wraps its children in a single layout container", () => {
    renderWithTheme(<Toolbar>content</Toolbar>);
    const toolbar = screen.getByText("content");
    expect(toolbar.tagName.toLowerCase()).toBe("div");
    expect(toolbar.textContent).toBe("content");
  });
});
