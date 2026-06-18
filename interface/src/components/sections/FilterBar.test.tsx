import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { FilterBar } from "./FilterBar";

const options = ["all", "draft"] as const;

describe("FilterBar", () => {
  it("renders a button per option", () => {
    render(<FilterBar options={options} value="all" onChange={vi.fn()} />);
    expect(screen.getByRole("button", { name: "all" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "draft" })).toBeTruthy();
    expect(screen.getAllByRole("button")).toHaveLength(2);
  });

  it("marks the selected option with aria-pressed=true", () => {
    render(<FilterBar options={options} value="all" onChange={vi.fn()} />);
    expect(
      screen.getByRole("button", { name: "all", pressed: true }),
    ).toBeTruthy();
  });

  it("marks the unselected options with aria-pressed=false", () => {
    render(<FilterBar options={options} value="all" onChange={vi.fn()} />);
    expect(
      screen.getByRole("button", { name: "draft", pressed: false }),
    ).toBeTruthy();
  });

  it("tracks the selected option when value changes", () => {
    render(<FilterBar options={options} value="draft" onChange={vi.fn()} />);
    expect(
      screen.getByRole("button", { name: "draft", pressed: true }),
    ).toBeTruthy();
    expect(
      screen.getByRole("button", { name: "all", pressed: false }),
    ).toBeTruthy();
  });

  it("calls onChange with the clicked option", () => {
    const onChange = vi.fn();
    render(<FilterBar options={options} value="all" onChange={onChange} />);
    fireEvent.click(screen.getByRole("button", { name: "draft" }));
    expect(onChange).toHaveBeenCalledTimes(1);
    expect(onChange).toHaveBeenCalledWith("draft");
  });

  it("calls onChange even when clicking the already-selected option", () => {
    const onChange = vi.fn();
    render(<FilterBar options={options} value="all" onChange={onChange} />);
    fireEvent.click(screen.getByRole("button", { name: "all" }));
    expect(onChange).toHaveBeenCalledWith("all");
  });

  it("renders nothing when options is empty", () => {
    render(<FilterBar options={[]} value="" onChange={vi.fn()} />);
    expect(screen.queryByRole("button")).toBeNull();
  });

  it("gives every button type=button", () => {
    render(<FilterBar options={options} value="all" onChange={vi.fn()} />);
    for (const button of screen.getAllByRole("button")) {
      expect(button.getAttribute("type")).toBe("button");
    }
  });
});
