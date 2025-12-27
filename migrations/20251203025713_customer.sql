-- Add migration script here
CREATE TABLE customers (
	id INTEGER PRIMARY KEY,
    created_at TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
	deleted_at TEXT,
    is_deleted INTEGER NOT NULL DEFAULT 0,
    number TEXT NOT NULL,
	name TEXT NOT NULL,
    address TEXT,
    email TEXT,
    phone TEXT,
    level INTEGER NOT NULL DEFAULT 0,
    metadata TEXT
);

CREATE INDEX idx_customers_is_deleted ON customers (is_deleted);
CREATE UNIQUE INDEX idx_customers_number_unique ON customers (number) WHERE is_deleted = 0;