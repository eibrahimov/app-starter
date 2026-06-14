-- Expense categories. Each expense optionally belongs to one category.
-- `monthly_budget_cents` is nullable: a category may have no budget. Amounts
-- are stored as integer minor units (cents) to avoid floating-point money bugs.
CREATE TABLE categories (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    color TEXT NOT NULL DEFAULT '#6366f1',
    monthly_budget_cents INTEGER,
    created_at TEXT NOT NULL
);

CREATE UNIQUE INDEX idx_categories_name ON categories (name);
