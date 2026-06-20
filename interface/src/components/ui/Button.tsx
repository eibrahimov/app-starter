import { Button as ThemesButton } from "@radix-ui/themes";
import type { ButtonHTMLAttributes } from "react";

// Public variant names are preserved so callers don't change. Each maps to a
// Radix Themes variant + color: primary -> solid (theme accent), ghost -> ghost,
// and the status variants stay ghost-styled text actions but carry a semantic
// color (danger -> red, success -> grass, warning -> amber).
export type ButtonVariant =
  | "primary"
  | "ghost"
  | "danger"
  | "success"
  | "warning";

type ThemesVariant = "solid" | "ghost";
type ThemesColor = "red" | "grass" | "amber" | undefined;

const VARIANT_MAP: Record<
  ButtonVariant,
  { variant: ThemesVariant; color: ThemesColor }
> = {
  primary: { variant: "solid", color: undefined },
  ghost: { variant: "ghost", color: undefined },
  danger: { variant: "ghost", color: "red" },
  success: { variant: "ghost", color: "grass" },
  warning: { variant: "ghost", color: "amber" },
};

// Omit the native `color` attribute: it collides with the Themes Button `color`
// prop we set from VARIANT_MAP (native color is `string`, Themes color is a hue
// union), and callers theme via `variant`, not a raw color.
type ButtonProps = Omit<ButtonHTMLAttributes<HTMLButtonElement>, "color"> & {
  variant?: ButtonVariant;
};

// Defaults to type="button" so callers satisfy the explicit-type rule for free;
// pass type="submit" for form submissions. Themes Button supplies focus rings
// and adequate touch targets, so no hand-rolled focus or coarse styling is needed.
export function Button({ variant = "primary", type, ...props }: ButtonProps) {
  const { variant: themesVariant, color } = VARIANT_MAP[variant];
  return (
    <ThemesButton
      type={type ?? "button"}
      variant={themesVariant}
      color={color}
      {...props}
    />
  );
}
