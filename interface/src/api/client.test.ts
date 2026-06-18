import { describe, expect, it } from "vitest";
import { api } from "./client";

describe("api client", () => {
  it("is defined", () => {
    expect(api).toBeTruthy();
  });

  it("exposes a GET method", () => {
    expect(typeof api.GET).toBe("function");
  });

  it("exposes a POST method", () => {
    expect(typeof api.POST).toBe("function");
  });

  it("exposes a DELETE method", () => {
    expect(typeof api.DELETE).toBe("function");
  });
});
