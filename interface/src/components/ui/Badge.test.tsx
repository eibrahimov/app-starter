import { Theme } from "@radix-ui/themes";
import { render, screen } from "@testing-library/react";
import type { ReactNode } from "react";
import { describe, expect, it } from "vitest";
import { Badge } from "./Badge";

// Themes Badge needs Theme context to resolve its tokens in jsdom.
function renderBadge(ui: ReactNode) {
  return render(<Theme>{ui}</Theme>);
}

// Radix Themes encodes the resolved color as a data-accent-color attribute on
// the rendered element; asserting on that proves the tone mapped to the right
// accessible hue without coupling to class strings.
function accentColorOf(text: string): string | null {
  return screen.getByText(text).getAttribute("data-accent-color");
}

describe("Badge", () => {
  it("renders its children", () => {
    renderBadge(<Badge>Draft</Badge>);
    expect(screen.getByText("Draft")).toBeTruthy();
  });

  it("defaults to the neutral tone (gray)", () => {
    renderBadge(<Badge>Draft</Badge>);
    expect(accentColorOf("Draft")).toBe("gray");
  });

  it("maps the emerald tone to grass", () => {
    renderBadge(<Badge tone="emerald">Active</Badge>);
    expect(accentColorOf("Active")).toBe("grass");
  });

  it("maps the amber tone to amber", () => {
    renderBadge(<Badge tone="amber">Pending</Badge>);
    expect(accentColorOf("Pending")).toBe("amber");
  });

  it("maps the red tone to red", () => {
    renderBadge(<Badge tone="red">Error</Badge>);
    expect(accentColorOf("Error")).toBe("red");
  });

  it("maps the zinc tone to gray", () => {
    renderBadge(<Badge tone="zinc">Muted</Badge>);
    expect(accentColorOf("Muted")).toBe("gray");
  });

  it("forwards arbitrary attributes via ...props", () => {
    const { container } = renderBadge(
      <Badge id="status-badge" title="status" data-testid="badge">
        Draft
      </Badge>,
    );
    const el = container.querySelector("#status-badge") as HTMLElement;
    expect(el).toBeTruthy();
    expect(el.getAttribute("title")).toBe("status");
    expect(el.getAttribute("data-testid")).toBe("badge");
  });
});
