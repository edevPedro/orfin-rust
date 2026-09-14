#!/usr/bin/env node
/** Minimal mock of orfin-rust for UI smoke tests. */
const http = require('http');
const crypto = require('crypto');

const categories = [
  { id: 'alimentacao', label: 'Alimentação' },
  { id: 'transporte', label: 'Transporte' },
  { id: 'moradia', label: 'Moradia' },
  { id: 'lazer', label: 'Lazer' },
  { id: 'saude', label: 'Saúde' },
  { id: 'educacao', label: 'Educação' },
  { id: 'assinaturas', label: 'Assinaturas' },
  { id: 'outros', label: 'Outros' },
];

const payments = [
  {
    id: '11111111-1111-1111-1111-111111111111',
    user_id: 'demo',
    source: 'android_notification',
    external_id: 'nubank|demo|1',
    amount: '45.90',
    currency: 'BRL',
    description: 'Compra aprovada',
    merchant: 'Mercado Extra',
    category: null,
    suggested_category: 'alimentacao',
    user_note: null,
    paid_at: new Date().toISOString(),
    status: 'awaiting_user',
    explained_at: null,
    created_at: new Date().toISOString(),
  },
  {
    id: '22222222-2222-2222-2222-222222222222',
    user_id: 'demo',
    source: 'pluggy',
    external_id: 'pluggy-tx-1',
    amount: '29.90',
    currency: 'BRL',
    description: 'Spotify',
    merchant: 'Spotify',
    category: 'assinaturas',
    suggested_category: 'assinaturas',
    user_note: 'família',
    paid_at: new Date(Date.now() - 86400000).toISOString(),
    status: 'categorized',
    explained_at: new Date().toISOString(),
    created_at: new Date().toISOString(),
  },
  {
    id: '33333333-3333-3333-3333-333333333333',
    user_id: 'demo',
    source: 'ocr',
    external_id: null,
    amount: '87.40',
    currency: 'BRL',
    description: 'Recibo',
    merchant: 'Farmácia São Paulo',
    category: 'saude',
    suggested_category: 'saude',
    user_note: null,
    paid_at: new Date(Date.now() - 172800000).toISOString(),
    status: 'categorized',
    explained_at: new Date().toISOString(),
    created_at: new Date().toISOString(),
  },
];

const alerts = [];

function json(res, status, body) {
  res.writeHead(status, {
    'Content-Type': 'application/json',
    'Access-Control-Allow-Origin': '*',
    'Access-Control-Allow-Headers': 'Content-Type, Accept',
    'Access-Control-Allow-Methods': 'GET,POST,OPTIONS',
  });
  res.end(JSON.stringify(body));
}

function readBody(req) {
  return new Promise((resolve) => {
    let body = '';
    req.on('data', (c) => (body += c));
    req.on('end', () => {
      try {
        resolve(JSON.parse(body || '{}'));
      } catch {
        resolve({});
      }
    });
  });
}

function suggestCategory(merchant = '') {
  const m = merchant.toLowerCase();
  if (/mercado|padaria|ifood|restaurante|cafe/.test(m)) return 'alimentacao';
  if (/uber|99|posto|shell|ipiranga/.test(m)) return 'transporte';
  if (/farm[aá]cia|drogaria|hospital/.test(m)) return 'saude';
  if (/spotify|netflix|disney|amazon/.test(m)) return 'assinaturas';
  return 'outros';
}

function buildSummary(userId) {
  const rows = payments.filter(
    (p) => (p.user_id === userId || userId === 'demo' || true) && p.status !== 'failed',
  );
  const byCat = {};
  const bySrc = {};
  let total = 0;
  for (const p of rows) {
    const amount = Number(p.amount) || 0;
    total += amount;
    const cat = p.category || p.suggested_category || 'outros';
    byCat[cat] = byCat[cat] || { category: cat, total: 0, count: 0 };
    byCat[cat].total += amount;
    byCat[cat].count += 1;
    bySrc[p.source] = bySrc[p.source] || { source: p.source, total: 0, count: 0 };
    bySrc[p.source].total += amount;
    bySrc[p.source].count += 1;
  }
  return {
    from: new Date(new Date().getFullYear(), new Date().getMonth(), 1).toISOString(),
    to: new Date().toISOString(),
    total_spent: total.toFixed(2),
    currency: 'BRL',
    by_category: Object.values(byCat)
      .map((r) => ({ ...r, total: r.total.toFixed(2) }))
      .sort((a, b) => Number(b.total) - Number(a.total)),
    by_source: Object.values(bySrc).map((r) => ({ ...r, total: r.total.toFixed(2) })),
  };
}

const server = http.createServer(async (req, res) => {
  if (req.method === 'OPTIONS') return json(res, 204, {});
  const url = new URL(req.url, 'http://localhost:3000');

  if (url.pathname === '/health') return json(res, 200, { status: 'ok' });
  if (url.pathname === '/categories') return json(res, 200, { categories });

  if (url.pathname === '/payments/awaiting') {
    return json(res, 200, {
      payments: payments.filter((p) => p.status === 'awaiting_user'),
    });
  }

  if (url.pathname === '/payments' && req.method === 'GET') {
    return json(res, 200, { payments });
  }

  if (url.pathname === '/payments/from-ocr' && req.method === 'POST') {
    const data = await readBody(req);
    const merchant = data.merchant || 'Recibo OCR';
    const payment = {
      id: crypto.randomUUID(),
      user_id: data.user_id || 'demo',
      source: 'ocr',
      external_id: null,
      amount: String(data.amount || '0'),
      currency: data.currency || 'BRL',
      description: data.raw_text?.slice(0, 80) || 'OCR',
      merchant,
      category: null,
      suggested_category: suggestCategory(merchant),
      user_note: null,
      paid_at: data.paid_at || new Date().toISOString(),
      status: 'awaiting_user',
      explained_at: null,
      created_at: new Date().toISOString(),
    };
    payments.unshift(payment);
    return json(res, 200, { status: 'ok', payment });
  }

  if (url.pathname.startsWith('/payments/') && url.pathname.endsWith('/explain')) {
    const id = url.pathname.split('/')[2];
    const data = await readBody(req);
    const p = payments.find((x) => x.id === id);
    if (!p) return json(res, 404, { error: 'not found' });
    p.category = data.category;
    p.user_note = data.note ?? null;
    p.status = 'categorized';
    p.explained_at = new Date().toISOString();
    return json(res, 200, { status: 'ok', payment: p });
  }

  if (url.pathname === '/channels/link' && req.method === 'POST') {
    return json(res, 200, { status: 'ok' });
  }

  if (url.pathname === '/reports/summary' && req.method === 'GET') {
    const userId = url.searchParams.get('user_id') || 'demo';
    return json(res, 200, buildSummary(userId));
  }

  if (url.pathname === '/alerts' && req.method === 'GET') {
    const userId = url.searchParams.get('user_id');
    return json(res, 200, {
      alerts: alerts.filter((a) => !userId || a.user_id === userId),
    });
  }

  if (url.pathname === '/alerts' && req.method === 'POST') {
    const data = await readBody(req);
    const alert = {
      id: crypto.randomUUID(),
      user_id: data.user_id || 'demo',
      kind: data.kind || 'budget_threshold',
      threshold: String(data.threshold || '0'),
      category: data.category ?? null,
      enabled: true,
      created_at: new Date().toISOString(),
    };
    alerts.push(alert);
    return json(res, 200, { status: 'ok', alert });
  }

  if (url.pathname === '/alerts/check' && req.method === 'GET') {
    const userId = url.searchParams.get('user_id') || 'demo';
    const summary = buildSummary(userId);
    const userAlerts = alerts.filter((a) => a.user_id === userId && a.enabled);
    const messages = [];
    let triggered = false;
    for (const rule of userAlerts) {
      const limit = Number(rule.threshold) || 0;
      let spent = Number(summary.total_spent) || 0;
      if (rule.category) {
        const row = summary.by_category.find((c) => c.category === rule.category);
        spent = row ? Number(row.total) : 0;
      }
      if (spent >= limit) {
        triggered = true;
        messages.push(
          `Gasto ${spent.toFixed(2)} atingiu o limite ${limit.toFixed(2)}${
            rule.category ? ` em ${rule.category}` : ''
          }.`,
        );
      }
    }
    if (!messages.length) {
      messages.push('Nenhum limite ultrapassado neste momento.');
    }
    return json(res, 200, { triggered, messages, rules: userAlerts });
  }

  if (url.pathname === '/reconcile/run' && req.method === 'POST') {
    await readBody(req);
    return json(res, 200, {
      status: 'ok',
      matched: 2,
      unmatched: 1,
      message: 'Comparação concluída: 2 batendo, 1 sem match no extrato Open Finance.',
    });
  }

  json(res, 404, { error: 'not found' });
});

server.listen(3000, () => console.log('mock orfin API on :3000'));
