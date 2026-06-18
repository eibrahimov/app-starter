import type { UseQueryResult } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { DataList } from "./DataList";

// Minimal stand-in for the fields DataList reads off a query result.
function fakeQuery<T>(
  state: Partial<UseQueryResult<T[]>>,
): UseQueryResult<T[]> {
  return {
    isLoading: false,
    isError: false,
    data: undefined,
    ...state,
  } as UseQueryResult<T[]>;
}

const renderItem = (item: string) => <li key={item}>{item}</li>;

describe("DataList", () => {
  it("shows the loading message while loading", () => {
    render(
      <DataList
        query={fakeQuery<string>({ isLoading: true })}
        renderItem={renderItem}
        emptyMessage="empty"
        loadingMessage="loading now"
      />,
    );
    expect(screen.getByText("loading now")).toBeTruthy();
  });

  it("shows the error message on error", () => {
    render(
      <DataList
        query={fakeQuery<string>({ isError: true })}
        renderItem={renderItem}
        emptyMessage="empty"
        errorMessage="broke"
      />,
    );
    expect(screen.getByText("broke")).toBeTruthy();
  });

  it("shows the empty message for an empty list", () => {
    render(
      <DataList
        query={fakeQuery<string>({ data: [] })}
        renderItem={renderItem}
        emptyMessage="nothing here"
      />,
    );
    expect(screen.getByText("nothing here")).toBeTruthy();
  });

  it("renders items via renderItem", () => {
    render(
      <DataList
        query={fakeQuery<string>({ data: ["alpha", "beta"] })}
        renderItem={renderItem}
        emptyMessage="empty"
      />,
    );
    expect(screen.getByText("alpha")).toBeTruthy();
    expect(screen.getByText("beta")).toBeTruthy();
  });
});
