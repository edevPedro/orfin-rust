CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TABLE IF NOT EXISTS payment_events (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id       TEXT NOT NULL,
    source        TEXT NOT NULL,
    external_id   TEXT,
    amount        NUMERIC(12, 2) NOT NULL,
    currency      TEXT NOT NULL DEFAULT 'BRL',
    description   TEXT,
    merchant      TEXT,
    category      TEXT,
    paid_at       TIMESTAMPTZ NOT NULL,
    raw_payload   JSONB,
    status        TEXT NOT NULL DEFAULT 'pending',
    explained_at  TIMESTAMPTZ,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (source, external_id)
);

CREATE INDEX IF NOT EXISTS idx_payment_events_status
    ON payment_events (status)
    WHERE status = 'pending';

CREATE INDEX IF NOT EXISTS idx_payment_events_user_paid_at
    ON payment_events (user_id, paid_at DESC);

CREATE TABLE IF NOT EXISTS payment_explanations (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    payment_event_id UUID NOT NULL REFERENCES payment_events (id),
    message_text     TEXT NOT NULL,
    sent_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS pluggy_item_users (
    item_id  TEXT PRIMARY KEY,
    user_id  TEXT NOT NULL,
    linked_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
