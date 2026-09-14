-- user_category_rules already exists from 002; add unique key for upsert on explain
CREATE UNIQUE INDEX IF NOT EXISTS idx_user_category_rules_upsert
    ON user_category_rules (user_id, match_type, match_value);

CREATE TABLE IF NOT EXISTS alert_rules (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id      TEXT NOT NULL,
    kind         TEXT NOT NULL,
    threshold    NUMERIC(12, 2),
    category_id  TEXT REFERENCES categories (id),
    enabled      BOOLEAN NOT NULL DEFAULT true,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_alert_rules_user
    ON alert_rules (user_id);

ALTER TABLE payment_events
    ADD COLUMN IF NOT EXISTS reconcile_status TEXT NOT NULL DEFAULT 'unmatched',
    ADD COLUMN IF NOT EXISTS reconciled_with UUID REFERENCES payment_events (id),
    ADD COLUMN IF NOT EXISTS ocr_text TEXT;

CREATE INDEX IF NOT EXISTS idx_payment_events_user_paid_at_reports
    ON payment_events (user_id, paid_at);

CREATE INDEX IF NOT EXISTS idx_payment_events_user_status
    ON payment_events (user_id, status);
