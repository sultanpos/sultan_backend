-- Add migration script here

CREATE TABLE units (
    id INTEGER PRIMARY KEY,
    created_at TEXT DEFAULT(
        strftime ('%Y-%m-%dT%H:%M:%fZ', 'now')
    ),
    updated_at TEXT DEFAULT(
        strftime ('%Y-%m-%dT%H:%M:%fZ', 'now')
    ),
    deleted_at TEXT,
    is_deleted INTEGER NOT NULL DEFAULT 0,
    name TEXT NOT NULL,
    description TEXT
);

-- product_type : 'product', 'service', 'bundle'
CREATE TABLE products (
    id INTEGER PRIMARY KEY,
    created_at TEXT DEFAULT(
        strftime ('%Y-%m-%dT%H:%M:%fZ', 'now')
    ),
    updated_at TEXT DEFAULT(
        strftime ('%Y-%m-%dT%H:%M:%fZ', 'now')
    ),
    deleted_at TEXT,
    is_deleted INTEGER NOT NULL DEFAULT 0,
    name TEXT NOT NULL,
    description TEXT,
    product_type TEXT NOT NULL,
    main_image TEXT,
    unit_id INTEGER,
    is_serial INTEGER NOT NULL DEFAULT 0,
    calculate_stock INTEGER NOT NULL DEFAULT 0,
    sellable INTEGER NOT NULL DEFAULT 1,
    buyable INTEGER NOT NULL DEFAULT 1,
    editable_price INTEGER NOT NULL DEFAULT 0,
    has_variant INTEGER NOT NULL DEFAULT 0,
    metadata TEXT
);

CREATE INDEX idx_products_is_deleted ON products (is_deleted);

CREATE TABLE product_variants (
    id INTEGER PRIMARY KEY,
    created_at TEXT DEFAULT(
        strftime ('%Y-%m-%dT%H:%M:%fZ', 'now')
    ),
    updated_at TEXT DEFAULT(
        strftime ('%Y-%m-%dT%H:%M:%fZ', 'now')
    ),
    deleted_at TEXT,
    is_deleted INTEGER NOT NULL DEFAULT 0,
    product_id INTEGER NOT NULL,
    barcode TEXT,
    name TEXT,
    metadata TEXT,
    FOREIGN KEY (product_id) REFERENCES products (id)
);

CREATE INDEX idx_product_variants_barcode ON product_variants (barcode)
WHERE
    barcode IS NOT NULL;

CREATE INDEX idx_product_variants_is_deleted ON product_variants (is_deleted);

CREATE TABLE product_categories (
    product_id INTEGER NOT NULL,
    category_id INTEGER NOT NULL,
    PRIMARY KEY (product_id, category_id),
    FOREIGN KEY (product_id) REFERENCES products (id),
    FOREIGN KEY (category_id) REFERENCES categories (id)
);