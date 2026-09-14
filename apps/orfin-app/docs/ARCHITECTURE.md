# Orfin App — Arquitetura do cliente

Cliente **Expo (React Native)** web + Android + iOS.  
Backend canônico: repo `orfin-rust` → leia também `docs/ARCHITECTURE.md` de lá (algoritmos, estados, API).

Este documento explica **como o app é organizado**, o módulo nativo de notificações, OCR e como evoluir a UI sem quebrar o contrato.

---

## 1. Por que Expo / React Native

| Opção | Decisão |
|---|---|
| **Expo / RN** | Escolhido — um código para web, Android e iOS; NotificationListener vira módulo local |
| Flutter | Viável, sem ganho claro aqui |
| KMP | Overkill para cliente HTTP + listas |
| Só Swift / só Kotlin | Dois apps = duplicidade; iOS não tem NotificationListener equivalente |

**Limitação importante:** o listener de notificações bancárias **só existe no Android**. iOS/web usam Pluggy (Open Finance) + OCR manual de comprovantes.

---

## 2. Mapa do repositório

```
app/
  _layout.tsx          fonts + Stack
  (tabs)/
    index.tsx          Aguardando (fila awaiting_user)
    payments.tsx       Histórico
    reports.tsx        Relatório do mês
    settings.tsx       user_id, API, NLS, canais, alertas, reconcile
    _layout.tsx        tabs
  payment/[id].tsx     Organizar compra (explain)
  capture.tsx          Foto/OCR ou colar texto
modules/bank-notifications/
  android/.../BankNotificationListenerService.kt   NLS + POST nativo
  android/.../BankNotificationsModule.kt           configure / isEnabled / openSettings
  src/*.ts                                         JS bridge + stub web/iOS
src/
  api.ts               cliente HTTP
  types.ts             tipos alinhados ao JSON do backend
  theme.ts             design tokens
  ocr.ts               wrapper ML Kit / fallback web
  parseExpenseText.ts  parser BRL (amount + merchant)
  bankListener.ts      fachada do módulo nativo
  storage.ts           AsyncStorage (user_id, apiUrl)
  config.ts            EXPO_PUBLIC_API_URL + default emulador
scripts/mock-api.js    API fake para smoke web sem Postgres
```

---

## 3. Conceitos que o app precisa respeitar

### 3.1 O NLS posta **sozinho**

No Android, `BankNotificationListenerService`:

1. Filtra pacotes de bancos
2. Extrai `R$` + merchant do texto da notificação
3. Monta `external_id` = `package|title|text|minuto`
4. Faz `POST /payments/from-notification` **em background Kotlin** (app morto continua funcionando)

O JavaScript **só** grava `apiUrl` + `user_id` em `SharedPreferences` via `configure()`.  
**Não** reenvie a mesma notificação pelo JS — isso gera soft-duplicates desnecessários.

### 3.2 OCR ≠ NotificationListener

| Fluxo | Entrada | ML? | Rota |
|---|---|---|---|
| Banco notificou | texto da notification | Não (já é texto) | `/payments/from-notification` (nativo) |
| Usuário fotografa recibo | imagem | **ML Kit** on-device | `/payments/from-ocr` |
| Web / fallback | texto colado | Não | `/payments/from-ocr` após `parseExpenseText` |

`src/ocr.ts` tenta `expo-mlkit-ocr` em builds nativos; na web retorna `supported: false` e a tela Capturar oferece colar texto.

### 3.3 Organizar compra = `explain`

`POST /payments/{id}/explain` com `{ category, note? }`.  
O backend aprende regra `merchant_contains` — a próxima sugestão melhora sozinha.

### 3.4 Relatório / alertas / reconcile

- Relatórios → `GET /reports/summary`
- Alertas → `POST /alerts`, `GET /alerts/check`
- Comparar Open Finance × notificações → `POST /reconcile/run`

Detalhe dos algoritmos: doc do backend.

---

## 4. Design system (`src/theme.ts`)

Direção visual: fintech BR limpa — **teal profundo + âmbar CTA**, tipografia **Fraunces** (marca/valores) + **DM Sans** (UI).

Evitar: roxo genérico, cream+terracotta clichê, dark-mode forçado, glow, cards demais no hero.

Tokens principais:

- `brand` `#0F4C5C`, `cta` `#D97706`, `ink` `#0B1220`, `surface` `#F3F6FA`
- Status labels em PT (`awaiting_user` → “Aguardando”)
- `formatBRL` / `formatDateTime` centralizados

Uma composição por tela; valor em destaque na fila e no explain.

---

## 5. Telas e navegação

| Rota | Papel |
|---|---|
| `(tabs)/index` | Fila do que precisa organização |
| `(tabs)/payments` | Histórico com status |
| `(tabs)/reports` | Total do mês + por categoria/fonte |
| `(tabs)/settings` | Identidade, NLS, canais, alertas, reconcile |
| `payment/[id]` | Categoria + nota |
| `capture` | OCR / texto → cria evento |

---

## 6. Setup

```bash
cp .env.example .env    # EXPO_PUBLIC_API_URL=http://localhost:3000
npm install
npm run web             # UI web
```

Smoke sem backend:

```bash
node scripts/mock-api.js
EXPO_PUBLIC_API_URL=http://localhost:3000 npm run web
```

### Android (NLS + OCR)

Não roda no Expo Go — precisa **dev build**:

```bash
npx expo prebuild
npx expo run:android
```

Depois: Ajustes → ativar acesso a notificações → salvar `user_id` + API URL  
(`http://10.0.2.2:3000` no emulador).

---

## 7. Contrato HTTP (app → backend)

Fonte da verdade: JSON do Rust. Se `types.ts` divergir, corrija o app.

| App | Backend |
|---|---|
| lista aguardando | `GET /payments/awaiting` |
| histórico | `GET /payments` |
| explicar | `POST /payments/{id}/explain` |
| OCR | `POST /payments/from-ocr` |
| notificação (nativo) | `POST /payments/from-notification` |
| categorias | `GET /categories` |
| canais | `POST /channels/link` |
| relatório | `GET /reports/summary` |
| alertas | `GET/POST /alerts`, `GET /alerts/check` |
| reconcile | `POST /reconcile/run` |

Campos típicos de `PaymentEvent` (espelham o JSON do Rust):  
`id`, `user_id`, `source`, `external_id`, `amount`, `currency`, `description`, `merchant`, `category`, `suggested_category`, `user_note`, `paid_at`, `status`, `explained_at`, `created_at`.

---

## 8. Algoritmos no cliente

### 8.1 `parseExpenseText`

1. Acha valor com regex BRL (`R$ 1.234,56` etc.) e normaliza para `1234.56`
2. Escolhe merchant = primeira linha “parecida com nome” (ignora PIX/cartão/banco/datas)
3. Hints: pix / cartão

Usado após OCR e no fallback web.

### 8.2 Listener nativo (Kotlin)

Ver backend doc §6 + código em `BankNotificationListenerService.kt`.  
Filtro de pacotes conhecidos (Nubank, Itaú, …) ou `bank`/`banco` no package name.

---

## 9. Como evoluir o app

1. **UI** — reutilize `theme.ts`; não invente cores soltas por tela
2. **Nova tela** — expo-router file-based; chame só `src/api.ts`
3. **Push FCM no app** — registrar via `POST /devices/push-token` (backend já aceita)
4. **Pluggy Connect Widget** — obter token em `/connect/token` e abrir widget; depois `/connect/items`
5. **Não** parsear notificação de novo em JS se o NLS já enviou
6. **OCR cloud** — só se ML Kit falhar em documentos densos; preferir worker no backend

### Dívidas / gaps comuns

- Expo Go não carrega NLS nem ML Kit
- iOS sem listener de notificação de banco
- Alinhar tipos TS se o backend renomear campos
- GitHub do app pode precisar ser criado manualmente (`gh` do agent às vezes não tem permissão de `createRepository`)
