import { Theme } from "@radix-ui/themes";
import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { VisuallyHidden } from "./VisuallyHidden";

describe("VisuallyHidden", () => {
  it("renders its children in the document", () => {
    render(
      <Theme>
        <VisuallyHidden>Close menu</VisuallyHidden>
      </Theme>,
    );
    expect(screen.getByText("Close menu")).toBeTruthy();
  });

  it("keeps text content available even though it is visually hidden", () => {
    render(
      <Theme>
        <VisuallyHidden>Loading</VisuallyHidden>
      </Theme>,
    );
    expect(screen.getByText("Loading").textContent).toBe("Loading");
  });

  it("renders nested elements as children", () => {
    render(
      <Theme>
        <VisuallyHidden>
          <strong>Step 1 of 3</strong>
        </VisuallyHidden>
      </Theme>,
    );
    expect(screen.getByText("Step 1 of 3").tagName).toBe("STRONG");
  });

  it("forwards extra props to the underlying element", () => {
    render(
      <Theme>
        <VisuallyHidden data-testid="vh">Saved</VisuallyHidden>
      </Theme>,
    );
    expect(screen.getByTestId("vh").textContent).toBe("Saved");
  });
});
