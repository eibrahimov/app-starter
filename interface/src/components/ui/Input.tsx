import { TextField } from "@radix-ui/themes";
import type { ComponentProps } from "react";

// Base the props on TextField.Root itself so the full prop set stays type-safe.
// Raw InputHTMLAttributes are wider than a text field (type="color", numeric
// size, readonly-string[] value) and collide with the Themes props; TextField
// still accepts the standard input attributes callers actually use.
type InputProps = ComponentProps<typeof TextField.Root>;

export function Input(props: InputProps) {
  return <TextField.Root size="2" {...props} />;
}
