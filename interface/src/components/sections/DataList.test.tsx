import type { UseQueryResult } from "@tanstack/react-query";
import { screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { renderWithTheme } from "../../test-utils";
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
  it("shows a spinner and the loading message while loading", () => {
    renderWithTheme(
      <DataList
        query={fakeQuery<string>({ isLoading: true })}
        renderItem={renderItem}
        emptyMessage="empty"
        loadingMessage="loading now"
      />,
    );
    expect(screen.getByText("loading now")).toBeTruthy();
    expect(screen.getByRole("status")).toBeTruthy();
  });

  it("shows the error message on error when there is no data", () => {
    renderWithTheme(
      <DataList
        query={fakeQuery<string>({ isError: true })}
        renderItem={renderItem}
        emptyMessage="empty"
        errorMessage="broke"
      />,
    );
    expect(screen.getByText("broke")).toBeTruthy();
  });

  it("keeps the previous list visible when a refetch errors", () => {
    renderWithTheme(
      <DataList
        query={fakeQuery<string>({ isError: true, data: ["alpha"] })}
        renderItem={renderItem}
        emptyMessage="empty"
        errorMessage="broke"
      />,
    );
    expect(screen.getByText("alpha")).toBeTruthy();
    expect(screen.queryByText("broke")).toBeNull();
  });

  it("shows the empty message for an empty list", () => {
    renderWithTheme(
      <DataList
        query={fakeQuery<string>({ data: [] })}
        renderItem={renderItem}
        emptyMessage="nothing here"
      />,
    );
    expect(screen.getByText("nothing here")).toBeTruthy();
  });

  it("renders items via renderItem", () => {
    renderWithTheme(
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
