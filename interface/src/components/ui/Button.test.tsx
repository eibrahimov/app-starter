import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { Button } from "./Button";

describe("Button", () => {
  it("renders its children", () => {
    render(<Button>Save</Button>);
    expect(screen.getByText("Save")).toBeTruthy();
  });

  it("defaults to type=button", () => {
    render(<Button>Save</Button>);
    expect(screen.getByText("Save").getAttribute("type")).toBe("button");
  });

  it("forwards onClick and respects disabled", () => {
    const onClick = vi.fn();
    render(
      <Button onClick={onClick} disabled>
        Save
      </Button>,
    );
    fireEvent.click(screen.getByText("Save"));
    expect(onClick).not.toHaveBeenCalled();
  });
});
