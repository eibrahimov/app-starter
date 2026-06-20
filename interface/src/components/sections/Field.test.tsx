import { Theme } from "@radix-ui/themes";
import { render, screen } from "@testing-library/react";
import type { ReactNode } from "react";
import { describe, expect, it } from "vitest";
import { Input } from "../ui/Input";
import { Field } from "./Field";

// Themes components (TextField via Input, Text labels) need Theme context to
// render in jsdom.
function renderWithTheme(ui: ReactNode) {
  return render(<Theme>{ui}</Theme>);
}

describe("Field", () => {
  it("associates the label with the control", () => {
    renderWithTheme(
      <Field label="Title">{(props) => <Input {...props} />}</Field>,
    );
    // getByLabelText resolves the control through the htmlFor/id association.
    expect(screen.getByLabelText("Title")).toBeTruthy();
  });

  it("marks the control invalid and announces the error", () => {
    renderWithTheme(
      <Field label="Title" error="Required">
        {(props) => <Input {...props} />}
      </Field>,
    );
    const input = screen.getByLabelText("Title");
    expect(input.getAttribute("aria-invalid")).toBe("true");
    expect(input.getAttribute("aria-describedby")).toBeTruthy();
    expect(screen.getByRole("alert").textContent).toBe("Required");
  });

  it("links a hint via aria-describedby", () => {
    renderWithTheme(
      <Field label="Title" hint="Keep it short">
        {(props) => <Input {...props} />}
      </Field>,
    );
    const input = screen.getByLabelText("Title");
    expect(input.getAttribute("aria-describedby")).toBeTruthy();
    expect(screen.getByText("Keep it short")).toBeTruthy();
  });
});
