import { StyleSheet, Text, View } from 'react-native';
import Animated, { FadeIn, ZoomIn } from 'react-native-reanimated';

import { colors, formatBRL, spacing, type } from '@/src/theme';

type Props = {
  amount: string;
  merchant?: string | null;
  subtitle?: string | null;
};

export function AmountHero({ amount, merchant, subtitle }: Props) {
  return (
    <View style={styles.wrap}>
      <Animated.Text entering={ZoomIn.springify().damping(14)} style={styles.amount}>
        {formatBRL(amount)}
      </Animated.Text>
      {merchant ? (
        <Animated.Text entering={FadeIn.delay(120)} style={styles.merchant}>
          {merchant}
        </Animated.Text>
      ) : null}
      {subtitle ? <Text style={styles.subtitle}>{subtitle}</Text> : null}
    </View>
  );
}

const styles = StyleSheet.create({
  wrap: {
    paddingVertical: spacing.xl,
  },
  amount: {
    ...type.heroAmount,
    color: colors.brand,
  },
  merchant: {
    ...type.title,
    color: colors.ink,
    marginTop: spacing.sm,
  },
  subtitle: {
    ...type.caption,
    color: colors.muted,
    marginTop: spacing.sm,
  },
});
