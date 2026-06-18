import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { Spinner } from "./Spinner";

describe("Spinner", () => {
  it("exposes a status role", () => {
    render(<Spinner />);
    expect(screen.getByRole("status")).toBeTruthy();
  });

  it("labels itself as Loading for assistive tech", () => {
    render(<Spinner />);
    expect(screen.getByRole("status").getAttribute("aria-label")).toBe(
      "Loading",
    );
  });

  it("applies the base spinner classes by default", () => {
    render(<Spinner />);
    const className = screen.getByRole("status").className;
    expect(className).toContain("animate-spin");
    expect(className).toContain("rounded-full");
  });

  it("appends a passed className alongside the base classes", () => {
    render(<Spinner className="size-8" />);
    const className = screen.getByRole("status").className;
    expect(className).toContain("size-8");
    expect(className).toContain("animate-spin");
  });

  it("renders without a passed className", () => {
    render(<Spinner />);
    expect(screen.getByRole("status").className).not.toContain("undefined");
  });
});
