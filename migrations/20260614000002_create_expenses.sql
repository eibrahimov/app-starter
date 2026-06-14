-- Expenses: the core record. `amount_cents` is an integer minor-unit amount
-- (e.g. 1299 = 12.99 in the app's base currency) so money math stays exact.
-- `category_id` is nullable so an expense can be uncategorized; deleting a
-- category nulls it rather than cascading. `spent_on` is a 'YYYY-MM-DD' date,
-- indexed for fast month filtering and ordering.
CREATE TABLE expenses (
    id TEXT PRIMARY KEY,
    amount_cents INTEGER NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    category_id TEXT,
    spent_on TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE INDEX idx_expenses_spent_on ON expenses (spent_on);
CREATE INDEX idx_expenses_category_id ON expenses (category_id);
