import { toHaveNoViolations } from "jest-axe";
import { expect } from "vitest";

// Make `expect(...).toHaveNoViolations()` available in every test. jest-axe
// ships Jest-namespace matcher types, so map them onto Vitest's matchers below.
// Lives under src/ (not the interface root) so tsc — whose include is ["src",
// "vite.config.ts"] — picks up this declaration-merging augmentation.
expect.extend(toHaveNoViolations);

interface AxeMatchers {
  toHaveNoViolations(): void;
}

declare module "vitest" {
  // biome-ignore lint/suspicious/noExplicitAny: must match Vitest's own Assertion<T = any> signature for declaration merging
  interface Assertion<T = any> extends AxeMatchers {}
  interface AsymmetricMatchersContaining extends AxeMatchers {}
}
