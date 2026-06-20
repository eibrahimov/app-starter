import { type ReactNode, useId } from "react";
import { Flex, Text } from "@radix-ui/themes";

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
    <Flex direction="column" gap="1">
      <Text as="label" htmlFor={id} size="2" weight="medium">
        {label}
      </Text>
      {children({
        id,
        "aria-describedby": describedBy,
        "aria-invalid": !!error,
      })}
      {hint && (
        <Text id={hintId} as="p" size="1" color="gray">
          {hint}
        </Text>
      )}
      {error && (
        <Text id={errorId} as="p" role="alert" size="1" color="red">
          {error}
        </Text>
      )}
    </Flex>
  );
}
