import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { ErrorState } from "./ErrorState";

describe("ErrorState", () => {
  it("renders the message prop text", () => {
    render(<ErrorState message="Something went wrong" />);
    expect(screen.getByText("Something went wrong")).toBeTruthy();
  });

  it("renders the message inside a paragraph element", () => {
    render(<ErrorState message="Boom" />);
    expect(screen.getByText("Boom").tagName).toBe("P");
  });

  it("applies the error styling classes", () => {
    render(<ErrorState message="Styled error" />);
    const node = screen.getByText("Styled error");
    expect(node.className).toContain("text-sm");
    expect(node.className).toContain("text-red-400");
  });

  it("renders a distinct message when the prop changes", () => {
    const { rerender } = render(<ErrorState message="First" />);
    expect(screen.getByText("First")).toBeTruthy();
    rerender(<ErrorState message="Second" />);
    expect(screen.getByText("Second")).toBeTruthy();
    expect(screen.queryByText("First")).toBeNull();
  });

  it("renders an empty paragraph for an empty message", () => {
    const { container } = render(<ErrorState message="" />);
    const paragraph = container.querySelector("p");
    expect(paragraph).toBeTruthy();
    expect(paragraph?.textContent).toBe("");
  });

  it("renders messages containing special characters verbatim", () => {
    render(<ErrorState message="Error: <code> & 'quotes' failed" />);
    expect(screen.getByText("Error: <code> & 'quotes' failed")).toBeTruthy();
  });
});
