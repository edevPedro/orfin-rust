# Guia do desenvolvedor

Como subir, testar e evoluir o Orfin sem quebrar o desenho KISS.

## Leitura obrigatória

1. [`ARCHITECTURE.md`](./ARCHITECTURE.md) — conceitos e algoritmos  
2. [`GLOSSARY.md`](./GLOSSARY.md) — valores canônicos de status/source/alerts  
3. App: `orfin-app/docs/ARCHITECTURE.md`

## Subir o backend

```bash
cp .env.example .env
# DATABASE_URL=postgres://...
# PLUGGY_CLIENT_ID / PLUGGY_CLIENT_SECRET
# opcional: FCM_SERVER_KEY, TELEGRAM_BOT_TOKEN, WEBHOOK_BASE_URL
cargo run
curl -s localhost:3000/health
```

Migrations: aplicadas em boot (`src/db.rs` inclui `001`…`003`).

## Subir o app

```bash
cd ../orfin-app   # ou clone do repo do app
cp .env.example .env
npm install
npm run web
```

Android (NLS + OCR): `npx expo prebuild && npx expo run:android` (não use Expo Go).

## Contratos que não pode quebrar

- Path `/payments/from-notification` (o Kotlin NLS usa esse path)
- Path `/payments/{id}/explain`
- Status string `awaiting_user` (não renomear sem migration + app + NLS)
- `UNIQUE (source, external_id)`

Se mudar nome de campo JSON, atualize `orfin-app/src/types.ts` na mesma mudança.

## Onde mexer para cada tipo de feature

| Quero… | Mexa em |
|---|---|
| Novo banco no listener | `BankNotificationListenerService.kt` (allowlist) |
| Melhor parse de notificação | Kotlin (amount/merchant regex) — **não** duplicar no JS |
| Melhor OCR / recibo | `orfin-app/src/ocr.ts` + `parseExpenseText.ts` + `POST /payments/from-ocr` |
| Melhor categoria | `categorize.rs` (heurísticas) ou regras aprendidas via explain |
| Novo alerta | `ALERT_*` em `models.rs` + branch em `reports::check_alerts` |
| Matching Open Finance | `reconcile.rs` (janela/score) |
| Nova tela | Expo Router em `orfin-app/app/` + `src/api.ts` |
| Auth de verdade | Middleware Axum; derive `user_id` do JWT |

## Testes manuais mínimos

1. `POST /payments/from-notification` com amount/merchant → aparece em `/payments/awaiting`  
2. `POST /payments/{id}/explain` → status `categorized`; segunda compra do merchant sugere a mesma categoria  
3. `GET /reports/summary?from&to` → totais  
4. `POST /reconcile/run` após ter um Pluggy + um Android com mesmo valor/hora → `matched`  
5. App web: fila → organizar → histórico  

Arquivo útil: `src/tests/requests/default.rest`.

## Princípios ao evoluir

1. **Uma fonte posta a notificação** — o serviço nativo; JS só configura  
2. **Pluggy = ledger; Android/OCR = realtime**  
3. **Regras antes de ML estatístico** — o explain já treina `user_category_rules`  
4. **OCR on-device primeiro** — cloud OCR só para PDF denso de extrato  
5. **Não invente um terceiro repo** para WhatsApp — worker separado ok, sem misturar neste binário
