# Orfin — Arquitetura do sistema

Documento canônico para qualquer desenvolvedor continuar o projeto.  
Repos: backend Rust (`orfin-rust` / este repo) + app Expo (`orfin-app`, pasta/repo separado).

---

## 1. Problema que o produto resolve

Bancos e carteiras (Apple/Google Pay) **não** entregam histórico de compra para apps de terceiros em tempo real. O Orfin combina dois sinais:

| Sinal | Quando chega | Qualidade |
|---|---|---|
| **Notificação Android** (`NotificationListenerService`) | Segundos após a compra | Rápido, texto livre, pode faltar no iOS |
| **Open Finance via Pluggy** | Minutos (webhook) | Extrato confiável, merchant/categoria do banco |
| **OCR de comprovante** (ML Kit on-device) | Quando o usuário fotografa | Complementa gaps (boleto, recibo, print) |

O loop do produto:

1. Detectar gasto → status `awaiting_user`
2. Sugerir categoria (regras do usuário → heurísticas → `outros`)
3. Notificar (FCM / Discord / Telegram): “o que foi essa compra?”
4. Usuário organiza (`explain`) → status `categorized` + **aprende** regra do merchant
5. Relatórios, alertas e reconciliação Open Finance ↔ notificação/OCR

---

## 2. Repositórios e stack

```
orfin-rust (este repo)          orfin-app (Expo)
─────────────────────          ─────────────────
Axum + Tokio                   Expo Router (web/Android/iOS)
sqlx + Postgres                TypeScript
Pluggy Open Finance            modules/bank-notifications (NLS Android)
FCM legacy HTTP                expo-mlkit-ocr (dev build)
Discord webhook / Telegram     AsyncStorage (user_id, api url)
```

### Por que essa stack

| Decisão | Motivo |
|---|---|
| **Rust/Axum** | API fina, tipada, barata de operar; sqlx evita ORM pesado |
| **Expo/RN** | Um código web+mobile; XP existente; NLS vira módulo local |
| **Não KMP / não só Swift** | Cliente HTTP + listas — KMP é overkill; Swift não cobre Android/web |
| **ML Kit on-device** | OCR de recibo sem mandar imagem para cloud; privacidade + custo zero |
| **Sem OCR em notificação** | O texto da notificação já é string; regex/parsers bastam |
| **user_id no body/query** | Auth JWT é dívida técnica consciente (MVP) |

---

## 3. Conceitos fundamentais

### 3.1 `payment_events` (fato central)

Cada linha é **uma observação de gasto**, não necessariamente “a verdade única do extrato”.

- `source`: `pluggy` | `android_notification` | `receipt_ocr`
- `external_id`: id estável da origem (tx Pluggy, fingerprint da notificação, hash OCR)
- `UNIQUE (source, external_id)` → idempotência dura na mesma origem
- `status`: máquina de estados (abaixo)
- `suggested_category` vs `category`: sugestão automática vs confirmação humana
- `reconcile_status` / `reconciled_with`: vínculo entre sinais diferentes do mesmo gasto (ver migration 003)

### 3.2 Máquina de estados

```
pending ──(raro)──► awaiting_user ──explain──► categorized
                         │
                         ├── soft-dedupe / conflict ──► duplicate
                         └── erro de ingestão ──► failed
```

Fluxo feliz quase sempre: ingestão já grava `awaiting_user` (não fica em `pending`).

### 3.3 Duas camadas de “é o mesmo gasto?”

1. **Hard dedupe** — `ON CONFLICT (source, external_id) DO NOTHING`  
   Mesma notificação / mesma tx Pluggy não duplica linha.

2. **Soft dedupe** — na ingestão Android/OCR, procura outro `source` com **mesmo `amount`** e `paid_at` ± 5 min.  
   Se achar, marca o novo como `duplicate` (ainda grava a linha para auditoria).

3. **Reconcile** — job explícito (`POST /reconcile/run`) que **liga** Pluggy ↔ Android/OCR sobreviventes (mesmo valor, janela configurável, default 30 min), setando `reconcile_status=matched`.

Pluggy = ledger (fonte de verdade do extrato).  
Android/OCR = sinal em tempo real.

### 3.4 Categorização (algoritmo)

Ordem em `suggest_category` (`src/categorize.rs`):

1. Se a origem já trouxe categoria (ex.: Pluggy) → normaliza slug e usa
2. Regras do usuário (`user_category_rules`: `merchant_contains` / `description_contains`)
3. Heurísticas BR por keyword (ifood→alimentacao, uber→transporte, …)
4. Fallback: `outros`

**Aprendizado:** em `explain_payment`, chama `learn_merchant_rule` → upsert  
`(user_id, merchant_contains, merchant) → category_id`.  
Próxima compra do mesmo merchant já sugere a categoria escolhida.

Isso **não** é ML estatístico; é rule-learning determinístico (KISS, auditável). Evolução natural: embeddings / classificador só quando regras + heurísticas saturarem.

### 3.5 Text recognition — onde entra ML de verdade

| Entrada | Técnica | Onde |
|---|---|---|
| Push do banco | Regex `R$`, merchant `em/no/na …`, fingerprint por minuto | Kotlin NLS |
| Foto/print de recibo | **Google ML Kit Text Recognition** (on-device) | App (`src/ocr.ts`) |
| Texto colado (web) | Mesmo parser de texto, sem ML Kit | App (`src/parseExpenseText.ts`) |
| Extrato PDF multi-página | **Fora do escopo atual** — Document AI / Textract no futuro | Backend |

Pipeline OCR:

```
imagem → ML Kit → texto bruto → parseExpenseText (valor/merchant)
       → POST /payments/from-ocr → suggest_category → awaiting_user
```

### 3.6 Alertas

Tabela `alert_rules`. Kinds:

| kind | Semântica |
|---|---|
| `budget_monthly` | Soma `categorized` do mês (opcionalmente por categoria) ≥ threshold |
| `large_purchase` | Algum gasto (7 dias) ≥ threshold |
| `uncategorized_streak` | Contagem `awaiting_user` ≥ threshold |

`GET /alerts/check` avalia e devolve alertas disparados (não persiste histórico ainda).

### 3.7 Relatórios

`GET /reports/summary?user_id&from&to` agrega `awaiting_user` + `categorized` no intervalo:

- `total`, `by_category`, `by_source`, `awaiting_count`, `uncategorized_count`

---

## 4. Mapa do backend

```
src/
  main.rs          boot, AppState
  config.rs        env
  db.rs            pool + migrations embutidas (001..003)
  models.rs        DTOs + constantes de domínio
  routes.rs        HTTP surface
  payments.rs      ingestão Pluggy/Android/OCR, explain, listagens
  categorize.rs    sugestão + aprendizado de regras
  reports.rs       summary + alerts
  reconcile.rs     matching Pluggy ↔ Android/OCR
  pluggy.rs        client Open Finance
  notify.rs        orquestra push + canais após ingest/explain
  push.rs          FCM
  channels/        discord.rs, telegram.rs
migrations/
  001_payment_events.sql
  002_realtime_loop.sql      device_tokens, channel_links, categories, rules
  003_reports_alerts_reconcile.sql
```

Migrations rodam em `run_migrations` (SQL split por `;`). Não usamos sqlx-cli migrate no hot path.

---

## 5. API (contrato canônico)

Base URL default: `http://localhost:3000` (emulador Android: `http://10.0.2.2:3000`).

| Método | Rota | Função |
|---|---|---|
| GET | `/health` | Liveness |
| POST | `/connect/token` | Pluggy Connect Token `{ user_id }` |
| POST | `/connect/items` | Vincula `{ user_id, item_id }` |
| POST | `/webhooks/pluggy` | Webhook Pluggy |
| POST | `/webhooks/pluggy/register` | Registra webhooks |
| GET | `/payments?user_id&status?&limit?` | Lista |
| GET | `/payments/awaiting?user_id` | Fila `awaiting_user` |
| POST | `/payments/from-notification` | Ingestão Android |
| POST | `/payments/from-ocr` | Ingestão OCR |
| POST | `/payments/{id}/explain` | `{ category, note? }` |
| POST | `/devices/push-token` | `{ user_id, platform, fcm_token }` |
| POST | `/channels/link` | `{ user_id, channel, target, enabled? }` |
| GET | `/categories` | Catálogo |
| GET | `/reports/summary?user_id&from&to` | Relatório |
| GET | `/alerts?user_id` | Lista regras |
| POST | `/alerts` | Cria regra |
| GET | `/alerts/check?user_id` | Avalia regras |
| POST | `/reconcile/run` | `{ user_id, window_minutes? }` |

### Payloads de ingestão (Android / OCR)

```json
{
  "user_id": "user-123",
  "external_id": "com.nu.production|Compra R$ 45,90|27890451",
  "amount": "45.90",
  "currency": "BRL",
  "description": "Compra aprovada",
  "merchant": "Mercado Extra",
  "paid_at": "2026-09-13T12:00:00Z",
  "raw_payload": { "package": "com.nu.production", "title": "...", "text": "..." },
  "ocr_text": "opcional — só em from-ocr"
}
```

### Notificação enviada ao usuário (`payment_ask`)

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

---

## 6. Algoritmos (detalhe operacional)

### 6.1 Fingerprint de notificação (Android)

```
external_id = "{package}|{title}|{text}|{floor(postTime / 60000)}"
```

Mesma notificação reentregue no mesmo minuto → mesmo `external_id` → hard dedupe.

### 6.2 Parse de valor BRL (NLS + JS)

Regex principal: `R$\s*(\d{1,3}(?:\.\d{3})*,\d{2}|\d+,\d{2}|\d+(?:\.\d{2})?)`  
Normalização: se tem vírgula, remove `.` de milhar e troca `,` → `.`.

### 6.3 Merchant na notificação

1. Captura após `em|no|na`  
2. Senão, usa o `title` da notificação

### 6.4 Soft dedupe (±5 min)

Na ingestão Android/OCR, busca outro source com mesmo `user_id` + `amount` e `paid_at` na janela.  
Prioriza Pluggy na ordenação do match.

### 6.5 Reconcile (greedy)

Para cada Pluggy `unmatched` (ordenado por `paid_at`):

- Achou Android/OCR unmatched com mesmo `amount` e `paid_at ∈ [t−w, t+w]`?
- Se sim: ambos `matched`, `reconciled_with` aponta um para o outro
- Senão: segue

Não faz matching global ótimo (Hungarian); é greedy KISS. Melhorar só se houver falsos positivos reais.

### 6.6 Pluggy ingest

Webhook `transactions/created` → fetch link → só débitos (`amount < 0`) → abs → `awaiting_user`.

---

## 7. Setup backend

```bash
cp .env.example .env
# DATABASE_URL, PLUGGY_CLIENT_ID/SECRET, FCM_SERVER_KEY?, TELEGRAM_BOT_TOKEN?, HOST/PORT
createdb orfin   # ou Docker Postgres
cargo run
```

Health: `curl localhost:3000/health`

---

## 8. Como evoluir com segurança

1. **Novas fontes** → novo `SOURCE_*` + rota de ingestão + dedupe; não misture parsers
2. **Melhor categoria** → primeiro enriquecer `user_category_rules` e heurísticas; ML só com dataset de `explain`
3. **Auth** → middleware JWT; manter `user_id` interno derivado do token
4. **Extrato PDF** → worker separado (Document AI) + `source=statement_ocr`, reusar reconcile
5. **WhatsApp** → outro worker; não inchá este binário
6. **Evitar** OCR no caminho de notificação; evitar duplicar POST no JS quando o NLS já posta

### Dívidas técnicas conhecidas

- Sem JWT
- FCM via HTTP legacy (`FCM_SERVER_KEY`)
- App pode ter drift de nomes de campos no TypeScript vs serde — **backend é a fonte da verdade**; alinhar `src/types.ts` do app ao JSON real
- Histórico de alertas disparados ainda não é persistido

---

## 9. App (resumo)

Ver `apps/orfin-app/docs/ARCHITECTURE.md` para telas, módulo NLS, OCR e design system.

Contrato mínimo que o app consome: awaiting, payments, explain, from-ocr, categories, channels/link, reports/summary, alerts, reconcile/run.
