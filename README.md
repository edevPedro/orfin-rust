# Orfin Backend

Backend Rust para detecção de pagamentos via Open Finance (Pluggy) e notificações Android, com persistência compartilhada para o projeto WhatsApp explicar compras.

## Requisitos

- Rust 1.85+ (recomendado: stable)
- PostgreSQL 14+

## Configuração

```bash
cp .env.example .env
# Edite PLUGGY_ID, PLUGGY_SECRET e DATABASE_URL
```

Variáveis:

| Variável | Descrição |
|---|---|
| `PLUGGY_ID` | Client ID da Pluggy |
| `PLUGGY_SECRET` | Client Secret da Pluggy |
| `DATABASE_URL` | Connection string Postgres |
| `HOST` | Host do servidor (padrão: `0.0.0.0`) |
| `PORT` | Porta (padrão: `3000`) |
| `WEBHOOK_BASE_URL` | URL pública HTTPS para webhooks Pluggy |

## Executar

```bash
cargo run
```

As migrations rodam automaticamente na inicialização.

## API

| Método | Rota | Descrição |
|---|---|---|
| `GET` | `/health` | Health check |
| `POST` | `/connect/token` | Gera Connect Token para Pluggy Connect widget |
| `POST` | `/connect/items` | Associa `item_id` Pluggy ao `user_id` |
| `POST` | `/webhooks/pluggy` | Recebe webhooks Pluggy |
| `POST` | `/webhooks/pluggy/register` | Registra webhooks na Pluggy |
| `GET` | `/payments` | Lista pagamentos (`?user_id=&status=&limit=`) |
| `POST` | `/payments/from-notification` | Recebe pagamento parseado do app Android |

### Fluxo Pluggy

1. App chama `POST /connect/token` com `{ "user_id": "..." }`
2. Abre Pluggy Connect com o `access_token` retornado
3. Após sucesso, chama `POST /connect/items` com `{ "user_id", "item_id" }`
4. Pluggy envia `transactions/created` para `/webhooks/pluggy`
5. Backend busca transações e grava em `payment_events`

### Fluxo Android

1. `NotificationListenerService` detecta notificação bancária
2. App envia `POST /payments/from-notification`
3. Backend deduplica contra eventos Pluggy (±5 min, mesmo valor)

## Contrato com projeto WhatsApp (DB compartilhado)

### Tabela `payment_events`

O worker WhatsApp deve consumir eventos com `status = 'pending'`:

```sql
SELECT id, user_id, amount, currency, description, merchant, category, paid_at, source
FROM payment_events
WHERE status = 'pending'
ORDER BY paid_at ASC
FOR UPDATE SKIP LOCKED
LIMIT 10;
```

Após enviar a explicação:

```sql
UPDATE payment_events
SET status = 'explained', explained_at = now()
WHERE id = $1;

INSERT INTO payment_explanations (payment_event_id, message_text)
VALUES ($1, $2);
```

### Status possíveis

| Status | Significado |
|---|---|
| `pending` | Aguardando explicação WhatsApp |
| `processing` | Worker WhatsApp processando |
| `explained` | Explicação enviada |
| `failed` | Falha no processamento |
| `duplicate` | Duplicata (ex: Android + Pluggy) |

### Tabela `pluggy_item_users`

Mapeia `item_id` Pluggy → `user_id` do Orfin.

## Deduplicação

Quando Android e Pluggy reportam o mesmo pagamento (`user_id` + `amount` + `paid_at` ±5 min):

- Dados Pluggy são preferidos (merchant, category)
- Evento Android recebe `status = 'duplicate'`

## Desenvolvimento local com Postgres

```bash
docker run -d --name orfin-pg \
  -e POSTGRES_USER=orfin \
  -e POSTGRES_PASSWORD=orfin \
  -e POSTGRES_DB=orfin \
  -p 5432:5432 \
  postgres:16
```
