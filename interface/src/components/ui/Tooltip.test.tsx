import { screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { renderWithTheme } from "../../test-utils";
import { Tooltip } from "./Tooltip";

describe("Tooltip", () => {
  it("renders the trigger children", () => {
    renderWithTheme(
      <Tooltip label="More info">
        <button type="button">Help</button>
      </Tooltip>,
    );
    expect(screen.getByText("Help")).toBeTruthy();
  });

  it("renders the trigger as a real button via the Themes trigger", () => {
    renderWithTheme(
      <Tooltip label="More info">
        <button type="button">Help</button>
      </Tooltip>,
    );
    const trigger = screen.getByText("Help");
    expect(trigger.tagName).toBe("BUTTON");
  });

  it("does not show the tooltip label before interaction", () => {
    renderWithTheme(
      <Tooltip label="hidden until hover">
        <button type="button">Help</button>
      </Tooltip>,
    );
    expect(screen.queryByText("hidden until hover")).toBeNull();
  });

  it("renders non-button trigger children", () => {
    renderWithTheme(
      <Tooltip label="info">
        <span>just text</span>
      </Tooltip>,
    );
    const trigger = screen.getByText("just text");
    expect(trigger.tagName).toBe("SPAN");
  });
});
