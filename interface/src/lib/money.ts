// Money helpers. Amounts cross the API as integer minor units (cents); we only
// convert to a human, currency-formatted string at the UI edge.

/** Formats integer cents as a localized currency string, e.g. 1299 -> "$12.99". */
export function formatCents(cents: number, currency: string): string {
  try {
    return new Intl.NumberFormat(undefined, {
      style: "currency",
      currency,
    }).format(cents / 100);
  } catch {
    // Unknown currency code: fall back to a plain number + code.
    return `${(cents / 100).toFixed(2)} ${currency}`;
  }
}

/**
 * Parses user-entered text like "12.34" or "1,234.50" into positive integer
 * cents. Returns null when the input is empty, non-numeric, or not positive.
 */
export function parseAmount(input: string): number | null {
  const cleaned = input.replace(/[^0-9.]/g, "");
  if (!cleaned) return null;
  const value = Number(cleaned);
  if (!Number.isFinite(value) || value <= 0) return null;
  return Math.round(value * 100);
}

/** Today's date as `YYYY-MM-DD` in the local timezone. */
export function today(): string {
  const now = new Date();
  const offset = now.getTimezoneOffset() * 60_000;
  return new Date(now.getTime() - offset).toISOString().slice(0, 10);
}

/** The current month as `YYYY-MM` in the local timezone. */
export function currentMonth(): string {
  return today().slice(0, 7);
}
