import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { Toolbar } from "./Toolbar";

describe("Toolbar", () => {
  it("renders its children", () => {
    render(
      <Toolbar>
        <button type="button">Add</button>
      </Toolbar>,
    );
    expect(screen.getByText("Add")).toBeTruthy();
  });

  it("renders multiple children", () => {
    render(
      <Toolbar>
        <span>first</span>
        <span>second</span>
      </Toolbar>,
    );
    expect(screen.getByText("first")).toBeTruthy();
    expect(screen.getByText("second")).toBeTruthy();
  });

  it("applies the base classes when no className is given", () => {
    render(<Toolbar>content</Toolbar>);
    const toolbar = screen.getByText("content");
    expect(toolbar.className).toBe("flex gap-2");
  });

  it("merges className with the base classes", () => {
    render(<Toolbar className="justify-end">content</Toolbar>);
    const toolbar = screen.getByText("content");
    expect(toolbar.className).toBe("flex gap-2 justify-end");
  });

  it("keeps only the base classes for an empty className", () => {
    render(<Toolbar className="">content</Toolbar>);
    const toolbar = screen.getByText("content");
    expect(toolbar.className).toBe("flex gap-2");
  });
});
