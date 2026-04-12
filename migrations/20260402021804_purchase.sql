-- Add migration script here
CREATE TABLE purchase_orders (
    id              INTEGER PRIMARY KEY,
    created_at      TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at      TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    deleted_at      TEXT,
    is_deleted      INTEGER NOT NULL DEFAULT 0,
    branch_id       INTEGER NOT NULL REFERENCES branches(id),
    supplier_id     INTEGER REFERENCES suppliers(id),
    number          TEXT NOT NULL,          -- human-readable, from number_sequences (prefix 'PO')
    reference_number TEXT,                  -- human-readable, the reference number from supplier invoice or order confirmation
    status          TEXT NOT NULL DEFAULT 'draft',  -- 'draft' | 'ordered' | 'received' | 'cancelled'
    order_date      TEXT,                   -- ISO 8601, date PO was sent to supplier
    expected_date   TEXT,                   -- ISO 8601, expected delivery date
    received_date   TEXT,                   -- ISO 8601, set when status → 'received'
    subtotal        INTEGER NOT NULL DEFAULT 0,  -- sum of (unit_cost * quantity)
    discount_amount INTEGER NOT NULL DEFAULT 0,
    total_amount    INTEGER NOT NULL DEFAULT 0,  -- subtotal - discount_amount
    payment_status       TEXT NOT NULL DEFAULT 'unpaid',  -- 'unpaid' | 'partial' | 'paid'
    payment_due_date     TEXT,                             -- ISO 8601, agreed payment deadline
    paid_amount          INTEGER NOT NULL DEFAULT 0,       -- sum of purchase_payments.amount
    returned_amount      INTEGER NOT NULL DEFAULT 0,       -- sum of confirmed purchase_return totals
    notes                TEXT,
    metadata             TEXT
);

CREATE INDEX idx_purchase_orders_branch           ON purchase_orders (branch_id);
CREATE INDEX idx_purchase_orders_supplier         ON purchase_orders (supplier_id);
CREATE INDEX idx_purchase_orders_status           ON purchase_orders (status);
CREATE INDEX idx_purchase_orders_payment_status   ON purchase_orders (payment_status);
CREATE INDEX idx_purchase_orders_created          ON purchase_orders (created_at, id);
CREATE UNIQUE INDEX idx_purchase_orders_number ON purchase_orders (branch_id, number);

CREATE TABLE purchase_order_items (
    id                  INTEGER PRIMARY KEY,
    created_at          TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at          TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    purchase_order_id   INTEGER NOT NULL REFERENCES purchase_orders(id),
    product_variant_id  INTEGER NOT NULL REFERENCES product_variants(id),
    product_name        TEXT NOT NULL,   -- snapshot at order time
    variant_name        TEXT,            -- snapshot at order time
    barcode             TEXT,            -- snapshot at order time
    quantity            INTEGER NOT NULL,
    unit_cost           INTEGER NOT NULL,   -- cost per unit (becomes last_buy_price on receipt)
    discount_amount     INTEGER NOT NULL DEFAULT 0,
    total_cost          INTEGER NOT NULL,   -- (unit_cost * quantity) - discount_amount
    metadata            TEXT
);

CREATE INDEX idx_po_items_order   ON purchase_order_items (purchase_order_id);
CREATE INDEX idx_po_items_variant ON purchase_order_items (product_variant_id);

CREATE TABLE payment_channels (
    id                  INTEGER PRIMARY KEY,
    created_at          TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at          TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    deleted_at          TEXT,
    is_deleted          INTEGER NOT NULL DEFAULT 0,
    branch_id           INTEGER REFERENCES branches(id),
    name                TEXT NOT NULL,
    priority            INTEGER NOT NULL DEFAULT 100,  -- lower number = higher priority
    metadata            TEXT
);

CREATE INDEX idx_payment_channels_branch  ON payment_channels (branch_id);
CREATE INDEX idx_payment_channels_deleted ON payment_channels (is_deleted);

CREATE TABLE purchase_payments (
    id                  INTEGER PRIMARY KEY,
    created_at          TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    purchase_order_id   INTEGER NOT NULL REFERENCES purchase_orders(id),
    amount              INTEGER NOT NULL,
    payment_channel_id  INTEGER NOT NULL REFERENCES payment_channels(id),
    paid_at             TEXT NOT NULL,    -- actual payment date (ISO 8601)
    reference           TEXT,             -- transfer ref, cheque number
    notes               TEXT
);

CREATE INDEX idx_purchase_payments_po ON purchase_payments (purchase_order_id);