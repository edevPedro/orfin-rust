# Orfin Backend

Detecção de pagamentos em tempo real (Pluggy + Android), com push FCM e canais Discord/Telegram para perguntar o que foi a compra e organizar por categoria.

## Setup

```bash
cp .env.example .env
cargo run
```

## Fluxo do app

1. App registra FCM token: `POST /devices/push-token`
2. Usuário liga Discord/Telegram: `POST /channels/link`
3. Usuário conecta banco (Pluggy) ou o app envia notificação Android
4. Backend grava o pagamento como `awaiting_user`, sugere categoria e notifica (push + canais)
5. App/usuário responde: `POST /payments/{id}/explain` com `{ "category", "note?" }`
6. Status vira `categorized` e os canais recebem confirmação

## API

| Método | Rota | Descrição |
|---|---|---|
| `GET` | `/health` | Health check |
| `POST` | `/connect/token` | Connect Token Pluggy |
| `POST` | `/connect/items` | Vincula `item_id` ao usuário |
| `POST` | `/webhooks/pluggy` | Webhook Pluggy |
| `POST` | `/webhooks/pluggy/register` | Registra webhooks Pluggy |
| `GET` | `/payments` | Lista pagamentos |
| `GET` | `/payments/awaiting` | Pagamentos esperando resposta |
| `POST` | `/payments/from-notification` | Ingestão Android |
| `POST` | `/payments/{id}/explain` | Categoria + nota do usuário |
| `POST` | `/devices/push-token` | Registra token FCM |
| `POST` | `/channels/link` | Liga Discord webhook ou Telegram chat_id |
| `GET` | `/categories` | Catálogo de categorias |

### Exemplos

```json
// POST /devices/push-token
{ "user_id": "user-123", "platform": "android", "fcm_token": "..." }

// POST /channels/link
{ "user_id": "user-123", "channel": "telegram", "target": "123456789" }
{ "user_id": "user-123", "channel": "discord", "target": "https://discord.com/api/webhooks/..." }

// POST /payments/{id}/explain
{ "category": "alimentacao", "note": "almoço com o time" }
```

Payload enviado no push / Discord / Telegram:

```json
{
  "type": "payment_ask",
  "payment_event_id": "uuid",
  "amount": "45.90",
  "currency": "BRL",
  "merchant": "Mercado Extra",
  "suggested_category": "alimentacao",
  "paid_at": "..."
}
```

## Status

`pending` → `awaiting_user` → `categorized` | `duplicate` | `failed`

## Categorias seed

`alimentacao`, `transporte`, `moradia`, `lazer`, `saude`, `educacao`, `assinaturas`, `outros`

## Auth

Por enquanto `user_id` vai no body/query (sem JWT). Dívida técnica documentada.

## Fora deste repo

- App mobile / NotificationListener
- WhatsApp (outro worker)
