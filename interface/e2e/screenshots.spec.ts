import { expect, type Page, type Route, test } from "@playwright/test";

// Deterministic README screenshot generator. Like the a11y smoke, this runs
// against the Vite dev server only — no Rust backend — so we intercept every
// /api call and fulfill it with a populated, schema-shaped body BEFORE
// navigating. The fixtures mirror `src/seed.rs` (what `just seed` inserts), so
// the committed images under docs/assets/ faithfully depict the live seeded app.
// Regenerate with `just screenshots`.
const HEALTH = { database: "ok", status: "ok", version: "0.2.0" };

// Mirrors the four items `src/seed.rs` inserts (two done, two not).
const ITEMS = [
  {
    id: "11111111-1111-1111-1111-111111111111",
    title: "Read AGENTS.md for the project conventions",
    done: true,
    created_at: "2026-01-02T10:00:00Z",
  },
  {
    id: "22222222-2222-2222-2222-222222222222",
    title: "Explore the items and posts worked examples",
    done: true,
    created_at: "2026-01-03T10:00:00Z",
  },
  {
    id: "33333333-3333-3333-3333-333333333333",
    title: "Add your first resource with the add-resource skill",
    done: false,
    created_at: "2026-01-04T10:00:00Z",
  },
  {
    id: "44444444-4444-4444-4444-444444444444",
    title: "Replace items and posts with your own domain",
    done: false,
    created_at: "2026-01-05T10:00:00Z",
  },
];

// Mirrors the four posts `src/seed.rs` inserts, one per lifecycle state plus a
// second published row — so every Badge tone (amber/emerald/zinc) is on screen.
const POSTS = [
  {
    id: "55555555-5555-5555-5555-555555555555",
    title: "Welcome to App Starter",
    body: "This post was created by the optional --seed routine.",
    status: "published",
    created_at: "2026-01-06T10:00:00Z",
    published_at: "2026-01-06T11:00:00Z",
  },
  {
    id: "66666666-6666-6666-6666-666666666666",
    title: "How the worked examples fit together",
    body: "Items shows minimal CRUD; posts adds a status lifecycle.",
    status: "published",
    created_at: "2026-01-07T10:00:00Z",
    published_at: "2026-01-07T11:00:00Z",
  },
  {
    id: "77777777-7777-7777-7777-777777777777",
    title: "A work-in-progress draft",
    body: "Drafts stay unpublished until you publish them.",
    status: "draft",
    created_at: "2026-01-08T10:00:00Z",
    published_at: null,
  },
  {
    id: "88888888-8888-8888-8888-888888888888",
    title: "An archived announcement",
    body: "Archived posts are retained but kept out of the active flow.",
    status: "archived",
    created_at: "2026-01-09T10:00:00Z",
    published_at: "2026-01-09T11:00:00Z",
  },
];

const POST_STATS = { draft: 1, published: 2, archived: 1 };

// Capture at a fixed desktop viewport so the three images share dimensions and
// stack cleanly in the README. Default device scale (1) keeps the PNGs small.
test.use({ viewport: { width: 1280, height: 800 } });

function json(route: Route, body: unknown) {
  return route.fulfill({
    status: 200,
    contentType: "application/json",
    body: JSON.stringify(body),
  });
}

// Match on the URL *pathname*, not a `**/api/**` glob: in Vite dev the app's own
// source module is served at /src/api/client.ts, which a glob would also catch —
// aborting it would stop the SPA from booting. Check /posts/stats before /posts
// so the longer path wins; honor ?status= so the mock mirrors the filtered list.
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

test("home screenshot (resolved health line)", async ({ page }) => {
  await page.goto("/");
  // Wait for the success branch so the shot shows the resolved state.
  await expect(page.getByText(/Backend ok/)).toBeVisible();
  await page.screenshot({ path: "../docs/assets/home.png" });
});

test("items screenshot (populated rows)", async ({ page }) => {
  await page.goto("/items");
  // Rows only mount after React Query resolves the mocked GET.
  await expect(
    page.getByText("Read AGENTS.md for the project conventions"),
  ).toBeVisible();
  await expect(
    page.getByText("Replace items and posts with your own domain"),
  ).toBeVisible();
  await page.screenshot({ path: "../docs/assets/items.png" });
});

test("posts screenshot (all Badge tones)", async ({ page }) => {
  await page.goto("/posts");
  // Each status row renders a differently-toned Badge; wait for all three.
  await expect(page.getByText("Welcome to App Starter")).toBeVisible();
  await expect(page.getByText("A work-in-progress draft")).toBeVisible();
  await expect(page.getByText("An archived announcement")).toBeVisible();
  await page.screenshot({ path: "../docs/assets/posts.png" });
});
