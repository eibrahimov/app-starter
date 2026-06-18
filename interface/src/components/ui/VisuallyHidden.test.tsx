import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { VisuallyHidden } from "./VisuallyHidden";

describe("VisuallyHidden", () => {
  it("renders its children in the document", () => {
    render(<VisuallyHidden>Close menu</VisuallyHidden>);
    expect(screen.getByText("Close menu")).toBeTruthy();
  });

  it("keeps text content available even though it is visually hidden", () => {
    render(<VisuallyHidden>Loading</VisuallyHidden>);
    expect(screen.getByText("Loading").textContent).toBe("Loading");
  });

  it("renders nested elements as children", () => {
    render(
      <VisuallyHidden>
        <span>Step 1 of 3</span>
      </VisuallyHidden>,
    );
    expect(screen.getByText("Step 1 of 3").tagName).toBe("SPAN");
  });

  it("forwards extra props to the underlying element", () => {
    render(<VisuallyHidden data-testid="vh">Saved</VisuallyHidden>);
    expect(screen.getByTestId("vh").textContent).toBe("Saved");
  });
});
