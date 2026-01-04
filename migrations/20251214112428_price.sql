-- Add migration script here
CREATE TABLE sell_prices (
    id INTEGER PRIMARY KEY,
    created_at TEXT DEFAULT(
        strftime ('%Y-%m-%dT%H:%M:%fZ', 'now')
    ),
    updated_at TEXT DEFAULT(
        strftime ('%Y-%m-%dT%H:%M:%fZ', 'now')
    ),
    deleted_at TEXT,
    is_deleted INTEGER NOT NULL DEFAULT 0,
    product_variant_id INTEGER NOT NULL,
    uom_id INTEGER,
    branch_id INTEGER,
    quantity INTEGER NOT NULL,
    price INTEGER NOT NULL,
    metadata TEXT,
    FOREIGN KEY (product_variant_id) REFERENCES product_variants (id) ON DELETE CASCADE
);

CREATE INDEX idx_sell_prices_is_deleted ON sell_prices (is_deleted);

CREATE INDEX idx_sell_prices_product_variant_id ON sell_prices (product_variant_id);

CREATE UNIQUE INDEX idx_sell_prices_unique_price ON sell_prices (
    product_variant_id,
    COALESCE(branch_id, 0)
)
WHERE
    is_deleted = 0;

CREATE TABLE sell_discounts (
    id INTEGER PRIMARY KEY,
    created_at TEXT DEFAULT(
        strftime ('%Y-%m-%dT%H:%M:%fZ', 'now')
    ),
    updated_at TEXT DEFAULT(
        strftime ('%Y-%m-%dT%H:%M:%fZ', 'now')
    ),
    deleted_at TEXT,
    is_deleted INTEGER NOT NULL DEFAULT 0,
    price_id INTEGER NOT NULL,
    quantity INTEGER,
    discount_formula TEXT NOT NULL,
    calculated_price INTEGER NOT NULL,
    customer_level INTEGER,
    metadata TEXT,
    FOREIGN KEY (price_id) REFERENCES sell_prices (id) ON DELETE CASCADE
);

CREATE INDEX idx_sell_discounts_price_id ON sell_discounts (price_id);