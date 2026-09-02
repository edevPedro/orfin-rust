# Orfin Backend

Detecção de pagamentos via Pluggy (Open Finance) e notificações Android, com Postgres compartilhado para o worker WhatsApp.

## Setup

```bash
cp .env.example .env
cargo run
```

## API

| Método | Rota | Descrição |
|---|---|---|
| `GET` | `/health` | Health check |
| `POST` | `/connect/token` | Connect Token Pluggy |
| `POST` | `/connect/items` | Vincula `item_id` ao usuário |
| `POST` | `/webhooks/pluggy` | Webhook Pluggy |
| `POST` | `/webhooks/pluggy/register` | Registra webhooks |
| `GET` | `/payments` | Lista pagamentos |
| `POST` | `/payments/from-notification` | Pagamento do Android |

## Contrato WhatsApp

Consumir eventos pendentes:

```sql
SELECT id, user_id, amount, currency, description, merchant, category, paid_at
FROM payment_events
WHERE status = 'pending'
ORDER BY paid_at ASC
FOR UPDATE SKIP LOCKED
LIMIT 10;
```

Após enviar explicação:

```sql
UPDATE payment_events SET status = 'explained', explained_at = now() WHERE id = $1;
INSERT INTO payment_explanations (payment_event_id, message_text) VALUES ($1, $2);
```

Status: `pending`, `explained`, `duplicate`, `failed`.
