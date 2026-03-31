-- Add migration script here
CREATE TABLE machines (
    id INTEGER PRIMARY KEY,
    created_at TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    deleted_at TEXT,
    is_deleted INTEGER NOT NULL DEFAULT 0,
    branch_id INTEGER NOT NULL REFERENCES branches(id),
    key TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    metadata TEXT
);

CREATE INDEX idx_machines_is_deleted ON machines (is_deleted);

CREATE UNIQUE INDEX idx_machines_unique_branch_key ON machines (branch_id, key) WHERE is_deleted = 0;
