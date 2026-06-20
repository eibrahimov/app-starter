import { Theme } from "@radix-ui/themes";
import { fireEvent, render, screen } from "@testing-library/react";
import type { ReactElement } from "react";
import { describe, expect, it, vi } from "vitest";
import { Input } from "./Input";

function renderInTheme(ui: ReactElement) {
  return render(<Theme>{ui}</Theme>);
}

describe("Input", () => {
  it("renders an input element", () => {
    renderInTheme(<Input />);
    expect(screen.getByRole("textbox")).toBeTruthy();
  });

  it("renders a usable textbox when no props are given", () => {
    renderInTheme(<Input />);
    const input = screen.getByRole("textbox");
    expect(input.tagName).toBe("INPUT");
  });

  it("applies a custom className to the rendered field", () => {
    renderInTheme(<Input className="w-full" />);
    const input = screen.getByRole("textbox");
    expect(input.closest(".w-full")).not.toBeNull();
  });

  it("forwards the placeholder prop", () => {
    renderInTheme(<Input placeholder="Search items" />);
    expect(screen.getByPlaceholderText("Search items")).toBeTruthy();
  });

  it("forwards arbitrary input attributes", () => {
    renderInTheme(<Input type="email" name="email" disabled />);
    const input = screen.getByRole("textbox");
    expect(input.getAttribute("type")).toBe("email");
    expect(input.getAttribute("name")).toBe("email");
    expect((input as HTMLInputElement).disabled).toBe(true);
  });

  it("renders a controlled value and fires onChange on input", () => {
    const onChange = vi.fn();
    renderInTheme(<Input value="hello" onChange={onChange} />);
    const input = screen.getByRole("textbox") as HTMLInputElement;
    expect(input.value).toBe("hello");

    fireEvent.change(input, { target: { value: "world" } });
    expect(onChange).toHaveBeenCalledTimes(1);
  });

  it("updates the displayed value for an uncontrolled input", () => {
    renderInTheme(<Input defaultValue="start" />);
    const input = screen.getByRole("textbox") as HTMLInputElement;
    expect(input.value).toBe("start");

    fireEvent.change(input, { target: { value: "changed" } });
    expect(input.value).toBe("changed");
  });
});
