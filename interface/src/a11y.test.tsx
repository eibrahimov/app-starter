import { render } from "@testing-library/react";
import { axe } from "jest-axe";
import { describe, expect, it } from "vitest";
import { FilterBar } from "./components/sections/FilterBar";
import { Badge } from "./components/ui/Badge";
import { Button } from "./components/ui/Button";
import { Card } from "./components/ui/Card";
import { EmptyState } from "./components/ui/EmptyState";
import { ErrorState } from "./components/ui/ErrorState";

// Landmark/region rules don't apply to isolated component fragments; the
// Playwright smoke (e2e/a11y.spec.ts) covers page-level landmarks + contrast.
const opts = { rules: { region: { enabled: false } } };

describe("accessibility (axe)", () => {
  it("Button has no violations", async () => {
    const { container } = render(<Button>Save</Button>);
    expect(await axe(container, opts)).toHaveNoViolations();
  });

  it("Badge has no violations", async () => {
    const { container } = render(<Badge tone="emerald">Active</Badge>);
    expect(await axe(container, opts)).toHaveNoViolations();
  });

  it("Card row has no violations", async () => {
    const { container } = render(
      <ul>
        <Card as="li">Row content</Card>
      </ul>,
    );
    expect(await axe(container, opts)).toHaveNoViolations();
  });

  it("FilterBar has no violations", async () => {
    const options = ["all", "draft"] as const;
    const { container } = render(
      <FilterBar options={options} value="all" onChange={() => {}} />,
    );
    expect(await axe(container, opts)).toHaveNoViolations();
  });

  it("EmptyState has no violations", async () => {
    const { container } = render(<EmptyState message="Nothing here" />);
    expect(await axe(container, opts)).toHaveNoViolations();
  });

  it("ErrorState has no violations", async () => {
    const { container } = render(<ErrorState message="Boom" />);
    expect(await axe(container, opts)).toHaveNoViolations();
  });
});
