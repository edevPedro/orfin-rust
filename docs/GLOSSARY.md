# Glossário Orfin

Termos e valores canônicos. Se o código e este glossário divergirem, **corrija o código ou atualize este arquivo na mesma PR**.

## Status de `payment_events`

| Valor | Significado |
|---|---|
| `pending` | Raro; reservado |
| `awaiting_user` | Detectado; esperando o usuário organizar |
| `categorized` | Usuário confirmou categoria (`explain`) |
| `duplicate` | Soft-dedupe: mesmo gasto já visto por outra fonte |
| `failed` | Falha de ingestão |

## Sources

| Valor | Origem |
|---|---|
| `pluggy` | Open Finance (extrato) |
| `android_notification` | NotificationListenerService |
| `receipt_ocr` | Foto/print + ML Kit / texto colado |

## Reconcile

| Valor | Significado |
|---|---|
| `unmatched` | Ainda sem par |
| `matched` | Ligado a outro `payment_events` via `reconciled_with` |

## Alert kinds

| Valor | Dispara quando |
|---|---|
| `budget_monthly` | Soma `categorized` do mês ≥ threshold |
| `large_purchase` | Max gasto (7 dias) ≥ threshold |
| `uncategorized_streak` | Contagem `awaiting_user` ≥ threshold |

## Canais

| Valor | `target` |
|---|---|
| `discord` | URL do webhook |
| `telegram` | `chat_id` |

## Categorias seed

`alimentacao`, `transporte`, `moradia`, `lazer`, `saude`, `educacao`, `assinaturas`, `outros`

## Regras de categoria

`user_category_rules.match_type`:

- `merchant_contains`
- `description_contains`

Aprendidas automaticamente no `explain` (merchant → categoria).

## Campos JSON importantes

| Campo | Uso |
|---|---|
| `external_id` | Idempotência por `(source, external_id)` |
| `suggested_category` | Sugestão automática |
| `category` | Confirmação humana |
| `user_note` | Nota do `explain` |
| `paid_at` | Quando o gasto ocorreu |
| `ocr_text` | Texto bruto do OCR (fonte `receipt_ocr`) |
| `reconcile_status` | `unmatched` \| `matched` |
| `reconciled_with` | UUID do pagamento pareado |

## Payload `payment_ask` (push/canais)

```json
{
  "type": "payment_ask",
  "payment_event_id": "uuid",
  "amount": "45.90",
  "currency": "BRL",
  "merchant": "...",
  "suggested_category": "alimentacao",
  "paid_at": "..."
}
```
