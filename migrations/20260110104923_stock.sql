-- Add migration script here
CREATE TABLE stocks (
    id INTEGER PRIMARY KEY,
    updated_at TEXT DEFAULT(
        strftime ('%Y-%m-%dT%H:%M:%fZ', 'now')
    ),
    branch_id INTEGER NOT NULL,
    product_variant_id INTEGER NOT NULL,
    quantity INTEGER NOT NULL,
    min_stock INTEGER,
    max_stock INTEGER,
    last_buy_price INTEGER,
    metadata TEXT,
    FOREIGN KEY (branch_id) REFERENCES branches (id),
    FOREIGN KEY (product_variant_id) REFERENCES product_variants (id) ON DELETE CASCADE
);

CREATE INDEX idx_stocks_branch_id ON stocks (branch_id);

CREATE INDEX idx_stocks_product_variant_id ON stocks (product_variant_id);

-- Unique constraint: one stock record per branch-variant combination
CREATE UNIQUE INDEX idx_stocks_unique ON stocks (branch_id, product_variant_id);