// Tiny class-name joiner — avoids a clsx dependency. Falsy entries drop out.
export function cx(...classes: (string | false | null | undefined)[]): string {
  return classes.filter(Boolean).join(" ");
}
