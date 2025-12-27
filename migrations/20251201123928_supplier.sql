-- Add migration script here
CREATE TABLE suppliers (
	id INTEGER PRIMARY KEY,
    created_at TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
	deleted_at TEXT,
    is_deleted INTEGER NOT NULL DEFAULT 0,
	name TEXT NOT NULL,
    code TEXT,
    address TEXT,
    email TEXT,
    phone TEXT,
    npwp TEXT,
    npwp_name TEXT,
    metadata TEXT
);
CREATE INDEX idx_suppliers_is_deleted ON suppliers (is_deleted);