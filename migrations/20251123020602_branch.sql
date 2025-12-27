-- Add migration script here
CREATE TABLE branches (
    id INTEGER PRIMARY KEY,
    created_at TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    deleted_at TEXT,
    is_deleted BOOLEAN not null default 0,
    is_main BOOLEAN not null default 0,
    name TEXT NOT NULL,
    code TEXT NOT NULL,
    address TEXT,
    phone TEXT,
    npwp TEXT,
    image TEXT
);

CREATE INDEX idx_branches_is_deleted ON branches (is_deleted);