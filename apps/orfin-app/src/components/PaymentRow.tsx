import { Link } from 'expo-router';
import { Pressable, StyleSheet, Text, View } from 'react-native';
import Animated, { FadeInDown } from 'react-native-reanimated';

import {
  categoryLabels,
  colors,
  formatBRL,
  formatDateTime,
  radius,
  spacing,
  statusColors,
  statusLabels,
  type,
} from '@/src/theme';
import type { PaymentEvent } from '@/src/types';

type Props = {
  payment: PaymentEvent;
  index?: number;
  showStatus?: boolean;
};

export function PaymentRow({ payment, index = 0, showStatus = false }: Props) {
  const status = statusLabels[payment.status] ?? payment.status;
  const tone = statusColors[payment.status] ?? { bg: colors.surface, fg: colors.muted };
  const categoryKey = payment.suggested_category ?? payment.category;
  const category = categoryKey
    ? (categoryLabels[categoryKey] ?? categoryKey)
    : 'Sem categoria';

  return (
    <Animated.View entering={FadeInDown.delay(Math.min(index, 8) * 45).springify().damping(18)}>
      <Link href={`/payment/${payment.id}`} asChild>
        <Pressable style={({ pressed }) => [styles.row, pressed && styles.pressed]}>
          <View style={styles.main}>
            <Text style={styles.amount}>{formatBRL(payment.amount)}</Text>
            <Text style={styles.merchant} numberOfLines={1}>
              {payment.merchant ?? payment.description ?? 'Sem estabelecimento'}
            </Text>
            <Text style={styles.meta} numberOfLines={1}>
              {category} · {formatDateTime(payment.paid_at)}
            </Text>
          </View>
          {showStatus && (
            <View style={[styles.pill, { backgroundColor: tone.bg }]}>
              <Text style={[styles.pillText, { color: tone.fg }]}>{status}</Text>
            </View>
          )}
        </Pressable>
      </Link>
    </Animated.View>
  );
}

const styles = StyleSheet.create({
  row: {
    backgroundColor: colors.surfaceRaised,
    borderRadius: radius.lg,
    padding: spacing.lg,
    marginBottom: spacing.md,
    borderWidth: 1,
    borderColor: colors.border,
    flexDirection: 'row',
    alignItems: 'center',
    gap: spacing.md,
  },
  pressed: { opacity: 0.88, transform: [{ scale: 0.992 }] },
  main: { flex: 1 },
  amount: { ...type.amount, color: colors.ink },
  merchant: { ...type.bodyMedium, color: colors.ink, marginTop: 4 },
  meta: { ...type.caption, color: colors.muted, marginTop: 6 },
  pill: {
    borderRadius: radius.pill,
    paddingHorizontal: 10,
    paddingVertical: 5,
  },
  pillText: { ...type.label, fontSize: 11 },
});
