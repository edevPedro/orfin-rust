# Orfin Backend

API Rust (Axum + Postgres) para detecção de pagamentos em tempo real: **Pluggy Open Finance** + **notificações Android** + **OCR de comprovantes**, com push FCM, Discord/Telegram, relatórios, alertas e reconciliação.

## Documentação completa

| Doc | Conteúdo |
|---|---|
| [`docs/ARCHITECTURE.md`](./docs/ARCHITECTURE.md) | Stack, conceitos, estados, algoritmos (dedupe, categorize, reconcile, OCR) |
| [`docs/GLOSSARY.md`](./docs/GLOSSARY.md) | Valores canônicos (status, source, alerts, campos JSON) |
| [`docs/DEVELOPING.md`](./docs/DEVELOPING.md) | Como subir, testar e onde mexer para cada feature |

Leia os três antes de alterar contratos HTTP ou o listener Android.

## Setup rápido

```bash
cp .env.example .env
# configure DATABASE_URL, PLUGGY_*, opcionalmente FCM / Telegram
cargo run
```

`GET /health` → `{ "status": "ok" }`.

## Fluxo resumido

1. App registra push / liga Discord ou Telegram  
2. Usuário conecta banco (Pluggy) **ou** o Android envia notificação **ou** fotografa comprovante (OCR)  
3. Backend grava `awaiting_user`, sugere categoria, notifica  
4. Usuário responde `POST /payments/{id}/explain` → `categorized` + aprende regra do merchant  
5. Relatórios (`/reports/summary`), alertas (`/alerts/check`), reconcile (`/reconcile/run`)

## API (visão rápida)

| Método | Rota |
|---|---|
| GET | `/health` |
| POST | `/connect/token`, `/connect/items` |
| POST | `/webhooks/pluggy`, `/webhooks/pluggy/register` |
| GET | `/payments`, `/payments/awaiting` |
| POST | `/payments/from-notification`, `/payments/from-ocr` |
| POST | `/payments/{id}/explain` |
| POST | `/devices/push-token`, `/channels/link` |
| GET | `/categories` |
| GET | `/reports/summary` |
| GET/POST | `/alerts`, GET `/alerts/check` |
| POST | `/reconcile/run` |

Status: `pending` → `awaiting_user` → `categorized` | `duplicate` | `failed`.

Categorias seed: `alimentacao`, `transporte`, `moradia`, `lazer`, `saude`, `educacao`, `assinaturas`, `outros`.

## Auth

`user_id` no body/query (sem JWT). Dívida técnica documentada na arquitetura.

## Cliente

App web/mobile: **`apps/orfin-app/`** (Expo + NotificationListener + ML Kit).  
Ver [`apps/orfin-app/docs/ARCHITECTURE.md`](./apps/orfin-app/docs/ARCHITECTURE.md).
