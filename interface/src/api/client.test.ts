import { afterEach, describe, expect, it, vi } from "vitest";
import { api, resolveApiBaseUrl } from "./client";

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

const TAURI = "__TAURI_INTERNALS__";

describe("resolveApiBaseUrl", () => {
  afterEach(() => {
    vi.unstubAllEnvs();
    delete (window as unknown as Record<string, unknown>)[TAURI];
  });

  it("defaults to a relative, same-origin base in the browser", () => {
    expect(resolveApiBaseUrl()).toBe("/");
  });

  it("uses the sidecar loopback URL inside the Tauri desktop shell", () => {
    (window as unknown as Record<string, unknown>)[TAURI] = {};
    expect(resolveApiBaseUrl()).toBe("http://127.0.0.1:8080");
  });

  it("honors VITE_API_BASE_URL over the browser default", () => {
    vi.stubEnv("VITE_API_BASE_URL", "https://api.example.com");
    expect(resolveApiBaseUrl()).toBe("https://api.example.com");
  });

  it("honors VITE_API_BASE_URL over the Tauri default", () => {
    (window as unknown as Record<string, unknown>)[TAURI] = {};
    vi.stubEnv("VITE_API_BASE_URL", "http://127.0.0.1:9090");
    expect(resolveApiBaseUrl()).toBe("http://127.0.0.1:9090");
  });

  it("falls back to the default when VITE_API_BASE_URL is blank", () => {
    vi.stubEnv("VITE_API_BASE_URL", "   ");
    expect(resolveApiBaseUrl()).toBe("/");
  });
});
