import { StyleSheet, Text, View } from 'react-native';

import { colors, spacing, type } from '@/src/theme';

type Props = {
  title: string;
  action?: string;
};

export function SectionTitle({ title, action }: Props) {
  return (
    <View style={styles.row}>
      <Text style={styles.title}>{title}</Text>
      {action ? <Text style={styles.action}>{action}</Text> : null}
    </View>
  );
}

const styles = StyleSheet.create({
  row: {
    flexDirection: 'row',
    alignItems: 'baseline',
    justifyContent: 'space-between',
    marginBottom: spacing.md,
    marginTop: spacing.sm,
  },
  title: {
    ...type.section,
    color: colors.ink,
    textTransform: 'uppercase',
  },
  action: {
    ...type.caption,
    color: colors.muted,
  },
});
