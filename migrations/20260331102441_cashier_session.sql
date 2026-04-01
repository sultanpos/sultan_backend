-- Add migration script here
CREATE TABLE cashier_sessions (
    id INTEGER PRIMARY KEY,
    created_at TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    deleted_at TEXT,
    is_deleted INTEGER NOT NULL DEFAULT 0,
    branch_id INTEGER NOT NULL REFERENCES branches(id),
    user_id INTEGER NOT NULL REFERENCES users(id),
    opened_at TEXT NOT NULL,
    closed_at TEXT,
    status TEXT NOT NULL DEFAULT 'open',     -- 'open' | 'closed'
    opening_cash BIGINT NOT NULL DEFAULT 0,  -- in smallest currency unit (e.g. cents/rupiah)
    closing_cash BIGINT,
    notes TEXT,
    metadata TEXT
);

CREATE INDEX idx_cashier_sessions_is_deleted ON cashier_sessions (is_deleted);
CREATE INDEX idx_cashier_sessions_branch_id ON cashier_sessions (branch_id);
CREATE INDEX idx_cashier_sessions_user_id ON cashier_sessions (user_id);
