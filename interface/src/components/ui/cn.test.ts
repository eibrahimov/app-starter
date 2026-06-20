import { describe, expect, it } from "vitest";
import { cn } from "./cn";

describe("cn", () => {
  it("joins truthy class names", () => {
    expect(cn("a", "b")).toBe("a b");
  });

  it("drops falsy entries", () => {
    expect(cn("a", false, null, undefined, "", "b")).toBe("a b");
  });

  it("supports conditional objects and arrays", () => {
    expect(cn("a", ["b", { c: true, d: false }])).toBe("a b c");
  });

  it("merges conflicting tailwind utilities so the last one wins", () => {
    // The behaviour the old `cx` joiner could not provide: a caller's override
    // replaces the component default instead of both classes shipping.
    expect(cn("bg-card px-2", "bg-background")).toBe("px-2 bg-background");
  });
});
