# Orfin Backend

Detecção de pagamentos em tempo real (Pluggy + Android + OCR), com push FCM e canais Discord/Telegram para perguntar o que foi a compra e organizar por categoria. Relatórios mensais, alertas de gasto e reconciliação entre fontes.

## Setup

```bash
cp .env.example .env
cargo run
```

## Fluxo do app

1. App registra FCM token: `POST /devices/push-token`
2. Usuário liga Discord/Telegram: `POST /channels/link`
3. Usuário conecta banco (Pluggy) ou o app envia notificação Android / OCR de recibo
4. Backend grava o pagamento como `awaiting_user`, sugere categoria e notifica (push + canais)
5. App/usuário responde: `POST /payments/{id}/explain` com `{ "category", "note?" }`
6. Status vira `categorized`, regra de merchant é aprendida, e os canais recebem confirmação

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
| `POST` | `/payments/from-ocr` | Ingestão OCR de recibo |
| `POST` | `/payments/{id}/explain` | Categoria + nota do usuário (aprende regra) |
| `POST` | `/devices/push-token` | Registra token FCM |
| `POST` | `/channels/link` | Liga Discord webhook ou Telegram chat_id |
| `GET` | `/categories` | Catálogo de categorias |
| `GET` | `/reports/summary` | Resumo por período (`user_id`, `from`, `to`) |
| `GET` | `/alerts` | Lista regras de alerta |
| `POST` | `/alerts` | Cria regra (`budget_monthly`, `large_purchase`, `uncategorized_streak`) |
| `GET` | `/alerts/check` | Avalia alertas do mês / últimos 7 dias |
| `POST` | `/reconcile/run` | Casa `android_notification`/`receipt_ocr` com `pluggy` |

### Exemplos

```json
// POST /devices/push-token
{ "user_id": "user-123", "platform": "android", "fcm_token": "..." }

// POST /channels/link
{ "user_id": "user-123", "channel": "telegram", "target": "123456789" }
{ "user_id": "user-123", "channel": "discord", "target": "https://discord.com/api/webhooks/..." }

// POST /payments/{id}/explain
{ "category": "alimentacao", "note": "almoço com o time" }

// POST /payments/from-ocr
{ "user_id": "user-123", "external_id": "ocr-1", "amount": "45.90", "paid_at": "2026-09-13T12:00:00Z", "ocr_text": "MERCADO EXTRA..." }

// GET /reports/summary?user_id=user-123&from=2026-09-01T00:00:00Z&to=2026-09-30T23:59:59Z

// POST /alerts
{ "user_id": "user-123", "kind": "budget_monthly", "threshold": "2000.00" }

// POST /reconcile/run
{ "user_id": "user-123", "window_minutes": 30 }
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

Reconciliação: `reconcile_status` = `unmatched` | `matched` (pares via `reconciled_with`).

## Categorias seed

`alimentacao`, `transporte`, `moradia`, `lazer`, `saude`, `educacao`, `assinaturas`, `outros`

## Auth

Por enquanto `user_id` vai no body/query (sem JWT). Dívida técnica documentada.

## Fora deste repo

- App web/mobile: pasta/repo separado `orfin-app` (Expo + NotificationListener Android)
- WhatsApp (outro worker)
