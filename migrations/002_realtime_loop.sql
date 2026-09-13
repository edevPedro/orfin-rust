ALTER TABLE payment_events
    ADD COLUMN IF NOT EXISTS suggested_category TEXT,
    ADD COLUMN IF NOT EXISTS user_note TEXT;

CREATE INDEX IF NOT EXISTS idx_payment_events_awaiting
    ON payment_events (user_id, paid_at DESC)
    WHERE status = 'awaiting_user';

CREATE TABLE IF NOT EXISTS device_tokens (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id    TEXT NOT NULL,
    platform   TEXT NOT NULL,
    fcm_token  TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (fcm_token)
);

CREATE INDEX IF NOT EXISTS idx_device_tokens_user
    ON device_tokens (user_id);

CREATE TABLE IF NOT EXISTS channel_links (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id    TEXT NOT NULL,
    channel    TEXT NOT NULL,
    target     TEXT NOT NULL,
    enabled    BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (user_id, channel, target)
);

CREATE TABLE IF NOT EXISTS categories (
    id    TEXT PRIMARY KEY,
    label TEXT NOT NULL
);

INSERT INTO categories (id, label) VALUES
    ('alimentacao', 'Alimentação'),
    ('transporte', 'Transporte'),
    ('moradia', 'Moradia'),
    ('lazer', 'Lazer'),
    ('saude', 'Saúde'),
    ('educacao', 'Educação'),
    ('assinaturas', 'Assinaturas'),
    ('outros', 'Outros')
ON CONFLICT (id) DO NOTHING;

CREATE TABLE IF NOT EXISTS user_category_rules (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id      TEXT NOT NULL,
    match_type   TEXT NOT NULL,
    match_value  TEXT NOT NULL,
    category_id  TEXT NOT NULL REFERENCES categories (id),
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_user_category_rules_user
    ON user_category_rules (user_id);
