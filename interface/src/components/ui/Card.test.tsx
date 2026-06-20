import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { Card } from "./Card";

describe("Card", () => {
  it("renders a <div> by default", () => {
    const { container } = render(<Card />);
    expect(container.querySelector("div")).toBeTruthy();
    expect(container.querySelector("li")).toBeNull();
  });

  it('renders an <li> when as="li"', () => {
    const { container } = render(<Card as="li" />);
    expect(container.querySelector("li")).toBeTruthy();
    expect(container.querySelector("div")).toBeNull();
  });

  it("applies the base classes", () => {
    const { container } = render(<Card />);
    const el = container.querySelector("div");
    expect(el?.className).toContain("flex");
    expect(el?.className).toContain("items-center");
    expect(el?.className).toContain("gap-3");
    expect(el?.className).toContain("rounded-md");
    expect(el?.className).toContain("border");
    expect(el?.className).toContain("border-border");
    expect(el?.className).toContain("px-3");
    expect(el?.className).toContain("py-2");
  });

  it("merges className with the base classes", () => {
    const { container } = render(<Card className="custom-class" />);
    const el = container.querySelector("div");
    expect(el?.className).toContain("custom-class");
    expect(el?.className).toContain("flex");
  });

  it("keeps only the base classes when no className is given", () => {
    const { container } = render(<Card />);
    const el = container.querySelector("div");
    expect(el?.className).toBe(
      "flex items-center gap-3 rounded-md border border-border px-3 py-2",
    );
  });

  it("forwards arbitrary props to the underlying element", () => {
    const { container } = render(
      <Card data-testid="card-el" aria-label="a card" role="group" />,
    );
    const el = container.querySelector("div");
    expect(el?.getAttribute("data-testid")).toBe("card-el");
    expect(el?.getAttribute("aria-label")).toBe("a card");
    expect(el?.getAttribute("role")).toBe("group");
  });

  it("renders children", () => {
    render(
      <Card>
        <span>inside the card</span>
      </Card>,
    );
    expect(screen.getByText("inside the card")).toBeTruthy();
  });

  it("renders children inside an <li> shell", () => {
    const { container } = render(
      <Card as="li">
        <span>row content</span>
      </Card>,
    );
    const li = container.querySelector("li");
    expect(li).toBeTruthy();
    expect(li?.textContent).toBe("row content");
  });
});
