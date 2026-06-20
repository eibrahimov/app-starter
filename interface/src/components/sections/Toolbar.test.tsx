import { Theme } from "@radix-ui/themes";
import { render, screen } from "@testing-library/react";
import type { ReactElement } from "react";
import { describe, expect, it } from "vitest";
import { Toolbar } from "./Toolbar";

function renderThemed(ui: ReactElement) {
  return render(<Theme>{ui}</Theme>);
}

describe("Toolbar", () => {
  it("renders its children", () => {
    renderThemed(
      <Toolbar>
        <button type="button">Add</button>
      </Toolbar>,
    );
    expect(screen.getByRole("button", { name: "Add" })).toBeTruthy();
  });

  it("renders multiple children", () => {
    renderThemed(
      <Toolbar>
        <span>first</span>
        <span>second</span>
      </Toolbar>,
    );
    expect(screen.getByText("first")).toBeTruthy();
    expect(screen.getByText("second")).toBeTruthy();
  });

  it("wraps its children in a single layout container", () => {
    renderThemed(<Toolbar>content</Toolbar>);
    const toolbar = screen.getByText("content");
    expect(toolbar.tagName.toLowerCase()).toBe("div");
    expect(toolbar.textContent).toBe("content");
  });

  it("forwards a className onto the layout container", () => {
    renderThemed(<Toolbar className="justify-end">content</Toolbar>);
    const toolbar = screen.getByText("content");
    expect(toolbar.classList.contains("justify-end")).toBe(true);
  });

  it("renders without a className", () => {
    renderThemed(<Toolbar className="">content</Toolbar>);
    expect(screen.getByText("content")).toBeTruthy();
  });
});
