import { describe, expect, it } from "vitest";
import { cx } from "./cx";

describe("cx", () => {
  it("returns an empty string when given no arguments", () => {
    expect(cx()).toBe("");
  });

  it("returns a single class unchanged", () => {
    expect(cx("btn")).toBe("btn");
  });

  it("joins multiple truthy classes with a single space", () => {
    expect(cx("btn", "btn-primary", "rounded")).toBe("btn btn-primary rounded");
  });

  it("drops false entries", () => {
    expect(cx("btn", false, "active")).toBe("btn active");
  });

  it("drops null entries", () => {
    expect(cx("btn", null, "active")).toBe("btn active");
  });

  it("drops undefined entries", () => {
    expect(cx("btn", undefined, "active")).toBe("btn active");
  });

  it("drops empty-string entries", () => {
    expect(cx("btn", "", "active")).toBe("btn active");
  });

  it("drops a mix of all falsy kinds at once", () => {
    expect(cx(false, "btn", null, undefined, "", "active")).toBe("btn active");
  });

  it("returns an empty string when every argument is falsy", () => {
    expect(cx(false, null, undefined, "")).toBe("");
  });

  it("preserves leading and trailing truthy classes", () => {
    expect(cx("first", false, "middle", null, "last")).toBe(
      "first middle last",
    );
  });
});
