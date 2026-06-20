import { Theme } from "@radix-ui/themes";
import { render, screen } from "@testing-library/react";
import type { ReactElement } from "react";
import { describe, expect, it } from "vitest";
import { Card } from "./Card";

function renderThemed(ui: ReactElement) {
  return render(<Theme>{ui}</Theme>);
}

describe("Card", () => {
  it("renders a <div> by default", () => {
    const { container } = renderThemed(<Card />);
    expect(container.querySelector("div")).toBeTruthy();
    expect(container.querySelector("li")).toBeNull();
  });

  it('renders an <li> when as="li"', () => {
    const { container } = renderThemed(
      <ul>
        <Card as="li" />
      </ul>,
    );
    expect(container.querySelector("li")).toBeTruthy();
  });

  it("forwards arbitrary props to the underlying element", () => {
    renderThemed(
      <Card data-testid="card-el" aria-label="a card" role="group" />,
    );
    const el = screen.getByTestId("card-el");
    expect(el.getAttribute("aria-label")).toBe("a card");
    expect(el.getAttribute("role")).toBe("group");
  });

  it("forwards props onto the <li> shell", () => {
    renderThemed(
      <ul>
        <Card as="li" data-testid="li-card" aria-label="a row" />
      </ul>,
    );
    const el = screen.getByTestId("li-card");
    expect(el.tagName.toLowerCase()).toBe("li");
    expect(el.getAttribute("aria-label")).toBe("a row");
  });

  it("renders children", () => {
    renderThemed(
      <Card>
        <span>inside the card</span>
      </Card>,
    );
    expect(screen.getByText("inside the card")).toBeTruthy();
  });

  it("renders children inside an <li> shell", () => {
    const { container } = renderThemed(
      <ul>
        <Card as="li">
          <span>row content</span>
        </Card>
      </ul>,
    );
    const li = container.querySelector("li");
    expect(li).toBeTruthy();
    expect(li?.textContent).toBe("row content");
  });
});
