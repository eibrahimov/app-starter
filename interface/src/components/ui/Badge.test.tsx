import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { Badge } from "./Badge";

describe("Badge", () => {
  it("renders its children", () => {
    render(<Badge>Draft</Badge>);
    expect(screen.getByText("Draft")).toBeTruthy();
  });

  it("applies the base text-xs class", () => {
    render(<Badge>Draft</Badge>);
    expect(screen.getByText("Draft").className).toContain("text-xs");
  });

  it("defaults to the neutral tone", () => {
    render(<Badge>Draft</Badge>);
    expect(screen.getByText("Draft").className).toContain("text-zinc-400");
  });

  it("applies the emerald tone class", () => {
    render(<Badge tone="emerald">Active</Badge>);
    expect(screen.getByText("Active").className).toContain("text-emerald-400");
  });

  it("applies the amber tone class", () => {
    render(<Badge tone="amber">Pending</Badge>);
    expect(screen.getByText("Pending").className).toContain("text-amber-400");
  });

  it("applies the red tone class", () => {
    render(<Badge tone="red">Error</Badge>);
    expect(screen.getByText("Error").className).toContain("text-red-400");
  });

  it("applies the zinc tone class", () => {
    render(<Badge tone="zinc">Muted</Badge>);
    expect(screen.getByText("Muted").className).toContain("text-zinc-500");
  });

  it("appends a passed className alongside the tone classes", () => {
    render(
      <Badge tone="emerald" className="extra-class">
        Active
      </Badge>,
    );
    const el = screen.getByText("Active");
    expect(el.className).toContain("extra-class");
    expect(el.className).toContain("text-emerald-400");
    expect(el.className).toContain("text-xs");
  });

  it("forwards arbitrary span attributes via ...props", () => {
    const { container } = render(
      <Badge id="status-badge" title="status" data-testid="badge">
        Draft
      </Badge>,
    );
    const el = container.querySelector("#status-badge") as HTMLSpanElement;
    expect(el).toBeTruthy();
    expect(el.getAttribute("title")).toBe("status");
    expect(el.getAttribute("data-testid")).toBe("badge");
  });

  it("renders as a span element", () => {
    const { container } = render(<Badge>Draft</Badge>);
    expect(container.querySelector("span")).toBeTruthy();
  });
});
