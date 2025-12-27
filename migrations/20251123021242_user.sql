-- Add migration script here
PRAGMA foreign_keys = ON;

CREATE TABLE users (
	id INTEGER PRIMARY KEY,
	created_at TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
	deleted_at TEXT,
    is_deleted INTEGER NOT NULL DEFAULT 0,
    username TEXT NOT NULL UNIQUE,
    password TEXT NOT NULL,
	name TEXT NOT NULL,
    email TEXT,
	photo TEXT,
    pin TEXT,
    address TEXT,
    phone TEXT
);

CREATE INDEX idx_users_is_deleted ON users (is_deleted);