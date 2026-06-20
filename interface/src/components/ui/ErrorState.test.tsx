import { Theme } from "@radix-ui/themes";
import { render, screen } from "@testing-library/react";
import type { ReactNode } from "react";
import { describe, expect, it } from "vitest";
import { ErrorState } from "./ErrorState";

function renderWithTheme(ui: ReactNode) {
  return render(<Theme>{ui}</Theme>);
}

describe("ErrorState", () => {
  it("renders the message prop text", () => {
    renderWithTheme(<ErrorState message="Something went wrong" />);
    expect(screen.getByText("Something went wrong")).toBeTruthy();
  });

  it("exposes the message through an assertive alert region", () => {
    renderWithTheme(<ErrorState message="Boom" />);
    const alert = screen.getByRole("alert");
    expect(alert).toBeTruthy();
    expect(alert.textContent).toBe("Boom");
  });

  it("renders a distinct message when the prop changes", () => {
    const { rerender } = renderWithTheme(<ErrorState message="First" />);
    expect(screen.getByText("First")).toBeTruthy();
    rerender(
      <Theme>
        <ErrorState message="Second" />
      </Theme>,
    );
    expect(screen.getByText("Second")).toBeTruthy();
    expect(screen.queryByText("First")).toBeNull();
  });

  it("renders an alert with no text for an empty message", () => {
    renderWithTheme(<ErrorState message="" />);
    const alert = screen.getByRole("alert");
    expect(alert).toBeTruthy();
    expect(alert.textContent).toBe("");
  });

  it("renders messages containing special characters verbatim", () => {
    renderWithTheme(<ErrorState message="Error: <code> & 'quotes' failed" />);
    expect(screen.getByText("Error: <code> & 'quotes' failed")).toBeTruthy();
  });
});
