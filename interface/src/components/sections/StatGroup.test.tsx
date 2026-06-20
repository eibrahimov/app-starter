import { screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { renderWithTheme } from "../../test-utils";
import { StatGroup } from "./StatGroup";

describe("StatGroup", () => {
  it("joins multiple stats with a slash separator", () => {
    renderWithTheme(
      <StatGroup
        stats={[
          { label: "draft", value: 1 },
          { label: "published", value: 0 },
        ]}
      />,
    );
    expect(screen.getByText("1 draft / 0 published")).toBeTruthy();
  });

  it("renders a single stat without a separator", () => {
    renderWithTheme(<StatGroup stats={[{ label: "archived", value: 2 }]} />);
    expect(screen.getByText("2 archived")).toBeTruthy();
  });

  it("joins three stats in order", () => {
    renderWithTheme(
      <StatGroup
        stats={[
          { label: "draft", value: 1 },
          { label: "published", value: 0 },
          { label: "archived", value: 2 },
        ]}
      />,
    );
    expect(screen.getByText("1 draft / 0 published / 2 archived")).toBeTruthy();
  });

  it("renders string values as-is", () => {
    renderWithTheme(<StatGroup stats={[{ label: "total", value: "many" }]} />);
    expect(screen.getByText("many total")).toBeTruthy();
  });

  it("renders an empty paragraph when given no stats", () => {
    const { container } = renderWithTheme(<StatGroup stats={[]} />);
    const paragraph = container.querySelector("p");
    expect(paragraph).toBeTruthy();
    expect(paragraph?.textContent).toBe("");
  });
});
