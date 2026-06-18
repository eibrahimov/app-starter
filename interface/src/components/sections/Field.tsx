import { type ReactNode, useId } from "react";

interface FieldRenderProps {
  id: string;
  "aria-describedby"?: string;
  "aria-invalid"?: boolean;
}

interface FieldProps {
  label: string;
  hint?: string;
  error?: string;
  children: (props: FieldRenderProps) => ReactNode;
}

// Wires a label, optional hint, and error message to a control via
// id / aria-describedby / aria-invalid — the accessible-form pattern a bare
// <Input> lacks (a placeholder is not a label). Render-prop so it composes with
// the existing Input, which spreads the id and aria-* straight through.
export function Field({ label, hint, error, children }: FieldProps) {
  const id = useId();
  const hintId = `${id}-hint`;
  const errorId = `${id}-error`;
  const describedBy =
    [hint && hintId, error && errorId].filter(Boolean).join(" ") || undefined;

  return (
    <div className="space-y-1">
      <label htmlFor={id} className="font-medium text-sm">
        {label}
      </label>
      {children({
        id,
        "aria-describedby": describedBy,
        "aria-invalid": !!error,
      })}
      {hint && (
        <p id={hintId} className="text-muted-foreground text-xs">
          {hint}
        </p>
      )}
      {error && (
        <p id={errorId} role="alert" className="text-destructive text-xs">
          {error}
        </p>
      )}
    </div>
  );
}
