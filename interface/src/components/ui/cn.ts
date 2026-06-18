import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";

// Join + de-duplicate Tailwind class names. Unlike the old `cx` joiner, this
// reconciles conflicting utilities (the last one wins), so a caller's
// `className` can reliably override a default already baked into a component.
export function cn(...inputs: ClassValue[]): string {
  return twMerge(clsx(inputs));
}
