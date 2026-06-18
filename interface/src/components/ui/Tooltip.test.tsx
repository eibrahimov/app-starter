import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { Tooltip } from "./Tooltip";

describe("Tooltip", () => {
  it("renders the trigger children", () => {
    render(
      <Tooltip label="More info">
        <button type="button">Help</button>
      </Tooltip>,
    );
    expect(screen.getByText("Help")).toBeTruthy();
  });

  it("renders the trigger as a real button via asChild", () => {
    render(
      <Tooltip label="More info">
        <button type="button">Help</button>
      </Tooltip>,
    );
    const trigger = screen.getByText("Help");
    expect(trigger.tagName).toBe("BUTTON");
  });

  it("does not show the tooltip label before interaction", () => {
    render(
      <Tooltip label="hidden until hover">
        <button type="button">Help</button>
      </Tooltip>,
    );
    expect(screen.queryByText("hidden until hover")).toBeNull();
  });

  it("renders non-button trigger children", () => {
    render(
      <Tooltip label="info">
        <span>just text</span>
      </Tooltip>,
    );
    const trigger = screen.getByText("just text");
    expect(trigger.tagName).toBe("SPAN");
  });
});
