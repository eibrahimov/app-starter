// Ambient typings for jest-axe, which ships no types of its own. We deliberately
// avoid the `@types/jest-axe` package: it depends on `@types/jest`, which would
// inject Jest's global namespace into this Vitest-only project and clobber
// `expect`. This file has no imports/exports, so `declare module` is an ambient
// declaration (not a module augmentation) and can type the otherwise-untyped
// module. Only the two symbols the suite uses are declared.
declare module "jest-axe" {
  // axe() audits a DOM node and resolves to a results object; typed `unknown`
  // because its only consumer is the toHaveNoViolations matcher below.
  export function axe(
    html: Element | Document | string,
    options?: Record<string, unknown>,
  ): Promise<unknown>;
  export const toHaveNoViolations: {
    toHaveNoViolations(received: unknown): { pass: boolean; message(): string };
  };
}
