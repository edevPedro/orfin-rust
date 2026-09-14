export type PaymentEvent = {
  id: string;
  user_id: string;
  source: string;
  external_id: string | null;
  amount: string;
  currency: string;
  description: string | null;
  merchant: string | null;
  category: string | null;
  suggested_category: string | null;
  user_note: string | null;
  paid_at: string;
  status: string;
  explained_at: string | null;
  created_at: string;
};

export type Category = {
  id: string;
  label: string;
};

export type ReportCategoryRow = {
  category: string;
  total: string;
  count: number;
};

export type ReportSourceRow = {
  source: string;
  total: string;
  count: number;
};

export type ReportSummary = {
  from: string;
  to: string;
  total_spent: string;
  currency: string;
  by_category: ReportCategoryRow[];
  by_source: ReportSourceRow[];
};

export type AlertRule = {
  id: string;
  user_id: string;
  kind: string;
  threshold: string;
  category: string | null;
  enabled: boolean;
  created_at: string;
};

export type AlertCheckResult = {
  triggered: boolean;
  messages: string[];
  rules: AlertRule[];
};

export type OcrPaymentPayload = {
  user_id: string;
  raw_text: string;
  amount?: string;
  merchant?: string;
  paid_at?: string;
  currency?: string;
};

export type ReconcileResult = {
  status: string;
  matched: number;
  unmatched: number;
  message?: string;
};
