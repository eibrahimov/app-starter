import { fireEvent, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { renderWithTheme } from "../../test-utils";
import { ErrorFallback } from "./ErrorFallback";

describe("ErrorFallback", () => {
  it("surfaces the error message as an alert", () => {
    renderWithTheme(
      <ErrorFallback
        error={new Error("kaboom")}
        resetErrorBoundary={vi.fn()}
      />,
    );

    expect(screen.getByRole("alert")).toBeTruthy();
    expect(screen.getByText(/something went wrong/i)).toBeTruthy();
    expect(screen.getByText("kaboom")).toBeTruthy();
  });

  it("invokes resetErrorBoundary when Try again is clicked", () => {
    const reset = vi.fn();
    renderWithTheme(
      <ErrorFallback error={new Error("x")} resetErrorBoundary={reset} />,
    );

    fireEvent.click(screen.getByRole("button", { name: "Try again" }));

    expect(reset).toHaveBeenCalledTimes(1);
  });
});
