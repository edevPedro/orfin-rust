import { StyleSheet, Text, View } from 'react-native';

import { colors, spacing, type } from '@/src/theme';

type Props = {
  tagline?: string;
  compact?: boolean;
};

export function BrandHeader({
  tagline = 'Organize o que acabou de sair da conta',
  compact = false,
}: Props) {
  return (
    <View style={[styles.wrap, compact && styles.compact]}>
      <Text style={[styles.brand, compact && styles.brandCompact]}>Orfin</Text>
      {!compact && <Text style={styles.tagline}>{tagline}</Text>}
    </View>
  );
}

const styles = StyleSheet.create({
  wrap: {
    paddingTop: spacing.xl,
    paddingBottom: spacing.lg,
  },
  compact: {
    paddingTop: spacing.md,
    paddingBottom: spacing.sm,
  },
  brand: {
    ...type.brand,
    color: colors.brand,
  },
  brandCompact: {
    fontSize: 28,
    lineHeight: 34,
  },
  tagline: {
    ...type.body,
    color: colors.muted,
    marginTop: spacing.sm,
    maxWidth: 320,
  },
});
