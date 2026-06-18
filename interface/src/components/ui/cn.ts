import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";

// Join + de-duplicate Tailwind class names. Unlike the old `cx` joiner, this
// reconciles conflicting utilities (the last one wins), so a caller's
// `className` can reliably override a default already baked into a component.
//
// twMerge handles the default Tailwind scales plus the semantic color tokens
// used here (verified by cn.test.ts). If you later add CUSTOM utilities twMerge
// can't infer (e.g. a bespoke `text-*` size group or a new spacing scale), build
// a configured instance with `extendTailwindMerge({ extend: { classGroups: {…} }})`
// so those conflicting custom utilities still de-dupe correctly.
export function cn(...inputs: ClassValue[]): string {
  return twMerge(clsx(inputs));
}
