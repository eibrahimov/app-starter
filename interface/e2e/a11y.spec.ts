import AxeBuilder from "@axe-core/playwright";
import { expect, type Page, type Route, test } from "@playwright/test";

// The page smoke runs against the Vite dev server only — no Rust backend — so we
// intercept every /api call and fulfill it with a populated, schema-shaped body
// BEFORE navigating. Without this, the backend is unreachable and axe would only
// ever audit the loading/error/empty states; the populated rows (and every Badge
// tone, where the AA contrast actually matters) would never render.
const HEALTH = { database: "ok", status: "ok", version: "0.1.1" };

const ITEMS = [
  {
    id: "11111111-1111-1111-1111-111111111111",
    title: "Buy groceries",
    done: false,
    created_at: "2026-01-02T10:00:00Z",
  },
  {
    id: "22222222-2222-2222-2222-222222222222",
    title: "Ship the release",
    done: true,
    created_at: "2026-01-03T10:00:00Z",
  },
];

// One post per status so every Badge tone (amber/emerald/zinc) is on screen.
const POSTS = [
  {
    id: "33333333-3333-3333-3333-333333333333",
    title: "Draft post",
    body: "b",
    status: "draft",
    created_at: "2026-01-04T10:00:00Z",
    published_at: null,
  },
  {
    id: "44444444-4444-4444-4444-444444444444",
    title: "Published post",
    body: "b",
    status: "published",
    created_at: "2026-01-05T10:00:00Z",
    published_at: "2026-01-06T10:00:00Z",
  },
  {
    id: "55555555-5555-5555-5555-555555555555",
    title: "Archived post",
    body: "b",
    status: "archived",
    created_at: "2026-01-07T10:00:00Z",
    published_at: "2026-01-06T10:00:00Z",
  },
];

const POST_STATS = { draft: 1, published: 1, archived: 1 };

const AXE_TAGS = ["wcag2a", "wcag2aa", "wcag21a", "wcag21aa", "wcag22aa"];

function json(route: Route, body: unknown) {
  return route.fulfill({
    status: 200,
    contentType: "application/json",
    body: JSON.stringify(body),
  });
}

// Match on the URL *pathname*, not a `**/api/**` glob: in Vite dev the app's own
// source module is served at /src/api/client.ts, which a glob would also catch —
// aborting it would stop the SPA from booting. The client uses baseUrl "/"
// (api/client.ts), so real calls are same-origin /api/... paths. Check
// /posts/stats before /posts so the longer path wins; honor ?status= so the mock
// mirrors the real filtered list.
async function mockApi(page: Page) {
  await page.route(
    (url) => url.pathname.startsWith("/api/"),
    (route) => {
      const url = new URL(route.request().url());
      const path = url.pathname;
      const status = url.searchParams.get("status");

      if (path === "/api/health") return json(route, HEALTH);
      if (path === "/api/v1/items") return json(route, ITEMS);
      if (path === "/api/v1/posts/stats") return json(route, POST_STATS);
      if (path === "/api/v1/posts") {
        return json(
          route,
          status ? POSTS.filter((p) => p.status === status) : POSTS,
        );
      }
      // Unknown API path: abort so a real coverage gap surfaces loudly instead
      // of silently hanging until the test times out.
      return route.abort("blockedbyclient");
    },
  );
}

test.beforeEach(async ({ page }) => {
  await mockApi(page);
});

async function expectNoViolations(page: Page) {
  const { violations } = await new AxeBuilder({ page })
    .withTags(AXE_TAGS)
    .analyze();
  expect(violations).toEqual([]);
}

test("no detectable a11y violations on / (resolved health line)", async ({
  page,
}) => {
  await page.goto("/");
  // Wait for the success branch so axe audits the resolved state, not loading.
  await expect(page.getByText(/Backend ok/)).toBeVisible();
  await expectNoViolations(page);
});

test("no detectable a11y violations on /items (populated rows)", async ({
  page,
}) => {
  await page.goto("/items");
  // Rows only mount after React Query resolves the mocked GET.
  await expect(page.getByText("Buy groceries")).toBeVisible();
  await expect(page.getByText("Ship the release")).toBeVisible();
  await expectNoViolations(page);
});

test("no detectable a11y violations on /posts (all Badge tones)", async ({
  page,
}) => {
  await page.goto("/posts");
  // Each status row renders a differently-toned Badge; wait for all three.
  await expect(page.getByText("Draft post")).toBeVisible();
  await expect(page.getByText("Published post")).toBeVisible();
  await expect(page.getByText("Archived post")).toBeVisible();
  await expectNoViolations(page);
});
