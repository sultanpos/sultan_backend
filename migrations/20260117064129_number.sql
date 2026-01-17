-- Add migration script here
-- Number sequence tracking table
CREATE TABLE number_sequences (
    id INTEGER PRIMARY KEY,
    created_at TEXT DEFAULT(
        strftime ('%Y-%m-%dT%H:%M:%fZ', 'now')
    ),
    updated_at TEXT DEFAULT(
        strftime ('%Y-%m-%dT%H:%M:%fZ', 'now')
    ),
    prefix TEXT NOT NULL,
    branch_id INTEGER REFERENCES branches (id) ON DELETE CASCADE,
    year INTEGER NOT NULL,
    month INTEGER,
    last_number INTEGER NOT NULL DEFAULT 0,
    UNIQUE (
        prefix,
        branch_id,
        year,
        month
    )
);

CREATE INDEX idx_number_sequences_lookup ON number_sequences (
    prefix,
    branch_id,
    year,
    month
);