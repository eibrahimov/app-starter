import { Theme } from "@radix-ui/themes";
import { fireEvent, render, screen } from "@testing-library/react";
import type { ReactNode } from "react";
import { describe, expect, it, vi } from "vitest";
import { Button } from "./Button";

function renderWithTheme(ui: ReactNode) {
  return render(<Theme>{ui}</Theme>);
}

describe("Button", () => {
  it("renders its children", () => {
    renderWithTheme(<Button>Save</Button>);
    expect(screen.getByRole("button", { name: "Save" })).toBeTruthy();
  });

  it("defaults to type=button", () => {
    renderWithTheme(<Button>Save</Button>);
    expect(
      screen.getByRole("button", { name: "Save" }).getAttribute("type"),
    ).toBe("button");
  });

  it("honors an explicit type", () => {
    renderWithTheme(<Button type="submit">Save</Button>);
    expect(
      screen.getByRole("button", { name: "Save" }).getAttribute("type"),
    ).toBe("submit");
  });

  it("forwards onClick and respects disabled", () => {
    const onClick = vi.fn();
    renderWithTheme(
      <Button onClick={onClick} disabled>
        Save
      </Button>,
    );
    const button = screen.getByRole("button", { name: "Save" });
    expect(button.hasAttribute("disabled")).toBe(true);
    fireEvent.click(button);
    expect(onClick).not.toHaveBeenCalled();
  });

  it("renders each variant as an accessible button", () => {
    for (const variant of [
      "primary",
      "ghost",
      "danger",
      "success",
      "warning",
    ] as const) {
      const { unmount } = renderWithTheme(
        <Button variant={variant}>{variant}</Button>,
      );
      expect(screen.getByRole("button", { name: variant })).toBeTruthy();
      unmount();
    }
  });
});
