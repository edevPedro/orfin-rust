export type ParsedExpense = {
  amount: string | null;
  merchant: string | null;
  currency: string;
  hints: string[];
};

const AMOUNT_PATTERNS = [
  /R\$\s*(\d{1,3}(?:\.\d{3})*,\d{2}|\d+,\d{2}|\d+\.\d{2})/i,
  /(?:valor|total|pago|compra)\s*[:=]?\s*R?\$?\s*(\d{1,3}(?:\.\d{3})*,\d{2}|\d+,\d{2}|\d+\.\d{2})/i,
  /(\d{1,3}(?:\.\d{3})*,\d{2})\s*(?:BRL|reais)?/i,
];

const MERCHANT_SKIP =
  /^(compra|aprovada|pix|cart[aã]o|nubank|ita[uú]|bradesco|inter|c6|banco|valor|total|data|hora|recibo|cupom|nf-e|nota)/i;

function normalizeAmount(raw: string): string {
  const trimmed = raw.trim();
  if (trimmed.includes(',')) {
    return trimmed.replace(/\./g, '').replace(',', '.');
  }
  return trimmed;
}

function guessMerchant(lines: string[]): string | null {
  for (const line of lines) {
    const clean = line.replace(/\s+/g, ' ').trim();
    if (clean.length < 3 || clean.length > 48) continue;
    if (/^\d/.test(clean)) continue;
    if (/R\$/.test(clean)) continue;
    if (MERCHANT_SKIP.test(clean)) continue;
    if (/^\d{1,2}[\/\-]\d{1,2}/.test(clean)) continue;
    return clean;
  }
  return null;
}

/** Client-side structured parse of receipt / notification text (BRL). */
export function parseExpenseText(text: string): ParsedExpense {
  const normalized = text.replace(/\r/g, '\n').trim();
  const lines = normalized
    .split('\n')
    .map((l) => l.trim())
    .filter(Boolean);

  let amount: string | null = null;
  for (const pattern of AMOUNT_PATTERNS) {
    const match = normalized.match(pattern);
    if (match?.[1]) {
      amount = normalizeAmount(match[1]);
      break;
    }
  }

  const merchant = guessMerchant(lines);
  const hints: string[] = [];
  if (amount) hints.push(`valor ${amount}`);
  if (merchant) hints.push(`loja ${merchant}`);
  if (/pix/i.test(normalized)) hints.push('pix');
  if (/cart[aã]o|crédito|débito/i.test(normalized)) hints.push('cartão');

  return {
    amount,
    merchant,
    currency: 'BRL',
    hints,
  };
}
