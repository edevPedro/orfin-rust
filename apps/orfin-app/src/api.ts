import { API_URL } from './config';
import { getStoredApiUrl } from './storage';
import type {
  AlertCheckResult,
  AlertRule,
  Category,
  OcrPaymentPayload,
  PaymentEvent,
  ReconcileResult,
  ReportSummary,
} from './types';

async function baseUrl(): Promise<string> {
  return (await getStoredApiUrl()) ?? API_URL;
}

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const response = await fetch(`${await baseUrl()}${path}`, {
    ...init,
    headers: {
      Accept: 'application/json',
      'Content-Type': 'application/json',
      ...(init?.headers ?? {}),
    },
  });
  const body = await response.json().catch(() => ({}));
  if (!response.ok) {
    throw new Error((body as { error?: string }).error ?? `HTTP ${response.status}`);
  }
  return body as T;
}

export const api = {
  listAwaiting: (userId: string) =>
    request<{ payments: PaymentEvent[] }>(
      `/payments/awaiting?user_id=${encodeURIComponent(userId)}&limit=50`,
    ),

  listPayments: (userId: string, status?: string) => {
    const params = new URLSearchParams({ user_id: userId, limit: '50' });
    if (status) params.set('status', status);
    return request<{ payments: PaymentEvent[] }>(`/payments?${params}`);
  },

  explain: (paymentId: string, category: string, note?: string) =>
    request<{ status: string; payment: PaymentEvent }>(`/payments/${paymentId}/explain`, {
      method: 'POST',
      body: JSON.stringify({ category, note }),
    }),

  fromOcr: (payload: OcrPaymentPayload) =>
    request<{ status: string; payment: PaymentEvent }>('/payments/from-ocr', {
      method: 'POST',
      body: JSON.stringify(payload),
    }),

  categories: () => request<{ categories: Category[] }>('/categories'),

  linkChannel: (userId: string, channel: 'discord' | 'telegram', target: string) =>
    request<{ status: string }>('/channels/link', {
      method: 'POST',
      body: JSON.stringify({
        user_id: userId,
        channel,
        target,
        enabled: true,
      }),
    }),

  reportSummary: (userId: string, from: string, to: string) => {
    const params = new URLSearchParams({ user_id: userId, from, to });
    return request<ReportSummary>(`/reports/summary?${params}`);
  },

  listAlerts: (userId: string) =>
    request<{ alerts: AlertRule[] }>(`/alerts?user_id=${encodeURIComponent(userId)}`),

  createAlert: (input: {
    user_id: string;
    kind: string;
    threshold: string;
    category?: string | null;
  }) =>
    request<{ status: string; alert: AlertRule }>('/alerts', {
      method: 'POST',
      body: JSON.stringify(input),
    }),

  checkAlerts: (userId: string) =>
    request<AlertCheckResult>(`/alerts/check?user_id=${encodeURIComponent(userId)}`),

  reconcileRun: (userId: string) =>
    request<ReconcileResult>('/reconcile/run', {
      method: 'POST',
      body: JSON.stringify({ user_id: userId }),
    }),
};
