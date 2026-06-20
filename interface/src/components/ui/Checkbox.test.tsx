import { Theme } from "@radix-ui/themes";
import { fireEvent, render, screen } from "@testing-library/react";
import type { ReactNode } from "react";
import { describe, expect, it, vi } from "vitest";
import { Checkbox } from "./Checkbox";

function renderWithTheme(ui: ReactNode) {
  return render(<Theme>{ui}</Theme>);
}

describe("Checkbox", () => {
  it("renders a checkbox exposed by its aria-label", () => {
    renderWithTheme(
      <Checkbox
        checked={false}
        onCheckedChange={vi.fn()}
        aria-label="Accept terms"
      />,
    );
    expect(screen.getByRole("checkbox", { name: "Accept terms" })).toBeTruthy();
  });

  it("reflects an unchecked state via aria-checked", () => {
    renderWithTheme(
      <Checkbox
        checked={false}
        onCheckedChange={vi.fn()}
        aria-label="Subscribe"
      />,
    );
    const checkbox = screen.getByRole("checkbox", { name: "Subscribe" });
    expect(checkbox.getAttribute("aria-checked")).toBe("false");
  });

  it("reflects a checked state via aria-checked", () => {
    renderWithTheme(
      <Checkbox
        checked={true}
        onCheckedChange={vi.fn()}
        aria-label="Subscribe"
      />,
    );
    const checkbox = screen.getByRole("checkbox", { name: "Subscribe" });
    expect(checkbox.getAttribute("aria-checked")).toBe("true");
  });

  it("calls onCheckedChange with true when toggling an unchecked box", () => {
    const onCheckedChange = vi.fn();
    renderWithTheme(
      <Checkbox
        checked={false}
        onCheckedChange={onCheckedChange}
        aria-label="Toggle"
      />,
    );
    fireEvent.click(screen.getByRole("checkbox", { name: "Toggle" }));
    expect(onCheckedChange).toHaveBeenCalledTimes(1);
    expect(onCheckedChange).toHaveBeenCalledWith(true);
  });

  it("calls onCheckedChange with false when toggling a checked box", () => {
    const onCheckedChange = vi.fn();
    renderWithTheme(
      <Checkbox
        checked={true}
        onCheckedChange={onCheckedChange}
        aria-label="Toggle"
      />,
    );
    fireEvent.click(screen.getByRole("checkbox", { name: "Toggle" }));
    expect(onCheckedChange).toHaveBeenCalledTimes(1);
    expect(onCheckedChange).toHaveBeenCalledWith(false);
  });

  it("always reports a boolean to onCheckedChange", () => {
    const onCheckedChange = vi.fn();
    renderWithTheme(
      <Checkbox
        checked={false}
        onCheckedChange={onCheckedChange}
        aria-label="Boolean check"
      />,
    );
    fireEvent.click(screen.getByRole("checkbox", { name: "Boolean check" }));
    expect(typeof onCheckedChange.mock.calls[0][0]).toBe("boolean");
  });

  it("forwards a custom className to the rendered control", () => {
    renderWithTheme(
      <Checkbox
        checked={false}
        onCheckedChange={vi.fn()}
        aria-label="Styled"
        className="custom-class"
      />,
    );
    const checkbox = screen.getByRole("checkbox", { name: "Styled" });
    const styled = checkbox.closest(".custom-class");
    expect(styled).not.toBeNull();
  });

  it("renders without an aria-label when none is provided", () => {
    renderWithTheme(<Checkbox checked={false} onCheckedChange={vi.fn()} />);
    const checkbox = screen.getByRole("checkbox");
    expect(checkbox.getAttribute("aria-label")).toBeNull();
  });
});
