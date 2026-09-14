export const colors = {
  brand: '#0F4C5C',
  brandSoft: '#1A6B7C',
  cta: '#D97706',
  ctaPressed: '#B45309',
  ink: '#0B1220',
  muted: '#5B6B7C',
  surface: '#F3F6FA',
  surfaceRaised: '#FFFFFF',
  success: '#0F766E',
  successSoft: '#CCFBF1',
  danger: '#B91C1C',
  dangerSoft: '#FEE2E2',
  border: '#D7E0EA',
  mistTop: '#DCE7F2',
  mistMid: '#E8EEF6',
  mistBottom: '#F3F6FA',
  amberSoft: '#FEF3C7',
  tealSoft: '#E0F2F1',
} as const;

export const spacing = {
  xs: 4,
  sm: 8,
  md: 12,
  lg: 16,
  xl: 24,
  xxl: 32,
  xxxl: 48,
} as const;

export const radius = {
  sm: 8,
  md: 12,
  lg: 16,
  xl: 22,
  pill: 999,
} as const;

export const type = {
  brand: {
    fontFamily: 'Fraunces_700Bold',
    fontSize: 42,
    letterSpacing: -0.8,
    lineHeight: 48,
  },
  heroAmount: {
    fontFamily: 'Fraunces_700Bold',
    fontSize: 40,
    letterSpacing: -1,
    lineHeight: 46,
  },
  title: {
    fontFamily: 'Fraunces_600SemiBold',
    fontSize: 24,
    letterSpacing: -0.4,
    lineHeight: 30,
  },
  section: {
    fontFamily: 'DMSans_700Bold',
    fontSize: 15,
    letterSpacing: 0.2,
    lineHeight: 20,
  },
  body: {
    fontFamily: 'DMSans_400Regular',
    fontSize: 15,
    lineHeight: 22,
  },
  bodyMedium: {
    fontFamily: 'DMSans_500Medium',
    fontSize: 15,
    lineHeight: 22,
  },
  label: {
    fontFamily: 'DMSans_600SemiBold',
    fontSize: 13,
    letterSpacing: 0.3,
    lineHeight: 18,
  },
  caption: {
    fontFamily: 'DMSans_400Regular',
    fontSize: 13,
    lineHeight: 18,
  },
  amount: {
    fontFamily: 'Fraunces_600SemiBold',
    fontSize: 22,
    letterSpacing: -0.4,
    lineHeight: 28,
  },
  button: {
    fontFamily: 'DMSans_700Bold',
    fontSize: 16,
    letterSpacing: 0.2,
  },
} as const;

export const statusLabels: Record<string, string> = {
  awaiting_user: 'Aguardando',
  categorized: 'Organizado',
  pending: 'Pendente',
  duplicate: 'Duplicado',
  failed: 'Falhou',
};

export const statusColors: Record<string, { bg: string; fg: string }> = {
  awaiting_user: { bg: colors.amberSoft, fg: colors.cta },
  categorized: { bg: colors.successSoft, fg: colors.success },
  pending: { bg: colors.tealSoft, fg: colors.brand },
  duplicate: { bg: colors.surface, fg: colors.muted },
  failed: { bg: colors.dangerSoft, fg: colors.danger },
};

export const categoryLabels: Record<string, string> = {
  alimentacao: 'Alimentação',
  transporte: 'Transporte',
  moradia: 'Moradia',
  lazer: 'Lazer',
  saude: 'Saúde',
  educacao: 'Educação',
  assinaturas: 'Assinaturas',
  outros: 'Outros',
};

export function formatBRL(amount: string | number): string {
  const n = typeof amount === 'string' ? Number(amount) : amount;
  if (Number.isNaN(n)) return `R$ ${amount}`;
  return n.toLocaleString('pt-BR', { style: 'currency', currency: 'BRL' });
}

export function formatDateTime(iso: string): string {
  try {
    return new Date(iso).toLocaleString('pt-BR', {
      day: '2-digit',
      month: 'short',
      hour: '2-digit',
      minute: '2-digit',
    });
  } catch {
    return iso;
  }
}
