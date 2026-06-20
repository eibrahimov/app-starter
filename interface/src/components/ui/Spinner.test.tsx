import { screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { renderWithTheme } from "../../test-utils";
import { Spinner } from "./Spinner";

describe("Spinner", () => {
  it("exposes a status role", () => {
    renderWithTheme(<Spinner />);
    expect(screen.getByRole("status")).toBeTruthy();
  });

  it("labels itself as Loading for assistive tech", () => {
    renderWithTheme(<Spinner />);
    expect(screen.getByRole("status").getAttribute("aria-label")).toBe(
      "Loading",
    );
  });

  it("renders a visible spinner indicator", () => {
    renderWithTheme(<Spinner />);
    expect(screen.getByRole("status").childElementCount).toBeGreaterThan(0);
  });

  it("applies a passed className to the status element", () => {
    renderWithTheme(<Spinner className="size-8" />);
    expect(screen.getByRole("status").className).toContain("size-8");
  });

  it("renders without a passed className", () => {
    renderWithTheme(<Spinner />);
    expect(screen.getByRole("status").className).not.toContain("undefined");
  });
});
