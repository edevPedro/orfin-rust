# Orfin App

Cliente web + mobile do Orfin — **Expo (React Native)** com visual de produto financeiro brasileiro.

## Documentação

| Doc | Conteúdo |
|---|---|
| [`docs/ARCHITECTURE.md`](./docs/ARCHITECTURE.md) | Telas, NLS, OCR, design system, contrato HTTP |
| Backend `docs/ARCHITECTURE.md` | Algoritmos de dedupe/categoria/reconcile/alertas |

Backend: [orfin-rust](https://github.com/edevPedro/orfin-rust).

## Por que Expo

Um código para **web**, **Android** e **iOS**. NotificationListener é módulo nativo Android; iOS/web usam Pluggy + OCR de comprovantes.

## Tipografia e tema

- **Fraunces** (marca / valores) + **DM Sans** (UI)
- Tokens em `src/theme.ts`: teal `#0F4C5C`, CTA âmbar `#D97706`, névoa slate-blue

## Telas

1. **Aguardando** — marca Orfin + fila `awaiting_user` + atalho Capturar  
2. **Pagamentos** — histórico com pills PT-BR  
3. **Relatórios** — `GET /reports/summary` + comparar Open Finance  
4. **Ajustes** — user_id, API, NLS, canais, alertas  
5. **Organizar** — categoria + nota  
6. **Capturar** — foto/OCR ou colar texto → `/payments/from-ocr`

## Estratégia OCR / texto

| Fonte | Como |
|---|---|
| Notificações bancárias (Android) | Texto nativo do NotificationListener — **sem OCR**; posta sozinho em `/payments/from-notification` |
| Recibos / screenshots | **ML Kit on-device** (`expo-mlkit-ocr` + `expo-image-picker`; precisa dev build) |
| Web | Sem OCR nativo — colar texto; parser em `src/parseExpenseText.ts` |
| Extrato Open Finance | Backend `POST /reconcile/run` (botão em Relatórios) |

iOS: `expo-build-properties` com `deploymentTarget: 16.4` (SDK 57 + ML Kit).

## Setup

```bash
cp .env.example .env   # EXPO_PUBLIC_API_URL
npm install
npm run web
```

Smoke sem backend:

```bash
npm run mock-api
EXPO_PUBLIC_API_URL=http://localhost:3000 npm run web
```

Emulador Android: `http://10.0.2.2:3000`.

### Android (NotificationListener + OCR)

Precisa de **dev build** (não Expo Go):

```bash
npx expo prebuild
npx expo run:android
```

Ajustes → ativar *Acesso a notificações* → salvar `user_id` + API URL.

O serviço nativo posta em `/payments/from-notification` **sem JS** (funciona com app morto).
