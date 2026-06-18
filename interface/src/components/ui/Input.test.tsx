import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { Input } from "./Input";

describe("Input", () => {
  it("renders an input element", () => {
    render(<Input />);
    expect(screen.getByRole("textbox")).toBeTruthy();
  });

  it("keeps its base classes when no className is given", () => {
    render(<Input />);
    const input = screen.getByRole("textbox");
    expect(input.className).toContain("rounded-md");
    expect(input.className).toContain("border-input");
  });

  it("merges a custom className with the base classes", () => {
    render(<Input className="w-full" />);
    const input = screen.getByRole("textbox");
    expect(input.className).toContain("w-full");
    expect(input.className).toContain("rounded-md");
  });

  it("forwards the placeholder prop", () => {
    render(<Input placeholder="Search items" />);
    expect(screen.getByPlaceholderText("Search items")).toBeTruthy();
  });

  it("forwards arbitrary input attributes", () => {
    render(<Input type="email" name="email" disabled />);
    const input = screen.getByRole("textbox");
    expect(input.getAttribute("type")).toBe("email");
    expect(input.getAttribute("name")).toBe("email");
    expect((input as HTMLInputElement).disabled).toBe(true);
  });

  it("renders a controlled value and fires onChange on input", () => {
    const onChange = vi.fn();
    render(<Input value="hello" onChange={onChange} />);
    const input = screen.getByRole("textbox") as HTMLInputElement;
    expect(input.value).toBe("hello");

    fireEvent.change(input, { target: { value: "world" } });
    expect(onChange).toHaveBeenCalledTimes(1);
  });

  it("updates the displayed value for an uncontrolled input", () => {
    render(<Input defaultValue="start" />);
    const input = screen.getByRole("textbox") as HTMLInputElement;
    expect(input.value).toBe("start");

    fireEvent.change(input, { target: { value: "changed" } });
    expect(input.value).toBe("changed");
  });
});
