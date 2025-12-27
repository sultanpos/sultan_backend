-- Add migration script here
CREATE TABLE categories (
	id INTEGER PRIMARY KEY,
    created_at TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
	deleted_at TEXT,
    is_deleted INTEGER NOT NULL DEFAULT 0,
    parent_id INTEGER,
	name TEXT NOT NULL,
    description TEXT,
    FOREIGN KEY (parent_id) REFERENCES categories(id) ON DELETE CASCADE
);

CREATE INDEX idx_categories_is_deleted ON categories (is_deleted);
CREATE INDEX idx_categories_parent_id ON categories (parent_id) WHERE is_deleted = 0;