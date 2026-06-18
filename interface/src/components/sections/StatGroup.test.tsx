import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { StatGroup } from "./StatGroup";

describe("StatGroup", () => {
  it("joins multiple stats with a slash separator", () => {
    render(
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
    render(<StatGroup stats={[{ label: "archived", value: 2 }]} />);
    expect(screen.getByText("2 archived")).toBeTruthy();
  });

  it("joins three stats in order", () => {
    render(
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
    render(<StatGroup stats={[{ label: "total", value: "many" }]} />);
    expect(screen.getByText("many total")).toBeTruthy();
  });

  it("renders an empty paragraph when given no stats", () => {
    const { container } = render(<StatGroup stats={[]} />);
    const paragraph = container.querySelector("p");
    expect(paragraph).toBeTruthy();
    expect(paragraph?.textContent).toBe("");
  });
});
